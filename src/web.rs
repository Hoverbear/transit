extern crate iron;
extern crate mount;

use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::path::Path;

use iron::{Iron, Request, Response, IronResult};
use iron::status;
use mount::Mount;

use git2::{Repository, Oid};
use rustc_serialize::json;

use processor;

pub fn start(port: u16) {
    let addr = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), port);
    let mut mount = Mount::new();

    mount.mount("/", index);
    mount.mount("/api", api);

    Iron::new(mount).http(addr).unwrap();
}

fn index(req: &mut Request) -> IronResult<Response> {
    println!("Running index handler, URL path: {:?}", req.url.path);
    Ok(Response::with((status::Ok, "Index!!")))
}

fn api(req: &mut Request) -> IronResult<Response> {
    let query_pairs = req.url.clone().into_generic_url().query_pairs().unwrap_or(vec![]);
    let mut repo = None;
    let mut old = None;
    let mut new = None;
    for (key, val) in query_pairs {
        match &key[..] {
            "repo" => repo = match Repository::discover(&val) {
                Ok(repo) => Some(repo),
                Err(_) => return Ok(Response::with((status::BadRequest, "Repository Invalid."))),
            },
            "old" => old = match Oid::from_str(&val) {
                Ok(commit) => Some(commit),
                Err(_) => return Ok(Response::with((status::BadRequest, "Old Commit Invalid."))),
            },
            "new" => new = match Oid::from_str(&val) {
                Ok(commit) => Some(commit),
                Err(_) => return Ok(Response::with((status::BadRequest, "New Commit Invalid"))),
            },
            _ => return Ok(Response::with((status::BadRequest, "Your input falls short of expectations"))),
        }
    }
    match (repo, old, new) {
        (Some(repo), Some(old), Some(new)) => {
            let out = match processor::commits(repo, old, new) {
                Ok(output) => output,
                Err(_) => return Ok(Response::with((status::InternalServerError, "Hold steadfast and report bugs."))),
            };
            Ok(Response::with((status::Ok, json::encode(&out).unwrap())))
        },
        (Some(repo), None, None) => {
            let out = match processor::repo(repo) {
                Ok(output) => output,
                Err(_) => return Ok(Response::with((status::InternalServerError, "Hold steadfast and report bugs."))),
            };
            Ok(Response::with((status::Ok, json::encode(&out).unwrap())))
        },
        _ => Ok(Response::with((status::BadRequest, "Your input falls short of expectations")))
    }
}
