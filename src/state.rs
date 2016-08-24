use util::{ServerId, Term, LogIndex};
use std::collections::{HashSet, HashMap};
use std::net::SocketAddr;

#[derive(Clone, Copy,Eq,PartialEq,Debug)]
pub enum State {
    Follower,
    Leader,
    Candidate,
}

#[derive(Clone)]
pub struct StateHandler {
    server_id: ServerId,
    pub current_term: Term,
    pub commit_index: LogIndex,
    last_applied: LogIndex,
    pub leader_id: Option<ServerId>,
    pub state: State,

    // states
    follower_state: Option<FollowerState>,
    leader_state: Option<LeaderState>,
    candidate_state: Option<CandidateState>,
}

impl StateHandler {
    pub fn new(server_id: ServerId) -> Self {
        StateHandler {
            server_id: server_id,
            current_term: Term(0),
            commit_index: LogIndex(0),
            last_applied: LogIndex(0),
            leader_id: None,
            state: State::Follower,
            follower_state: Some(FollowerState::new()),
            leader_state: None,
            candidate_state: None,
        }
    }

    pub fn set_term(&mut self, i: u64) {
        self.current_term = Term(i);
    }

    pub fn next_term(&mut self) {
        let old = self.current_term.as_u64();

        self.current_term = Term(old + 1);
    }

    pub fn set_leader(&mut self, leader_id: ServerId) {
        self.leader_id = Some(leader_id);
    }

    pub fn commit(&mut self) {
        // self.commit_index += 1;
        unimplemented!()
    }

    pub fn heartbeat_timeout(self) {
        debug!("Heartbeat timeout");
    }

    pub fn transition_to_follower(&mut self, leader_id: ServerId) {
        self.state = State::Follower;
        self.set_leader(leader_id);
        self.follower_state = Some(FollowerState::new());
    }

    pub fn transition_to_candidate(&mut self) {
        self.next_term();
        self.state = State::Candidate;
        self.candidate_state = Some(CandidateState::new(self.server_id));
    }

    pub fn transition_to_leader(&mut self,
                                peers: &HashMap<ServerId, SocketAddr>,
                                log_length: u64) {
        assert_eq!(self.state, State::Candidate);
        self.leader_id = Some(self.server_id);
        self.state = State::Leader;
        self.leader_state = Some(LeaderState::new(peers, log_length));
    }
}

#[derive(Clone, Copy)]
struct FollowerState {
    voted_for: Option<ServerId>,
}

impl FollowerState {
    fn new() -> Self {
        FollowerState { voted_for: None }
    }

    fn vote_for(&mut self, v: ServerId) {
        self.voted_for = Some(v);
    }
}

#[derive(Clone)]
struct CandidateState {
    voted_for: ServerId, // vote for self
    votes: HashSet<ServerId>,
}

impl CandidateState {
    fn new(s: ServerId) -> Self {
        let mut c = CandidateState {
            voted_for: s,
            votes: HashSet::new(),
        };

        c.vote(s);

        c
    }

    fn vote(&mut self, v: ServerId) {
        self.votes.insert(v);
    }
}

#[derive(Clone)]
struct LeaderState {
    next_index: HashMap<ServerId, LogIndex>,
    match_index: HashMap<ServerId, LogIndex>,
}

impl LeaderState {
    fn new(peers: &HashMap<ServerId, SocketAddr>, log_length: u64) -> Self {
        let mut next_index: HashMap<ServerId, LogIndex> = HashMap::new();
        let mut match_index: HashMap<ServerId, LogIndex> = HashMap::new();

        peers.keys().map(|peer| {
            next_index.insert(*peer, LogIndex(log_length + 1));
            match_index.insert(*peer, LogIndex(0));
        });

        LeaderState {
            next_index: next_index,
            match_index: match_index,
        }
    }
}
