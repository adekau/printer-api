use std::sync::mpsc::Sender as ThreadOut;
use std::thread;
use std::time::Duration;
use serde_json;
use config::Config;
use std::io;
use std::sync::{Arc, Mutex};
use api;

pub fn job_runner(available_hosts: Arc<Mutex<Vec<String>>>, config: &Config, tx: ThreadOut<String>) {

    thread::spawn(move || {
        // Setup the authentication.
        auth_setup(available_hosts, config);

        loop {
            tx.send("hello world".to_string()).ok();
            thread::sleep(Duration::from_millis(5000));
        }

    });

}

// The hosts are already determined to be available or unavailable. 
fn auth_setup (available_hosts: Arc<Mutex<Vec<String>>>, config: &Config) -> io::Result<()> {
    api::get_available_hosts(&config, available_hosts);
    Ok(())
}