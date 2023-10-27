use std::{
    io::{BufRead, BufReader},
    net::{TcpListener, TcpStream},
    thread,
};

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7777")
        .expect("Should be able to open debug server on port 7777.");

    println!(
        "Start listening for connections on {}",
        listener
            .local_addr()
            .expect("Address should be getting returned when binding is successful.")
    );

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_connection(stream));
            }
            Err(error) => {
                println!("A connection failed to be established: {error}");
            }
        }
    }
}

fn handle_connection(mut connection: TcpStream) {
    let address = connection
        .peer_addr()
        .expect("Should be able to get local client address.");

    println!("Connected ({address})");

    let mut line = String::new();
    let mut reader = BufReader::new(&mut connection);

    while let Ok(bytes) = reader.read_line(&mut line) {
        if bytes == 0 {
            break;
        }

        print!("({address}) {line}");
        line.clear();
    }

    println!("Disconnected ({address})");
}
