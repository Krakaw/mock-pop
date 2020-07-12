mod pop;
use clap::Clap;
use log::info;
use std::error::Error;
use std::net::SocketAddr;

#[derive(Clap)]
#[clap(
    version = "0.1",
    author = "Krakaw <41575888+Krakaw@users.noreply.github.com>"
)]
struct Opts {
    #[clap(subcommand)]
    pop: PopSubCommand,
}

#[derive(Clap)]
enum PopSubCommand {
    Pop(PopConfig),
}

/// POP3 server config
#[derive(Clap)]
struct PopConfig {
    /// Listening address, run as root for port 110
    #[clap(short, long, default_value = "127.0.0.1:110")]
    listen: SocketAddr,
    /// Do not start the pop server
    #[clap(short, long)]
    no_start: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let opts: Opts = Opts::parse();
    let (addr, no_pop_start) = match opts.pop {
        PopSubCommand::Pop(pop_config) => (pop_config.listen, pop_config.no_start),
    };
    if no_pop_start {
        info!("Not starting POP3 mock server");
    } else {
        pop::server::start(addr).await?;
    }

    Ok(())
}
