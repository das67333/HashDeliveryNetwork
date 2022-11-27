use indoc::indoc;
use log::info;
use std::{
    collections::HashMap,
    io::{Error, ErrorKind, Read, Result, Write},
    net::{IpAddr, Shutdown, TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

pub struct Server {
    listener: TcpListener,
    threads: Vec<std::thread::JoinHandle<()>>,
    storage: Arc<Mutex<HashMap<String, String>>>,
}

impl Server {
    pub fn start(ip: IpAddr, port: u16) -> Self {
        Self {
            listener: TcpListener::bind((ip, port)).unwrap(),
            threads: Vec::new(),
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        for client in self.listener.incoming().map(|x| x.unwrap()) {
            info!("Client connected: {}", client.peer_addr().unwrap());
            let storage_ref = self.storage.clone();
            self.threads.push(std::thread::spawn(move || {
                Self::client_handler(client, storage_ref)
            }));
        }
        Ok(())
    }

    fn client_handler(mut client: TcpStream, mut storage: Arc<Mutex<HashMap<String, String>>>) {
        match Self::write(
            &mut client,
            indoc! {
            b"{
              \"student_name\": \"Your Name\"
            }"},
        ) {
            Ok(_) => {}
            Err(_) => {
                let _ = client.shutdown(Shutdown::Both);
                return;
            }
        };
        loop {
            let message = match Self::read(&mut client) {
                Ok(val) => val,
                Err(_) => break,
            };
            let respond = match Self::respond(&message, &mut storage) {
                Ok(val) => val,
                Err(_) => break,
            };
            match Self::write(&mut client, &respond) {
                Ok(_) => {}
                Err(_) => break,
            };
        }
        let _ = client.shutdown(Shutdown::Both);
    }

    fn respond(
        request: &[u8],
        storage: &mut Arc<Mutex<HashMap<String, String>>>,
    ) -> Result<Vec<u8>> {
        let request: HashMap<String, String> =
            serde_json::from_str(&String::from_utf8(request.to_vec()).unwrap())?;

        let request_type = match request.get("request_type") {
            Some(val) => val.as_str(),
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Request doesn't contain request_type field",
                ))
            }
        };
        let key = match request.get("key") {
            Some(val) => val.as_str(),
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Request doesn't contain key field",
                ))
            }
        };
        let response: Vec<u8> = match request_type {
            "store" => {
                let hash = match request.get("hash") {
                    Some(val) => val.as_str(),
                    None => {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "Store request doesn't contain hash field",
                        ))
                    }
                };
                {
                    storage
                        .lock()
                        .unwrap()
                        .insert(key.to_owned(), hash.to_owned());
                }
                indoc! {
                b"{
                  \"response_status\": \"success\"
                }"}
                .to_vec()
            }
            "load" => match { storage.lock().unwrap().get(key) } {
                Some(hash) => format!(
                    indoc! {
                    "{{
                      \"response_status\": \"success\",
                      \"requested_key\": \"{}\",
                      \"requested_hash\": \"{}\"
                    }}"},
                    key, hash
                )
                .as_bytes()
                .to_vec(),
                None => indoc! {
                b"{
                  \"response_status\": \"key not found\"
                }"
                }
                .to_vec(),
            },
            _ => {
                return Err(Error::new(
                    ErrorKind::Unsupported,
                    "Unsupported request type",
                ))
            }
        };
        Ok(response)
    }

    fn read(stream: &mut TcpStream) -> Result<Vec<u8>> {
        let mut buffer = [0; 8];
        stream.read_exact(&mut buffer)?;
        let length = u64::from_le_bytes(buffer) as usize;
        let mut message = vec![0; length];
        stream.read_exact(&mut message)?;
        info!(
            "Server received:\n{}",
            String::from_utf8(message.clone()).unwrap()
        );
        Ok(message)
    }

    fn write(stream: &mut TcpStream, message: &[u8]) -> Result<()> {
        assert_ne!(
            0,
            message.len(),
            "Server is trying to send an empty message"
        );
        let buffer: [u8; 8] = (message.len() as u64).to_le_bytes();
        stream.write_all(&buffer)?;
        stream.write_all(message)?;
        info!(
            "Server sent:\n{}",
            String::from_utf8(message.to_vec()).unwrap()
        );
        Ok(())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let mut moved = vec![];
        std::mem::swap(&mut self.threads, &mut moved);
        for thread in moved.into_iter() {
            thread.join().unwrap();
        }
    }
}
