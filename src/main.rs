use anyhow::{bail, Result};
use clap::{App, Arg};

mod dialer;
mod listener;

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

    if matches.is_present("dialer") {
        let addr = match matches.value_of("address") {
            None => bail!("IP address required to run ping-pong as dialer"),
            Some(addr) => addr,
        };

        let addr = match addr.parse() {
            Err(e) => bail!("failed to parse multiaddr: {:?}", e),
            Ok(addr) => addr,
        };

        dialer::run(addr)
    } else {
        listener::run()
    }
}
