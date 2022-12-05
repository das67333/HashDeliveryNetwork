use clap::Parser;
use std::{
    net::{AddrParseError, IpAddr, SocketAddr},
    str::FromStr,
};

#[derive(Debug, Parser)]
pub struct ClArgs {
    #[clap(short, long, default_value = "127.0.0.1")]
    ip: String,

    #[clap(short, long, default_value = "0")]
    port: u16,
}

impl ClArgs {
    /// Returns network address from command line arguments.
    ///
    /// If no matching arguments are provided, IPv4 is used,
    /// the ip is set to `127.0.0.1` and the OS assigns the port.
    pub fn get_addr(&self) -> Result<SocketAddr, AddrParseError> {
        Ok(SocketAddr::from((IpAddr::from_str(&self.ip)?, self.port)))
    }
}
