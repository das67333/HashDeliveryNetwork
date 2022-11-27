// use simplelog::*;
use hash_delivery_network::client::Client;
use indoc::indoc;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::{
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

#[macro_use]
extern crate bencher;

use bencher::Bencher;

fn single_client_store(bench: &mut Bencher) {
    let server = ServerWrapper::start(IpVersion::V4);
    let mut client = Client::start(0, server.addr).unwrap();
    let _greeting = client.read().unwrap();

    let mut rng = ChaCha8Rng::seed_from_u64(0);
    bench.iter(|| {
        let _ = client.write(
            format!(
                indoc! {
                "{{
                  \"request_type\": \"store\",
                  \"key\": \"{}\",
                  \"hash\": \"{:0>32X}\"
                }}"
                },
                rng.gen::<u32>(),
                rng.gen::<u128>()
            )
            .as_bytes(),
        );
        assert_eq!(
            indoc! {
            b"{
              \"response_status\": \"success\"
            }"},
            &client.read().unwrap()[..]
        );
    });

    client.shutdown(Shutdown::Both);
}

fn single_client_failed_load(bench: &mut Bencher) {
    let server = ServerWrapper::start(IpVersion::V4);
    let mut client = Client::start(0, server.addr).unwrap();
    let _greeting = client.read().unwrap();

    let mut rng = ChaCha8Rng::seed_from_u64(0);
    bench.iter(|| {
        let _ = client.write(
            format!(
                indoc! {
                    "{{
                      \"request_type\": \"load\",
                      \"key\": \"{}\"
                    }}"
                },
                rng.gen::<u32>()
            )
            .as_bytes(),
        );
        assert_eq!(
            indoc! {
            b"{
              \"response_status\": \"key not found\"
            }"
            },
            &client.read().unwrap()[..]
        );
    });

    client.shutdown(Shutdown::Both);
}

benchmark_group!(benches, single_client_store, single_client_failed_load);
benchmark_main!(benches);
