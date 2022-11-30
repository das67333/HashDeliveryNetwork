use crate::{logger::LogEvent, server::Server};
use std::{collections::HashMap, sync::Arc};
use tokio::{
    io::{Error, ErrorKind, Result},
    net::TcpStream,
    sync::Mutex,
};

impl Server {
    /// Processes client requests until the connection is terminated.
    pub async fn client_handler(
        mut client: TcpStream,
        mut storage: Arc<Mutex<HashMap<String, String>>>,
    ) {
        Self::log(LogEvent::NewConnection(&client), &mut storage).await;
        Self::greet(&mut client).await.ok();
        while Self::accept_request(&mut client, &mut storage)
            .await
            .is_ok()
        {}
        drop(client);
    }

    async fn greet(client: &mut TcpStream) -> Result<()> {
        Self::write(
            client,
            b"{\
                    \n  \"student_name\": \"das67333\"\
                    \n}",
        )
        .await
    }

    async fn accept_request(
        client: &mut TcpStream,
        storage: &mut Arc<Mutex<HashMap<String, String>>>,
    ) -> Result<()> {
        let request_json = Self::read(client).await?;
        let request = Self::deserialize_request(&request_json)?;
        Self::log(LogEvent::NewRequest(&request, client), storage).await;
        let respond = Self::respond(request, storage).await?;
        let respond_json = Self::serialize_respond(respond);
        Self::write(client, &respond_json).await?;
        Ok(())
    }

    fn deserialize_request(request: &[u8]) -> Result<Request> {
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
            Some(val) => val,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Request doesn't contain key field",
                ))
            }
        }
        .to_owned();
        match request_type {
            "store" => match request.get("hash") {
                Some(hash) => Ok(Request::Store(key, hash.to_owned())),
                None => Err(Error::new(
                    ErrorKind::InvalidData,
                    "Store request doesn't contain hash field",
                )),
            },
            "load" => Ok(Request::Load(key)),
            _ => Err(Error::new(
                ErrorKind::Unsupported,
                "Unsupported request type",
            )),
        }
    }

    fn serialize_respond(respond: Respond) -> Vec<u8> {
        match respond {
            Respond::StoreSuccess => {
                b"{\
                \n  \"response_status\": \"success\"\
                \n}"
            }
            .to_vec(),
            Respond::LoadSuccess(key, hash) => format!(
                "{{\
                \n  \"response_status\": \"success\",\
                \n  \"requested_key\": \"{key}\",\
                \n  \"requested_hash\": \"{hash}\"\
                \n}}"
            )
            .as_bytes()
            .to_vec(),
            Respond::LoadKeyNotFound => {
                b"{\
                \n  \"response_status\": \"key not found\"\
                \n}"
            }
            .to_vec(),
        }
    }

    async fn respond(
        request: Request,
        storage: &mut Arc<Mutex<HashMap<String, String>>>,
    ) -> Result<Respond> {
        match request {
            Request::Store(key, hash) => {
                {
                    storage.lock().await.insert(key, hash);
                };
                Ok(Respond::StoreSuccess)
            }
            Request::Load(key) => match { storage.lock().await.get(&key) } {
                Some(hash) => Ok(Respond::LoadSuccess(key, hash.to_owned())),
                None => Ok(Respond::LoadKeyNotFound),
            },
        }
    }
}

/// Implements a client's request to the `Server`
pub enum Request {
    Store(String, String),
    Load(String),
}

enum Respond {
    StoreSuccess,
    LoadSuccess(String, String),
    LoadKeyNotFound,
}
