use log::info;
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
        let buffer: [u8; 8] = (message.len() as u64).to_le_bytes();
        self.stream.write_all(&buffer)?;
        self.stream.write_all(message)?;
        info!(
            "Client {} sent:\n{}",
            self.id,
            String::from_utf8(message.to_vec()).unwrap()
        );

        Ok(())
    }

    pub fn read(&mut self) -> Result<Vec<u8>> {
        let mut buffer = [0; 8];
        self.stream.read_exact(&mut buffer)?;
        let length = u64::from_le_bytes(buffer) as usize;
        let mut message = vec![0; length];
        self.stream.read_exact(&mut message)?;
        info!(
            "Client {} received:\n{}",
            self.id,
            String::from_utf8(message.clone()).unwrap()
        );
        Ok(message)
    }

    pub fn shutdown(&mut self, how: Shutdown) {
        let _ = self.stream.shutdown(how);
    }
}
