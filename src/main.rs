#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate ws;
extern crate env_logger;
extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate serde_json;

mod jobs;
mod api;

use ws::{Sender, Message, Handler, Factory};

use std::thread;
use std::time::Duration;
use std::sync::mpsc::channel;

use std::fs::File;
use rocket::response::content;

struct ServerFactory;
struct ServerHandler {
    ws: Sender,
}

impl Handler for ServerHandler {
    fn on_open(&mut self, _: ws::Handshake) -> ws::Result<()> {
        println!("Connection opened.");
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        println!("Got message {}", msg);
        Ok(())
    }
}

impl Factory for ServerFactory {
    type Handler = ServerHandler;

    fn connection_made(&mut self, ws: Sender) -> ServerHandler {
        ServerHandler {
            ws: ws,
        }
    }

    fn client_connected(&mut self, ws: Sender) -> ServerHandler {
        ServerHandler {
            ws: ws,
        }
    }
}

#[get("/")]
fn index() -> Option<content::Html<File>> {
    let path = "index.html";
    File::open(&path).map(|f| content::Html(f)).ok()
}

fn main() {
    let me = ws::WebSocket::new(ServerFactory).unwrap();

    // Get a sender for ALL connections to the websocket
    let broadcaster = me.broadcaster();

    let server = thread::spawn(move || {
        me.listen("127.0.0.1:80").unwrap();
    });

    thread::sleep(Duration::from_millis(10));

    // Create a channels for communication between the job runner and
    // the broadcaster thread.
    let (tx, rx) = channel();

    // Spawn a thread to run job updates.
    jobs::job_runner(tx.clone());

    thread::spawn(move || {
        while let Ok(data) = rx.recv() {
            println!("Received data {}", data);
            broadcaster.send(data).unwrap();
        }
    });

    rocket::ignite().mount("/", routes![index]).launch();

    // in case rocket fails somehow..
    println!("Rocket crashed.");
    server.join().unwrap();
}