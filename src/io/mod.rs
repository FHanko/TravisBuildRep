use util::{ServerId, Term, LogIndex};

pub trait Log: Clone + 'static {
    fn vote_for(&mut self, id: ServerId);
    fn get_voted_for(&self) -> Option<ServerId>;
    fn reset_vote(&mut self);

    fn set_term(&mut self, t: Term);
    fn get_term(&self) -> Option<Term>;

    fn append_entries(&mut self, m: Vec<(Term, &[u8])>);
    fn read(&self, index: u64) -> (Term, &Vec<u8>);

    // Delete entry at i
    fn truncate(&mut self, i: u16);

    fn get_latest_log_index(&self) -> LogIndex;
    fn get_latest_log_term(&self) -> Term;

    fn len(&self) -> usize;
}

// TODO seperate to an extra file
#[derive(Debug,Default,Clone)]
pub struct VLog {
    currentTerm: Term,
    votedFor: Option<ServerId>,
    log: Vec<(Term, Vec<u8>)>,
}

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
    fn vote_for(&mut self, id: ServerId) {
        self.votedFor = Some(id);
    }

    fn get_voted_for(&self) -> Option<ServerId> {
        self.votedFor
    }

    fn reset_vote(&mut self) {
        self.votedFor = None;
    }

    fn set_term(&mut self, t: Term) {
        self.currentTerm = t;
    }

    fn get_term(&self) -> Option<Term> {
        Some(self.currentTerm)
    }

    fn append_entries(&mut self, m: Vec<(Term, &[u8])>) {
        // m.iter().map(|entry| self.log.push(entry));
        self.log.extend(m.iter().map(|&(term, command)| (term, command.to_vec())));
    }

    fn read(&self, index: u64) -> (Term, &Vec<u8>) {
        let (term, ref bytes) = self.log[(index - 1) as usize];
        (term, bytes)
    }

    fn truncate(&mut self, i: u16) {
        self.log.truncate(i as usize);
    }

    fn get_latest_log_term(&self) -> Term {
        let length = self.log.len() as usize;

        if (length != 0) {
            self.log[length - 1].0
        } else {
            Term(0)
        }
    }

    fn get_latest_log_index(&self) -> LogIndex {
        LogIndex(self.log.len() as u64)
    }

    fn len(&self) -> usize {
        self.log.len()
    }
}
