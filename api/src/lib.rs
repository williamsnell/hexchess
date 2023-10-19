use hexchesscore::{Hexagon, PieceType, Board};
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum PlayerColor {
    Black,
    White,
    Both
}



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

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "op")]
pub enum OutgoingMessage {
    ValidMoves {
        moves: Vec<Hexagon>,
        promotion_moves: Vec<Hexagon>,
    },
    BoardState {
        board: Board,
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
