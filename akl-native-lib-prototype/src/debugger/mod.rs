use std::{io::Write, net::TcpStream, sync::Mutex};

static mut DEBUGGER: Option<Debugger> = None;

pub struct Debugger(Mutex<TcpStream>);

impl Debugger {
    pub fn init() {
        unsafe {
            DEBUGGER = Some(Self(Mutex::new(
                TcpStream::connect("127.0.0.1:7777")
                    .expect("Debug server should run for prototyping the native lib."),
            )))
        };

        Debugger::write("Successfully connected.");
    }

    pub fn destroy() {
        Debugger::write("Ending connection.");
        let _ = unsafe { DEBUGGER.take() };
    }

    pub fn write(line: &str) {
        let mut line = line.to_owned();
        line.push('\n');

        // Instead of messing with the rest of the program it's better to just
        // not handle any errors that could have occurred while writing to the
        // tcp stream.
        let _ = unsafe {
            DEBUGGER
                .as_mut()
                .expect("Connection to debug server should have been established.")
                .0
                .lock()
                .expect("No other thread will ever panic on this single write call.")
                .write(line.as_bytes())
        };
    }
}
