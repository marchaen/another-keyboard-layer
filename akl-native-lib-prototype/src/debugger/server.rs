use std::{
    io::{BufRead, BufReader},
    net::{TcpListener, TcpStream},
    thread,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7777")
        .expect("Should be able to open debug server on port 7777.");

    println!("Start listening for connections");

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
