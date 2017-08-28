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

    // redo this with just conn.query inside the loop
    let q = conn.prepare("SELECT * FROM auth WHERE application=$1 AND host=$2").unwrap();
    for host in (*data).iter() {
        for row in &q.execute(&[&conf.application(), &host]).unwrap() {
            let id: i32 = row.get(0);
            println!("ID: {}", id);
        }
        println!("Host: {}, Query: {:?}", host, q);
    }

    Ok(())
}