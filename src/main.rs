use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::{env, fs, thread};

use itertools::Itertools;

enum RequestType {
    Blank,
    Echo(String),
    UserAgent,
    Error,
    File(String),
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let args = env::args().collect_vec();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let args = args.clone();
                thread::spawn(move || {
                    process_stream(&mut _stream, args);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

pub fn process_stream(mut _stream: &mut TcpStream, args: Vec<String>) {
    let mut directory: Option<PathBuf> = None;
    if args.len() > 2 && args[1] == "--directory" {
        directory = Some(PathBuf::from(&args[2]));
    }

    let mut data = vec![0; 512];
    _stream.read(&mut data).unwrap();
    let start_line: String = String::from_utf8(data.clone())
        .unwrap()
        .split("\r\n")
        .into_iter()
        .take(1)
        .collect();

    match get_request_type(&start_line) {
        RequestType::Blank => {
            _stream.write(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
        }
        RequestType::Echo(str) => {
            respond_with_msg(&mut _stream, &str, "text/plain").unwrap();
        }
        RequestType::UserAgent => {
            let headers: String = String::from_utf8(data)
                .unwrap()
                .split("\r\n")
                .into_iter()
                .skip(1)
                .filter_map(|x| {
                    if x.starts_with("User-Agent: ") {
                        return Some(x.replace("User-Agent: ", ""));
                    }
                    return None;
                })
                .collect();

            respond_with_msg(&mut _stream, &headers, "text/plain").unwrap();
        }
        RequestType::File(filename) => {
            if let Some(mut dir) = directory {
                dir.push(&filename);
                if dir.exists() && dir.is_file() {
                    let file_data = fs::read(dir).unwrap();
                    respond_with_msg(
                        _stream,
                        String::from_utf8(file_data).unwrap().as_str(),
                        "application/octet-stream"
                    ).unwrap();
                } else {
                    _stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
                }
            } else {
                _stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
            }
        }
        RequestType::Error => {
            _stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
        }
    }
}

fn respond_with_msg(
    _stream: &mut dyn Write,
    msg: &str,
    content_type: &str,
) -> Result<(), std::io::Error> {
    let resp = format!(
        "HTTP/1.1 200 OK\r\n\
        Content-Type: {}\r\n\
        Content-Length: {}\r\n\
        \r\n\
        {}\r\n",
        content_type,
        msg.len(),
        msg
    );
    _stream.write(resp.as_bytes())?;
    Ok(())
}

fn get_request_type(request: &str) -> RequestType {
    let parts: Vec<String> = request.split(" ").map(|x| x.to_string()).collect();
    if parts[0] != "GET" || parts[2] != "HTTP/1.1" {
        return RequestType::Error;
    }

    let path = parts[1].split("/").collect_vec();
    return match path[1] {
        "" => RequestType::Blank,
        "echo" => RequestType::Echo(path[2..].join("/").to_string()),
        "user-agent" => RequestType::UserAgent,
        "files" => RequestType::File(path[2..].join("/").to_string()),
        _ => RequestType::Error,
    };
}
