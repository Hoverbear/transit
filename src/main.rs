#![feature(core)]
#![feature(path)]

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

fn print_or_none<T: std::fmt::Display> (s: &str, opt: Option<T>) {
    print!("{}=", s);
    match opt {
        Some(value) => println!("{}", value),
        None => println!("NONE"),
    }
}

fn find_moves(repo: &Repository, old: &Commit, new: &Commit) -> Result<Vec<Output>, Error> {

    println!("\nFIND MOVES---------------------");

	let old_tree = try!(old.tree());
	let new_tree = try!(new.tree());
	// Build up a diff of the two trees.
	let diff = try!(Diff::tree_to_tree(repo, Some(&old_tree), Some(&new_tree), None));
	// State. TODO: Don't just use a u64.
	//let moved = 0;

    //let mut results: Vec<Output> = Vec::new();

    let mut has_addition = false;
    let mut has_deletion = false;

	//let mut current_hunk: Vec<String> = Vec::new();
	// Read about this function in http://alexcrichton.com/git2-rs/git2/struct.Diff.html#method.print
	// It's a bit weird, but I think it will provide the necessary information.
	diff.print(DiffFormat::Patch, |delta, maybe_hunk, line| -> bool {
		// Thinking:
		//  * If is not a hunk, keep going.
		//  * If it's a hunk, do regex magic.
		//  * Stick regex output into a hashmap as a hash.
		//  * Later, we will iterate through and output pased on the values.
		// Filter out all the boring context lines.
		// If we're not interested in this line just return since it will iterate to the next.
		if maybe_hunk.is_none() { return true }; // Return early.

        println!("");
        print_or_none("line.old_lineno()", line.old_lineno());
        print_or_none("line.new_lineno()", line.new_lineno());
        println!("line.num_lines()={}",  line.num_lines());
        println!("line.content_offset={}", line.content_offset());
        println!("|line.origin()|=|{}|", line.origin());  // Wrap in |'s to detect space characters.
        print!("line.content()={}", str::from_utf8(line.content()).unwrap());

        println!("delta.nfiles()={}", delta.nfiles());
        println!("delta.status()={:?}", delta.status());    // :? is for debug trait.
        println!("delta.old_file():\n\tid={}\n\tpath_bytes=TODO\n\tpath=TODO\n\tsize={}", delta.old_file().id(), delta.old_file().size());
        println!("delta.new_file():\n\tid={}\n\tpath_bytes=TODO\n\tpath=TODO\n\tsize={}", delta.new_file().id(), delta.new_file().size());

		match line.origin() {
			// Context
			' ' | '=' => {
                print!("= {}", str::from_utf8(line.content()).unwrap());
                true
            },
			// Headers
			'F' | 'H' => {
                print!("F {}", str::from_utf8(line.content()).unwrap());
                true
            },
			// Additions
			'+' | '>' => {
				print!("+ {}", str::from_utf8(line.content()).unwrap());
                has_addition = true;
				true
			},
			// Deletions
			'-' | '<' => {
				print!("- {}", str::from_utf8(line.content()).unwrap());
                has_deletion = true;
				true
			},
			// Other (We don't care about these.)
			_         => {
                print!("_ {}", str::from_utf8(line.content()).unwrap());
                true
            }
		}
			// Look at that match statement, what a hunk. So hunky.
			// match maybe_hunk {
			//     Some(hunk) => unimplemented!(),
			//     None => unimplemented!(),
			// }
	});


    println!("\nhas_addition={}, has_deletion={}", has_addition, has_deletion);

   // if has_addition && has_deletion {
/*
        results.push(Output {
		    old_commit: old.id(),
		    new_commit: new.id(),
		    origin_line: 0,
		    destination_line: 0,
		    num_lines: 0
        });
*/
    //}


    if !has_addition || !has_deletion {
        println!("Does not have addition or deletion.");
        Ok(vec![])  // Return empty vector.
    } else {

	    Ok(vec![Output {
		    old_commit: old.id(),
		    new_commit: new.id(),
		    origin_line: 0,
		    destination_line: 0,
		    num_lines: 0
	    }])
    }

   // Ok(results)
}

	
#[derive(Show)]	
struct Output {
    old_commit: Oid,
    new_commit: Oid,
    origin_line: u32,
    destination_line: u32,
    num_lines: u32,
}
