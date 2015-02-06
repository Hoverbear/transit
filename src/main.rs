#![feature(core)]
#![feature(path)]

extern crate git2;
extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;

use git2::{Repository, Branch, BranchType, DiffLine,
    Commit, Diff, DiffFormat, DiffDelta, DiffHunk};
use docopt::Docopt;
use std::old_io as io;
use std::old_io::BufferedReader;
use std::old_io::File;
use std::str;

// Write the Docopt usage string.
static USAGE: &'static str = "
Usage: transit <repo>
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_repo: String,
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
        diff.print(DiffFormat::Patch, handler);
    }
}

// Read about this function in http://alexcrichton.com/git2-rs/git2/struct.Diff.html#method.print
// It's a bit weird, but I think it will provide the necessary information.
fn handler(delta: DiffDelta, hunk: Option<DiffHunk>, line: DiffLine) -> bool {
    println!("----\n{}----\n", str::from_utf8(line.content()).unwrap());
    false
}
