mod debugger;
mod events;
mod translation;

#[cfg(target_os = "linux")]
mod linux_main;

use std::{error::Error, net::Ipv4Addr};

use argh::FromArgs;

/// Prototype for the linux core system lib implementation.
#[derive(FromArgs)]
struct ProgramArgs {
    /// sets the ip address of the debug server
    #[argh(option, short = 'd')]
    debug_server: Ipv4Addr,
    /// prints all events in there raw format to the debug server
    #[argh(switch, short = 'a')]
    display_all_events: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: ProgramArgs = argh::from_env();
    debugger::init(args.debug_server);

    println!("Initialized.");

    #[cfg(target_os = "linux")]
    linux_main::main(args.display_all_events)?;

    #[cfg(windows)]
    println!("Currently not prototyping for windows");

    Ok(())
}
