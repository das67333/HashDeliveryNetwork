use crate::logger::LogEvent;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Error, ErrorKind, Result},
    net::{TcpListener, TcpStream},
    spawn,
    sync::Mutex,
    task::JoinHandle,
};

/// An asynchronous server that stores hashes and processes TCP requests.
pub struct Server {
    listener: TcpListener,
    request_handles: Vec<JoinHandle<()>>,
    storage: Arc<Mutex<HashMap<String, String>>>,
}

impl Server {
    /// Attempts to create a TCP server with the given address and port.
    pub async fn new(addr: SocketAddr) -> Result<Self> {
        let mut storage = Arc::new(Mutex::new(HashMap::new()));
        let listener = TcpListener::bind(addr).await?;
        Self::log(LogEvent::ServerStart(&listener), &mut storage).await;
        Ok(Self {
            listener,
            request_handles: vec![],
            storage,
        })
    }

    /// Starts accepting TCP connections and processing client requests.
    pub async fn run(&mut self) -> Result<()> {
        loop {
            let (client, _addr) = self.listener.accept().await?;
            let storage_ref = self.storage.clone();
            self.request_handles
                .push(spawn(Self::request_handler(client, storage_ref)));
        }
    }

    /// Reads bytes from the provided TCP stream until `b'}'` or `EOF` is found.
    /// Once found, all bytes to, and including, `b'}'` (if found)
    /// will be returned.
    pub async fn read(stream: &mut TcpStream) -> Result<Vec<u8>> {
        let mut buffer = vec![];
        let ret = BufReader::new(stream).read_until(b'}', &mut buffer).await?;
        if ret == 0 {
            Err(Error::new(
                ErrorKind::AddrNotAvailable,
                "Connection terminated unexpectedly",
            ))
        } else {
            Ok(buffer)
        }
    }

    /// Writes an entire buffer into the provided TCP stream.
    pub async fn write(stream: &mut TcpStream, buffer: &[u8]) -> Result<()> {
        stream.write_all(buffer).await
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let mut tasks = vec![];
        std::mem::swap(&mut tasks, &mut self.request_handles);
        for task in tasks.into_iter() {
            task.abort();
        }
    }
}
