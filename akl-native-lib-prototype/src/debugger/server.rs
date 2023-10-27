use std::{
    io::{BufRead, BufReader},
    net::TcpListener,
};

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7777")
        .expect("Should be able to open debug server on port 7777.");

    println!("Start listening for connections");

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let address = stream
            .peer_addr()
            .expect("Should be able to get local client address.");

        println!("Connected ({address})");

        let mut line = String::new();
        let mut reader = BufReader::new(&mut stream);

        while let Ok(bytes) = reader.read_line(&mut line) {
            if bytes == 0 {
                break;
            }

            print!("({address}) {line}");
            line.clear();
        }

        println!("Disconnected ({address})");
    }
}
