

use std::collections::HashMap;

use hexchesscore::Hexagon;

use serde::{Serialize, Deserialize};
use warp::ws::Message;

use hexchesscore::Board;

use uuid::Uuid;

use hexchesscore::Color;

#[derive(Debug)]
pub struct PlayersPerGame {
    pub black: Option<PlayerID>,
    pub white: Option<PlayerID>,
}

impl PlayersPerGame {
    pub fn new(first_player: PlayerID) -> (Color, PlayersPerGame) {
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
    pub fn try_add_player(&mut self, second_player: PlayerID) -> Option<Color> {
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

    pub fn check_color(&self, player: PlayerID, color: Color) -> bool {
        if color == Color::Black {
            self.black == Some(player)
        } else if color == Color::White {
            self.white == Some(player)
        } else {
            false
        }

    }



}

pub type SessionID = Uuid;

pub type PlayerID = Uuid;

#[derive(Debug)]
pub struct Game {
    pub board: Board,
    pub players: PlayersPerGame,
    pub channels: Vec<tokio::sync::mpsc::UnboundedSender<Message>>
}

impl Game {
    pub fn new(user_id: PlayerID, transmitter: &tokio::sync::mpsc::UnboundedSender<Message>) -> (SessionID, Game, Color) {
        let board = Board::setup_default_board();
        let (color, players) = PlayersPerGame::new(user_id);
        let session_id = Uuid::new_v4();
        (session_id, Game {board: board, players: players, channels: vec![transmitter.clone()]}, color)
    }
}

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

#[derive(Debug)]
pub struct SessionHandler {
    pub sessions: HashMap<SessionID, Game>,
    pub players: HashMap<PlayerID, SessionID>,
}

impl SessionHandler {
    pub fn new() -> SessionHandler {
        SessionHandler {
            sessions: HashMap::<SessionID, Game>::new(),
            players: HashMap::<PlayerID, SessionID>::new(),
        }
    }

    pub fn get_session_if_exists(&self, user_id: Uuid) -> Option<&Game> {
        let session_id = self.players.get(&user_id);
        match session_id {
            Some(session) => self.sessions.get(&session),
            None => None,
        }
    }

    pub fn get_mut_session_if_exists(&mut self, user_id: Uuid) -> Option<&mut Game> {
        let session_id = self.players.get(&user_id);
        match session_id {
            Some(session) => self.sessions.get_mut(&session),
            None => None,
        }
    }

    pub fn add_session(&mut self, user_id: Uuid, is_multiplayer: bool, transmitter: tokio::sync::mpsc::UnboundedSender<Message>) -> (SessionID, &mut Game, Option<Color>) {
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

    pub fn try_join_session(&mut self, user_id: PlayerID, session_id: SessionID, transmitter: tokio::sync::mpsc::UnboundedSender<Message>) -> Option<Color> {
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