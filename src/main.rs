extern crate git2;
extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;

use git2::Repository;
use docopt::Docopt;

// Write the Docopt usage string.
static USAGE: &'static str = "
Usage: transit [options] <repo>

Options:
    -f, --flag  Flags a flag, note the multiple spaces!
";

#[derive(RustcDecodable, Show)]
struct Args {
    arg_repo: String,
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
    let mut branches = repo.branches(None)
        .ok().expect("Could find any branches.");
    for (branch, variant) in branches {
        let name = branch.name()
            .ok().expect("Could not get branch name.")
            .expect("Branch name was not valid UTF-8");
        println!("{:?} is {:?}", name, variant);
    }
}
