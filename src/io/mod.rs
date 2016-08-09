use util::{ServerId, Term};

pub type Message = [u8];

pub trait Log {
    fn vote_for(&self, id: ServerId);
    fn get_voted_for(&self) -> Option<ServerId>;

    fn set_term(&self, t: Term);
    fn get_term(&self) -> Option<Term>;

    fn get_commitIndex(&self) -> u64;
    fn set_commitIndex(&self, index: u64);

    fn write(&self, m: &Message);
    fn read(&self) -> &Message;

    // Delete all entries starting from i
    fn truncate(&self, i: u16);
}
