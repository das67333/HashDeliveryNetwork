use crate::{logger::LogEvent, server::Server};
use std::{
    collections::HashMap,
    io::{Error, ErrorKind, Result},
    net::TcpStream,
    sync::{Arc, Mutex},
};

impl Server {
    /// Processes client requests until the connection is terminated.
    pub fn request_handler(
        mut client: TcpStream,
        mut storage: Arc<Mutex<HashMap<String, String>>>,
    ) {
        Self::log(LogEvent::NewConnection(&client), &mut storage);
        Self::greet(&mut client).ok();
        while Self::accept_request(&mut client, &mut storage).is_ok() {}
        Self::log(LogEvent::Disconnected, &mut storage);
    }

    fn accept_request(
        client: &mut TcpStream,
        storage: &mut Arc<Mutex<HashMap<String, String>>>,
    ) -> Result<()> {
        let request_json = Self::read(client)?;
        // eprintln!("{:?}", request_json);
        let request = Self::deserialize_request(&request_json);
        let is_invalid = matches!(request, Request::Invalid(_));
        Self::log(LogEvent::NewRequest(client, &request), storage);
        let respond = Self::respond(request, storage);
        let respond_json = Self::serialize_respond(respond);
        Self::write(client, &respond_json)?;
        if is_invalid {
            Err(Error::new(ErrorKind::InvalidData, "Invalid request"))
        } else {
            Ok(())
        }
    }

    fn try_deserialize_request(request: &[u8]) -> Result<Request> {
        let request: HashMap<String, String> =
            serde_json::from_str(&String::from_utf8(request.to_vec()).map_err(|_| {
                Error::new(ErrorKind::InvalidData, "Request contains non-utf8 data")
            })?)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Request parsing error"))?;

        let request_type = match request.get("request_type") {
            Some(val) => val.as_str(),
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Request doesn't contain \"request_type\" field",
                ))
            }
        };

        match request_type {
            "store" => match request.get("hash") {
                Some(hash) => {
                    let key = match request.get("key") {
                        Some(val) => val,
                        None => {
                            return Err(Error::new(
                                ErrorKind::InvalidData,
                                "Request doesn't contain \"key\" field",
                            ))
                        }
                    }
                    .to_owned();
                    Ok(Request::Store(key, hash.to_owned()))
                }
                None => Err(Error::new(
                    ErrorKind::InvalidData,
                    "Store request doesn't contain hash field",
                )),
            },
            "load" => {
                let key = match request.get("key") {
                    Some(val) => val,
                    None => {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "Request doesn't contain \"key\" field",
                        ))
                    }
                }
                .to_owned();
                Ok(Request::Load(key))
            }
            "shutdown" => Ok(Request::Shutdown),
            _ => Err(Error::new(
                ErrorKind::Unsupported,
                "Unsupported request type",
            )),
        }
    }

    fn deserialize_request(request: &[u8]) -> Request {
        match Self::try_deserialize_request(request) {
            Ok(val) => val,
            Err(err) => Request::Invalid(err),
        }
    }

    fn serialize_respond(respond: Respond) -> Vec<u8> {
        match respond {
            Respond::StoreSuccess => {
                b"{\
                \n\t\"response_status\": \"success\"\
                \n}"
            }
            .to_vec(),
            Respond::LoadSuccess(key, hash) => format!(
                "{{\
                \n\t\"response_status\": \"success\",\
                \n\t\"requested_key\": \"{key}\",\
                \n\t\"requested_hash\": \"{hash}\"\
                \n}}"
            )
            .as_bytes()
            .to_vec(),
            Respond::LoadKeyNotFound => {
                b"{\
                \n\t\"response_status\": \"key not found\"\
                \n}"
            }
            .to_vec(),
            Respond::InvalidRequest(err) => {
                format!(
                    "{{\
                    \n\t\"response_status\": \"invalid request\",\
                    \n\t\"error\": \"{err}\"
                    \n}}",
                )
            }
            .as_bytes()
            .to_vec(),
        }
    }

    fn respond(request: Request, storage: &mut Arc<Mutex<HashMap<String, String>>>) -> Respond {
        match request {
            Request::Store(key, hash) => {
                {
                    storage.lock().unwrap().insert(key, hash);
                };
                Respond::StoreSuccess
            }
            Request::Load(key) => match { storage.lock().unwrap().get(&key) } {
                Some(hash) => Respond::LoadSuccess(key, hash.to_owned()),
                None => Respond::LoadKeyNotFound,
            },
            Request::Shutdown => {
                std::process::exit(0);
            }
            Request::Invalid(err) => Respond::InvalidRequest(err.to_string()),
        }
    }

    fn greet(client: &mut TcpStream) -> Result<()> {
        Self::write(
            client,
            b"{\
                    \n\t\"student_name\": \"das67333\"\
                    \n}",
        )
    }
}

/// Implements a client's request to the `Server`
pub enum Request {
    /// Stores key and hash
    Store(String, String),
    /// Stores key
    Load(String),
    Shutdown,
    /// Stores the error that occurred
    Invalid(Error),
}

enum Respond {
    StoreSuccess,
    /// stores key and hash
    LoadSuccess(String, String),
    LoadKeyNotFound,
    /// stores error message
    InvalidRequest(String),
}
