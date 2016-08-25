use network::{Server, ServerTimeout, Connection};
use mio::{EventLoop, Timeout, Token};
use mio::util::Slab;
use rand::{self, Rng};
use state::{StateHandler, State};
use util::{LogIndex, ServerId, ClientId, Term};
use capnp::message::{Reader, ReaderSegments};
use messages_capnp::{message, append_entries_request, request_vote_request,
                     append_entries_response, request_vote_response};
use io::Log;
use messages::rpc::request_vote_request;
use std::collections::HashMap;
use std::net::SocketAddr;

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
    pub fn election_timeout(&mut self,
                            server: &mut Server<L>,
                            event_loop: &mut EventLoop<Server<L>>) {
        let handler = server.set_timeout(event_loop,
                         ServerTimeout::ConsensusTimeout(ConsensusTimeout::ElectionTimeout))
            .unwrap();
        self.election_handler = handler;

        self.transition_to_candidate(&mut server.connections);
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

    pub fn apply_message<S>(&mut self, from: ServerId, message: &Reader<S>)
        where S: ReaderSegments
    {
        let reader = message.get_root::<message::Reader>().unwrap().which().unwrap();

        match reader {
            message::Which::AppendEntriesRequest(Ok(request)) => {
                self.append_entries_request(from, request);
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

    fn append_entries_request(&mut self, from: ServerId, request: append_entries_request::Reader) {

        let leader_term = Term(request.get_term());
        let my_term = self.state_handler.current_term;

        if leader_term.as_u64() < my_term.as_u64() {
            let message = messages::append_entries_response_stale_term(my_term);
            actions.peer_messages.push((from, message));
            return;
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
                    let message = messages::append_entries_response_inconsistent_prev_entry(my_term, leader_prev_log_index)
                    actions.peer_messages.push((from, message));
                    return;
                }

                let term = if leader_prev_log_index == LogIndex(0) {
                    Term(0)
                } else {
                    self.log.read(leader_prev_log_index.as_u64()).0
                };

                if leader_prev_log_term.as_u64() != term.as_u64() {
                    let message = messages::append_entries_response_inconsistent_prev_entry(my_term, leader_prev_log_index)
                    actions.peer_messages.push((from, message));
                    return;
                } else {
                    if let Ok(entries) = request.get_entries() {
                        let entries_vec: Vec<(Term, &[u8])> = entries.iter()
                            .map(|entry| {
                                (Term::from(entry.get_term()), entry.get_data().unwrap_or(b""))
                            })
                            .collect();

                        self.log.append_entries(entries_vec);

                        let message = messages::append_entries_response_success(my_term,
                                                                      self.log
                                                                          .latest_log_index()
                                                                          .unwrap());
                        actions.peer_messages.push((from, message));
                    } else {
                        // TODO allow empty append_entries_request; (heartbeats)
                        panic!("no entries in append_entries_request");
                    }
                }
            }
            State::Candidate => {
                self.transition_to_follower(from);
                return self.append_entries_request(from, request);
            }
            State::Leader => {
                if leader_term.as_u64() == my_term.as_u64() {
                    panic!("Panic! This term has two leaders");
                } else if my_term.as_u64() < leader_term.as_u64() {
                    self.transition_to_follower(from);
                }
                return self.append_entries_request(from, request);
            }
        }
    }

    pub fn transition_to_follower(&mut self, leader_id: ServerId) {
        debug!("Transition to follower");
        self.log.reset_vote();
        self.state_handler.transition_to_follower(leader_id);
    }

    fn transition_to_candidate(&mut self, peers: &mut Slab<Connection>) {
        self.state_handler.transition_to_candidate();
        let message = request_vote_request(self.state_handler.current_term,
                                           self.latest_log_index(),
                                           self.latest_log_term());

        for peer in peers.iter_mut() {
            peer.write(message.clone());
        }
    }

    fn transition_to_leader(&mut self, peers: &HashMap<ServerId, SocketAddr>) {
        self.state_handler.transition_to_leader(peers, self.log.len() as u64);
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

    fn latest_log_term(&self) -> Term {
        self.log.get_latest_log_term()
    }

    fn latest_log_index(&self) -> LogIndex {
        self.log.get_latest_log_index()
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;
    use std::io::Cursor;
    use std::collections::HashMap;
    use mio::EventLoop;
    use capnp::message::{Allocator, Builder, HeapAllocator, ReaderOptions, Reader, ReaderSegments};

    use util::*;
    use network::Server;
    use io::{Log, VLog};
    use state::State;
    use capnp::serialize::{self, OwnedSegments};
    use messages::rpc::append_entries_request;
    use messages_capnp::message;

    fn new_server() -> (Server<VLog>, EventLoop<Server<VLog>>) {
        let local_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let log = VLog::new();

        Server::new(local_addr, HashMap::new(), log)
    }

    fn into_reader<A>(message: &Builder<A>) -> Reader<OwnedSegments>
        where A: Allocator
    {
        let mut buf = Cursor::new(Vec::new());

        serialize::write_message(&mut buf, message).unwrap();
        buf.set_position(0);
        serialize::read_message(&mut buf, ReaderOptions::new()).unwrap()
    }

    #[test]
    fn test_append_entries_follower() {
        let (myServer, event_loop) = new_server();
        let mut entries: Vec<(Term, &[u8])> = Vec::new();
        entries.push((Term(0), b"my custom command"));

        let mut consensus = myServer.consensus.unwrap().clone();

        let message = append_entries_request(Term(0), LogIndex(0), Term(0), &entries, LogIndex(0));

        let req = into_reader(&*message);

        consensus.apply_message(ServerId(1), &req);

        assert_eq!(consensus.state_handler.state, State::Follower);
        assert_eq!(consensus.state_handler.current_term, Term(0));
        assert_eq!(consensus.log.len(), 1);
    }

    #[test]
    fn test_transition_to_follower() {
        let (myServer, event_loop) = new_server();

        let mut consensus = myServer.consensus.unwrap().clone();

        consensus.transition_to_follower(ServerId(1000));

        assert_eq!(consensus.state_handler.state, State::Follower);
        assert!(consensus.state_handler.leader_id != Some(ServerId(0)));
    }

    #[test]
    fn test_transition_to_candidate() {
        let (mut myServer, event_loop) = new_server();

        let mut consensus = myServer.consensus.unwrap().clone();

        consensus.transition_to_candidate(&mut myServer.connections);

        assert_eq!(consensus.state_handler.state, State::Candidate);
        assert_eq!(consensus.state_handler.current_term, Term(1));
    }

    #[test]
    fn test_transition_to_leader() {
        let (myServer, event_loop) = new_server();

        let mut consensus = myServer.consensus.unwrap().clone();

        consensus.state_handler.transition_to_candidate();

        consensus.transition_to_leader(&myServer.peers);

        assert_eq!(consensus.state_handler.state, State::Leader);
        assert_eq!(consensus.state_handler.leader_id, Some(ServerId(0)));
    }
}
