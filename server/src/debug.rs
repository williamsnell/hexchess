use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::{env, net::TcpStream, thread, time::Duration};

use futures::{channel::mpsc::UnboundedSender, SinkExt, StreamExt, TryFutureExt};
use hexchesscore::{Board, Color};
use serde::Deserialize;
use tokio::sync::Mutex;
use tokio::{self, sync::mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;
use url::Url;
use warp::ws::Message;
use warp::Filter;

use tungstenite::{connect, stream::MaybeTlsStream, WebSocket};
use uuid::{self, Uuid};

use std::io::BufReader;

use api::{IncomingMessage, OutgoingMessage, PlayerColor};

pub fn send_board(transmitter: &mpsc::UnboundedSender<Message>, board: Board) {
    let _result = transmitter.send(Message::text(
        serde_json::to_string(&OutgoingMessage::BoardState { board: board }).unwrap(),
    ));
}

pub async fn debug_sender(tx: Arc<Mutex<mpsc::UnboundedSender<Message>>>, board: &'static str) {
    loop {
        let f = File::open(board).expect("Couldn't open file");
        let reader = BufReader::new(f);
        let board: Result<Board, serde_json::Error>  = serde_json::from_reader(reader);

        if let Ok(board) = board {
            let tx = tx.lock().await;
            send_board(&tx, board.clone());
        }
    }
}

async fn handle_websocket_async(socket: warp::ws::WebSocket, board: &'static str) {
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

    let tx = Arc::new(Mutex::new(tx));

    tokio::task::spawn(debug_sender(tx.clone(), board.clone()));
}

pub async fn spawn_debug_server(file_to_watch: PathBuf) {
    let websocket = warp::path("ws").and(warp::ws()).map(|ws: warp::ws::Ws| {
        ws.on_upgrade(|socket| handle_websocket_async(socket, "./debug/board.json"))
    });

    warp::serve(websocket).run(([127, 0, 0, 1], 7878)).await;
}
