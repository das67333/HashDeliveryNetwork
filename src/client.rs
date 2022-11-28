use std::io::{prelude::*, Result};
use std::net::{Shutdown, SocketAddr, TcpStream};

#[derive(Debug)]
pub struct Client {
    pub id: u64,
    stream: TcpStream,
}

impl Client {
    pub fn start(id: u64, server_addr: SocketAddr) -> Result<Self> {
        Ok(Self {
            id,
            stream: TcpStream::connect(server_addr)?,
        })
    }

    pub fn write(&mut self, message: &[u8]) -> Result<()> {
        assert_ne!(
            0,
            message.len(),
            "Client {} is trying to send an empty message",
            self.id
        );
        let buffer = [
            (message.len() as u64).to_le_bytes().to_vec(),
            message.to_vec(),
        ]
        .concat();
        self.stream.write_all(&buffer)?;
        Ok(())
    }

    pub fn read(&mut self) -> Result<Vec<u8>> {
        let mut buffer = [0; 8];
        self.stream.read_exact(&mut buffer)?;
        let length = u64::from_le_bytes(buffer) as usize;
        let mut message = vec![0; length];
        self.stream.read_exact(&mut message)?;
        Ok(message)
    }

    pub fn shutdown(&mut self) {
        self.stream.shutdown(Shutdown::Both).ok();
    }
}

impl Clone for Client {
    fn clone(&self) -> Self {
        Client {
            id: self.id,
            stream: self.stream.try_clone().unwrap(),
        }
    }
}
