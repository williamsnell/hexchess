use hexchesscore::{moves, Board, Hexagon, get_valid_moves, register_move};
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
use serde::{Serialize, Deserialize};
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

#[derive(Serialize, Deserialize)]
#[serde(tag = "op")]
enum Message {
    GetMoves { user_id: String, hexagon: Hexagon},
    RegisterMove {user_id: String, start_hexagon: Hexagon, final_hexagon: Hexagon},
}

impl SessionHandler {
    fn new() -> SessionHandler {
        SessionHandler {
            sessions: HashMap::<Uuid, Board>::new(),
        }
    }

    fn get_session_if_exists(&mut self, user_id: Uuid) -> Option<&mut Session> {
        self.sessions.get_mut(&user_id)
    }

    fn add_session(&mut self, user_id: Uuid) -> &mut Board {
        let board = Board::setup_default_board();
        self.sessions.insert(user_id, board);
        self.sessions.get_mut(&user_id).expect("board wasn't added for some reason")
    }
}

fn handle_websocket(mut websocket: WebSocket<TcpStream>, sessions: Arc<Mutex<SessionHandler>>) {
    loop {
        let message = websocket.read().unwrap();

        if let Text(message) = message {
            println!("{:?}", message);
            let decoded: Message = serde_json::from_str(message.as_str()).unwrap();

            match decoded {
                Message::GetMoves { user_id, hexagon } => {          
                    let user_id = Uuid::parse_str(&user_id).unwrap();      
                    let mut session = sessions.lock().unwrap();
    
                    let board: &Board;
                    
                    if let Some(valid_session) = session.get_session_if_exists(user_id) {
                        board = valid_session;                
                    } else {
                        board = session.add_session(user_id);
                    }
                    
                    // try and process the move
                    if let Some(piece) = board.occupied_squares.get(&hexagon) {
                        // match piece type to valid moves
                        let moves = get_valid_moves(&hexagon, &piece, &board);
                        let moves_json = serde_json::to_string(&moves).unwrap();
                        let json = format!("{{\"moves\": {moves_json}}}");
                        websocket.send(Text(json)).unwrap();
                    }      
                    drop(session);},
                Message::RegisterMove { user_id, start_hexagon, final_hexagon } => {
                    let user_id = Uuid::parse_str(&user_id).unwrap();      
                    let mut session = sessions.lock().unwrap();
    
                    let board: &mut Board;
                    
                    if let Some(valid_session) = session.get_session_if_exists(user_id) {
                        board = valid_session;                
                    } else {
                        board = session.add_session(user_id);
                    }
                    
                    // try and process the move
                    if let Some(piece) = board.occupied_squares.get(&start_hexagon) {
                        // match piece type to valid moves
                        let moves = get_valid_moves(&start_hexagon, &piece, board);
                        
                        if moves.contains(&final_hexagon) {
                            register_move(&start_hexagon, &final_hexagon, board);
                        }
                    }

                    // send back the board state
                    let new_board_state = serde_json::to_string(board).expect("failed to serialize board");
                    websocket.send(Text(new_board_state));      
                    drop(session);
                }
            }
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
