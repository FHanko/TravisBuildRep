use util::ServerId;
use std::collections::HashSet;

#[derive(Clone)]
enum State {
    Follower,
    Leader,
    Candidate,
}

#[derive(Clone)]
pub struct StateMachine {
    server_id: ServerId,
    current_term: u64,
    commit_index: u64,
    last_applied: u64,
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
            current_term: 0,
            commit_index: 0,
            last_applied: 0,
            leader_id: None,
            state: State::Follower,
            follower_state: Some(FollowerState::new()),
            leader_state: None,
            candidate_state: None,
        }
    }

    pub fn set_term(&mut self, i: u64) {
        self.current_term = i;
    }

    pub fn set_leader(&mut self, leader_id: ServerId) {
        self.leader_id = Some(leader_id);
    }

    pub fn commit(&mut self) {
        self.commit_index += 1;
    }

    pub fn apply(&mut self) {
        match self.state {
            State::Follower => {
                match self.follower_state.clone() {
                    Some(mut follower_state) => {
                        follower_state.apply();
                    }
                    None => panic!("follower_state is None"),
                }
            }
            State::Candidate => {
                match self.candidate_state.clone() {
                    Some(mut candidate_state) => {
                        candidate_state.apply();
                    }
                    None => panic!("candidate_state is None"),
                }
            }
            State::Leader => {
                match self.leader_state.clone() {
                    Some(mut leader_state) => {
                        leader_state.apply(self);
                    }
                    None => panic!("leader_state is None"),
                }
            }
        }
    }

    pub fn timeout(self) {
        match self.state {
            State::Follower => {
                match self.follower_state.clone() {
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
                match self.leader_state.clone() {
                    Some(leader_state) => {
                        leader_state.timeout();
                    }
                    None => panic!("leader_state is None"),
                }
            }
        }
    }
}

#[derive(Clone)]
struct FollowerState {
    voted_for: Option<ServerId>,
}

impl FollowerState {
    fn new() -> Self {
        FollowerState { voted_for: None }
    }

    fn apply(&mut self) {
        unimplemented!()
    }

    fn timeout(self) {
        unimplemented!()
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

    fn apply(&mut self) {
        unimplemented!()
    }

    fn timeout(self) {
        unimplemented!()
    }

    fn got_voted(&mut self, v: ServerId) {
        self.votes.insert(v);
    }
}

#[derive(Clone)]
struct LeaderState;

impl LeaderState {
    fn new() -> Self {
        LeaderState
    }

    fn apply(&mut self, s: &StateMachine) {
        match s.leader_id {
            Some(leader_id) => {
                assert_eq!(leader_id, s.server_id);

                unimplemented!()
            }
            None => {
                panic!("Something didn't worked out! You cannot apply a message when no leader is \
                        chosen")
            }
        }
    }

    fn timeout(self) {
        unimplemented!()
    }
}
