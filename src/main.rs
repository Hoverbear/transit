#![feature(core)]
#![feature(path)]

#![feature(plugin)]
#![plugin(regex_macros)]
extern crate regex;
extern crate git2;
extern crate core;
extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;

use git2::{Repository, Branch, BranchType, DiffLine,
    Commit, Diff, DiffFormat, DiffDelta, DiffHunk, Oid};
use docopt::Docopt;
use std::old_io as io;
use std::collections::HashMap;
use std::old_io::BufferedReader;
use std::old_io::File;
use std::str;
use regex::Regex;
use rustc_serialize::json::{self, ToJson, Json};
use std::fmt::Display;

// Write the Docopt usage string.
static USAGE: &'static str = "
Usage: transit <repo> [--json] [<old> <new>]

If no commits are given, transit will revwalk from latest to oldest.
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_repo: String,
    arg_old: Option<String>,
    arg_new: Option<String>,
    flag_json: bool,
}

fn main() {
    // Parse the args above or die.
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());
    // Pull up the revwalk.
    let path = Path::new(args.arg_repo);
    let repo = Repository::discover(&path)
        .ok().expect("Unable to find repo.");
    // If we can destructures the two optional args into real things.
    if let (Some(old_string), Some(new_string)) = (args.arg_old, args.arg_new) {
        // Compare a specific commit pair.
        // Fist, get the commits. (Error checked)
        let old = Oid::from_str(&old_string[]).and_then(|oid| repo.find_commit(oid));
        let new = Oid::from_str(&new_string[]).and_then(|oid| repo.find_commit(oid));
        if old.is_ok() && new.is_ok() {
            let output = find_moves(&repo, &old.unwrap(), &new.unwrap()).unwrap();
            make_output(output);
        } else {
            panic!("Commit ids were not valid.");
        }
    } else {
        // Revwalk.
        let mut revwalk = repo.revwalk()
            .ok().expect("Unable to get revwalk.");
        // Setup some options.
        revwalk.simplify_first_parent(); // TODO: Maybe remove?
        let mut flags = git2::Sort::empty();
        flags.insert(git2::SORT_TIME);
        flags.insert(git2::SORT_TOPOLOGICAL);
        revwalk.set_sorting(flags);
        // Push HEAD to the revwalk.
        revwalk.push_head()
            .ok().expect("Unable to push HEAD.");
        // We sadly must collect here to use `.windows()`
        let history = revwalk.filter_map(|id| repo.find_commit(id).ok())
            .collect::<Vec<Commit>>();
        // Walk through each pair of commits.
        for pair in history.windows(2) {let (old, new) = (&pair[1], &pair[0]);
            let output = find_moves(&repo, old.clone(), new.clone()).unwrap();
            if args.flag_json {
                make_json(output);
            } else {
                make_output(output);
            }
        }
    }
}

fn make_output(output: Vec<Output>) {
    println!("make_output: output.len()={}", output.len());
    for i in range(0, output.len()) {
            println!("\told_commit={}",         output[i].old_commit);
            println!("\tnew_commit={}",         output[i].new_commit);
            println!("\torigin_line={}",        output[i].origin_line);
            println!("\tdestintation_line={}",  output[i].destination_line);
            println!("\tnum_lines={}",          output[i].num_lines);
            println!("\tnew_filename={}",       output[i].new_filename);
            println!("\told_filename={}",       output[i].old_filename);
    }
}

fn make_json(output: Vec<Output>) {
    println!("{}", json::encode(&output).unwrap());
}

#[derive(Debug, PartialEq, Eq)]
enum FoundState {
    Added, Deleted
}

fn dump_diffline(line: &DiffLine) {
    // 'origin' is wrapped in pipes to ease displaying space characters.
    print!("line: old={:?} new={:?} offset={} |origin|=|{}|\n      content={}",
             line.old_lineno(), line.new_lineno(), line.content_offset(),
             line.origin(), str::from_utf8(line.content()).unwrap());
}

