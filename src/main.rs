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

    let path = Path::new(args.arg_repo);
    let repo = Repository::discover(&path)
        .ok().expect("Could not discover a repository.");

    // Which branch?
    let branch = match args.arg_branch {
        Some(branch) => repo.find_branch(&branch[], BranchType::Local),
        None => choose_branch(&repo),
    }.ok().expect("Could not get branch.");

    // Mostly debugging.
    println!("Repo: {:?}, Branch: {:?}", path, branch.name()
       .ok().expect("Couldn't get branch name.")
       .expect("Branch name is not UTF-8."));

    // Get the reference of the branch.
    let reference = branch.into_reference();
    let oid = reference.target()
        .expect("Could not get target.");
    println!("Target Commit: {:?}", oid);

    let top_commit = repo.find_commit(oid)
        .ok().expect("Could not get commit.");

    // There might be more than one parent of a commit... Hrm...
    for commit in top_commit.parents() {
        println!("Parent: {:?}, {:?}", commit.id(), commit.message()
            .expect("Could not get commit message"));
    }
}

fn choose_branch(repo: &Repository) -> Result<Branch, git2::Error> {
    println!("Please choose a branch:");
    let mut branches = repo.branches(Some(BranchType::Local))
        .ok().expect("Could find any branches.");
    for (branch, _) in branches {
        let name = branch.name()
            .ok().expect("Could not get branch name.")
            .expect("Branch name was not valid UTF-8");
        println!("  {}", name);
    }
    io::stdout().write_str(&"Choose your branch: "[]);
    let chosen = io::stdin().lock().read_line()
        .ok().expect("Could not read from stdin");
    repo.find_branch(&chosen.trim()[], BranchType::Local)
}
