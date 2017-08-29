use std::sync::mpsc::Sender as ThreadOut;
use std::thread;
use std::time::Duration;
use serde_json;
use config::Config;
use std::io;
use std::sync::{Arc, Mutex};
use postgres::{Connection, TlsMode};
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
    let conf = config.clone();
    let conn = Connection::connect("postgres://Alex@127.0.0.1:5432", TlsMode::None)?;

    api::get_available_hosts(config, &available_hosts);
    let data = available_hosts.lock().unwrap();

    println!("Available Hosts: {:?}", *data);

    for host in (*data).iter() {
        let p = conn.query("SELECT * FROM auth WHERE application=$1 AND host=$2", &[&conf.application(), &host]).unwrap();

        // Auth Keys should maybe be a vec of AuthKey structs with the following components:
        // - Host
        // - Id
        // - Key
        // - Status (AuthKeyStatus::Authorized, AuthKeyStatus::Unknown, AuthKeyStatus::Unauthorized)
        // AuthKeyStatus::Unauthorized means that the key will need to be regenerated and that host should
        // not be contacted in future data retrieval steps.

        if p.len() > 0 {
            for row in &p {
                let host: String = row.get(3);
                let id: String = row.get(4);
                let key: String = row.get(5);
                println!("HOST: {:?} ID: {:?}, KEY: {:?}", host, id, key);
            }
        } else {
            // Generate a key and store it in database and data structure here.
        }
    }

    Ok(())
}

// Step 1: Check host availability.
// Step 2: Check auth keys /api/v1/auth/check/{key}. Plan is to make it so it just makes a
//         host unavailable if the key expires while the api is running and just send a
//         message to the front end to press a button to regenerate it.
// Step 3: Gather data and store in the database.

// fn loop_step () {}