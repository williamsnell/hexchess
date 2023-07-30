use futures::FutureExt;
use futures::StreamExt;
use server::ThreadPool;
use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};
use warp::Filter;

#[tokio::main]
async fn main() {
    let echo = warp::path("echo").and(warp::ws()).map(|ws: warp::ws::Ws| {
        ws.on_upgrade(|websocket| {
            let (tx, rx) = websocket.split();
            rx.forward(tx).map(|result| {
                if let Err(e) = result {
                    eprintln!("websocket error: {:?}", e);
                }
            })
        })
    });

    let current_dir = std::env::current_dir().expect("failed to read current directory");
    let routes = warp::get().and(echo.or(warp::fs::dir(current_dir)));
    warp::serve(routes)
        .tls()
        .cert_path("cert.pem")
        .key_path("key.rsa")
        .run(([0, 0, 0, 0], 9231))
        .await;
}
// let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
// let pool = ThreadPool::new(8);

// for stream in listener.incoming() {
//     let stream = stream.unwrap();

//     pool.execute(|| {
//         handle_connection(stream);
//     })
// }
// }

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /draw_hexagon.js HTTP/1.1" => ("HTTP/1.1 200 OK", "draw_hexagon.js"),
        "GET /hex_frontend_funcs.js HTTP/1.1" => ("HTTP/1.1 200 OK", "hex_frontend_funcs.js"),
        "GET /moves.json HTTP/1.1" => ("HTTP/1.1 200 OK", "moves.json"),
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let content_type = if filename.ends_with(".js") {
        "text/javascript"
    } else if filename.ends_with(".html") {
        "text/html"
    } else {
        ""
    };

    let response = format!("{status_line}\r\nContent-Length: {length}\r\nContent-Type: {content_type}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
