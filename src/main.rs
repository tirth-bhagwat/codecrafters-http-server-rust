use std::io::{Read, Write};
use std::net::TcpListener;

use itertools::Itertools;

enum RequestType {
    Blank,
    Echo(String),
    UserAgent,
    Error,
}


fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
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
                        respond_with_msg(&str, &mut _stream).unwrap();
                    }
                    RequestType::UserAgent => {
                        let headers: String = String::from_utf8(data)
                            .unwrap()
                            .split("\r\n")
                            .into_iter()
                            .skip(1)
                            .filter_map(
                                |x| {
                                    if x.starts_with("User-Agent: ") {
                                        return Some(x.replace("User-Agent: ", ""));
                                    }
                                    return None;
                                }
                            )
                            .collect();

                        respond_with_msg(&headers, &mut _stream).unwrap();
                    }
                    RequestType::Error => {
                        _stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn respond_with_msg(msg: &str, _stream: &mut dyn Write) -> Result<(), std::io::Error>
{
    let resp = format!(
        "HTTP/1.1 200 OK\r\n\
        Content-Type: text/plain\r\n\
        Content-Length: {}\r\n\
        \r\n\
        {}\r\n",
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


    let path = parts[1].split("/").take(2).collect_vec();
    return match path[1] {
        "" => { RequestType::Blank }
        "echo" => { RequestType::Echo(path[2..].join("/").to_string()) }
        "user-agent" => { RequestType::UserAgent }
        _ => { RequestType::Error }
    };
}
