use hexchesscore::Board;
use hexchesscore::Color;
use hexchesscore::Hexagon;
use hexchesscore::register_move;

use hexchesscore::get_valid_moves;

use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use tokio::sync::mpsc;

use tokio::sync::RwLock;

use std::sync::Arc;

use warp::ws::Message;

use crate::session_handling;


#[derive(Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum IncomingMessage {
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
    CreateGame {
        user_id: String,
        is_multiplayer: bool,
    },
    JoinGame {
        user_id: String,
        game_id: String
    }
}

#[derive(Serialize, Debug)]
#[serde(tag = "op")]
pub enum OutgoingMessage<'a> {
    ValidMoves { moves: &'a Vec<Hexagon> },
    BoardState { board: &'a Board },
    JoinGameSuccess { color: Option<Color>, session: String},
    JoinGameFailure
}


pub async fn handle_incoming_ws_message(message: Message, sessions: &Arc<RwLock<session_handling::SessionHandler>>, tx: &mpsc::UnboundedSender<Message>) {
    let decoded: IncomingMessage = serde_json::from_str(message.to_str().unwrap()).unwrap();

    match decoded {
        IncomingMessage::CreateGame { user_id, is_multiplayer } => {
            let user_id = Uuid::parse_str(&user_id).unwrap();
            let mut session = sessions.write().await;

            let (session_id, session, color) = session.add_session(user_id, is_multiplayer, tx.clone()); 
    
            send_join_success(color, session_id, tx, session);                        
        }
        IncomingMessage::GetBoard { user_id } => {
            // Get the state of the board associated with the user's ID
            //
            // If the game doesn't exist, do nothing
            let user_id = Uuid::parse_str(&user_id).unwrap();

            let session = sessions.read().await;

            if let Some(valid_session) = session.get_session_if_exists(user_id) {
                send_board(valid_session, tx);
            } else {
                eprintln!("User doesn't have an existing game");
            }
            drop(session);
        }
        IncomingMessage::GetMoves { user_id, hexagon } => {
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

                    let outgoing = OutgoingMessage::ValidMoves { moves: &moves };
                    tx.send(warp::ws::Message::text(
                        serde_json::to_string(&outgoing).unwrap(),
                    ))
                    .unwrap();
                }
            }
            drop(session);
        }
        IncomingMessage::RegisterMove {
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
        IncomingMessage::JoinGame { user_id, game_id } => {
            let user_id = Uuid::parse_str(&user_id).unwrap();
            let session_id = Uuid::parse_str(&game_id).unwrap();

            let mut session: tokio::sync::RwLockWriteGuard<'_, session_handling::SessionHandler> = sessions.write().await;

            let color = session.try_join_session(user_id, session_id, tx.clone());

            if let Some(valid_session) = session.get_mut_session_if_exists(user_id) {
                send_join_success(color, session_id, tx, valid_session);
            }

            drop(session);
        }
    }
}

fn send_join_success(color: Option<Color>, session_id: Uuid, tx: &mpsc::UnboundedSender<warp::ws::Message>, session: &mut session_handling::Game) {
    let message = OutgoingMessage::JoinGameSuccess { color: color, session: session_id.to_string() };
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
    let message = OutgoingMessage::BoardState { board: board };
    if let Ok(new_board_state) = serde_json::to_string(&message) {
        tx.send(warp::ws::Message::text(new_board_state)).unwrap();
    } else {
        eprintln!("Failed to send board state");
    }
}