
use futures::{SinkExt, StreamExt, TryFutureExt};
use hexchesscore::{get_valid_moves, register_move, Color};
use serde::{Deserialize, Serialize};

use server::session_handling;
use warp::ws::Message;
use std::{collections::HashMap, sync::Arc};
use tokio;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::Filter;

async fn handle_websocket_async(
    websocket: warp::ws::WebSocket,
    sessions: Arc<RwLock<session_handling::SessionHandler>>,
) {
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
            let decoded: session_handling::IncomingMessage = serde_json::from_str(message.to_str().unwrap()).unwrap();

            match decoded {
                session_handling::IncomingMessage::CreateGame { user_id, is_multiplayer } => {
                    let user_id = Uuid::parse_str(&user_id).unwrap();
                    let mut session = sessions.write().await;

                    let (session_id, session, color) = session.add_session(user_id, is_multiplayer, tx.clone()); 
                    
                    send_join_success(color, session_id, &tx, session);                        
                }
                session_handling::IncomingMessage::GetBoard { user_id } => {
                    // Get the state of the board associated with the user's ID
                    //
                    // If the game doesn't exist, do nothing
                    let user_id = Uuid::parse_str(&user_id).unwrap();

                    let session = sessions.read().await;

                    if let Some(valid_session) = session.get_session_if_exists(user_id) {
                        send_board(valid_session, &tx);
                    } else {
                        eprintln!("User doesn't have an existing game");
                    }
                    drop(session);
                }
                session_handling::IncomingMessage::GetMoves { user_id, hexagon } => {
                    let user_id = Uuid::parse_str(&user_id).unwrap();

                    // we need the board mutable because we do some intermediate mutations
                    // while checking for check, before returning the board to its original state.
                    // probably, we should just clone the board if doing so is fast enough.
                    let mut session = sessions.write().await;

                    if let Some(valid_session) = session.get_mut_session_if_exists(user_id) {
                        // try and process the move
                        if let Some(piece) = valid_session.board.occupied_squares.get(&hexagon).cloned() {
                            // match piece type to valid moves
                            let (moves, _) = get_valid_moves(&hexagon, &piece, &mut valid_session.board);

                            let outgoing = session_handling::OutgoingMessage::ValidMoves { moves: &moves };
                            tx.send(warp::ws::Message::text(
                                serde_json::to_string(&outgoing).unwrap(),
                            ))
                            .unwrap();
                        }
                    }
                    drop(session);
                }
                session_handling::IncomingMessage::RegisterMove {
                    user_id,
                    start_hexagon,
                    final_hexagon,
                } => {
                    let user_id = Uuid::parse_str(&user_id).unwrap();

                    let mut session = sessions.write().await;

                    // check if it is the player's turn to make a move
                    let test = session.get_mut_session_if_exists(user_id);
                    
                    if let Some(valid_session) =  test {
                        let board = &mut valid_session.board;
                        // check this player really has the right to play the next move
                        if valid_session.players.check_color(user_id, board.current_player) {
                            // try and process the move
                            if let Some(piece) = board.occupied_squares.get(&start_hexagon).cloned() {
                                // match piece type to valid moves
                                let (moves, double_jump) = get_valid_moves(&start_hexagon, &piece, board);
    
                                if moves.contains(&final_hexagon) {
                                    let _ =
                                        register_move(&start_hexagon, &final_hexagon, board, double_jump);

                                    // TODO broadcast an update to both the players
                                    for transmitter in &valid_session.channels {
                                        send_board(valid_session, transmitter);
                                    }

                                }
                            }
                        }

                    }
                    drop(session);
                }
                session_handling::IncomingMessage::JoinGame { user_id, game_id } => {
                    let user_id = Uuid::parse_str(&user_id).unwrap();
                    let session_id = Uuid::parse_str(&game_id).unwrap();

                    let mut session: tokio::sync::RwLockWriteGuard<'_, session_handling::SessionHandler> = sessions.write().await;

                    let color = session.try_join_session(user_id, session_id, tx.clone());

                    if let Some(valid_session) = session.get_mut_session_if_exists(user_id) {
                        send_join_success(color, session_id, &tx, valid_session);
                    }

                    drop(session);
                }
            }
        }
    }
}

fn send_join_success(color: Option<Color>, session_id: Uuid, tx: &mpsc::UnboundedSender<warp::ws::Message>, session: &mut session_handling::Game) {
    let message = session_handling::OutgoingMessage::JoinGameSuccess { color: color, session: session_id.to_string() };
    if let Ok(success_message) = serde_json::to_string(&message) {
        tx.send(warp::ws::Message::text(success_message)).unwrap();
    
        // send back the new board state
        send_board(&session, tx);
    } else {
        eprintln!("Failed to send back join confirmation");
    }
}

fn send_board(valid_session: &session_handling::Game, tx: &mpsc::UnboundedSender<warp::ws::Message>) {
    let board = &valid_session.board;
    let message = session_handling::OutgoingMessage::BoardState { board: board };
    if let Ok(new_board_state) = serde_json::to_string(&message) {
        tx.send(warp::ws::Message::text(new_board_state)).unwrap();
    } else {
        eprintln!("Failed to send board state");
    }
}

#[tokio::main]
async fn main() {
    // handle the page-serving side of the website
    let default = warp::path::end().and(warp::fs::file("./server_files/hello.html"));

    let pages = warp::fs::dir("./server_files/");

    let sessions: Arc<RwLock<session_handling::SessionHandler>> = Arc::new(RwLock::new(session_handling::SessionHandler::new()));

    let sessions = warp::any().map(move || sessions.clone());

    let websocket =
        warp::path::end()
            .and(warp::ws())
            .and(sessions)
            .map(|ws: warp::ws::Ws, sessions| {
                ws.on_upgrade(move |socket| handle_websocket_async(socket, sessions))
            });

    let routes = pages
        .or(websocket)
        .or(default)
        // serve 404s if the file doesn't exist and the client isn't asking for the default page
        .or(warp::fs::file("./server_files/404.html"));

    warp::serve(routes)
        .run(([127, 0, 0, 1], 7878))
        .await;
}
