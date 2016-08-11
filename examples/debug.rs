extern crate PaenkoDb;
extern crate env_logger;

use std::collections::HashMap;
use PaenkoDb::network::*;
use PaenkoDb::util::*;
use std::net::*;

fn main() {
    env_logger::init().unwrap();
    let mut map: HashMap<ServerId, SocketAddr> = HashMap::new();
    // map.insert(ServerId(1), "192.168.1.18:1337".parse().unwrap());
    let (myServer, event_loop) = Server::new("127.0.0.1:1337".parse().unwrap(), map);
}
