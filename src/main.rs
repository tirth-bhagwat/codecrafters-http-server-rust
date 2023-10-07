use std::io::{Read, Write};
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let mut data = vec![0; 512];
                _stream.read(&mut data).unwrap();
                if String::from_utf8(data).unwrap().starts_with("GET / HTTP/1.1") {
                    _stream.write(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
                } else {
                    _stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
