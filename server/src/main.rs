
use futures::{SinkExt, StreamExt, TryFutureExt};
use hexchesscore::{get_valid_moves, register_move, Board, Color, Hexagon};
use serde::{Deserialize, Serialize};

use warp::ws::Message;
use std::{collections::HashMap, sync::Arc};
use tokio;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::Filter;

#[derive(Debug)]
struct PlayersPerGame {
    black: Option<PlayerID>,
    white: Option<PlayerID>,
}

impl PlayersPerGame {
    fn new(first_player: PlayerID) -> (Color, PlayersPerGame) {
        // pseudo-randomly pick whether the first player is
        // black or white
        match first_player.as_fields().0 % 2 {
            0 => (Color::Black, PlayersPerGame {
                black: Some(first_player),
                white: None,
            }),
            1 => (Color::White, PlayersPerGame {
                black: None,
                white: Some(first_player),
            }),
            _ => panic!("shouldn't be able to get here!")
        }
    }
    fn try_add_player(&mut self, second_player: PlayerID) -> Option<Color> {
        // this function tries to add a player, but 
        // if the second slot is already occupied,
        // it silently fails.
        // worth thinking about if this should raise an error
        // instead
        let players_color;
        match self.black {
            Some(_) => {
                if self.white == None {
                    self.white = Some(second_player);
                    players_color = Some(Color::White);
                } else {
                    players_color = None;
                }
            }
            None => {
                self.black = Some(second_player);
                players_color = Some(Color::Black);
                    },
        }
        players_color
    }

    fn check_color(&self, player: PlayerID, color: Color) -> bool {
        if color == Color::Black {
            self.black == Some(player)
        } else if color == Color::White {
            self.white == Some(player)
        } else {
            false
        }

    }



}
// each session has a uuid.
// when a player joins a multiplayer game, their session is added as a key that can
// access that session

type SessionID = Uuid;
type PlayerID = Uuid;

#[derive(Debug)]
pub struct Game {
    board: Board,
    players: PlayersPerGame,
    channels: Vec<tokio::sync::mpsc::UnboundedSender<Message>>
}

impl Game {
    fn new(user_id: PlayerID, transmitter: &tokio::sync::mpsc::UnboundedSender<Message>) -> (SessionID, Game, Color) {
        let board = Board::setup_default_board();
        let (color, players) = PlayersPerGame::new(user_id);
        let session_id = Uuid::new_v4();
        (session_id, Game {board: board, players: players, channels: vec![transmitter.clone()]}, color)
    }
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
enum OutgoingMessage<'a> {
    ValidMoves { moves: &'a Vec<Hexagon> },
    BoardState { board: &'a Board },
    JoinGameSuccess { color: Option<Color>, session: String},
    JoinGameFailure
}

#[derive(Debug)]
pub struct SessionHandler {
    sessions: HashMap<SessionID, Game>,
    players: HashMap<PlayerID, SessionID>,
}

impl SessionHandler {
    fn new() -> SessionHandler {
        SessionHandler {
            sessions: HashMap::<SessionID, Game>::new(),
            players: HashMap::<PlayerID, SessionID>::new(),
        }
    }

    fn get_session_if_exists(&self, user_id: Uuid) -> Option<&Game> {
        let session_id = self.players.get(&user_id);
        match session_id {
            Some(session) => self.sessions.get(&session),
            None => None,
        }
    }

    fn get_mut_session_if_exists(&mut self, user_id: Uuid) -> Option<&mut Game> {
        let session_id = self.players.get(&user_id);
        match session_id {
            Some(session) => self.sessions.get_mut(&session),
            None => None,
        }
    }

    fn add_session(&mut self, user_id: Uuid, is_multiplayer: bool, transmitter: tokio::sync::mpsc::UnboundedSender<Message>) -> (SessionID, &mut Game, Option<Color>) {
        let (session_id, mut new_session, player_color) = Game::new(user_id, &transmitter);
        // if multiplayer, just add the one player for the moment,
        // which is performed in the session::new() setup.
        // if single-player, both player slots are the same player
        let mut player_color = Some(player_color);
        if !is_multiplayer {
            player_color = None;
            new_session.players.try_add_player(user_id);
        }
        // store the session so we can find it later
        self.sessions.insert(session_id, new_session);

        // add the player to players so we can find their game easily in the future
        self.players.insert(user_id, session_id);

        (session_id, self.get_mut_session_if_exists(user_id)
            .expect("Failure creating session"), player_color)
    }

    fn try_join_session(&mut self, user_id: PlayerID, session_id: SessionID, transmitter: tokio::sync::mpsc::UnboundedSender<Message>) -> Option<Color> {
        // try and join a session. If the session is already full,
        // or it doesn't exist, silently fail.
        let game = self.sessions.get_mut(&session_id);
        if let Some(valid_game) = game {
            let players = &mut valid_game.players;
            if let Some(player_color) = players.try_add_player(user_id) {
                self.players.insert(user_id, session_id);
                valid_game.channels.push(transmitter.clone());
                return Some(player_color)
            }
        }
        None
    }
}

async fn handle_websocket_async(
    websocket: warp::ws::WebSocket,
    sessions: Arc<RwLock<SessionHandler>>,
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
            let decoded: IncomingMessage = serde_json::from_str(message.to_str().unwrap()).unwrap();

            match decoded {
                IncomingMessage::CreateGame { user_id, is_multiplayer } => {
                    let user_id = Uuid::parse_str(&user_id).unwrap();
                    let mut session = sessions.write().await;

                    let (session_id, session, color) = session.add_session(user_id, is_multiplayer, tx.clone()); 
                    
                    send_join_success(color, session_id, &tx, session);                        
                }
                IncomingMessage::GetBoard { user_id } => {
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

                    let mut session: tokio::sync::RwLockWriteGuard<'_, SessionHandler> = sessions.write().await;

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

fn send_join_success(color: Option<Color>, session_id: Uuid, tx: &mpsc::UnboundedSender<warp::ws::Message>, session: &mut Game) {
    let message = OutgoingMessage::JoinGameSuccess { color: color, session: session_id.to_string() };
    if let Ok(success_message) = serde_json::to_string(&message) {
        tx.send(warp::ws::Message::text(success_message)).unwrap();
    
        // send back the new board state
        send_board(&session, tx);
    } else {
        eprintln!("Failed to send back join confirmation");
    }
}

fn send_board(valid_session: &Game, tx: &mpsc::UnboundedSender<warp::ws::Message>) {
    let board = &valid_session.board;
    let message = OutgoingMessage::BoardState { board: board };
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

    let sessions: Arc<RwLock<SessionHandler>> = Arc::new(RwLock::new(SessionHandler::new()));

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
