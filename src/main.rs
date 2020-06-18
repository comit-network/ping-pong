use clap::{App, Arg};
use std::process;

mod dialer;
mod listener;

fn main() {
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
        if !matches.is_present("address") {
            eprintln!("IP address required to run ping-pong as dialer");
            process::exit(1);
        }
        dialer::run(matches.value_of("address").unwrap())
    } else {
        listener::run()
    }
}
