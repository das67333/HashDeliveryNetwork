use clap::Parser;
use hash_delivery_network::cl_parser::ClArgs;
use hash_delivery_network::server::Server;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let args = ClArgs::parse();
    Server::new(args.get_addr()?)?.run()?;
    Ok(())
}
