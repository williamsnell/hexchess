use std::{
    fs,
    io::{BufReader, prelude::*},
};
use std::net::{TcpListener, TcpStream};
use std::thread::spawn;
use tungstenite::{
    WebSocket,
    accept,
    Message::Text,
};
use hexchesscore::Hexagon;

fn handle_websocket(mut websocket: WebSocket<TcpStream>) {
    loop {
        let msg = websocket.read().unwrap();
        if msg.is_text() {
            let rook_moves: Vec<Hexagon> =
                hexchesscore::moves::RookMoves::new(Hexagon::new(msg.to_text().expect("not text")).unwrap()).collect();
            let moves_json = serde_json::to_string(&rook_moves).unwrap();
            let json = format!("{{\"moves\": {moves_json}}}");
            websocket.send(Text(json)).unwrap();
        }
    }
}

fn handle_tcp_stream(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /draw_hexagon.js HTTP/1.1" => ("HTTP/1.1 200 OK", "draw_hexagon.js"),
        "GET /hex_frontend_funcs.js HTTP/1.1" => ("HTTP/1.1 200 OK", "hex_frontend_funcs.js"),
        "GET /horse.jpg HTTP/1.1" => ("HTTP/1.1 200 OK", "horse.jpg"),
        "GET /moves.json HTTP/1.1" => ("HTTP/1.1 200 OK", "moves.json"),
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let content_type = if filename.ends_with(".js") {
        "text/javascript"
    } else if filename.ends_with(".html") {
        "text/html"
    } else if filename.ends_with(".jpg") {
        "image/jpeg"
    } else {
        ""
    };

    let response = format!("{status_line}\r\nContent-Length: {length}\r\nContent-Type: {content_type}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}

fn main() {

    let websocket_server = TcpListener::bind("127.0.0.1:8080").unwrap();
    let server = TcpListener::bind("127.0.0.1:7878").unwrap();


    spawn(move 
        || {
        loop {
            for stream in websocket_server.incoming() {
                spawn (|| {          
                    let mut websocket = accept(stream.unwrap()).unwrap();
                    handle_websocket(websocket);
                });
            }
        }
    }
    );

    spawn(move || {
        loop {
            for stream in server.incoming() {
                spawn (move || {
                    handle_tcp_stream(stream.unwrap());
                    }
                );
            }
        }
    });

    loop {};
}