//! The `killport` command-line utility is designed to kill processes
//! listening on specified ports.
//!
//! The utility accepts a list of port numbers as input and attempts to
//! terminate any processes listening on those ports.

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
use linux::kill_processes_by_port;
#[cfg(target_os = "macos")]
use macos::kill_processes_by_port;

use clap::Parser;
use clap_verbosity_flag::{Verbosity, WarnLevel};
use log::error;
use nix::sys::signal::Signal;
use std::{process::exit, str::FromStr};

/// `killport` utility.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct KillPortArgs {
    /// A list of port numbers to kill processes on.
    #[arg(
        name = "ports",
        help = "The list of port numbers to kill processes on",
        required = true
    )]
    ports: Vec<u16>,

    /// An option to specify the type of signal to be sent.
    #[arg(
        long,
        short = 's',
        name = "SIG",
        help = "SIG is a signal name",
        default_value = "sigterm",
        value_parser = parse_signal
    )]
    signal: Signal,

    /// A verbosity flag to control the level of logging output.
    #[command(flatten)]
    verbose: Verbosity<WarnLevel>,
}

fn parse_signal(arg: &str) -> Result<Signal, std::io::Error> {
    let str_arg = arg.parse::<String>();
    match str_arg {
        Ok(str_arg) => {
            let signal_str = str_arg.to_uppercase();
            let signal = Signal::from_str(signal_str.as_str())?;
            return Ok(signal)
        },
        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

/// The `main` function is the entry point of the `killport` utility.
///
/// It parses command-line arguments, sets up the logging environment, and
/// attempts to kill processes listening on the specified ports.
fn main() {
    // Parse command-line arguments
    let args = KillPortArgs::parse();

    // Set up logging environment
    let log_level = args
        .verbose
        .log_level()
        .map(|level| level.to_level_filter())
        .unwrap();

    env_logger::Builder::new()
        .format_module_path(log_level == log::LevelFilter::Trace)
        .format_target(log_level == log::LevelFilter::Trace)
        .format_timestamp(Option::None)
        .filter_level(log_level)
        .init();

    // Determine a signal to be sent.
    // If an option for signal number is added, we can determine a signal to be sent by signal number.
    let signal = args.signal;

    // Attempt to kill processes listening on specified ports
    for port in args.ports {
        match kill_processes_by_port(port, signal) {
            Ok(killed) => {
                if killed {
                    println!("Successfully killed process listening on port {}", port);
                } else {
                    println!("No processes found using port {}", port);
                }
            }
            Err(err) => {
                error!("{}", err);
                exit(1);
            }
        }
    }
}
