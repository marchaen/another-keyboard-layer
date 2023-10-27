use std::net::Ipv4Addr;

use log::LevelFilter;
use simplelog::{ConfigBuilder, ThreadLogMode, WriteLogger};

pub fn init(server: Ipv4Addr) {
    let connection = std::net::TcpStream::connect((server, 7777));

    match connection {
        Ok(connection) => {
            let config = {
                match ConfigBuilder::new()
                    .set_thread_level(LevelFilter::Error)
                    .set_thread_mode(ThreadLogMode::Both)
                    .set_target_level(LevelFilter::Error)
                    .set_time_offset_to_local()
                {
                    Ok(config_builder) | Err(config_builder) => config_builder.build(),
                }
            };

            // Errors if the global logger is already initialized. Can be
            // ignored safely without any consequences.
            let _ = WriteLogger::init(LevelFilter::Trace, config, connection);
        }
        Err(error) => eprintln!("Connection failed: {}", error.to_string()),
    }
}
