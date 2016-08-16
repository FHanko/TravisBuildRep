use mio::{EventLoop, Handler, EventSet, PollOpt, Token, Timeout};
use mio::tcp::{TcpListener, TcpStream};
use mio::util::Slab;
use std::net::SocketAddr;
use capnp::message::{Builder, HeapAllocator, ReaderOptions, Reader};
use capnp_nonblock::{MessageStream, Segments};
use capnp::Result;
use std::rc::Rc;
use consensus::{Consensus, ConsensusTimeout};
use state::StateMachine;
use util::ServerId;
use messages::rpc::server_connection_preamble;
use std::collections::HashMap;
use io::{Log, VLog};

pub struct Server<L: Log + Clone> {
    pub id: ServerId,
    pub server: TcpListener,
    pub connections: Slab<Connection>,
    addr: SocketAddr,
    consensus: Option<Consensus<L>>,
}

#[derive(Copy,Clone)]
pub enum ServerTimeout {
    NetworkTimeout,
    ConsensusTimeout(ConsensusTimeout),
}

const SERVER: Token = Token(0);

impl<L: Log + Clone> Server<L> {
    pub fn new(localAddr: SocketAddr,
               peers: HashMap<ServerId, SocketAddr>,
               log: L)
               -> (Server<L>, EventLoop<Server<L>>) {

        debug!("Start program");

        let mut event_loop = EventLoop::new().unwrap();

        let server = TcpListener::bind(&localAddr).unwrap();

        event_loop.register(&server, SERVER, EventSet::readable(), PollOpt::edge()).unwrap();


        let mut myServer = Server {
            id: ServerId(0),
            server: server,
            connections: Slab::new_starting_at(Token(1), 257),
            addr: localAddr,
            consensus: None,
        };

        let consensus = myServer.init_consensus(&mut event_loop, log);
        myServer.consensus = Some(consensus);

        // TODO better error handling
        // TODO code refactoring
        for (peer_id, peer_addr) in peers {
            match TcpStream::connect(&peer_addr) {
                Ok(stream) => {
                    let token = myServer.connections
                        .insert_with(|token| Connection::new_peer(token, stream))
                        .unwrap();

                    match event_loop.register(myServer.connections[token].socket.inner(),
                                              token,
                                              EventSet::readable(),
                                              PollOpt::edge() | PollOpt::oneshot()) {
                        Ok(_) => info!("peer added"),
                        Err(err) => error!("{}", err),
                    }

                }
                Err(_) => {
                    error!("Cannot connect to peer {}", peer_addr);
                }
            };
        }

        (myServer, event_loop)
    }

    pub fn run(local_addr: SocketAddr,
               peers: HashMap<ServerId, SocketAddr>,
               log: L)
               -> (Server<L>, EventLoop<Server<L>>) {
        let (mut server, mut event_loop) = Server::new(local_addr, peers, log);

        event_loop.run(&mut server);

        (server, event_loop)
    }

    fn init_consensus(&self, event_loop: &mut EventLoop<Server<L>>, log: L) -> Consensus<L> {
        let heartbeat = ConsensusTimeout::HeartbeatTimeout;
        let electionTimeout = ConsensusTimeout::ElectionTimeout;

        let heartbeat = self.set_timeout(event_loop, ServerTimeout::ConsensusTimeout(heartbeat))
            .unwrap();
        let election =
            self.set_timeout(event_loop, ServerTimeout::ConsensusTimeout(electionTimeout)).unwrap();

        let state_machine = StateMachine::new(self.id);

        Consensus {
            heartbeat_handler: heartbeat,
            election_handler: election,
            state_machine: state_machine,
            log: log,
        }
    }


    pub fn set_timeout(&self,
                       event_loop: &mut EventLoop<Server<L>>,
                       timeout_type: ServerTimeout)
                       -> Option<Timeout> {
        match timeout_type {
            ServerTimeout::ConsensusTimeout(c) => {
                Some(event_loop.timeout_ms(timeout_type, c.get_duration()).unwrap())
            }
            ServerTimeout::NetworkTimeout => None,
        }
    }
}

impl<L: Log + Clone> Handler for Server<L> {
    type Message = ();
    type Timeout = ServerTimeout;

    fn ready(&mut self, event_loop: &mut EventLoop<Server<L>>, token: Token, events: EventSet) {
        match token {
            SERVER => {
                match self.server.accept() {
                    Ok(Some(socket)) => {
                        let token = self.connections
                            .insert_with(|token| Connection::new(socket, token))
                            .unwrap();

                        event_loop.register(self.connections[token].socket.inner(),
                                      token,
                                      EventSet::readable(),
                                      PollOpt::edge() | PollOpt::oneshot())
                            .unwrap();

                        let connection_preamble = server_connection_preamble(self.id, &self.addr);
                        self.connections[token].write(connection_preamble);
                    } 
                    Ok(None) => error!("socket was not actually ready"),
                    Err(_) => error!("listener.accept() errored"),
                }
            } 
            _ => {
                debug!("client socket is ready");
                self.connections[token].ready(event_loop, events, self.id, self.addr);
            }
        }
    }

