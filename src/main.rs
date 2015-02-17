#![feature(core)]
#![feature(path)]

#![feature(plugin)]
#![plugin(regex_macros)]
extern crate regex;
extern crate git2;
extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;

use git2::{Repository, Branch, BranchType, DiffLine,
    Commit, Diff, DiffFormat, DiffDelta, DiffHunk, Error, Oid};
use docopt::Docopt;
use std::old_io as io;
use std::old_io::BufferedReader;
use std::old_io::File;
use std::str;
use regex::Regex;

// Write the Docopt usage string.
static USAGE: &'static str = "
Usage: transit <repo> [<old> <new>]

If no commits are given, transit will revwalk from latest to oldest.
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
			// mangle the vectors here to make pretty output
			make_output(output);
		}
	}
}

fn make_output(output: Vec<Output>) {
	println!("make_output:");
    println!("\toutput.len()={}", output.len());
    for i in range(0, output.len()) {
            println!("\told_commit={}",         output[i].old_commit);
            println!("\tnew_commit={}",         output[i].new_commit);
            println!("\torigin_line={}",        output[i].origin_line);
            println!("\tdestintation_line={}",  output[i].destination_line);
            println!("\tnum_lines={}",          output[i].num_lines);
    }
}


#[derive(Debug)]
enum State {
    Other, Addition, Deletion
}

#[derive(Debug)]
enum FoundState {
    Added, Deleted
}

fn dump_diffdelta(delta: &DiffDelta) {
    println!("delta: nfiles={} status={:?} old_file=(id={} path_bytes={:?} path={:?} tsize={}) new_file=(id={} path_bytes={:?} path={:?} tsize={})",
            delta.nfiles(), delta.status(),
            delta.old_file().id(), delta.old_file().path_bytes(), delta.old_file().path(), delta.old_file().size(),
            delta.new_file().id(), delta.new_file().path_bytes(), delta.new_file().path(), delta.new_file().size());
}

fn format_key(key: String) -> String {
    let removeWhiteSpace = regex!(r"\s{2,}"); // 2 or more whitespaces    // TODO Removes whitespace from a string.
    removeWhiteSpace.replace_all(key.as_slice(), "")
}

#[derive(Debug)]
struct Found {
    filename: Path,
    key: String,
    state: FoundState,
}

fn find_moves(repo: &Repository, old: &Commit, new: &Commit) -> Result<Vec<Output>, Error> {

    println!("\nFIND MOVES---------------------");

	let old_tree = try!(old.tree());
	let new_tree = try!(new.tree());
	// Build up a diff of the two trees.
	let diff = try!(Diff::tree_to_tree(repo, Some(&old_tree), Some(&new_tree), None));

    let mut state = State::Other;
    let mut added = String::new();
    let mut deleted = String::new();
    let mut old_path: Path = Path::new("");
    let mut new_path: Path = Path::new("");


    let mut keys: Vec<String> = Vec::new();  // TODO Will become hashmap.

    let mut founds: Vec<Found> = Vec::new();

	// Read about this function in http://alexcrichton.com/git2-rs/git2/struct.Diff.html#method.print
	// It's a bit weird, but I think it will provide the necessary information.
	diff.print(DiffFormat::Patch, |delta, maybe_hunk, line| -> bool {

        assert!(delta.nfiles() == 2, "This only works on diffs between exactly 2 files.");

		// Thinking:
		//  * If is not a hunk, keep going.
		//  * If it's a hunk, do regex magic.
		//  * Stick regex output into a hashmap as a hash.
		//  * Later, we will iterate through and output pased on the values.
		// Filter out all the boring context lines.
		// If we're not interested in this line just return since it will iterate to the next.
		if maybe_hunk.is_none() { return true }; // Return early.

        //println!("top of loop: keys={:?}", keys);
        println!("top of loop: founds={:?}", founds);
        println!("top of loop: added={:?} deleted={:?}", added, deleted);

        // 'origin' is wrapped in pipes to ease displaying space characters.
        print!("line: old={:?} new={:?} offset={} |origin|=|{}|\n      content={}",
                 line.old_lineno(), line.new_lineno(), line.content_offset(),
                 line.origin(), str::from_utf8(line.content()).unwrap());

        //dump_diffdelta(&delta);

        old_path = delta.old_file().path().unwrap().clone();
        new_path = delta.new_file().path().unwrap().clone();

		match line.origin() {
			// Additions
			'+' | '>' => {
                println!("In additions. state={:?}", state);

                added.push_str(str::from_utf8(line.content()).unwrap());

                match state {
                    State::Deletion => {
                        founds.push(Found {
                            filename: delta.old_file().path().unwrap().clone(),
                            key: format_key(deleted.clone()),
                            state: FoundState::Deleted,
                        });
                        deleted = String::new();
                    },
                    _ => (),
                }

                state = State::Addition;
				true
			},
			// Deletions
			'-' | '<' => {
                println!("In deletion. state={:?}", state);

                deleted.push_str(str::from_utf8(line.content()).unwrap());

                match state {
                    State::Addition => {
                        founds.push(Found {
                            filename: delta.new_file().path().unwrap().clone(),
                            key: format_key(added.clone()),
                            state: FoundState::Added,
                        });
                        added = String::new();
                    },
                    _ => (),
                }

                state = State::Deletion;
				true
			},
			// Other
			_         => {
                println!("in _. state={:?}", state);

                match state {
                    State::Addition => {
                        founds.push(Found {
                            filename: delta.new_file().path().unwrap().clone(),
                            key: format_key(added.clone()),
                            state: FoundState::Added,
                        });
                        added = String::new();
                    },
                    State::Deletion => {
                        founds.push(Found {
                            filename: delta.old_file().path().unwrap().clone(),
                            key: format_key(deleted.clone()),
                            state: FoundState::Deleted,
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

    // Grab one.
    match state {
        State::Addition => {
            if added.len() > 0 {
                founds.push(Found {
                    filename: new_path,
                    key: format_key(added.clone()),
                    state: FoundState::Added,
                });
            }
        },
        State::Deletion => {
            if deleted.len() > 0 {
                founds.push(Found {
                    filename: old_path,
                    key: format_key(deleted.clone()),
                    state: FoundState::Deleted,
                });
            }
        },
        _ => (),
    }

    println!("KEYS={:?}", keys);
    println!("FOUNDS={:?}", founds);

    Ok(vec![Output {
	    old_commit: old.id(),
	    new_commit: new.id(),
        old_filename: String::from_str("oldfilename"),
        new_filename: String::from_str("newfilename"),
	    origin_line: 0,
	    destination_line: 0,
	    num_lines: 0
    }])
}

	
#[derive(Debug)]
struct Output {
    old_commit: Oid,
    new_commit: Oid,
    old_filename: String,
    new_filename: String,
    origin_line: u32,
    destination_line: u32,
    num_lines: u32,
}
