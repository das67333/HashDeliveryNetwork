// use simplelog::*;
use hash_delivery_network::client::Client;
use indoc::indoc;
use std::{
    collections::HashMap,
    net::{IpAddr, Shutdown, SocketAddr, TcpListener},
    process::{Child, Command},
    str::FromStr,
    thread, time,
};

#[allow(dead_code)]
enum IpVersion {
    V4,
    V6,
}

struct ServerWrapper {
    proc: Option<Child>,
    addr: SocketAddr,
}

#[allow(dead_code)]
impl ServerWrapper {
    fn start(ip_version: IpVersion) -> Self {
        // TermLogger::init(
        //     LevelFilter::Info,
        //     Config::default(),
        //     TerminalMode::Stderr,
        //     ColorChoice::Auto,
        // )
        // .unwrap();
        let ip = match ip_version {
            IpVersion::V4 => IpAddr::from_str("127.0.0.1").unwrap(),
            IpVersion::V6 => IpAddr::from_str("::1").unwrap(),
        };
        let port = TcpListener::bind((ip, 0))
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let proc = Command::new(env!("CARGO_BIN_EXE_hash-delivery-network"))
            .arg("--ip")
            .arg(ip.to_string())
            .arg("--port")
            .arg(port.to_string())
            .spawn()
            .unwrap();
        thread::sleep(time::Duration::from_millis(10));
        Self {
            proc: Some(proc),
            addr: SocketAddr::new(ip, port),
        }
    }

    fn is_alive(&mut self) -> bool {
        self.proc
            .as_mut()
            .map_or(false, |proc| proc.try_wait().unwrap().is_none())
    }

    fn expected_to_be_dead(&mut self) {
        let _ = self.stop();
    }

    fn stop(&mut self) -> std::io::Result<()> {
        self.proc.take().map_or(Ok(()), |mut proc| proc.kill())
    }
}

impl Drop for ServerWrapper {
    fn drop(&mut self) {
        let _ = self.stop().unwrap();
    }
}

fn are_json_equal(data1: &[u8], data2: &[u8]) -> bool {
    let map1: HashMap<String, String> =
        serde_json::from_str(&String::from_utf8(data1.to_vec()).unwrap()).unwrap();
    let map2: HashMap<String, String> =
        serde_json::from_str(&String::from_utf8(data2.to_vec()).unwrap()).unwrap();
    map1.len() == map2.len() && map1.keys().all(|k| map1.get(k) == map2.get(k))
}

#[test]
fn test_ddos() {
    let server = ServerWrapper::start(IpVersion::V4);

    let n = 1_000;
    let mut clients: Vec<Client> = (0..n)
        .into_iter()
        .filter_map(|i| Client::start(i, server.addr).ok())
        .collect();
    for client in clients.iter_mut() {
        let _greeting = client.read().unwrap();
        let message = format!("client {}: corrupted json", client.id);
        client.write(message.as_bytes()).unwrap();
    }
    for client in clients.iter_mut() {
        client.shutdown(Shutdown::Both);
    }
}

#[test]
fn test_store_and_load() {
    let server = ServerWrapper::start(IpVersion::V4);

    let mut client = Client::start(0, server.addr).unwrap();

    // test greeting
    {
        assert!(are_json_equal(
            indoc! {
            b"{
              \"student_name\": \"Your Name\"
            }"},
            &client.read().unwrap()
        ));
    }
    // test failed load
    {
        let message = indoc! {
        b"{
          \"request_type\": \"load\",
          \"key\": \"some_key\"
        }"};
        client.write(message).unwrap();
        assert!(are_json_equal(
            indoc! {
            b"{
              \"response_status\": \"key not found\"
            }"
            },
            &client.read().unwrap()
        ));
    }
    // test store
    {
        let message = indoc! {
        b"{
          \"request_type\": \"store\",
          \"key\": \"some_key\",
          \"hash\": \"0b672dd94fd3da6a8d404b66ee3f0c83\"
        }"};
        client.write(message).unwrap();
        assert!(are_json_equal(
            indoc! {
            b"{
              \"response_status\": \"success\"
            }"
            },
            &client.read().unwrap()
        ));
    }
    // test successful load
    {
        let message = indoc! {
        b"{
          \"request_type\": \"load\",
          \"key\": \"some_key\"
        }"};
        client.write(message).unwrap();
        assert!(are_json_equal(
            indoc! {
                b"{
                  \"response_status\": \"success\",
                  \"requested_key\": \"some_key\",
                  \"requested_hash\": \"0b672dd94fd3da6a8d404b66ee3f0c83\"
                }"
            },
            &client.read().unwrap()
        ));
    }
    client.shutdown(Shutdown::Both);
}