    fn timeout(&mut self, event_loop: &mut EventLoop<Server<L>>, timeout: Self::Timeout) {
        match timeout {
            ServerTimeout::NetworkTimeout => {
                error!("There is a network timeout");
            }
            ServerTimeout::ConsensusTimeout(ct) => {
                match ct {
                    ConsensusTimeout::HeartbeatTimeout => {
                        self.consensus.clone().unwrap().heartbeat_timeout(&self, event_loop);
                    }
                    ConsensusTimeout::ElectionTimeout => {
                        self.consensus.clone().unwrap().election_timeout(&self, event_loop);
                    }
                }
            }
        }
    }
}

pub struct Connection {
    pub socket: MessageStream<TcpStream, HeapAllocator, Rc<Builder<HeapAllocator>>>,
    token: Token,
}

impl Connection {
    pub fn new((socket, _): (TcpStream, SocketAddr), token: Token) -> Self {
        Connection {
            socket: MessageStream::new(socket, ReaderOptions::new()),
            token: token,
        }
    }

    pub fn ready<L: Log + Clone>(&mut self,
                                 event_loop: &mut EventLoop<Server<L>>,
                                 events: EventSet,
                                 id: ServerId,
                                 addr: SocketAddr) {

        if events.is_readable() {
            let message = Self::read(self);

            match message {
                Ok(op) => {
                    match op {
                        Some(m) => {
                            debug!("Message received");
                        }
                        None => info!("empty message received"),
                    }
                }
                Err(err) => error!("{}", err),
            }
        } else if events.is_writable() {

            self.flush();
        }

        Self::reregister(self, event_loop, events);
    }

    fn reregister<L: Log + Clone>(&self,
                                  event_loop: &mut EventLoop<Server<L>>,
                                  old_event_set: EventSet) {
        let mut event_set = EventSet::none();
        if old_event_set.is_readable() {
            event_set = EventSet::writable();
        } else if old_event_set.is_writable() {
            event_set = EventSet::readable();
        }

        event_loop.reregister(self.socket.inner(),
                        self.token,
                        event_set,
                        PollOpt::edge() | PollOpt::oneshot())
            .unwrap();
    }

    fn read(&mut self) -> Result<Option<Reader<Segments>>> {
        self.socket.read_message().map_err(From::from)
    }

    fn write(&mut self, message: Rc<Builder<HeapAllocator>>) -> Result<()> {
        try!(self.socket.write_message(message));

        Ok(())
    }

    fn flush(&mut self) {
        self.socket.write();
    }

    pub fn new_peer(token: Token, stream: TcpStream) -> Self {
        Connection {
            socket: MessageStream::new(stream, ReaderOptions::new()),
            token: token,
        }
    }
}

#[cfg(test)]
mod tests {
    use mio::*;
    use std::net::{SocketAddr, TcpListener, TcpStream};
    use std::collections::HashMap;

    use capnp::message::ReaderOptions;
    use capnp::serialize;
    use std::io::Read;
    use messages_capnp::connection_preamble;

    use network::Server;
    use util::*;
    use std::thread;

    fn new_server(peers: HashMap<ServerId, SocketAddr>) -> (Server, EventLoop<Server>) {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let (mut server, mut event_loop) = Server::new(addr, peers);

        (server, event_loop)
    }

    fn read_server_preamble<R>(read: &mut R) -> ServerId
        where R: Read
    {
        let message = serialize::read_message(read, ReaderOptions::new()).unwrap();
        let preamble = message.get_root::<connection_preamble::Reader>().unwrap();

        match preamble.get_id().which().unwrap() {
            connection_preamble::id::Which::Server(peer) => ServerId(peer.unwrap().get_id()),
            _ => panic!("unexpected preamble id"),
        }
    }

    // FIXME  serialize::read_message blocks
    #[test]
    fn test_peer_connect() {
        let peer_id = ServerId(1);
        let peer_listener = TcpListener::bind("127.0.0.1:0").unwrap();

        let mut peers = HashMap::new();
        peers.insert(peer_id, peer_listener.local_addr().unwrap());

        let (mut server, mut event_loop) = new_server(peers);

        let (mut stream, _) = peer_listener.accept().unwrap();

        assert_eq!(ServerId(0), read_server_preamble(&mut stream));
    }
}
