use clap::{App, Arg, SubCommand};

mod dialer;
mod listener;

fn main() {
    let matches = App::new("ping-pong")
        .version("0.1")
        .subcommand(
            SubCommand::with_name("dialer")
                .about("Run application as a dialer, does the ping")
                .arg(
                    Arg::with_name("address")
                        .help("IP address to dial")
                        .index(1)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("listener").about("Run application as a listener, does the pong"),
        )
        .get_matches();

    match matches.subcommand_name() {
        Some("dialer") => dialer::run(),
        Some("listener") => listener::run(),
        None => println!("No subcommand was used: specify dialer or listener"),
        _ => println!("Unknown subcommand, pingpong -h for help"), // We never actually hit this.
    }
}
