use util::{ServerId, Term, LogIndex};
use std::collections::HashSet;

#[derive(Clone, Copy)]
enum State {
    Follower,
    Leader,
    Candidate,
}

#[derive(Clone)]
pub struct StateMachine {
    server_id: ServerId,
    current_term: Term,
    commit_index: LogIndex,
    last_applied: LogIndex,
    leader_id: Option<ServerId>,
    state: State,

    // states
    follower_state: Option<FollowerState>,
    leader_state: Option<LeaderState>,
    candidate_state: Option<CandidateState>,
}

impl StateMachine {
    pub fn new(server_id: ServerId) -> Self {
        StateMachine {
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

    // TODO
    pub fn set_term(&mut self, i: u64) {
        // self.current_term = i;
        unimplemented!()
    }

    pub fn set_leader(&mut self, leader_id: ServerId) {
        self.leader_id = Some(leader_id);
    }

    pub fn commit(&mut self) {
        // self.commit_index += 1;
        unimplemented!()
    }

    pub fn append_entry(&mut self) {
        match self.state {
            State::Follower => {
                match self.follower_state {
                    Some(mut follower_state) => {
                        follower_state.append_entry();
                    }
                    None => panic!("follower_state is None"),
                }
            }
            State::Candidate => {
                match self.candidate_state.clone() {
                    Some(mut candidate_state) => {
                        candidate_state.append_entry();
                    }
                    None => panic!("candidate_state is None"),
                }
            }
            State::Leader => {
                match self.leader_state {
                    Some(mut leader_state) => {
                        leader_state.append_entry(self);
                    }
                    None => panic!("leader_state is None"),
                }
            }
        }
    }

    pub fn election_timeout(self) {
        match self.state {
            State::Follower => {
                match self.follower_state {
                    Some(follower_state) => {

                        // Transition to candidate


                        follower_state.timeout();
                    }
                    None => panic!("follower_state is None"),
                }
            }
            State::Candidate => {
                match self.candidate_state.clone() {
                    Some(candidate_state) => {
                        candidate_state.timeout();
                    }
                    None => panic!("candidate_state is None"),
                }
            }
            State::Leader => {
                match self.leader_state {
                    Some(leader_state) => {
                        leader_state.timeout();
                    }
                    None => panic!("leader_state is None"),
                }
            }
        }
    }

    pub fn heartbeat_timeout(self) {
        debug!("Heartbeat timeout");
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

    fn append_entry(&mut self) {
        unimplemented!()
    }

    fn timeout(self) {
        debug!("Follower timeout")
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

        c.got_voted(s);

        c
    }

    fn append_entry(&mut self) {
        unimplemented!()
    }

    fn timeout(self) {
        debug!("Candidate timeout")
    }

    fn got_voted(&mut self, v: ServerId) {
        self.votes.insert(v);
    }
}

#[derive(Clone,Copy)]
struct LeaderState;

impl LeaderState {
    fn new() -> Self {
        LeaderState
    }

    fn append_entry(&mut self, s: &StateMachine) {
        match s.leader_id {
            Some(leader_id) => {
                assert_eq!(leader_id, s.server_id);

                unimplemented!()
            }
            None => panic!("Something didn't work! There is no leader!"),
        }
    }

    fn timeout(self) {
        debug!("Leader timeout")
    }
}
