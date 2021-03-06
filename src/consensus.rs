use network::{Server, ServerTimeout};
use mio::{EventLoop, Timeout};
use rand::{self, Rng};
use state::StateMachine;

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
pub struct Consensus {
    pub heartbeat_handler: Timeout,
    pub election_handler: Timeout,
    pub state_machine: StateMachine,
}

impl Consensus {
    pub fn election_timeout(&mut self, server: &Server, event_loop: &mut EventLoop<Server>) {
        let handler = server.set_timeout(event_loop,
                         ServerTimeout::ConsensusTimeout(ConsensusTimeout::ElectionTimeout))
            .unwrap();
        self.election_handler = handler;

        self.state_machine.clone().election_timeout();
    }

    pub fn heartbeat_timeout(&mut self, server: &Server, event_loop: &mut EventLoop<Server>) {
        let handler = server.set_timeout(event_loop,
                         ServerTimeout::ConsensusTimeout(ConsensusTimeout::HeartbeatTimeout))
            .unwrap();
        self.heartbeat_handler = handler;

        self.state_machine.clone().heartbeat_timeout();
    }
}
