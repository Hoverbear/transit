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
    println!("Running repo handler, URL path: {:?}, Query: {:?}", req.url.path, req.url.query);
    // match req.url.query {
    //     3 => {
    //         let path = Path::new(&req.url.path[0]);
    //         let repo = match Repository::discover(&path) {
    //             Ok(repo) => repo,
    //             Err(_) => return Ok(Response::with((status::BadRequest, "Your input falls short of expectations"))),
    //         };
    //
    //         let old = match Oid::from_str(&req.url.path[1][..]) {
    //                Ok(oid) => oid,
    //                Err(_) => return Ok(Response::with((status::BadRequest, "Your input falls short of expectations"))),
    //         };
    //         let new = match Oid::from_str(&req.url.path[2][..]) {
    //                Ok(oid) => oid,
    //                Err(_) => return Ok(Response::with((status::BadRequest, "Your input falls short of expectations"))),
    //         };
    //         let out = match processor::commits(repo, old, new) {
    //             Ok(output) => output,
    //             Err(_) => return Ok(Response::with((status::InternalServerError, "Hold steadfast and report bugs."))),
    //         };
    //
    //         Ok(Response::with((status::Ok, json::encode(&out).unwrap())))
    //     },
    //     1 => {
    //         let path = Path::new(&req.url.path[0]);
    //         let repo = match Repository::discover(&path) {
    //             Ok(repo) => repo,
    //             Err(_) => return Ok(Response::with((status::BadRequest, "Your input falls short of expectations"))),
    //         };
    //
    //         let out = match processor::repo(repo) {
    //             Ok(output) => output,
    //             Err(_) => return Ok(Response::with((status::InternalServerError, "Hold steadfast and report bugs."))),
    //         };
    //         Ok(Response::with((status::Ok, json::encode(&out).unwrap())))
    //     },
    //     _ => Ok(Response::with((status::NotFound, "Your work is nothingness.")))
    // }

}
