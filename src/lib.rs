#![feature(plugin)]
#![plugin(clippy)]
//Perm allow (don't care)
#![allow(non_snake_case)]
#![allow(enum_variant_names)]

//Dev allow (dont't care at the moment)
#![allow(unused_variables)]
#![allow(single_match)]
#![allow(dead_code)]
#![allow(clone_on_copy)]

extern crate mio;
extern crate capnp;
extern crate capnp_nonblock;
extern crate uuid;
extern crate rand;

pub mod network;
pub mod messages;
pub mod util;
pub mod io;
pub mod state;
pub mod consensus;

pub mod messages_capnp {
    include!("messages/messages_capnp.rs");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
