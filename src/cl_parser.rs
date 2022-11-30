use clap::Parser;
use std::net::{IpAddr, SocketAddr};

#[derive(Debug, Parser)]
pub struct ClArgs {
    #[clap(short, long, default_value = None)]
    ip: Option<IpAddr>,

    #[clap(short, long, default_value = "0")]
    port: u16,
}

impl ClArgs {
    /// Returns network address from command line arguments.
    pub fn get_addr(&self) -> SocketAddr {
        let ip = if let Some(val) = self.ip {
            val
        } else {
            IpAddr::from([127, 0, 0, 1])
        };
        SocketAddr::from((ip, self.port))
    }
}
