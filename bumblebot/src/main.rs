use std::{net::TcpStream, env};

use hexchesscore::{get_all_valid_moves, Board, Color, Move, PieceType, Piece};
use rand::{self, Rng};

use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};
use url::Url;
use uuid::{self, Uuid};

use server::{
    session_handling::PlayerColor,
    websocket_messaging::{IncomingMessage, OutgoingMessage},
};

fn match_player_color(color: PlayerColor) -> Color {
    match color {
        PlayerColor::Black => Color::Black,
        PlayerColor::White => Color::White,
        PlayerColor::Both => {
            panic!("the bot is fighting.... itself????")
        }
    }
}

fn handle_message(
    message: Message,
    user_id: Uuid,
    current_color: &mut Color,
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
) {
    let decoded: OutgoingMessage = serde_json::from_str(&message.into_text().unwrap()).unwrap();
    match decoded {
        OutgoingMessage::JoinGameSuccess { color, session } => {
            *current_color = match_player_color(color);
        }
        OutgoingMessage::OpponentJoined { session: _ } => {
            let _ = socket.send(Message::Text(
                serde_json::to_string(&IncomingMessage::GetBoard {
                    user_id: user_id.to_string(),
                })
                .unwrap(),
            ));
        }
        OutgoingMessage::BoardState { mut board } => {
            if board.current_player == *current_color {
                let intended_move = make_a_move(&mut board);
                let _ = socket.send(Message::Text(
                    serde_json::to_string(&IncomingMessage::RegisterMove {
                        user_id: user_id.to_string(),
                        start_hexagon: intended_move.start_hex,
                        final_hexagon: intended_move.final_hex,
                        // TODO fix the behaviour around promotions
                        promotion_choice: None,
                    })
                    .unwrap(),
                ));
            }
        }
        _ => {}
    }
}

fn main() {
    // spool up a bot that will respond to a board state with its
    // suggested move
    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    
    let (mut socket, response) =
        connect(Url::parse("ws://127.0.0.1:7878/ws").unwrap()).expect("Can't connect");

    let user_id = Uuid::new_v4();

    let message = IncomingMessage::JoinGame {
        user_id: user_id.to_string(),
        game_id: args[1].to_string()
    };


    // initialize the session_id with something useless
    let mut current_color = Color::Black;

    socket.send(Message::Text(
        serde_json::to_string(&message).expect("Couldn't serialize message"),
    ));

    loop {
        let msg = socket.read().expect("Error reading WS message");
        handle_message(msg, user_id, &mut current_color, &mut socket);
    }
}

fn evaluate_board(board: &Board) -> i16 {
    // count each player's pieces
    let mut black_count = 0;
    let mut white_count = 0;
    for (_hex, piece) in &board.occupied_squares {
        let value = match piece.piece_type {
            PieceType::Pawn => 1,
            PieceType::Rook => 4,
            PieceType::Bishop => 3, 
            PieceType::Knight => 3, 
            PieceType::Queen => 9,
            PieceType::King => 0
        };
        match piece.color {
            Color::White => {white_count += value},
            Color::Black => {black_count += value}
        }
    }
    white_count - black_count
}

fn apply_move(board: &mut Board, movement: Move) -> (&mut Board, Option<Piece>) {
    // this function assumes the move is legal. The legality checking
    // should have already happened in the move generation
    let moving_piece = board.occupied_squares.remove(&movement.start_hex).expect("Piece wasn't present at start hex");
    let taken_piece = board.occupied_squares.insert(movement.final_hex, moving_piece);
    board.current_player.invert();
    (board, taken_piece)
}

fn revert_move(board: &mut Board, movement: Move, taken_piece: Option<Piece>) -> (&mut Board, Option<Piece>) {
    // this function assumes the move is legal. The legality checking
    // should have already happened in the move generation
    let moving_piece = board.occupied_squares.remove(&movement.final_hex).expect("Piece wasn't present at final hex");
    board.occupied_squares.insert(movement.start_hex, moving_piece);
    if let Some(taken_piece) = taken_piece {
        board.occupied_squares.insert(movement.final_hex, taken_piece);
    }
    board.current_player.invert();
    (board, taken_piece)
}

fn evaluate_move(board: &mut Board, movement: Move) -> i16 {
    let (mut new_board, taken_piece) = apply_move(board, movement);
    let rating = evaluate_board(new_board);
    revert_move(new_board, movement, taken_piece);

    rating
}

fn make_a_move(board: &mut Board) -> Move {
    let move_options = get_all_valid_moves(board);
    let mut best_move = move_options[0];
    let mut best_move_rating = evaluate_move(board, best_move);


    for player_move in move_options {
        let rating = evaluate_move(board, player_move);
        dbg!(player_move, rating);
        if rating > best_move_rating {
            best_move_rating = rating;
            best_move = player_move
        }
    }
    return best_move;
}
