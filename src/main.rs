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

fn main() {
    // TermLogger::init(
    //     LevelFilter::Info,
    //     Config::default(),
    //     TerminalMode::Stderr,
    //     ColorChoice::Auto,
    // )
    // .unwrap();

    let opts = Opts::parse();
    Server::start(opts.ip, opts.port).run().unwrap();
}
