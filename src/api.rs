use std::io;
use std::collections::HashMap;
use std::str;
use std::time::Duration;
use reqwest;
use std::sync::{Arc, Mutex};
use config::Config;
use std::thread::{self, JoinHandle};
use auth_key::{AuthKey,AuthKeyStatus};

#[derive(Deserialize, Debug)]
pub struct AuthResponse {
    pub id: String,
    pub key: String,
}

#[derive(Deserialize, Debug)]
pub struct AuthCheckResponse {
    pub message: String,
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
    pub fn auth_check_all (&mut self, host_auth: Arc<Mutex<Vec<AuthKey>>>) -> io::Result<()> {
        let mut data = match host_auth.lock() {
            Ok(data) => data,
            Err(_) => return Err(io::Error::new(io::ErrorKind::Other, "Couldn't lock host_auth")),  
        };
        let mut handles: Vec<JoinHandle<Result<(String, AuthKeyStatus), String>>> = Vec::new();

        for host in (*data).iter() {
            // Clone to make lifetime 'static for thread::spawn
            let (host, id) = (host.host().clone(), host.id().clone());
            // Note: This will panic if the OS fails to make the thread.
            // Use a thread Builder to error handle if needed.
            let handle: JoinHandle<Result<(String, AuthKeyStatus), String>> = thread::spawn(move || {
                let result: AuthKeyStatus = match auth_check(&host, &id) {
                    Ok(result) => result,
                    Err(e) => return Err(e),
                };
                
                Ok((host, result))
            });

            handles.push(handle);
        }

        for handle in handles {
            match handle.join() {
                Ok(Ok((thost, result))) => {

                    for host in (*data).iter_mut() {
                        let result_copy = result.clone();
                        if host.host().to_owned() == thost {
                            host.set_status(result_copy);
                        }
                    }

                },
                Ok(Err(e)) => println!("Thread handle encountered an error: {:?}", e),
                Err(e) => println!("Thread handle encountered an error: {:?}", e),
            };
        }

        Ok(())
    }
}

    // Contact the host (http://{host}/api/v1/auth/check/{id})
fn auth_check (host: &String, id: &String) -> Result<AuthKeyStatus, String> {
    let copy_uri = format!("http://{}/api/v1/auth/check/{}", host, id);
    let uri: reqwest::Url = copy_uri.parse().unwrap();

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
    {
        Ok(client) => client,
        Err(e) => return Err(e.to_string()),
    };

    let mut response = match client.get(uri).send() {
        Ok(res) => res,
        Err(e) => return Err(e.to_string()),
    };

    let rr: AuthCheckResponse = match response.json() {
        Ok(r) => r,
        Err(e) => return Err(e.to_string()),
    };

    let result: AuthKeyStatus = match rr.message.as_ref() {
        "authorized" => AuthKeyStatus::Authorized,
        "unauthorized" => AuthKeyStatus::Unauthorized,
        "unknown" => AuthKeyStatus::Unknown,
        _ => AuthKeyStatus::None,
    };

    Ok(result)
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