fn dump_diffdelta(delta: &DiffDelta) {
    println!("delta: nfiles={} status={:?} old_file=(id={} path_bytes={:?} path={:?} tsize={}) new_file=(id={} path_bytes={:?} path={:?} tsize={})",
            delta.nfiles(), delta.status(),
            delta.old_file().id(), delta.old_file().path_bytes(), delta.old_file().path(), delta.old_file().size(),
            delta.new_file().id(), delta.new_file().path_bytes(), delta.new_file().path(), delta.new_file().size());
}

fn dump_diffhunk(hunk: &DiffHunk) {
    println!("hunk: old_start={} old_lines={} new_start={} new_lines={} header={}",
            hunk.old_start(), hunk.old_lines(),
            hunk.new_start(), hunk.new_lines(),
            str::from_utf8(hunk.header()).unwrap());
}

fn format_key(key: String) -> String {
    let remove_whitespace = regex!(r"\s{2,}"); // 2 or more whitespaces    // TODO Removes whitespace from a string.
    let trim = regex!(r"^[\s]+|[\s]+$");
    let result = remove_whitespace.replace_all(key.as_slice(), "");
    trim.replace_all(result.as_slice(), "")
}

#[derive(Debug)]
struct Found {
    filename: Path,
    key: String,
    state: FoundState,
    start_position: u32,
    line_count: u32,
}

fn find_additions_and_deletions(diff: Diff) -> Vec<Found> {

    #[derive(Debug)]
    enum State {
        Other, Addition, Deletion
    }

    let mut founds: Vec<Found> = Vec::new();

    let mut state = State::Other;
    let mut added = String::new();
    let mut deleted = String::new();
    let mut old_path = Path::new("");
    let mut new_path = Path::new("");

    let mut line_count: u32 = 0;
    let mut start_position: u32 = 0;

    // Read about this function in http://alexcrichton.com/git2-rs/git2/struct.Diff.html#method.print
    // It's a bit weird, but I think it will provide the necessary information.
    diff.print(DiffFormat::Patch, |delta, maybe_hunk, line| -> bool {

        assert!(delta.nfiles() == 2, "This only works on diffs between exactly 2 files. Found {} files.", delta.nfiles());

        // Thinking:
        //  * If is not a hunk, keep going.
        //  * If it's a hunk, do regex magic.
        //  * Stick regex output into a hashmap as a hash.
        //  * Later, we will iterate through and output pased on the values.
        // Filter out all the boring context lines.
        // If we're not interested in this line just return since it will iterate to the next.
        if maybe_hunk.is_none() { return true }; // Return early.

        //dump_diffline(&line);
        //dump_diffdelta(&delta);
        //dump_diffhunk(&maybe_hunk.unwrap());

        old_path = delta.old_file().path().unwrap().clone();
        new_path = delta.new_file().path().unwrap().clone();

        match line.origin() {
            // Additions
            '+' | '>' => {
                added.push_str(str::from_utf8(line.content()).unwrap());

                match state {
                    State::Deletion => {
                        founds.push(Found {
                            filename: old_path.clone(),
                            key: format_key(deleted.clone()),
                            state: FoundState::Deleted,
                            start_position: start_position,
                            line_count: line_count,
                        });
                        deleted = String::new();
                        line_count = 0;
                        start_position = line.new_lineno().unwrap();
                    },
                    State::Addition => {
                        line_count += 1;
                    },
                    State::Other => {
                        line_count = 1;
                        start_position = line.new_lineno().unwrap();
                    },
                }

                state = State::Addition;
                true
            },
            // Deletions
            '-' | '<' => {
                deleted.push_str(str::from_utf8(line.content()).unwrap());

                match state {
                    State::Addition => {
                        founds.push(Found {
                            filename: new_path.clone(),
                            key: format_key(added.clone()),
                            state: FoundState::Added,
                            start_position: start_position,
                            line_count: line_count,
                        });
                        added = String::new();
                        line_count = 0;
                        start_position = line.old_lineno().unwrap();
                    },
                    State::Deletion => {
                        line_count += 1;
                    },
                    State::Other => {
                        line_count = 1;
                        start_position = line.old_lineno().unwrap();
                    },
                }

                state = State::Deletion;
                true
            },
            // Other
            _         => {
                match state {
                    State::Addition => {
                        founds.push(Found {
                            filename: new_path.clone(),
                            key: format_key(added.clone()),
                            state: FoundState::Added,
                            start_position: start_position,
                            line_count: line_count,
                        });
                        added = String::new();
                    },
                    State::Deletion => {
                        founds.push(Found {
                            filename: old_path.clone(),
                            key: format_key(deleted.clone()),
                            state: FoundState::Deleted,
                            start_position: start_position,
                            line_count: line_count,
                        });
                        deleted = String::new();
                    },
                    _ => (),
                }

                state = State::Other;
                true
            }
        }
    });

    // Grab last one.
    match state {
        State::Addition => {
            if added.len() > 0 {
                founds.push(Found {
                    filename: new_path,
                    key: format_key(added.clone()),
                    state: FoundState::Added,
                    start_position: start_position,
                    line_count: line_count,
                });
            }
        },
        State::Deletion => {
            if deleted.len() > 0 {
                founds.push(Found {
                    filename: old_path,
                    key: format_key(deleted.clone()),
                    state: FoundState::Deleted,
                    start_position: start_position,
                    line_count: line_count,
                });
            }
        },
        _ => (),
    }

    return founds;
}

