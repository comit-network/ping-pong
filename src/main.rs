#![warn(rust_2018_idioms)]
#![forbid(unsafe_code)]
use anyhow::{Context, Result};
use async_std::task;
use clap::{App, Arg};
use ping_pong::{dialer, listener};

const ADDR: &str = "/ip4/127.0.0.1/tcp/4444";

fn main() -> Result<()> {
    let matches = App::new("ping-pong")
        .version("0.1")
        .arg(
            Arg::with_name("dialer")
                .help("Run as the dialer i.e., do the ping")
                .long("dialer")
                .short("d"),
        )
        .arg(
            Arg::with_name("listener")
                .help("Run as the listener i.e., do the pong [default]")
                .long("listener")
                .short("l"),
        )
        .args(&[Arg::with_name("address")
            .help("IP address to use")
            .index(1)
            .required(false)])
        .get_matches();

    let addr = matches.value_of("address").unwrap_or(ADDR);

    let addr = addr
        .parse()
        .with_context(|| format!("failed to parse multiaddr: {}", addr))?;

    if matches.is_present("dialer") {
        task::block_on(dialer::run(addr)).expect("failed to run dialer");
    } else {
        task::block_on(listener::run(addr)).expect("failed to run listener");
    }

    Ok(())
}
