use hexchesscore::{check_for_mates, get_valid_moves, register_move, Board, Hexagon, Mate, PieceType};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use tokio::sync::{mpsc, RwLock};

use std::sync::Arc;

use std::collections::HashSet;

use warp::ws::Message;

use crate::session_handling::{self, PlayerColor};

#[derive(Serialize, Deserialize, Debug)]

pub enum GameOutcome {
    Won,
    Drew,
    Lost,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameEndReason {
    Checkmate,
    Stalemate,
    Resignation,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "op")]
pub enum IncomingMessage {
    GetBoard {
        user_id: String,
    },
    GetMoves {
        user_id: String,
        hexagon: Hexagon,
    },
    GetGameState {
        user_id: String,
    },
    RegisterMove {
        user_id: String,
        start_hexagon: Hexagon,
        final_hexagon: Hexagon,
        promotion_choice: Option<PieceType>
    },
    CreateGame {
        user_id: String,
        is_multiplayer: bool,
    },
    JoinGame {
        user_id: String,
        game_id: String,
    },
    JoinAnyGame {
        user_id: String,
    },
    TryReconnect {
        user_id: String,
    },
}

#[derive(Serialize, Debug)]
#[serde(tag = "op")]
pub enum OutgoingMessage<'a> {
    ValidMoves {
        moves: &'a Vec<Hexagon>,
        promotion_moves: &'a Vec<Hexagon>,
    },
    BoardState {
        board: &'a Board,
    },
    JoinGameSuccess {
        color: PlayerColor,
        session: String,
    },
    OpponentJoined {
        session: String,
    },
    JoinGameFailure,
    GameEnded {
        game_outcome: GameOutcome,
        reason: GameEndReason,
    },
    GameStatus {
        game_started: bool
    }
}

pub async fn handle_incoming_ws_message(
    message: Message,
    sessions: &Arc<RwLock<session_handling::SessionHandler>>,
    tx: &mpsc::UnboundedSender<Message>,
    user_ids_on_websocket: &mut HashSet<Uuid>,
) {
    let decoded: IncomingMessage = serde_json::from_str(message.to_str().unwrap()).unwrap();

    let uuid_user_id;
    
    match decoded {
        IncomingMessage::CreateGame {
            user_id,
            is_multiplayer,
        } => {
            uuid_user_id = Uuid::parse_str(&user_id).unwrap();

            let mut session = sessions.write().await;

            let (session_id, session, color) =
                session.add_session(uuid_user_id, is_multiplayer, false, tx.clone());

            send_join_success(color, session_id, tx, &session.board);
        }
        IncomingMessage::JoinAnyGame { user_id } => {
            uuid_user_id = Uuid::parse_str(&user_id).unwrap();
            let mut session = sessions.write().await;

            let (session_id, session, color) =
                session.try_join_any_sessions(uuid_user_id, tx.clone());

            send_join_success(color, session_id, tx, &session.board);
        }
        IncomingMessage::GetBoard { user_id } => {
            // Get the state of the board associated with the user's ID
            //
            // If the game doesn't exist, do nothing
            uuid_user_id = Uuid::parse_str(&user_id).unwrap();

            let session = sessions.read().await;

            if let Some(valid_session) = session.get_session_if_exists(uuid_user_id) {
                send_board(&valid_session.board, tx);
            } else {
                eprintln!("User doesn't have an existing game");
            }
            drop(session);
        }
        IncomingMessage::GetMoves { user_id, hexagon } => {
            uuid_user_id = Uuid::parse_str(&user_id).unwrap();

            // we need the board mutable because we do some intermediate mutations
            // while checking for check, before returning the board to its original state.
            // probably, we should just clone the board if doing so is fast enough.
            let mut session = sessions.write().await;

            if let Some(valid_session) = session.get_mut_session_if_exists(uuid_user_id) {
                // try and process the move
                if let Some(piece) = valid_session.board.occupied_squares.get(&hexagon).cloned() {
                    // match piece type to valid moves
                    let (moves, _, promotion_moves) =
                        get_valid_moves(&hexagon, &piece, &mut valid_session.board);

                    let outgoing = OutgoingMessage::ValidMoves {
                        moves: &moves,
                        promotion_moves: &promotion_moves,
                    };
                    tx.send(warp::ws::Message::text(
                        serde_json::to_string(&outgoing).unwrap(),
                    ))
                    .unwrap();
                }
            }
            drop(session);
        },
        IncomingMessage::GetGameState { user_id } => {
            uuid_user_id = Uuid::parse_str(&user_id).unwrap();
            let mut session = sessions.write().await;

            let maybe_session = session.get_mut_session_if_exists(uuid_user_id);

            if let Some(valid_session) = maybe_session {
                let outgoing;
                if valid_session.players.black.is_some() & valid_session.players.white.is_some() {
                    outgoing = OutgoingMessage::GameStatus {
                        game_started: true
                    }
                } else {
                    outgoing = OutgoingMessage::GameStatus {
                        game_started: false
                    }
                }
                tx.send(warp::ws::Message::text(
                    serde_json::to_string(&outgoing).unwrap())).unwrap();
            }
        }
        IncomingMessage::RegisterMove {
            user_id,
            start_hexagon,
            final_hexagon,
            promotion_choice
        } => {
            uuid_user_id = Uuid::parse_str(&user_id).unwrap();

            let mut session = sessions.write().await;

            // check if it is the player's turn to make a move
            let maybe_session = session.get_mut_session_if_exists(uuid_user_id);

            if let Some(valid_session) = maybe_session {
                let board = &mut valid_session.board;
                // check this player really has the right to play the next move
                if valid_session
                    .players
                    .check_color(uuid_user_id, board.current_player)
                {
                    // try and process the move
                    if let Some(piece) = board.occupied_squares.get(&start_hexagon).cloned() {
                        // match piece type to valid moves
                        let (moves, double_jump, promotion_moves) =
                            get_valid_moves(&start_hexagon, &piece, board);

                        if moves.contains(&final_hexagon) {
                            let _ = register_move(
                                &start_hexagon,
                                &final_hexagon,
                                board,
                                double_jump,
                                promotion_moves,
                                promotion_choice,
                            );

                            if let Some(mate) = check_for_mates(board) {
                                // the player registering the move has just won

                                // send a win message to the player
                                send_game_end(
                                    Some(mate),
                                    true,
                                    valid_session.channels.get(&uuid_user_id).expect(
                                        "No channels
                                to communicate with the player who sent this move in!",
                                    ),
                                );

                                // send a lose message to the opponent
                                let loser_channel =
                                    valid_session.channels.iter().find_map(|(player, channel)| {
                                        if player != &uuid_user_id {
                                            Some(channel)
                                        } else {
                                            None
                                        }
                                    });

                                if loser_channel.is_some() {
                                    send_game_end(Some(mate), false, loser_channel.unwrap());
                                }
                            }

                            // TODO broadcast an update to both the players
                            for (_, transmitter) in &valid_session.channels {
                                send_board(&valid_session.board, transmitter);
                            }
                        }
                    }
                }
            }
            drop(session);
        }
        IncomingMessage::JoinGame { user_id, game_id } => {
            uuid_user_id = Uuid::parse_str(&user_id).unwrap();
            let session_id = Uuid::parse_str(&game_id).unwrap();

            let mut session: tokio::sync::RwLockWriteGuard<'_, session_handling::SessionHandler> =
                sessions.write().await;

            let color = session.try_join_session(uuid_user_id, session_id, tx.clone());

            if let (Some(valid_session), Some(color)) =
                (session.get_mut_session_if_exists(uuid_user_id), color)
            {
                send_join_success(color, session_id, tx, &valid_session.board);
            }

            drop(session);
        }
        IncomingMessage::TryReconnect { user_id } => {
            println!("trying to reconnect");
            uuid_user_id = Uuid::parse_str(&user_id).unwrap();
            // see if the user_id already has some games
            let session = sessions.read().await;
            let session_id = session.players.get(&uuid_user_id).cloned();
            drop(session);

            if let Some(session_id) = session_id {
                let mut session = sessions.write().await;

                // have to identify a different way of figuring out if the player doesn't exist
                let res = session.reconnect_player(uuid_user_id, tx.clone());

                if let Some((color, board)) = res {
                    println!("trying to send a success message");
                    send_join_success(color, session_id, tx, board)
                }
            }
        }
    }
}

