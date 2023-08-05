use hexchesscore::{moves, Hexagon};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, self};
use std::thread::spawn;
use std::{
    fs,
    io::{prelude::*, BufReader},
};
use tungstenite::{accept, Message::Text, WebSocket};

fn handle_websocket(mut websocket: WebSocket<TcpStream>) {
    loop {
        let msg = websocket.read().unwrap();
        if msg.is_text() {
            let rook_moves: Vec<Hexagon> =
                // moves::RookMoves::new(Hexagon::new(msg.to_text().expect("not text")).unwrap()).collect();
                moves::KnightMoves::new(Hexagon::new(msg.to_text().expect("not text")).unwrap()).collect();
            let moves_json = serde_json::to_string(&rook_moves).unwrap();
            let json = format!("{{\"moves\": {moves_json}}}");
            websocket.send(Text(json)).unwrap();
        }
    }
}

fn handle_tcp_stream(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    if let Some(valid_http_request) = request_line.split("GET ").last()
                                        .expect("no GET in header")
                                        .split(" HTTP/1.1").next()
        {
            let status_line;
            let filename;
            if valid_http_request == "/".to_string() {
                status_line = "HTTP/1.1 200 OK";
                filename = String::from("server_files/hello.html");
            }
            else if Path::new(&(String::from("server_files") + valid_http_request)).exists() {
                status_line = "HTTP/1.1 200 OK";
                filename = String::from("server_files") + valid_http_request;
            } else {
                status_line = "HTTP/1.1 404 NOT FOUND";
                filename = String::from("server_files/404.html");
            }
            let contents = fs::read_to_string(&filename).unwrap();
            let length = contents.len();
        
            let content_type = match Path::new(&filename).extension().unwrap().to_str() {
                Some("js") => "text/javascript",
                Some("html") => "text/html",
                Some("jpg") => "text/jpeg",
                Some("svg") => "image/svg+xml",
                _ => ""
            };

            let response = format!("{status_line}\r\nContent-Length: {length}\r\nContent-Type: {content_type}\r\n\r\n{contents}");
            stream.write_all(response.as_bytes()).unwrap();

        }
}

fn main() {
    let websocket_server = TcpListener::bind("127.0.0.1:8080").unwrap();
    let server = TcpListener::bind("127.0.0.1:7878").unwrap();

    spawn(move || loop {
        for stream in websocket_server.incoming() {
            spawn(|| {
                let mut websocket = accept(stream.unwrap()).unwrap();
                handle_websocket(websocket);
            });
        }
    });

    spawn(move || loop {
        for stream in server.incoming() {
            spawn(move || {
                handle_tcp_stream(stream.unwrap());
            });
        }
    });

    loop {}
}
