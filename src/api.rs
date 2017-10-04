use std::io::{self, stdout, copy};
use std::collections::HashMap;
use std::str;
use std::time::Duration;
use reqwest;
use std::sync::{Arc, Mutex};
use config::Config;
use std::thread::{self, JoinHandle};
use auth_key::AuthKey;

#[derive(Deserialize, Debug)]
pub struct AuthResponse {
    pub id: String,
    pub key: String,
}

pub struct Api {
    config: Config,
}

impl Api {
    pub fn new() -> Api {
        let config = Config::new();

        Api {
            config: config,
        }
    }

    pub fn get_available_hosts (&mut self, available_hosts: &Arc<Mutex<Vec<String>>>) {
        let hosts = self.config.get_hosts().unwrap().clone();
        let mut handles: Vec<JoinHandle<_>> = Vec::new();

        // Check if the hosts are reachable.
        for elem in hosts.iter() {
            let elem_copy = elem.to_owned();
            let av = available_hosts.to_owned();

            let handle = thread::spawn(move || {
                match check_host_availability(&elem_copy) {
                    Ok(_) => {
                        let mut lock = av.lock().unwrap();
                        (*lock).push(elem_copy);
                    },
                    Err(e) => {
                        if e == "Client timed out while connecting.".to_string() {
                            println!("The client timed out, host {} is unavailable.", elem_copy);
                        } else {
                            println!("An error occurred while attempting to connect to {}: {}", elem_copy, e);
                        }
                    }   
                };
            });

            handles.push(handle);
        }

        for handle in handles {
            if let Err(e) = handle.join() {
                println!("Thread handle encountered an error: {:?}", e);   
            };
        }
    }

    pub fn auth_request (&mut self, host: String) -> Result<AuthResponse, String> {
        let application = self.config.application();
        let appuser = self.config.appuser();
        
        let mut json = HashMap::new();
        json.insert("application", application);
        json.insert("user", appuser);

        let uri: reqwest::Url = format!("http://{}/api/v1/auth/request", host)[..].parse().unwrap();

        let client = match reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
        {
            Ok(client) => client,
            Err(e) => return Err(e.to_string()),
        };

        let mut response = match client.post(uri)
            .json(&json)
            .send()
        {
            Ok(res) => res,
            Err(e) => return Err(e.to_string()),
        };

        let ar: AuthResponse = match response.json() {
            Ok(ar) => ar,
            Err(e) => return Err(e.to_string()),
        };

        Ok(ar)
    }

    // Spawn a thread to auth check a host.
    // TODO: This should probably return a result for error checking.
    pub fn auth_check_all (&mut self, host_auth: Arc<Mutex<Vec<AuthKey>>>) -> io::Result<()> {
        let data = match host_auth.lock() {
            Ok(data) => data,
            Err(_) => return Err(io::Error::new(io::ErrorKind::Other, "Couldn't lock host_auth")),  
        };
        let mut handles: Vec<JoinHandle<_>> = Vec::new();

        for host in (*data).iter() {
            // Clone to make lifetime 'static for thread::spawn
            let (host, id) = (host.host().clone(), host.id().clone());
            // Note: This will panic if the OS fails to make the thread.
            // Use a thread Builder to error handle if needed.
            let handle = thread::spawn(move || {
                if let Err(e) = auth_check(&host, &id) {
                    return Err(e);   
                };
                Ok(())
            });

            handles.push(handle);
        }

        for handle in handles {
            if let Err(e) = handle.join() {
                println!("Thread handle encountered an error: {:?}", e);   
            };
        }

        Ok(())
    }
}

    // Contact the host (http://{host}/api/v1/auth/check/{id})
fn auth_check (host: &String, id: &String) -> io::Result<()> {
    // let copy_uri = format!("http://{}/api/v1/auth/check/{}", host, id);
    // let uri = copy_uri.parse().unwrap();
    // let timeout = tokio_core::reactor::Timeout::new(Duration::from_secs(10), &core.handle()).unwrap();
    // let request = client.get(uri).map(|res| {
    //     println!("Got status: {}", res.status());
    // });
    // let work = request.select2(timeout).then(|res| match res {
    //     Ok(Either::A((got, _timeout))) => Ok(got),
    //     Ok(Either::B((_timeout_error, _get))) => {
    //         Err(hyper::Error::Io(io::Error::new(
    //             io::ErrorKind::TimedOut,
    //             "Client timed out while connecting.",
    //         )))
    //     },
    //     Err(Either::A((get_error, _timeout))) => Err(get_error),
    //     Err(Either::B((timeout_error, _get))) => Err(From::from(timeout_error)),
    // });
    Ok(())
}

pub fn check_host_availability (host: &String) -> Result<(), String> {
    let copy_uri = format!("http://{}", host);
    let uri: reqwest::Url = copy_uri.parse().unwrap();

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
    {
        Ok(client) => client,
        Err(e) => return Err(e.to_string()),
    };

    if let Err(e) = client.get(uri).send() {
        return Err(e.to_string());
    };

    Ok(())
}