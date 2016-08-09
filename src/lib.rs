#![feature(plugin)]
#![plugin(clippy)]

extern crate mio;
extern crate capnp;
extern crate capnp_nonblock;

pub mod network;
pub mod messages;
pub mod util;
pub mod io;

pub mod messages_capnp {
    include!("messages/messages_capnp.rs");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
