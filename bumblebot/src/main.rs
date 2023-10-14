use std::net::TcpStream;

use hexchesscore::{get_all_valid_moves, Board, Move};
use rand::{self, Rng};

use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};
use url::Url;
use uuid::{self, Uuid};

use server::{
    session_handling::PlayerColor,
    websocket_messaging::{IncomingMessage, OutgoingMessage},
};

fn handle_message(
    message: Message,
    user_id: Uuid,
    mut session_id: Uuid,
    mut current_color: PlayerColor,
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
) {
    let decoded: OutgoingMessage = serde_json::from_str(&message.into_text().unwrap()).unwrap();
    dbg!(&decoded);
    match decoded {
        OutgoingMessage::JoinGameSuccess { color, session } => {
            session_id = Uuid::parse_str(&session).unwrap().clone();
            current_color = color;
        }
        OutgoingMessage::BoardState { mut board } => {
            if matches!(board.current_player, current_color) {
                let intended_move = make_a_move(&mut board);
                socket.send(Message::Text(
                    serde_json::to_string(&IncomingMessage::RegisterMove {
                        user_id: user_id.to_string(),
                        start_hexagon: intended_move.start_hex,
                        final_hexagon: intended_move.final_hex,
                        // TODO fix the behaviour around promotions
                        promotion_choice: None,
                    }).unwrap()
                ));
            }
        }
        _ => {}
    }
}

fn main() {
    // spool up a bot that will respond to a board state with its
    // suggested move
    let (mut socket, response) =
        connect(Url::parse("ws://127.0.0.1:7878/ws").unwrap()).expect("Can't connect");

    println!("Connected to the server");

    let user_id = Uuid::new_v4();

    let message = IncomingMessage::JoinAnyGame { user_id: user_id.to_string()};

    socket.send(Message::Text(serde_json::to_string(&message).expect("Couldn't serialize message")));

    // initialize the session_id with something useless
    let mut session_id = Uuid::new_v4();
    let mut current_color = PlayerColor::Black;

    loop {
        let msg = socket.read().expect("Error reading WS message");
        handle_message(msg, user_id, session_id, current_color, &mut socket);
    }

    let mut board = Board::setup_default_board();
    dbg!(make_a_move(&mut board));
}

fn make_a_move(board: &mut Board) -> Move {
    let move_options = get_all_valid_moves(board);
    return move_options[rand::thread_rng().gen_range(0..move_options.len())];
}
