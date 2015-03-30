#![feature(core)]
#![feature(path)]
#![feature(collections)]

#![feature(plugin)]
#![plugin(regex_macros)]
extern crate regex;
extern crate git2;
extern crate core;
extern crate rustc_serialize;
extern crate docopt;

use git2::{Repository, Commit, Diff, DiffFormat, Oid};
use docopt::Docopt;
use std::collections::HashMap;
// use std::old_io::BufferedReader;
// use std::old_io::File;
use std::str;
// use regex::Regex;
use rustc_serialize::json;
use std::fmt::Display;
use std::path::Path;

// Write the Docopt usage string.
static USAGE: &'static str = "
Usage: transit <repo> [<old> <new>]

If no commits are given, transit will revwalk from latest to oldest. Output is ing JSON.
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_repo: String,
    arg_old: Option<String>,
    arg_new: Option<String>,
}

fn main() {
    // Parse the args above or die.
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());
    // Pull up the revwalk.
    let path = Path::new(&args.arg_repo);
    let repo = Repository::discover(&path)
        .ok().expect("Unable to find repo.");
    // If we can destructures the two optional args into real things.
    if let (Some(old_string), Some(new_string)) = (args.arg_old, args.arg_new) {
        // Compare a specific commit pair.
        // Fist, get the commits. (Error checked)
        let old_id = Oid::from_str(&old_string[..])
            .ok().expect("Couldn't parse old ID");
        let old = repo.find_commit(old_id);
        let new_id = Oid::from_str(&new_string[..])
            .ok().expect("Couldn't parse new ID");
        let new = repo.find_commit(new_id);
        if old.is_ok() && new.is_ok() {
            let (old_commit, new_commit) = (old.unwrap(), new.unwrap());
            let output = find_moves(&repo, &old_commit, &new_commit).unwrap();
            make_json(vec![OutputSet {
                old: TransitOid(old_id),
                old_time: old_commit.time().seconds(), // Seconds from Epoch
                new: TransitOid(new_id),
                new_time: new_commit.time().seconds(), // Seconds from Epoch
                outputs: output,
            }]);
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
        let history = revwalk.collect::<Vec<Oid>>();
        let mut output = Vec::with_capacity(history.len());
        // Walk through each pair of commits.
        for pair in history.windows(2) {
            let (old_id, new_id) = (pair[0], pair[1]);
            if let (Ok(old), Ok(new)) = (repo.find_commit(old_id), repo.find_commit(new_id)) {
                let detected = find_moves(&repo, &old, &new).unwrap();
                output.push(OutputSet {
                    old: TransitOid(old_id),
                    old_time: old.time().seconds(), // Seconds from Epoch
                    new: TransitOid(new_id),
                    new_time: new.time().seconds(), // Seconds from Epoch
                    outputs: detected,
                });
            } else {
                continue;
            }
        }
        make_json(output);
    }
}

fn make_json(output: Vec<OutputSet>) {
    let out = json::as_pretty_json(&output).indent(4);
    println!("{}", out);
}

#[derive(Debug, PartialEq, Eq)]
enum FoundState {
    Added, Deleted
}

fn format_key(key: String) -> String {
    let remove_whitespace = regex!(r"\s{2,}"); // 2 or more whitespaces    // TODO Removes whitespace from a string.
    let trim = regex!(r"^[\s]+|[\s]+$");
    let result = remove_whitespace.replace_all(key.as_slice(), "");
    trim.replace_all(result.as_slice(), "")
}

#[derive(Debug)]
struct Found {
    filename: String,
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
    let mut old_path = String::new();
    let mut new_path = String::new();

    let mut line_count: u32 = 0;
    let mut start_position: u32 = 0;

    // Read about this function in http://alexcrichton.com/git2-rs/git2/struct.Diff.html#method.print
    // It's a bit weird, but I think it will provide the necessary information.
    diff.print(DiffFormat::Patch, |delta, maybe_hunk, line| -> bool {

        // if delta.nfiles() != 2 {
        //     // This is diff with only one side, and thus can't have a move.
        //     return true;
        // }
        // assert!(delta.nfiles() == 2, "This only works on diffs between exactly 2 files. Found {} files.", delta.nfiles());

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

        old_path = match delta.old_file().path()
        .and_then(|x| x.to_str()) {
            Some(path) => String::from_str(path),
            None => return false,
        };
        new_path = match delta.new_file().path()
        .and_then(|x| x.to_str()) {
            Some(path) => String::from_str(path),
            None => return false,
        };

        match line.origin() {
            // Additions
            '+' | '>' => {
                // If we attempt to unwrap and get `InvalidBytes(_)` it's probably just junk.
                // TODO: Is it?
                let line_str = str::from_utf8(line.content()).unwrap_or("");
                added.push_str(line_str);

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
                        start_position = match line.new_lineno() {
                            Some(lineno) => lineno,
                            None => return false, // Can't be a move if deleted.
                        }
                    },
                    State::Addition => {
                        line_count += 1;
                    },
                    State::Other => {
                        line_count = 1;
                        start_position = match line.new_lineno() {
                            Some(lineno) => lineno,
                            None => return false, // Can't be a move if deleted.
                        }
                    },
                }

                state = State::Addition;
                true
            },
            // Deletions
            '-' | '<' => {
                // If we attempt to unwrap and get `InvalidBytes(_)` it's probably just junk.
                // TODO: Is it?
                let line_str = str::from_utf8(line.content()).unwrap_or("");
                deleted.push_str(line_str);

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
                        start_position = match line.old_lineno() {
                            Some(lineno) => lineno,
                            None => return false, // Can't be a move if deleted.
                        }
                    },
                    State::Deletion => {
                        line_count += 1;
                    },
                    State::Other => {
                        line_count = 1;
                        start_position = match line.old_lineno() {
                            Some(lineno) => lineno,
                            None => return false, // Can't be a move if deleted.
                        }
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
    }).ok(); // We don't care if we exit early. That's fine.

    // Grab last one.
    match state {
        State::Addition => {
            if added.len() > 0 {
                founds.push(Found {
                    filename: new_path.clone(),
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
                    filename: old_path.clone(),
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

            // If both states are added, there are no moves.
            // assert!(f.state != q.state, format!("States {:?}, {:?}. Should be Addition/Deletion ", f.state, q.state));
            if f.state == q.state {
                return Ok(Vec::<Output>::new())
            }

            let output: Output;

            match f.state {
                FoundState::Added => {
                    output = Output {
                        old_filename: q.filename.clone(),
                        new_filename: f.filename.clone(),
                        origin_line: q.start_position,
                        destination_line: f.start_position,
                        num_lines: f.line_count,
                    };
                },
                FoundState::Deleted => {
                    output = Output {
                        old_filename: f.filename.clone(),
                        new_filename: q.filename.clone(),
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
struct OutputSet {
    old: TransitOid,
    old_time: i64, // Seconds from Epoch
    new: TransitOid,
    new_time: i64, // Seconds from Epoch
    outputs: Vec<Output>,
}

#[derive(Debug, RustcEncodable)]
struct Output {
    old_filename: String,
    new_filename: String,
    origin_line: u32,
    destination_line: u32,
    num_lines: u32,
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct TransitOid(Oid);
impl Display for TransitOid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        Display::fmt(&self.0, f)
    }
}

impl rustc_serialize::Encodable for TransitOid {
    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_str(&format!("{}", self)[..])
    }
}
