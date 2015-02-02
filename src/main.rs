#![feature(io)]
#![feature(core)]
#![feature(path)]
extern crate git2;
extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;

use git2::{Repository, Branch, BranchType};
use docopt::Docopt;
use std::old_io as io;

// Write the Docopt usage string.
static USAGE: &'static str = "
Usage: transit [options] <repo> [<branch>]

Options:
    -f, --flag  Flags a flag, note the multiple spaces!
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_repo: String,
    arg_branch: Option<String>,
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
    for id in revwalk {
        match repo.find_commit(id) {
            Ok(commit) =>
                println!("Commit: {:?} - {:?}", commit.id(), commit.message()
                    .expect("Commit message was not UTF-8.")),
            Err(error) => (),
        }
    }
}
