#[allow(dead_code)]
pub mod tests {
    use rand::Rng;
    use std::{
        io::{BufRead, BufReader, Result, Write},
        net::{IpAddr, Shutdown, SocketAddr, TcpListener, TcpStream},
        process::{Child, Command},
        str::FromStr,
        thread, time,
    };

    pub enum IpVersion {
        V4,
        V6,
    }

    pub struct ServerWrapper {
        proc: Option<Child>,
        pub addr: SocketAddr,
    }

    impl ServerWrapper {
        pub fn start(ip_version: IpVersion) -> Result<Self> {
            let ip = match ip_version {
                IpVersion::V4 => IpAddr::from_str("127.0.0.1"),
                IpVersion::V6 => IpAddr::from_str("::1"),
            }
            .unwrap();
            let port = TcpListener::bind((ip, 0))?.local_addr()?.port();
            let proc = Command::new(env!("CARGO_BIN_EXE_hash-delivery-network"))
                .arg("--ip")
                .arg(ip.to_string())
                .arg("--port")
                .arg(port.to_string())
                .spawn()?;
            thread::sleep(time::Duration::from_millis(100));
            Ok(Self {
                proc: Some(proc),
                addr: SocketAddr::new(ip, port),
            })
        }

        pub fn is_alive(&mut self) -> bool {
            self.proc
                .as_mut()
                .map_or(false, |proc| proc.try_wait().unwrap().is_none())
        }

        pub fn expected_to_be_dead(&mut self) {
            self.stop().ok();
        }

        pub fn stop(&mut self) -> std::io::Result<()> {
            self.proc.take().map_or(Ok(()), |mut proc| proc.kill())
        }
    }

    impl Drop for ServerWrapper {
        fn drop(&mut self) {
            self.stop().ok();
        }
    }

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
            self.stream.write_all(message)
        }

        pub fn read(&mut self) -> Result<Vec<u8>> {
            let mut buffer = vec![];
            BufReader::new(&mut self.stream).read_until(b'}', &mut buffer)?;
            Ok(buffer)
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

    /// Generate random store and load requests and responses to them
    pub fn gen_requests_set(
        index_begin: usize,
        index_end: usize,
    ) -> (Vec<(Vec<u8>, Vec<u8>)>, Vec<(Vec<u8>, Vec<u8>)>) {
        let mut rng = rand::thread_rng();

        (index_begin..index_end)
            .map(|index| {
                let (key, hash) = (index, rng.gen::<u128>());
                (
                    (
                        format!(
                            "{{\
                            \n\t\"request_type\": \"store\",\
                            \n\t\"key\": \"{key}\",\
                            \n\t\"hash\": \"{hash:0>32X}\"\
                            \n}}"
                        )
                        .as_bytes()
                        .to_vec(),
                        b"{\
                        \n\t\"response_status\": \"success\"\
                        \n}"
                        .to_vec(),
                    ),
                    (
                        format!(
                            "{{\
                            \n \"request_type\": \"load\",\
                            \n\t\"key\": \"{key}\"\
                            \n}}",
                        )
                        .as_bytes()
                        .to_vec(),
                        format!(
                            "{{\
                            \n\t\"response_status\": \"success\",\
                            \n\t\"requested_key\": \"{key}\",\
                            \n\t\"requested_hash\": \"{hash:0>32X}\"\
                            \n}}",
                        )
                        .as_bytes()
                        .to_vec(),
                    ),
                )
            })
            .unzip()
    }

    pub fn gen_requests_failed_load(
        index_begin: usize,
        index_end: usize,
    ) -> Vec<(Vec<u8>, Vec<u8>)> {
        (index_begin..index_end)
            .map(|index| {
                (
                    format!(
                        "{{\
                        \n\t\"request_type\": \"load\",\
                        \n\t\"key\": \"corrupted-{index}\"\
                        \n}}",
                    )
                    .as_bytes()
                    .to_vec(),
                    b"{\
                    \n\t\"response_status\": \"key not found\"\
                    \n}"
                    .to_vec(),
                )
            })
            .collect()
    }
}
