use std::{net::TcpStream, env};

use hexchesscore::Color;

use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};
use url::Url;
use uuid::{self, Uuid};

use server::{
    session_handling::PlayerColor,
    websocket_messaging::{IncomingMessage, OutgoingMessage},
};

use bumblebot::bot_mind::make_a_move;

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
                let intended_move = make_a_move(&mut board, *current_color);
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