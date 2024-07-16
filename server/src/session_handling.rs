use std::collections::{HashMap, VecDeque};
use warp::ws::Message;
use hexchesscore::{Board, Color};
use uuid::Uuid;

use api::PlayerColor;

#[derive(Debug, Clone, Copy)]
pub struct PlayersPerGame {
    pub black: Option<PlayerID>,
    pub white: Option<PlayerID>,
}

impl PlayersPerGame {
    pub fn new(first_player: PlayerID) -> (PlayerColor, PlayersPerGame) {
        // pseudo-randomly pick whether the first player is
        // black or white
        match first_player.as_fields().0 % 2 {
            0 => (PlayerColor::Black, PlayersPerGame {
                black: Some(first_player),
                white: None,
            }),
            1 => (PlayerColor::White, PlayersPerGame {
                black: None,
                white: Some(first_player),
            }),
            _ => panic!("shouldn't be able to get here!")
        }
    }
    pub fn try_add_player(&mut self, second_player: PlayerID) -> Option<PlayerColor> {
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
                    players_color = Some(PlayerColor::White);
                } else {
                    players_color = None;
                }
            }
            None => {
                self.black = Some(second_player);
                players_color = Some(PlayerColor::Black);
                    },
        }
        players_color
    }

    pub fn check_color(&self, player: PlayerID, color: Color) -> bool {
        // Look at whether it is a player's turn, given the player's ID
        // and the color that is currently allowed to move
        if color == Color::Black {
            self.black == Some(player)
        } else if color == Color::White {
            self.white == Some(player)
        } else {
            false
        }

    }

    pub fn check_for_player(&self, player: PlayerID) -> Option<PlayerColor> {
        if Some(player) == self.black {
            if Some(player) == self.white {
                Some(PlayerColor::Both)
            } else {
                Some(PlayerColor::Black)
            }
        } else if Some(player) == self.white {
            Some(PlayerColor::White)
        } else {
            None
        }
    }



}

pub type SessionID = Uuid;

pub type PlayerID = Uuid;

#[derive(Debug)]
pub struct Game {
    pub board: Board,
    pub players: PlayersPerGame,
    pub channels: HashMap<PlayerID, tokio::sync::mpsc::UnboundedSender<Message>>
}

impl Game {
    pub fn new(user_id: PlayerID, transmitter: &tokio::sync::mpsc::UnboundedSender<Message>) -> (SessionID, Game, PlayerColor) {
        let board = Board::setup_default_board();
        let (color, players) = PlayersPerGame::new(user_id);
        let session_id = Uuid::new_v4();
        let mut channels = HashMap::new();
        channels.insert(user_id, transmitter.clone());
        (session_id, Game {board: board, players: players, channels: channels}, color)
    }
}

#[derive(Debug)]
pub struct SessionHandler {
    pub sessions: HashMap<SessionID, Game>,
    pub players: HashMap<PlayerID, SessionID>,
    pub joinable_sessions: VecDeque<SessionID>
}

