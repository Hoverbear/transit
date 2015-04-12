#![feature(core)]
#![feature(collections)]
#![feature(ip_addr)]

#![feature(plugin)]
#![plugin(regex_macros)] extern crate regex;
extern crate git2;
extern crate core;
extern crate rustc_serialize;
extern crate docopt;
extern crate iron;
extern crate mount;

use git2::{Repository, Oid};
use docopt::Docopt;
use rustc_serialize::json;
use std::path::Path;

mod processor;
mod web;

// Write the Docopt usage string.
static USAGE: &'static str = "
Usage: transit [--web=<port> | <repo> [<old> <new>] | --help]

Examples:
  transit --web=$PORT       Spawn a web service.
  transit $REPO             Output the results of a revwalk through a repo.
  transit $REPO $ID1 $ID2   Output the data for a pair of commits.
  transit --help            Display this message.


Output is in JSON.
";

#[derive(RustcDecodable, Debug)]
struct Args {
    flag_web: Option<u16>,
    arg_repo: Option<String>,
    arg_old: Option<String>,
    arg_new: Option<String>,
}

fn main() {
    // Parse the args above or die.
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    if let Some(port) = args.flag_web {
        web::start(port);
    } else if let Some(path_string) = args.arg_repo {
        // Validate values.
        let path = Path::new(&path_string);
        let repo = Repository::discover(&path)
            .ok().expect("Unable to find repo.");
        let old = args.arg_old
            .and_then(|string| Oid::from_str(&string[..]).ok());
        let new = args.arg_new
            .and_then(|string| Oid::from_str(&string[..]).ok());
        // Dispatch.
        if let (Some(old_id), Some(new_id)) = (old, new) {
            let output = processor::process_commits(repo, old_id, new_id).unwrap();
            println!("{}", json::as_pretty_json(&output).indent(4));
        } else {
            let output = processor::process_repo(repo).unwrap();
            println!("{}", json::as_pretty_json(&output).indent(4));
        };

    } else {
        unreachable!();
    }
}
