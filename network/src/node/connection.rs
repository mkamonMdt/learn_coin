use tokio::net::tcp::OwnedWriteHalf;

#[derive(Debug)]
pub struct UnverifiedConnection {
    addr: String,
    writer: OwnedWriteHalf,
}

impl UnverifiedConnection {
    pub fn new(addr: String, writer: OwnedWriteHalf) -> Self {
        Self { addr, writer }
    }
}
