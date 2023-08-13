use futures::{SinkExt, StreamExt, TryFutureExt};
use hexchesscore::{get_valid_moves, register_move, Board, Color, Hexagon};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::Filter;

struct PlayersPerGame {
    black: Option<PlayerID>,
    white: Option<PlayerID>,
}

impl PlayersPerGame {
    fn new(first_player: PlayerID) -> PlayersPerGame {
        // pseudo-randomly pick whether the first player is
        // black or white
        match first_player.as_fields().0 % 2 {
            0 => PlayersPerGame {
                black: Some(first_player),
                white: None,
            },
            1 => PlayersPerGame {
                black: None,
                white: Some(first_player),
            },
        }
    }
    fn try_add_player(&mut self, second_player: PlayerID) {
        // this function tries to add a player, but 
        // if the second slot is already occupied,
        // it silently fails.
        // worth thinking about if this should raise an error
        // instead
        match self.black {
            Some(_) => {
                if self.white == None {
                    self.white = Some(second_player)
                }
            }
            None => self.black = Some(second_player),
        }
    }
}
// each session has a uuid.
// when a player joins a multiplayer game, their session is added as a key that can
// access that session

type SessionID = Uuid;
type PlayerID = Uuid;

pub struct Session {
    board: Board,
    players: PlayersPerGame,
}

impl Session {
    fn new(user_id: PlayerID) -> (SessionID, Session) {
        let board = Board::setup_default_board();
        let players = PlayersPerGame::new(user_id);
        let session_id = Uuid::new_v4();
        (session_id, Session {board: board, players: players})
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
}

#[derive(Serialize, Debug)]
#[serde(tag = "op")]
enum OutgoingMessage<'a> {
    ValidMoves { moves: &'a Vec<Hexagon> },
    BoardState { board: &'a Board },
}


pub struct SessionHandler {
    sessions: HashMap<SessionID, Session>,
    players: HashMap<PlayerID, SessionID>
}

impl SessionHandler {
    fn new() -> SessionHandler {
        SessionHandler {
            sessions: HashMap::<SessionID, Session>::new(),
            players: HashMap::<PlayerID, SessionID>::new(),
        }
    }

    fn get_session_if_exists(&mut self, user_id: Uuid) -> Option<&mut Session> {
        let session_id = self.players.get(&user_id);
        match session_id {
            Some(session) => self.sessions.get_mut(&session),
            None => None,
        }
    }

    fn add_session(&mut self, user_id: Uuid, is_multiplayer: bool) -> (SessionID, &mut Session) {
        let (session_id, mut new_session) = Session::new(user_id);
        // if multiplayer, just add the one player for the moment,
        // which is performed in the session::new() setup.
        // if single-player, both player slots are the same player
        if !is_multiplayer {
            new_session.players.try_add_player(user_id);
        }
        // store the session so we can find it later
        self.sessions.insert(session_id, new_session);

        // add the player to players so we can find their game easily in the future
        self.players.insert(user_id, session_id);

        (session_id, self.sessions
            .get_mut(&user_id)
            .expect("Failure creating session"))
    }

    fn try_join_session(&mut self, user_id: PlayerID, session_id: SessionID) {
        // try and join a session. If the session is already full,
        // or it doesn't exist, silently fail.
        let session = self.sessions.get(&session_id);
        if let Some(valid_session) = session {
            valid_session.players.try_add_player(user_id);
        }
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
                        let (moves, _) = get_valid_moves(&hexagon, &piece, &board);

                        let outgoing = OutgoingMessage::ValidMoves { moves: &moves };
                        tx.send(warp::ws::Message::text(
                            serde_json::to_string(&outgoing).unwrap(),
                        ))
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
                        let (moves, double_jump) = get_valid_moves(&start_hexagon, &piece, board);

                        if moves.contains(&final_hexagon) {
                            let _ =
                                register_move(&start_hexagon, &final_hexagon, board, double_jump);
                        }
                    }
                    drop(session);
                }
            }
        }
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
        .tls()
        .cert_path("./cert/playhexchess.com.crt")
        .key_path("./cert/playhexchess.com.key")
        // .run(([0, 0, 0, 0], 443))
        .run(([127, 0, 0, 1], 7878))
        .await;
}
