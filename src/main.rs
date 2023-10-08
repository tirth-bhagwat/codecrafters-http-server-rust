use std::io::{Read, Write};
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let mut data = vec![0; 512];
                _stream.read(&mut data).unwrap();
                let request: String = String::from_utf8(data)
                    .unwrap()
                    .split("\r\n")
                    .into_iter()
                    .take(1)
                    .collect();

                // println!("{}", request);
                // println!("{:?}", request);
                if request.starts_with("GET /echo/") && request.ends_with(" HTTP/1.1") {
                    let req_len = request.len();
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\n\r\nContent-Type: text/plain\r\nContent-Length: {}\n\r\r\n{}\r\n",
                        req_len - 19,
                        request.get(10..req_len - 9).unwrap()
                    );
                    println!("op: \n{}", resp);
                    _stream.write(resp.as_bytes()).unwrap();
                } else {
                    _stream.write(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
