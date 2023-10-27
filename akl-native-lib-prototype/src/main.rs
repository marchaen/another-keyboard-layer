mod debugger;
mod events;
mod translation;

use std::{error::Error, net::Ipv4Addr};

#[cfg(not(target_os = "windows"))]
use x11rb::{
    connection::Connection,
    protocol::{xproto::{ConnectionExt, WindowClass}, Event},
    COPY_DEPTH_FROM_PARENT,
};

use argh::FromArgs;
use log::info;

/// Prototype for the linux core system lib implementation.
#[derive(FromArgs)]
struct ProgramArgs {
    /// sets the ip address of the debug server
    #[argh(option, short = 'd')]
    debug_server: Ipv4Addr,
}

#[cfg(target_os = "windows")]
fn main() {
    println!("Currently not testing for windows!");
}

#[cfg(not(target_os = "windows"))]
fn main() -> Result<(), Box<dyn Error>> {
    let args: ProgramArgs = argh::from_env();
    debugger::init(args.debug_server);
    println!("Initialized.");

    let (x11, screen_num) = x11rb::connect(None)?;

    let screen = &x11.setup().roots[screen_num];
    info!("Screen: {screen:?}");

    info!("Try to start event loop.");

    while let Ok(Some(event)) = x11.poll_for_event() {
        info!("Event: {event:#?}");
    }

    Ok(())
}
