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
            find_moves(&repo, &old.unwrap(), &new.unwrap()).unwrap();
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
        for pair in history.windows(2) {
            let (old, new) = (&pair[1], &pair[0]);
            find_moves(&repo, old.clone(), new.clone()).unwrap();
        }
    }


fn find_moves(repo: &Repository, old: &Commit, new: &Commit) -> Result<u64, Error> {
        let old_tree = try!(old.tree());
        let new_tree = try!(new.tree());
        // Build up a diff of the two trees.
        let diff = try!(Diff::tree_to_tree(repo, Some(&old_tree), Some(&new_tree), None));
        // State. TODO: Don't just use a u64.
        let moved = 0;
        // Read about this function in http://alexcrichton.com/git2-rs/git2/struct.Diff.html#method.print
        // It's a bit weird, but I think it will provide the necessary information.
        diff.print(DiffFormat::Patch, |delta, maybe_hunk, line| -> bool {
            // Filter out all the boring context lines.
            // If we're not interested in this line just return since it will iterate to the next.
            match line.origin() {
                // Context
                ' ' | '=' => true,
                // Headers
                'F' | 'H' => true,
                // Additions
                '+' | '>' => {
                    println!("+ {}", str::from_utf8(line.content()).unwrap());
                    true
                },
                // Deletions
                '-' | '<' => {
                    println!("- {}", str::from_utf8(line.content()).unwrap());
                    true
                },
                // Other (We don't care about these.)
                _         => true,
            }

            // Look at that match statement, what a hunk. So hunky.
            // match maybe_hunk {
            //     Some(hunk) => unimplemented!(),
            //     None => unimplemented!(),
            // }
        });
        Ok(moved)
    }
}
