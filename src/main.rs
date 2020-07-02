#![warn(rust_2018_idioms)]
#![forbid(unsafe_code)]
use anyhow::{bail, Context, Result};
use libp2p::Multiaddr;
use log::{warn, Level};
use structopt::StructOpt;

use ping_pong::{run_dialer, run_listener, OnionAddr, Opt};

/// Local ping-pong server address.
const LISTENER_ADDR: &str = "/ip4/127.0.0.1/tcp/7777";
/// Local port as well as the onion service port.
const PORT: u16 = 7777;

#[tokio::main]
async fn main() -> Result<()> {
    simple_logger::init_with_level(Level::Debug).unwrap();

    let opt = Opt::from_args();

    if opt.dialer {
        let addr = match opt.onion {
            Some(addr) => addr,
            None => bail!("onion address required to dial"),
        };

        let addr = multiaddr(&addr)?;
        if !is_valid_onion_addr(addr.clone()) {
            bail!("invalid multiaddr: {}", addr);
        }

        run_dialer(addr).await?;
    } else {
        let addr = multiaddr(LISTENER_ADDR)?;
        run_listener(addr, PORT).await?;
    }

    Ok(())
}

fn multiaddr(s: &str) -> Result<Multiaddr> {
    let addr = s
        .parse()
        .with_context(|| format!("failed to parse multiaddr: {}", s))?;
    Ok(addr)
}

fn is_valid_onion_addr(multi: Multiaddr) -> bool {
    let onion = OnionAddr::from_multiaddr(multi);
    onion.is_ok()
}
