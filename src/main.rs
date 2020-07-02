#![warn(rust_2018_idioms)]
#![forbid(unsafe_code)]
use anyhow::{bail, Context, Result};
use libp2p::Multiaddr;
use log::{warn, Level};
use structopt::StructOpt;

use ping_pong::{run_dialer, run_listener, OnionAddr, Opt};

/// The ping-pong onion service address.
const ONION: &str = "/onion3/r4nttccifklkruvrztwxuhk2iy4xx7cnnex2sgogbo4zw6rnx3cq2bid:7";

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
        let onion = multiaddr(ONION)?;
        run_listener(onion).await?;
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
