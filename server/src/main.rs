
use futures::{SinkExt, StreamExt, TryFutureExt};

use server::{session_handling, websocket_messaging, debug};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use tokio;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::Filter;

async fn handle_websocket_async(
    websocket: warp::ws::WebSocket,
    sessions: Arc<RwLock<session_handling::SessionHandler>>,
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
            websocket_messaging::handle_incoming_ws_message(message, &sessions, &tx).await;
        }
    }

    // // when the websocket closes, do some cleanup
    // // go through all the sessions that the player was subscribed to
    // let mut session = sessions.write().await;
    // for id in user_ids_on_websocket {
    //     session.delete_player(id);
    // }


}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        // special debug mode where we just send the board state right at the frontend
        let mut path = PathBuf::new();
        path.push(args[0].to_string());

        debug::spawn_debug_server(path).await;

    } else {
        // handle the page-serving side of the website
        let default = warp::path::end().and(warp::fs::file("./server_files/hello.html"));
    
        let join = warp::path("join").and(warp::fs::file("./server_files/hello.html"));
    
        let pages = warp::fs::dir("./server_files/");
    
        let sessions: Arc<RwLock<session_handling::SessionHandler>> = Arc::new(RwLock::new(session_handling::SessionHandler::new()));
    
        let sessions = warp::any().map(move || sessions.clone());
    
        let websocket =
            warp::path("ws")
                .and(warp::ws())
                .and(sessions)
                .map(|ws: warp::ws::Ws, sessions| {
                    ws.on_upgrade(move |socket| handle_websocket_async(socket, sessions))
                });
                
        let routes = pages
            .or(join)
            .or(websocket)
            .or(default)
            // serve 404s if the file doesn't exist and the client isn't asking for the default page
            .or(warp::fs::file("./server_files/404.html"));
    
        warp::serve(routes)
            .run(([127, 0, 0, 1], 7878))
            .await;
    }

}
