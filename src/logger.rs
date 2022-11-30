use crate::{client_handler::Request, server::Server};
use std::{collections::HashMap, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

impl Server {
    /// Creates a log message and writes it to stderr.
    pub async fn log<'a>(event: LogEvent<'a>, storage: &mut Arc<Mutex<HashMap<String, String>>>) {
        use chrono::prelude::*;

        eprintln!(
            "[{}]    Storage size: {}. {}",
            Local::now().format("%d/%b/%Y:%H:%M:%S %z"),
            storage.lock().await.len(),
            match event {
                LogEvent::ServerStart(listener) => format!(
                    "Server was started with address: {}.",
                    listener.local_addr().unwrap()
                ),
                LogEvent::NewRequest(request, client) => match &request {
                    Request::Store(key, value) => format!(
                        "Client address: {}. Received request to write new value \"{}\" by key \"{}\".",
                        client.peer_addr().unwrap(), key, value
                    ),
                    Request::Load(key) =>
                        format!("Client address: {}. Received request to get value by key \"{}\".", client.peer_addr().unwrap(), key),
                },
                LogEvent::NewConnection(client) => format!("Client address: {}. Connection established.", client.peer_addr().unwrap()),
            }
        );
    }
}

/// Implements server events to logged.
pub enum LogEvent<'a> {
    ServerStart(&'a TcpListener),
    NewRequest(&'a Request, &'a TcpStream),
    NewConnection(&'a TcpStream),
}