fn send_join_success(
    color: PlayerColor,
    session_id: Uuid,
    tx: &mpsc::UnboundedSender<warp::ws::Message>,
    board: &Board,
) {
    let message = OutgoingMessage::JoinGameSuccess {
        color: color,
        session: session_id.to_string(),
    };
    if let Ok(success_message) = serde_json::to_string(&message) {
        tx.send(warp::ws::Message::text(success_message)).unwrap();

        // send back the new board state
        send_board(&board, tx);
    } else {
        eprintln!("Failed to send back join confirmation");
    }
}

fn send_board(board: &Board, tx: &mpsc::UnboundedSender<warp::ws::Message>) {
    let message = OutgoingMessage::BoardState { board: board };
    if let Ok(new_board_state) = serde_json::to_string(&message) {
        tx.send(warp::ws::Message::text(new_board_state)).unwrap();
    } else {
        eprintln!("Failed to send board state");
    }
}

fn send_game_end(mate: Option<Mate>, winner: bool, tx: &mpsc::UnboundedSender<warp::ws::Message>) {
    let (reason, outcome) = match (mate, winner) {
        (Some(Mate::Checkmate), true) => (GameEndReason::Checkmate, GameOutcome::Won),
        (Some(Mate::Stalemate), _) => (GameEndReason::Stalemate, GameOutcome::Drew),
        (Some(Mate::Checkmate), false) => (GameEndReason::Checkmate, GameOutcome::Lost),
        (None, true) => (GameEndReason::Resignation, GameOutcome::Won),
        (None, false) => (GameEndReason::Resignation, GameOutcome::Lost),
    };

    let message = OutgoingMessage::GameEnded {
        game_outcome: outcome,
        reason: reason,
    };
    if let Ok(outcome_message) = serde_json::to_string(&message) {
        tx.send(warp::ws::Message::text(outcome_message)).unwrap();
    } else {
        // do something at this point to make sure all the clients recieved their outcome message
        eprintln!("Failed to send outcome message");
    }
}