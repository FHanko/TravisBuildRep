use uuid::Uuid;

#[derive(Debug,Clone,Eq,PartialEq,Hash, Copy)]
pub struct ServerId(pub u64);

impl ServerId {
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

#[derive(Default)]
pub struct ClientId(pub Uuid);

impl ClientId {
    pub fn new() -> Self {
        ClientId(Uuid::new_v4())
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[derive(Copy,Clone)]
pub struct Term(pub u64);

impl Term {
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl From<u64> for Term {
    fn from(v: u64) -> Self {
        Term(v)
    }
}

impl Into<u64> for Term {
    fn into(self) -> u64 {
        self.0
    }
}

#[derive(Copy,Clone)]
pub struct LogIndex(pub u64);

impl LogIndex {
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Into<u64> for LogIndex {
    fn into(self) -> u64 {
        self.0
    }
}

impl From<u64> for LogIndex {
    fn from(v: u64) -> LogIndex {
        LogIndex(v)
    }
}
