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
        .ok().expect("Could not discover a repository.");
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
    for pair in history.windows(2) {
        let (prev, next) = (&pair[0], &pair[1]);
        println!("\n{:?} - {:?}\n", prev.id(), next.id());
        let prev_tree = prev.tree()
            .ok().expect("Couldn't get prev tree");
        let next_tree = next.tree()
            .ok().expect("Couldn't get next tree.");
        let diff = Diff::tree_to_tree(&repo, Some(&prev_tree), Some(&next_tree), None)
            .ok().expect("Couldn't diff trees.");
        for delta in diff.deltas() {
            let old_file = repo.find_blob(delta.old_file().id())
                .ok().expect("Couldn't get blob");
            let new_file = repo.find_blob(delta.new_file().id())
                .ok().expect("Couldn't get blob");
            let old_content = str::from_utf8(&mut old_file.content())
                .ok().expect("Couldn't get content");
            let new_content = str::from_utf8(&mut new_file.content())
                .ok().expect("Couldn't get content");
            for (old, new) in old_content.lines().zip(new_content.lines()) {
                if (old != new) {
                    println!("{:?} - {:?}", old, new);
                } else { println!("Same"); }
            }
        }
    }
}
