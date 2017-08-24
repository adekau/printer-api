use std::sync::mpsc::Sender as ThreadOut;
use std::thread;
use std::time::Duration;
use serde_json;

use api;

pub fn job_runner(tx: ThreadOut<String>) {

    thread::spawn(move || {
        let t: serde_json::Value = api::auth_request();
        println!("Got the result: {}", t["id"]);

        loop {
            tx.send("hello world".to_string()).ok();

            thread::sleep(Duration::from_millis(1000));
        }

    });

}