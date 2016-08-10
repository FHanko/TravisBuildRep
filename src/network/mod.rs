use mio::{EventLoop, Handler, EventSet, PollOpt, Token, Timeout};
use mio::tcp::{TcpListener, TcpStream};
use mio::util::Slab;
use std::net::SocketAddr;
use std::io::{Read, Write};
use capnp::message::{Builder, HeapAllocator, ReaderOptions, Reader};
use capnp_nonblock::{MessageStream, Segments};
use capnp::serialize::read_message;
use capnp::Result;
use std::rc::Rc;
use consensus::{Consensus, ConsensusTimeout};

pub struct Server {
    pub id: String,
    pub server: TcpListener,
    pub connections: Slab<Connection>,
    addr: SocketAddr,
    consensus: Option<Consensus>,
}

#[derive(Copy,Clone)]
pub enum ServerTimeout {
    NetworkTimeout,
    ConsensusTimeout(ConsensusTimeout),
}

const SERVER: Token = Token(0);

impl Server {
    pub fn new(event_loop: &mut EventLoop<Server>) -> (Server, EventLoop<Server>) {

        let mut event_loop = EventLoop::new().unwrap();

        let addr = "0.0.0.0:1337".parse().unwrap();
        let server = TcpListener::bind(&addr).unwrap();

        event_loop.register(&server, SERVER, EventSet::all(), PollOpt::edge()).unwrap();


        let mut myServer = Server {
            id: "".to_string(),
            server: server,
            connections: Slab::new_starting_at(Token(1), 257),
            addr: addr,
            consensus: None,
        };

        let consensus = myServer.init_consensus(&mut event_loop);
        myServer.consensus = Some(consensus);


        event_loop.run(&mut myServer).unwrap();

        (myServer, event_loop)
    }

    fn init_consensus(&self, event_loop: &mut EventLoop<Server>) -> Consensus {
        let heartbeat = ConsensusTimeout::HeartbeatTimeout;
        let electionTimeout = ConsensusTimeout::ElectionTimeout;

        let heartbeat = self.set_timeout(event_loop, ServerTimeout::ConsensusTimeout(heartbeat))
            .unwrap();
        let election =
            self.set_timeout(event_loop, ServerTimeout::ConsensusTimeout(electionTimeout)).unwrap();

        Consensus {
            heartbeat_handler: heartbeat,
            election_handler: election,
        }
    }


    pub fn set_timeout(&self,
                       event_loop: &mut EventLoop<Server>,
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

impl Handler for Server {
    type Message = ();
    type Timeout = ServerTimeout;

    fn ready(&mut self, event_loop: &mut EventLoop<Server>, token: Token, events: EventSet) {
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
                    } 
                    Ok(None) => println!("socket was not actually ready"),
                    Err(e) => println!("listener.accept() errored"),
                }
            } 
            _ => {
                // TODO send message when server gets connected
                // let connection_preamble = server_connection_preamble(self.id, &self.addr);
                // self.connections[token].ready(event_loop, events, connection_preamble);
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

    pub fn ready(&mut self,
                 event_loop: &mut EventLoop<Server>,
                 events: EventSet,
                 connection_preamble: Rc<Builder<HeapAllocator>>) {
        if events.is_readable() {
            Self::read(self);
        } else if events.is_writable() {
            Self::write(self, connection_preamble);
        }

        Self::reregister(self, event_loop, events);
    }

    fn reregister(&self, event_loop: &mut EventLoop<Server>, old_event_set: EventSet) {
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

    fn write(&mut self, message: Rc<Builder<HeapAllocator>>) {
        self.socket.write_message(message);
    }
}
