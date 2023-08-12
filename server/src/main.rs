use futures::{FutureExt, StreamExt, SinkExt, TryFutureExt};
use hexchesscore::{get_valid_moves, moves, register_move, Board, Hexagon};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::Message;
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::path::{self, Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::{
    fs,
    io::{prelude::*, BufReader},
};
use tokio;
use uuid::Uuid;
use warp::Filter;

pub struct SessionId {
    uuid: Uuid,
}

type Session = Board;

pub struct SessionHandler {
    sessions: HashMap<Uuid, Session>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "op")]
enum IncomingMessage {
    GetBoard {
        user_id: String,
    },
    GetMoves {
        user_id: String,
        hexagon: Hexagon,
    },
    RegisterMove {
        user_id: String,
        start_hexagon: Hexagon,
        final_hexagon: Hexagon,
    },
}

#[derive(Serialize, Debug)]
#[serde(tag = "op")]
enum OutgoingMessage<'a> {
    ValidMoves { moves: &'a Vec<Hexagon> },
    BoardState { board: &'a Board },
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
        self.sessions
            .get_mut(&user_id)
            .expect("board wasn't added for some reason")
    }
}

async fn handle_websocket_async(websocket: warp::ws::WebSocket, sessions: Arc<RwLock<SessionHandler>>) {
    // split the socket into a sender and a receiver
    let (mut ws_tx, mut ws_rx) = websocket.split();

    // use an unbounded channel to allow communication from within the tasks we're about to spawn,
    // back to the main thread that holds the actual websocket transmitter and receiver.
    //
    // everytime we send a message into tx, it will appear in rx which will then forward the message
    // along to the websocket (and back to the client)
    let (tx, rx) = mpsc::unbounded_channel();
    // turn the normal receiver into a stream
    let mut rx = UnboundedReceiverStream::new(rx);

    ws_tx.send(Message::text("hi")).await;

    // spawn a task that will do the sending for us
    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            ws_tx
            .send(message)
            .unwrap_or_else(|e| {
                eprintln!("websocket send error: {}", e);
            })
            .await;
        }
    });

    // Listen for messages from the client, and do something with them
    while let Some(result) = ws_rx.next().await {
        let message = match result {
            Ok(message) => message,
            Err(e) => {
                eprintln!("websocket error {}", e);
                break;
            }
        };
        if message.is_text() {
            let decoded: IncomingMessage = serde_json::from_str(message.to_str().unwrap()).unwrap();

            match decoded {
                IncomingMessage::GetBoard { user_id } => {
                    let user_id = Uuid::parse_str(&user_id).unwrap();

                    let board: &Board;

                    let mut session = sessions.write().await;

                    if let Some(valid_session) = session.get_session_if_exists(user_id) {
                        board = valid_session;
                    } else {
                        board = session.add_session(user_id);
                        // todo should release the write lock on sessions at this point
                    }
                    let message = OutgoingMessage::BoardState { board: board };
                    if let Ok(new_board_state) = serde_json::to_string(&message) {
                        tx.send(warp::ws::Message::text(new_board_state)).unwrap();
                    } else {
                        eprintln!("Failed to send board state");
                    }
                    drop(session);
                }
                IncomingMessage::GetMoves { user_id, hexagon } => {
                    let user_id = Uuid::parse_str(&user_id).unwrap();

                    let board: &Board;

                    let mut session = sessions.write().await;

                    if let Some(valid_session) = session.get_session_if_exists(user_id) {
                        board = valid_session;
                    } else {
                        board = session.add_session(user_id);
                    }
                    
                    // try and process the move
                    if let Some(piece) = board.occupied_squares.get(&hexagon) {
                        // match piece type to valid moves
                        let moves = get_valid_moves(&hexagon, &piece, &board);
                        
                        let outgoing = OutgoingMessage::ValidMoves { moves: &moves };
                        tx.send(warp::ws::Message::text(serde_json::to_string(&outgoing).unwrap()))
                        .unwrap();
                }
                drop(session);
                }
                IncomingMessage::RegisterMove {
                    user_id,
                    start_hexagon,
                    final_hexagon,
                } => {
                    let user_id = Uuid::parse_str(&user_id).unwrap();

                    let board: &mut Board;

                    let mut session = sessions.write().await;

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
                    drop(session);
                }
            }
        }
    };
}


#[tokio::main]
async fn main() {
    // handle the page-serving side of the website
    let default = warp::path::end().and(warp::fs::file("./server_files/hello.html"));

    let pages = warp::fs::dir("./server_files/");

    let sessions: Arc<RwLock<SessionHandler>> = Arc::new(RwLock::new(SessionHandler::new()));

    let sessions = warp::any().map(move || sessions.clone());

    let websocket =
        warp::path::end()
            .and(warp::ws())
            .and(sessions)
            .map(|ws: warp::ws::Ws, sessions| {
                ws.on_upgrade( move|socket| handle_websocket_async(socket, sessions)
                    
                    
                    //     {
                    //     let (tx, rx) = websocket.split();
                    //     rx.forward(tx).map(|result| {
                    //         if let Err(e) = result {
                    //             eprintln!("websocket error: {:?}", e);
                    //         }
                    //     })
                    // }
            )
            });

    let routes = pages
        .or(websocket)
        .or(default)
        // serve 404s if the file doesn't exist and the client isn't asking for the default page
        .or(warp::fs::file("./server_files/404.html"));

    warp::serve(routes)
        .tls()
        .cert_path("./cert/playhexchess.com.crt")
        .key_path("./cert/playhexchess.com.key")
        .run(([127, 0, 0, 1], 7878))
        .await;
}
