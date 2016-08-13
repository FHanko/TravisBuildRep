use util::{ServerId, Term};

pub type Message = [u8];

pub trait Log {
    fn vote_for(&mut self, id: ServerId);
    fn get_voted_for(&self) -> Option<ServerId>;

    fn set_term(&mut self, t: Term);
    fn get_term(&self) -> Option<Term>;

    fn get_commitIndex(&self) -> u64;
    fn set_commitIndex(&mut self, index: u64);

    fn write(&self, m: &Message);
    fn read(&self) -> &Message;

    // Delete all entries starting from i
    fn truncate(&mut self, i: u16);
}

pub struct VLog
{
    currentTerm: Term,
    votedFor: Option<ServerId>,
    commitIndex: u64,
    log: Vec<(Term, Vec<u8>)>,
}

impl Default for VLog { fn default() -> Self { Self::new() } }

impl VLog {
    pub fn new() -> VLog {
        VLog {
            currentTerm: Term::from(0),
            votedFor: None,
            log: Vec::new(),
        }
    }
}

impl Log for VLog {
    fn vote_for(&mut self, id: ServerId)
    {
        self.votedFor = Some(id);
    }
    fn get_voted_for(&self) -> Option<ServerId>
    {
        self.votedFor
    }

    fn set_term(&mut self, t: Term)
    {
        self.currentTerm = t;
    }
    fn get_term(&self) -> Option<Term>
    {
        Some(self.currentTerm)
    }

    fn get_commitIndex(&self) -> u64
    {
        self.commitIndex
    }
    fn set_commitIndex(&mut self, index: u64)
    {
        self.commitIndex = index;
    }

    fn write(&self, m: &Message)
    {
        unimplemented!();
    }
    fn read(&self) -> &Message
    {
        unimplemented!();
    }

    // Delete all entries starting from i
    fn truncate(&mut self, i: u16)
    {
        self.log.truncate(i as usize);
    }
}