impl SessionHandler {
    pub fn new() -> SessionHandler {
        SessionHandler {
            sessions: HashMap::<SessionID, Game>::new(),
            players: HashMap::<PlayerID, SessionID>::new(),
            joinable_sessions: VecDeque::<SessionID>::new()
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

    pub fn add_session(&mut self, user_id: Uuid, is_multiplayer: bool, joinable: bool, transmitter: tokio::sync::mpsc::UnboundedSender<Message>) -> (SessionID, &mut Game, PlayerColor) {
        let (session_id, mut new_session, mut player_color) = Game::new(user_id, &transmitter);
        // if multiplayer, just add the one player for the moment,
        // which is performed in the session::new() setup.
        // if single-player, both player slots are the same player
        if !is_multiplayer {
            player_color = PlayerColor::Both;
            new_session.players.try_add_player(user_id);
        }
        if joinable {
            self.joinable_sessions.push_back(session_id);
        }
        // store the session so we can find it later
        self.sessions.insert(session_id, new_session);

        // add the player to players so we can find their game easily in the future
        self.add_player_to_game(user_id, session_id);

        (session_id, self.get_mut_session_if_exists(user_id)
            .expect("Failure creating session"), player_color)
    }

    fn add_player_to_game(&mut self, user_id: Uuid, session_id: Uuid) {
        // Add a player to a game. If they have other active games,
        // delete them.
        match self.players.insert(user_id, session_id) {
            Some(session_id) => self.delete_session(user_id, session_id),
            None => ()
        }
    }

    pub fn delete_session(&mut self, user_id: PlayerID, session_id: SessionID) {
        // the player who asks to delete the game implicitly resigns
        let session = self.sessions.get(&session_id);
        if let Some(valid_session) = session {
            send_resignation(user_id, &valid_session.channels);
            // delete the session
            self.sessions.remove(&session_id);
        }
        // also, delete the session if it is in the joinable sessions vec
        self.joinable_sessions.retain(|val| val != &session_id);
    }

    pub fn try_join_session(&mut self, user_id: PlayerID, session_id: SessionID, transmitter: tokio::sync::mpsc::UnboundedSender<Message>) -> Option<PlayerColor> {
        // first, clean up any existing games the player might have
        self.delete_player(user_id);
        // try and join a session. If the session is already full,
        // or it doesn't exist, silently fail.
        let game = self.sessions.get_mut(&session_id);
        if let Some(valid_game) = game {
            let players = &mut valid_game.players;
            if let Some(player_color) = players.try_add_player(user_id) {
                valid_game.channels.insert(user_id, transmitter.clone());
                self.add_player_to_game(user_id, session_id);
                // delete the session from the list of joinable sessions
                self.joinable_sessions.retain(|val| val != &session_id);

                return Some(player_color)
            }
        }
        None
    }

    pub fn reconnect_player(&mut self, user_id: PlayerID, transmitter: tokio::sync::mpsc::UnboundedSender<Message>) -> Option<(PlayerColor, &Board)> {
        let game = self.get_mut_session_if_exists(user_id);
        if let Some(valid_game) = game {
            let color = valid_game.players.check_for_player(user_id);
            if color.is_some() {
                // overwrite the transmitter from the old websocket
                valid_game.channels.insert(user_id, transmitter.clone());
            }

            Some((color.unwrap(), &valid_game.board))
        }
        else {
            None
        }
    }

    pub fn try_join_any_sessions(&mut self, user_id: PlayerID, transmitter: tokio::sync::mpsc::UnboundedSender<Message>) -> (SessionID, &mut Game, PlayerColor) {
        // try join any of the joinable sessions

        // we pop this game off the list so that if it isn't joinable,
        // it doesn't stay in the list of joinables
        while self.joinable_sessions.len() > 0 {
            // try join a game
            if let Some(game) = self.joinable_sessions.pop_front() {
                let color = self.try_join_session(user_id, game, transmitter.clone());
                match color {
                    Some(color) => return (game, self.get_mut_session_if_exists(user_id).expect("couldn't get newly created game"), color),
                    None => ()
                }
            } 
            // delete it from the queue if you can't join
        }
        // can't find any games for some reason; time to make one
        let (session_id, game, color) = self.add_session(user_id, true, true, transmitter.clone());

        (session_id, game, color)
    }

    pub fn delete_player(&mut self, user_id: PlayerID) {
        let game = self.players.remove(&user_id);
        match game {
            Some(session_id) => {self.delete_session(user_id, session_id);},
            None => ()
        };
    }
}

// TODO this is a really hacky way of killing bots for the moment
pub fn send_resignation(_initiating_player: PlayerID, channels: &HashMap<PlayerID, tokio::sync::mpsc::UnboundedSender<Message>>) {
    for channel in channels.values() {
        let _ = channel.send(warp::ws::Message::text("{{\"op\"= \"GameEnded\"}}"));
    }
}