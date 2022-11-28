use clap::Parser;
use hash_delivery_network::server::Server;
use std::net::IpAddr;

#[derive(Debug, Parser)]
struct Opts {
    #[clap(short, long)]
    ip: IpAddr,

    #[clap(short, long, default_value = "0")]
    port: u16,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();
    Server::start(opts.ip, opts.port).await.run().await.unwrap();
}
