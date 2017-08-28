#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate ws;
extern crate env_logger;
extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate serde_json;
extern crate toml;
#[macro_use]
extern crate serde_derive;

mod jobs;
mod api;
mod config;

use config::Config;

use ws::{Sender, Message, Handler, Factory};

use std::thread;
use std::time::Duration;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};

use std::fs::File;
use rocket::response::content;

struct ServerFactory;
struct ServerHandler {
    // ws: Sender,
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

    fn connection_made(&mut self, _: Sender) -> ServerHandler {
        ServerHandler {
            // ws: ws,
        }
    }

    fn client_connected(&mut self, _: Sender) -> ServerHandler {
        ServerHandler {
            // ws: ws,
        }
    }
}

#[get("/")]
fn index() -> Option<content::Html<File>> {
    let path = "index.html";
    File::open(&path).map(|f| content::Html(f)).ok()
}

fn main() {
    // Set up an Arc container for the available hosts, so that multiple
    // references to it can be active at once.
    let available_hosts: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let appconfig: Config = Config::new();

    // Create a channels for communication between the job runner and
    // the broadcaster thread.
    let (tx, rx) = channel();

    // Initialize the websocket server. TODO ERROR HANDLE.
    let me = match ws::WebSocket::new(ServerFactory) { 
        Ok(ws) => ws,
        Err(e) => panic!("Unable to create websocket: {}", e.to_string()),
    };

    // Get a sender for ALL connections to the websocket
    let broadcaster = me.broadcaster();

    let server = thread::spawn(move || {
        match me.listen("127.0.0.1:80") {
            Ok(server) => server,
            Err(e) => panic!("Unable to start the websocket server: {}", e.to_string()),
        }
    });

    thread::sleep(Duration::from_millis(10));

    //TODO: Remove these, just to shut warnings up.
    println!("{:?}", available_hosts);
    println!("{} {}", appconfig.application(), appconfig.appuser());


    // Spawn a thread to run job updates.
    jobs::job_runner(available_hosts.clone(), appconfig.clone(), tx.clone());


    thread::spawn(move || {
        while let Ok(data) = rx.recv() {
            println!("Received data {}", data);
            broadcaster.send(data).unwrap();
        }
    });

    rocket().launch();

    // in case rocket fails somehow..
    println!("Rocket crashed.");
    server.join().unwrap();
}

// Returns an instance of Rocket with the correct routes and config.
fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/",
        routes![
            index,
        ])
}