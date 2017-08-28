use std::sync::mpsc::Sender as ThreadOut;
use std::thread;
use std::time::Duration;
use serde_json;
use config::Config;
use std::io;
use std::sync::{Arc, Mutex};
use api;

pub fn job_runner(available_hosts: Arc<Mutex<Vec<String>>>, config: Config, tx: ThreadOut<String>) {

    thread::spawn(move || {
        // Setup the authentication.
        auth_setup(available_hosts, config).expect("Did not setup properly");

        loop {
            tx.send("hello world".to_string()).ok();
            thread::sleep(Duration::from_secs(5));
        }

    });

}

// Step 1: Determine the available hosts.
// Step 2: For each host, check if the hosts already have keys in the database.
// Step 3: If true: auth_check
//         If false: generate the key then auth_check.
fn auth_setup (available_hosts: Arc<Mutex<Vec<String>>>, config: Config) -> io::Result<()> {
    api::get_available_hosts(config, &available_hosts);
    let data = available_hosts.lock().unwrap();

    (*data).iter().map(|host: &String| {
        println!("Host: {}", host);
    }).collect();

    Ok(())
}