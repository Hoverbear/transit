use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::path::Path;

use iron::{Iron, Handler, Request, Response, IronResult, status};
// use staticfile::Static;

pub fn start(port: u16) {
    let addr = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), port);

    // Fire up server.
    Iron::new(|_: &mut Request| {
        Ok(Response::with((status::Ok, "Hello world!")))
    }).http(addr).unwrap();
}