fn path_to_string(path: Path) -> String {
    String::from_utf8(path.into_vec()).unwrap()
}

fn find_moves(repo: &Repository, old: &Commit, new: &Commit) -> Result<Vec<Output>, git2::Error> {
    let old_tree = try!(old.tree());
    let new_tree = try!(new.tree());
    // Build up a diff of the two trees.
    let diff = try!(Diff::tree_to_tree(repo, Some(&old_tree), Some(&new_tree), None));

    let founds: Vec<Found> = find_additions_and_deletions(diff);

    let mut moves: Vec<Output> = Vec::new();
    let mut map: HashMap<String, Found> = HashMap::new();

    for f in founds {
        if f.key.len() == 0 { continue; }

        if map.contains_key(&f.key) {
            let q = map.get(&f.key).unwrap();

            assert!(f.state != q.state, "One state must be an addition and the other state must be a deletion.");

            let output: Output;

            match f.state {
                FoundState::Added => {
                    output = Output {
                        old_commit: TransitOid(old.id()),
                        new_commit: TransitOid(new.id()),
                        old_filename: path_to_string(q.filename.clone()),
                        new_filename: path_to_string(f.filename.clone()),
                        origin_line: q.start_position,
                        destination_line: f.start_position,
                        num_lines: f.line_count,
                    };
                },
                FoundState::Deleted => {
                    output = Output {
                        old_commit: TransitOid(old.id()),
                        new_commit: TransitOid(new.id()),
                        old_filename: path_to_string(f.filename.clone()),
                        new_filename: path_to_string(q.filename.clone()),
                        origin_line: f.start_position,
                        destination_line: q.start_position,
                        num_lines: f.line_count,
                    };
                },
            }

            moves.push(output);
        } else {
            map.insert(f.key.clone(), f);
        }
    }

    return Ok(moves);
}


#[derive(Debug, RustcEncodable)]
struct Output {
    old_commit: TransitOid,
    new_commit: TransitOid,
    old_filename: String,
    new_filename: String,
    origin_line: u32,
    destination_line: u32,
    num_lines: u32,
}

#[derive(Debug)]
struct TransitOid(Oid);
impl Display for TransitOid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        Display::fmt(&self.0, f)
    }
}

impl rustc_serialize::Encodable for TransitOid {
    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_str(&format!("{}", self)[])
    }
}
