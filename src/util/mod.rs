pub struct ServerId;

impl ServerId {
    pub fn as_u64(&self) -> u64 {
        unimplemented!()
    }
}

pub struct ClientId;

impl ClientId {
    pub fn as_bytes<'a>(&self) -> &'a [u8] {
        unimplemented!()
    }
}

#[derive(Copy,Clone)]
pub struct Term;

impl Term {
    pub fn as_u64(&self) -> u64 {
        unimplemented!()
    }
}

impl From<u64> for Term {
    fn from(v: u64) -> Self {
        unimplemented!()
    }
}

impl Into<u64> for Term {
    fn into(self) -> u64 {
        unimplemented!()
    }
}

pub struct LogIndex;

impl LogIndex {
    pub fn as_u64(&self) -> u64 {
        unimplemented!()
    }
}

impl Into<u64> for LogIndex {
    fn into(self) -> u64 {
        unimplemented!()
    }
}
