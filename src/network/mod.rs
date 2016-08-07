use mio::{EventLoop, Handler, EventSet, PollOpt, Token};
use mio::tcp::{TcpListener, TcpStream};
use mio::util::Slab;
use std::net::SocketAddr;
use std::io::{Read, Write};
use capnp::message::{Builder, HeapAllocator, ReaderOptions, Reader};
use capnp_nonblock::{MessageStream, Segments};
use capnp::serialize::read_message;
use capnp::Result;
use std::rc::Rc;

pub struct Server {
    pub id: String,
    pub server: TcpListener,
    pub connections: Slab<Connection>,
    addr: SocketAddr,
}

const SERVER: Token = Token(0);

impl Server {
    fn new() -> Self {
        let addr = "0.0.0.0:1337".parse().unwrap();
        let server = TcpListener::bind(&addr).unwrap();
        Server {
            id: "".to_string(),
            server: server,
            connections: Slab::new_starting_at(Token(1), 257),
            addr: addr,
        }
    }

    pub fn run() -> (Server, EventLoop<Server>) {
        let mut event_loop = EventLoop::new().unwrap();

        let mut server = Self::new();
        event_loop.register(&server.server, SERVER, EventSet::all(), PollOpt::edge())
            .unwrap();


        event_loop.run(&mut server).unwrap();

        (server, event_loop)
    }
}

impl Handler for Server {
    type Message = ();
    type Timeout = ();

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
