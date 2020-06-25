#![warn(rust_2018_idioms)]
#![forbid(unsafe_code)]
use anyhow::{Context, Result};
use log::{warn, Level};
use ping_pong::{run_dialer, run_listener, tor, Opt};
use structopt::StructOpt;

const ADDR: &str = "/ip4/127.0.0.1/tcp/7777";

#[tokio::main]
async fn main() -> Result<()> {
    simple_logger::init_with_level(Level::Info).unwrap();

    let opt = Opt::from_args();

    let addr = opt.address.unwrap_or_else(|| ADDR.to_string());
    let addr = addr
        .parse()
        .with_context(|| format!("failed to parse multiaddr: {}", addr))?;

    if opt.start_tor {
        tor::start_tor_instance()?;
    }

    if opt.dialer {
        run_dialer(addr).await?;
    } else {
        run_listener(addr).await?;
    }

    Ok(())
}
