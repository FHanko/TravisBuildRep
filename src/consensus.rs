use network::{Server, ServerTimeout};
use mio::{EventLoop, Timeout};
use rand::{self, Rng};
use state::{StateHandler, State};
use util::{LogIndex, ServerId, ClientId, Term};
use capnp::message::{Reader, ReaderSegments};
use messages_capnp::{message, append_entries_request, request_vote_request,
                     append_entries_response, request_vote_response};
use io::Log;

const HEART_BEAT: u64 = 1000;
const MIN_ELECTION: u64 = 1000;
const MAX_ELECTION: u64 = 3000;

#[derive(Clone,Copy)]
pub enum ConsensusTimeout {
    HeartbeatTimeout,
    ElectionTimeout,
}

impl ConsensusTimeout {
    pub fn get_duration(&self) -> u64 {
        match *self {
            ConsensusTimeout::HeartbeatTimeout => HEART_BEAT,
            ConsensusTimeout::ElectionTimeout => {
                rand::thread_rng().gen_range::<u64>(MIN_ELECTION, MAX_ELECTION)
            }
        }
    }
}

#[derive(Clone)]
pub struct Consensus<L: Log + Clone> {
    pub heartbeat_handler: Timeout,
    pub election_handler: Timeout,
    pub state_handler: StateHandler,
    pub log: L,
}

impl<L: Log + Clone> Consensus<L> {
    pub fn election_timeout(&mut self, server: &Server<L>, event_loop: &mut EventLoop<Server<L>>) {
        let handler = server.set_timeout(event_loop,
                         ServerTimeout::ConsensusTimeout(ConsensusTimeout::ElectionTimeout))
            .unwrap();
        self.election_handler = handler;

        self.state_handler.clone().election_timeout();
    }

    pub fn heartbeat_timeout(&mut self,
                             server: &Server<L>,
                             event_loop: &mut EventLoop<Server<L>>) {
        let handler = server.set_timeout(event_loop,
                         ServerTimeout::ConsensusTimeout(ConsensusTimeout::HeartbeatTimeout))
            .unwrap();
        self.heartbeat_handler = handler;

        self.state_handler.clone().heartbeat_timeout();
    }

    pub fn apply_message<S>(&mut self, from: ClientId, message: &Reader<S>)
        where S: ReaderSegments
    {
        let reader = message.get_root::<message::Reader>().unwrap().which().unwrap();

        match reader {
            message::Which::AppendEntriesRequest(Ok(request)) => {
                self.append_entries_request(request);
            }
            message::Which::RequestVoteRequest(Ok(request)) => {
                self.request_vote_request(request);
            }
            message::Which::AppendEntriesResponse(Ok(request)) => {
                self.append_entries_response(request);
            }
            message::Which::RequestVoteResponse(Ok(request)) => {
                self.request_vote_response(request);
            }
            _ => panic!("Do not panic, but I do not know what kind of message I got"),
        }
    }

    // TODO test
    fn append_entries_request(&mut self, request: append_entries_request::Reader) {

        let leader_term = Term(request.get_term());
        let my_term = self.state_handler.current_term;

        if leader_term.as_u64() < my_term.as_u64() {
            // TODO add response to leader and delete panic
            panic!("Leader has not a higher term");
        }

        match self.state_handler.state {
            State::Follower => {
                if my_term.as_u64() < leader_term.as_u64() {
                    self.state_handler.set_term(leader_term.as_u64());
                }

                let leader_prev_log_index = LogIndex(request.get_prev_log_index());
                let leader_prev_log_term = Term(request.get_prev_log_term());

                let my_prev_log_index = self.state_handler.commit_index;

                if my_prev_log_index.as_u64() < leader_prev_log_index.as_u64() {
                    // TODO reply that logs are inconsistent
                    panic!("logs inconsistent; different log index");
                }

                let term = if leader_prev_log_index == LogIndex(0) {
                    Term(0)
                } else {
                    self.log.read(leader_prev_log_index.as_u64()).0
                };

                if leader_prev_log_term.as_u64() != term.as_u64() {
                    // TODO reply that logs are inconsistent
                    panic!("logs inconsistent; different terms");
                } else {
                    // TODO append entries
                    unimplemented!()
                }

            }
            State::Candidate => {
                self.transition_to_follower();
                return self.append_entries_request(request);
            }
            State::Leader => {
                if leader_term.as_u64() == my_term.as_u64() {
                    panic!("Panic! This term has two leaders");
                } else if my_term.as_u64() < leader_term.as_u64() {
                    self.transition_to_follower();
                }
                return self.append_entries_request(request);
            }
        }
    }

    fn transition_to_follower(&mut self) {
        debug!("Transition to follower");
        unimplemented!()
    }

    fn request_vote_request(&mut self, request: request_vote_request::Reader) {
        unimplemented!()
    }

    fn append_entries_response(&mut self, request: append_entries_response::Reader) {
        unimplemented!()
    }

    fn request_vote_response(&mut self, request: request_vote_response::Reader) {
        unimplemented!()
    }
}
