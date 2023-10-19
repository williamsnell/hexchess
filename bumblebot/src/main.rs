use std::{env, net::TcpStream, thread, time::Duration};

use futures::{SinkExt, StreamExt, TryFutureExt, channel::mpsc::UnboundedSender};
use hexchesscore::{Color, Board};
use warp::Filter;
use warp::ws::Message;
use tokio::{self, sync::mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;

use tungstenite::{connect, stream::MaybeTlsStream, WebSocket};
use uuid::{self, Uuid};

use api::{PlayerColor, IncomingMessage, OutgoingMessage};

use bumblebot::{bot_mind::make_a_move, setup_test_boards};

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
    message: tungstenite::Message,
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
            let _ = socket.send(tungstenite::Message::Text(
                serde_json::to_string(&IncomingMessage::GetBoard {
                    user_id: user_id.to_string(),
                })
                .unwrap(),
            ));
        }
        OutgoingMessage::BoardState { mut board } => {
            if board.current_player == *current_color {
                let intended_move = make_a_move(&mut board, *current_color);
                let _ = socket.send(tungstenite::Message::Text(
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

fn send_board(transmitter: &mpsc::UnboundedSender<Message>, board: Board) {
    let _result = transmitter.send(Message::text(
        serde_json::to_string(&OutgoingMessage::BoardState { board: board}).unwrap()));
}

pub async fn spawn_bot(tx: &mpsc::UnboundedSender<Message>) {
    let (board, board2) = setup_test_boards();
    loop {
        send_board(tx, board.clone());
    }
}

async fn handle_websocket_async(socket: warp::ws::WebSocket) {
    let (mut ws_tx, mut ws_rx) = socket.split();


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

    // initialize the session by sending a join success message
    let message = OutgoingMessage::JoinGameSuccess {
        color: PlayerColor::Black,
        session: Uuid::new_v4().to_string(),
    };
    if let Ok(success_message) = serde_json::to_string(&message) {
        tx.send(warp::ws::Message::text(success_message)).unwrap();
    }
    // send a game status message, letting the gui know an opponent has joined
    let message = OutgoingMessage::GameStatus { game_started: true };
    if let Ok(success_message) = serde_json::to_string(&message) {
        tx.send(warp::ws::Message::text(success_message)).unwrap();
    }

    tokio::task::spawn(
        async move {
            spawn_bot(&tx).await;
        }
    );

    // while let Some(result) = ws_rx.next().await {
    //     let message = match result {
    //         Ok(message) => message,
    //         Err(e) => {
    //             eprintln!("websocket error {}", e);
    //             break;
    //         }
    //     };
    //     if message.is_text() {
    //         dbg!(message);
    //     }
    // }
}
#[tokio::main]
async fn main() {
    // spool up a bot that will respond to a board state with its
    // suggested move
    // let args: Vec<String> = env::args().collect();
    // dbg!(&args);

    // let (mut socket, response) =
    //     connect(Url::parse("ws://127.0.0.1:7878/ws").unwrap()).expect("Can't connect");

    // let user_id = Uuid::new_v4();

    // if args.len() > 1 {
    //     let message = IncomingMessage::JoinGame {
    //         user_id: user_id.to_string(),
    //         game_id: args[1].to_string(),
    //     };

    //     // initialize the session_id with something useless
    //     let mut current_color = Color::Black;

    //     socket.send(Message::Text(
    //         serde_json::to_string(&message).expect("Couldn't serialize message"),
    //     ));

    //     loop {
    //         let msg = socket.read().expect("Error reading WS message");
    //         handle_message(msg, user_id, &mut current_color, &mut socket);
    //     }
    // } else {
    //     make_a_move(&mut Board::setup_default_board(), Color::White);
    // }
    let websocket =
        warp::path("ws")
            .and(warp::ws())
            .map(|ws: warp::ws::Ws| {
                ws.on_upgrade(move |socket| handle_websocket_async(socket))
            });


    warp::serve(websocket)
        .run(([127, 0, 0, 1], 7878))
        .await;

}

