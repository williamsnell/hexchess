use hexchesscore::{moves, Board, Hexagon};
use tungstenite::http::HeaderValue;
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::path::{self, Path};
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::{
    fs,
    io::{prelude::*, BufReader},
};
use serde_json::from_str;
use tungstenite::{
    accept,
    handshake::server::{ErrorResponse, Request, Response},
    Message::Text,
    WebSocket,
};
use uuid::Uuid;

pub struct SessionId {
    uuid: Uuid,
}

type Session = Board;

pub struct SessionHandler {
    sessions: HashMap<Uuid, Session>,
}

impl SessionHandler {
    fn new() -> SessionHandler {
        SessionHandler {
            sessions: HashMap::<Uuid, Board>::new(),
        }
    }

    fn get_session_if_exists(&self, user_id: Uuid) -> Option<&Session> {
        self.sessions.get(&user_id)
    }

    fn add_session(&mut self, user_id: Uuid) {
        self.sessions.insert(user_id, Board::setup_default_board());
    }
}



// fn initialize_session(sessions: SessionHandler) -> SessionId {
//     let uuid = Uuid::new_v4();

// }

// fn handle_web_connection(msg: Message) {

// }

fn handle_websocket(mut websocket: WebSocket<TcpStream>, sessions: Arc<Mutex<SessionHandler>>) {
    let binding = websocket.read().unwrap();
    let user_id = Uuid::parse_str(binding.to_text().unwrap()).unwrap();
    println!("here");
    let mut session = sessions.lock().unwrap();

    if let Some(valid_session) = session.get_session_if_exists(user_id) {
        println!("here");
        let board = valid_session;
    } else {
        session.add_session(user_id);
    }

    drop(session);

    loop {
        let msg = websocket.read().unwrap();
        println!("{:?}", msg);
        if msg.is_text() {
            let rook_moves: Vec<Hexagon> =
                // moves::RookMoves::new(Hexagon::new(msg.to_text().expect("not text")).unwrap()).collect();
                moves::KnightMoves::new(Hexagon::new(msg.to_text().expect("not text")).unwrap()).collect();
            let moves_json = serde_json::to_string(&rook_moves).unwrap();
            println!("{:?}", moves_json);
            let json = format!("{{\"moves\": {moves_json}}}");
            websocket.send(Text(json)).unwrap();
        }
    }
}

fn handle_tcp_stream(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    if let Some(valid_http_request) = request_line
        .split("GET ")
        .last()
        .expect("no GET in header")
        .split(" HTTP/1.1")
        .next()
    {
        let status_line;
        let filename;
        if valid_http_request == "/".to_string() {
            status_line = "HTTP/1.1 200 OK";
            filename = String::from("server_files/hello.html");
        } else if Path::new(&(String::from("server_files") + valid_http_request)).exists() {
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
            _ => "",
        };

        let response = format!("{status_line}\r\nContent-Length: {length}\r\nContent-Type: {content_type}\r\n\r\n{contents}");
        stream.write_all(response.as_bytes()).unwrap();
    }
}

fn main() {
    let websocket_server = TcpListener::bind("127.0.0.1:7979").unwrap();
    let server = TcpListener::bind("127.0.0.1:7878").unwrap();

    let sessions: Arc<Mutex<SessionHandler>> = Arc::new(Mutex::new(SessionHandler::new()));

    spawn(move || loop {
        for stream in websocket_server.incoming() {
            let new_sessions = Arc::clone(&sessions);
            spawn(|| {
                let websocket = accept(stream.unwrap())
                .unwrap();
                handle_websocket(websocket, new_sessions);
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
