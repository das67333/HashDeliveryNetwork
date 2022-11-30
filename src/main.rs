use clap::Parser;
use hash_delivery_network::cl_parser::ClArgs;
use hash_delivery_network::server::Server;

#[tokio::main]
async fn main() {
    let args = ClArgs::parse();
    Server::start(args.get_addr()).await.run().await.unwrap();
}
