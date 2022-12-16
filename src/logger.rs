use crate::{client_handler::Request, server::Server};
use std::{
    collections::HashMap,
    io::{stderr, Write},
    sync::Arc,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

impl Server {
    /// Creates a log message and writes it to stderr.
    pub async fn log<'a>(event: LogEvent<'a>, storage: &mut Arc<Mutex<HashMap<String, String>>>) {
        if std::env::var("DISABLE_LOGS").is_ok() {
            return;
        }

        use chrono::prelude::*;

        let log_message = format!(
            "[{}]    Storage size: {}. {}",
            Local::now().format("%d/%b/%Y:%H:%M:%S %z"),
            storage.lock().await.len(),
            match event {
                LogEvent::ServerStart(listener) =>
                    if let Ok(addr) = listener.local_addr() {
                        format!("Server started with address: {addr}.",)
                    } else {
                        return;
                    },
                LogEvent::NewRequest(client, request) =>
                    if let Ok(addr) = client.peer_addr() {
                        match &request {
                        Request::Store(key, value) => format!(
                            "Client address: {addr}. Received request to write new value \"{value}\" by key \"{key}\".",
                        ),
                        Request::Load(key) =>
                            format!("Client address: {addr}. Received request to get value by key \"{key}\"."),
                        Request::Shutdown => format!("Client address: {addr}. Received request to shutdown."),
                        Request::Invalid(err) =>
                            format!("Client address: {addr}. Received invalid request: \"{err}\"."),
                        }
                    } else {
                        return;
                    },
                LogEvent::NewConnection(client) =>
                    if let Ok(addr) = client.peer_addr() {
                        format!("Client address: {addr}. Connection established.")
                    } else {
                        return;
                    },
                LogEvent::Disconnected => "Terminated connection with a client.".to_owned(),
            }
        );
        eprintln!("{log_message}");
        stderr().flush().ok();
    }
}

/// Implements server events to logged.
pub enum LogEvent<'a> {
    ServerStart(&'a TcpListener),
    NewRequest(&'a TcpStream, &'a Request),
    NewConnection(&'a TcpStream),
    Disconnected,
}
