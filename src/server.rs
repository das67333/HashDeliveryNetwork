use std::{collections::HashMap, net::IpAddr, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Error, ErrorKind, Result},
    net::{TcpListener, TcpStream},
    spawn,
    sync::Mutex,
    task::JoinHandle,
};

pub struct Server {
    listener: TcpListener,
    threads: Vec<JoinHandle<()>>,
    storage: Arc<Mutex<HashMap<String, String>>>,
}

impl Server {
    pub async fn start(ip: IpAddr, port: u16) -> Self {
        Self {
            listener: TcpListener::bind((ip, port)).await.unwrap(),
            threads: Vec::new(),
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            let (client, _addr) = self.listener.accept().await?;
            let storage_ref = self.storage.clone();
            self.threads
                .push(spawn(Self::client_handler(client, storage_ref)));
        }
    }

    async fn client_handler(
        mut client: TcpStream,
        mut storage: Arc<Mutex<HashMap<String, String>>>,
    ) {
        if Self::write(
            &mut client,
            b"{\
                    \n  \"student_name\": \"das67333\"\
                    \n}",
        )
        .await
        .is_err()
        {
            drop(client);
            return;
        };
        loop {
            let message = if let Ok(value) = Self::read(&mut client).await {
                value
            } else {
                break;
            };
            let respond = if let Ok(value) = Self::respond(&message, &mut storage).await {
                value
            } else {
                break;
            };
            if Self::write(&mut client, &respond).await.is_err() {
                break;
            };
        }
        drop(client);
    }

    async fn respond(
        request: &[u8],
        storage: &mut Arc<Mutex<HashMap<String, String>>>,
    ) -> Result<Vec<u8>> {
        let request: HashMap<String, String> =
            serde_json::from_str(match &String::from_utf8(request.to_vec()) {
                Ok(val) => val,
                Err(_) => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "Request contains non-utf8 data",
                    ))
                }
            })?;

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
                    storage.lock().await.insert(key.to_owned(), hash.to_owned());
                }
                b"{\
                \n  \"response_status\": \"success\"\
                \n}"
                .to_vec()
            }
            "load" => match { storage.lock().await.get(key) } {
                Some(hash) => format!(
                    "{{\
                    \n  \"response_status\": \"success\",\
                    \n  \"requested_key\": \"{}\",\
                    \n  \"requested_hash\": \"{}\"\
                    \n}}",
                    key, hash
                )
                .as_bytes()
                .to_vec(),
                None => b"{\
                        \n  \"response_status\": \"key not found\"\
                        \n}"
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

    async fn read(stream: &mut TcpStream) -> Result<Vec<u8>> {
        let mut buffer = [0; 8];
        stream.read_exact(&mut buffer).await?;
        let length = u64::from_le_bytes(buffer) as usize;
        let mut message = vec![0; length];
        stream.read_exact(&mut message).await?;
        Ok(message)
    }

    async fn write(stream: &mut TcpStream, message: &[u8]) -> Result<()> {
        assert_ne!(
            0,
            message.len(),
            "Server is trying to send an empty message"
        );
        let buffer = [
            (message.len() as u64).to_le_bytes().to_vec(),
            message.to_vec(),
        ]
        .concat();
        stream.write_all(&buffer).await?;
        Ok(())
    }
}
