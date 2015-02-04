#![feature(core)]
#![feature(path)]
extern crate git2;
extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;

use git2::{Repository, Branch, BranchType, Commit, Diff};
use docopt::Docopt;
use std::old_io as io;
use std::old_io::BufferedReader;
use std::old_io::File;
use std::str;

// Write the Docopt usage string.
static USAGE: &'static str = "
Usage: transit [options] <repo>

Options:
    -f, --flag  Flags a flag, note the multiple spaces!
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_repo: String,
    flag_flag: bool,
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
        println!("\nOLD: {:?} - NEW: {:?}\n", old.id(), new.id());
        let old_tree = old.tree()
            .ok().expect("Could not get tree.");
        let new_tree = new.tree()
            .ok().expect("Could not get tree.");
        // Build up a diff of the two trees.
        let diff = Diff::tree_to_tree(&repo, Some(&old_tree), Some(&new_tree), None)
            .ok().expect("Couldn't diff trees.");

        for delta in diff.deltas() {
            // If we don't get anything the file was probably newly created.
            let old_file = repo.find_blob(delta.old_file().id());
            let old_content = match old_file {
                Ok(ref file) => str::from_utf8(file.content()).unwrap(),
                Err(e)   => "".as_slice(),
            };
            // If we don't get anything the file was probably newly deleted.
            let new_file = repo.find_blob(delta.new_file().id());
            let new_content = match new_file {
                Ok(ref file) => str::from_utf8(file.content()).unwrap(),
                Err(e)   => "".as_slice(),
            };
            for (old, new) in old_content.lines().zip(new_content.lines()) {
                if old != new {
                    println!("--- {}\n+++ {}", old, new);
                } else {
                    println!("{}", new);
                }
                // Note that this zipped iter ends when ONE ends, and won't show all.
            }
        }
    }
}
