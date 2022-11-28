#[allow(dead_code)]
pub mod tests {
    use rand::Rng;
    use std::{
        io::Result,
        net::{IpAddr, SocketAddr, TcpListener},
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
                            \n  \"request_type\": \"store\",\
                            \n  \"key\": \"{}\",\
                            \n  \"hash\": \"{:0>32X}\"\
                            \n}}",
                            key, hash
                        )
                        .as_bytes()
                        .to_vec(),
                        b"{\
                        \n  \"response_status\": \"success\"\
                        \n}"
                        .to_vec(),
                    ),
                    (
                        format!(
                            "{{\
                            \n \"request_type\": \"load\",\
                            \n  \"key\": \"{}\"\
                            \n}}",
                            key
                        )
                        .as_bytes()
                        .to_vec(),
                        format!(
                            "{{\
                            \n  \"response_status\": \"success\",\
                            \n  \"requested_key\": \"{}\",\
                            \n  \"requested_hash\": \"{:0>32X}\"\
                            \n}}",
                            key, hash
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
                        \n  \"request_type\": \"load\",\
                        \n  \"key\": \"corrupted-{}\"\
                        \n}}",
                        index
                    )
                    .as_bytes()
                    .to_vec(),
                    b"{\
                    \n  \"response_status\": \"key not found\"\
                    \n}"
                    .to_vec(),
                )
            })
            .collect()
    }
}
