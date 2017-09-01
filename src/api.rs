use std::io;
use futures::{Future, Stream};
use futures::future::Either;
use hyper::{self, Method, Request};
use hyper::client::{Client, HttpConnector};
use hyper::header::{ContentLength, ContentType};
use tokio_core::reactor::Core;
use tokio_core;
use std::str;
use std::time::Duration;
use serde_json::{self, Value};
use std::sync::{Arc, Mutex};
use config::Config;
use std::thread::{self, JoinHandle};
use auth_key::AuthKey;

pub struct Api {
    core: Core,
    client: Client<HttpConnector>,
    config: Config,
}

impl Api {
    pub fn new() -> Api {
        let mut core = match Core::new() {
            Ok(core) => core,
            Err(e) => panic!("Failed to create API instance due to an error: {}", e.to_string()),
        };

        let client = Client::new(&core.handle());
        let config = Config::new();

        Api {
            core: core,
            client: client,
            config: config,
        }
    }

    pub fn get_available_hosts (&mut self, available_hosts: &Arc<Mutex<Vec<String>>>) {
        let hosts = self.config.get_hosts().unwrap();

        // Check if the hosts are reachable.
        for elem in hosts.iter() {
            let elem_copy = elem.clone();
            let core_ref = &mut self.core;
            let client_ref = &self.client;
            match check_host_availability(core_ref, client_ref, elem) {
                Ok(_) => {
                    let mut lock = available_hosts.lock().unwrap();
                    (*lock).push(elem_copy);
                },
                Err(e) => {
                    if e == "Client timed out while connecting.".to_string() {
                        println!("The client timed out, host {} is unavailable.", elem);
                    } else {
                        println!("An error occurred connecting while attempting to connect to {}: {}", elem, e);
                    }
                }   
            };
        }
    }

    pub fn auth_request (&mut self, host: String) -> io::Result<Value> {
        let application = self.config.application();
        let appuser = self.config.appuser();
        
        let json = format!(r#"{{"application":"{}","user":"{}"}}"#, application, appuser);
        let uri = format!("http://{}/api/v1/auth/request", host)[..].parse().unwrap();

        let mut req = Request::new(Method::Post, uri);
        req.headers_mut().set(ContentType::json());
        req.headers_mut().set(ContentLength(json.len() as u64));
        req.set_body(json);

        let post = self.client.request(req).and_then(|res| {
            res.body().concat2().and_then(move |body| {
                let v: Value = match serde_json::from_slice(&body).map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        e
                    )
                }) {
                    Ok(res) => res,
                    Err(e) => panic!("Error parsing JSON response: {}", e),
                };
                Ok(v)
            })
        });

        let post_result = match self.core.run(post) {
            Ok(result) => result,
            Err(e) => panic!("Error core.run: {}", e),
        };

        Ok(post_result)
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
            let core_ref = &mut self.core;
            let client_ref = &self.client;
            // Note: This will panic if the OS fails to make the thread.
            // Use a thread Builder to error handle if needed.
            let handle = thread::spawn(move || {
                if let Err(e) = auth_check(&host, &id, core_ref, client_ref) {
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
fn auth_check (host: &String, id: &String, core: &mut Core, 
client: &Client<HttpConnector>) -> io::Result<()> {
    let copy_uri = format!("http://{}/api/v1/auth/check/{}", host, id);
    let uri = copy_uri.parse().unwrap();
    let timeout = tokio_core::reactor::Timeout::new(Duration::from_secs(10), &core.handle()).unwrap();
    let request = client.get(uri).map(|res| {
        println!("Got status: {}", res.status());
    });
    let work = request.select2(timeout).then(|res| match res {
        Ok(Either::A((got, _timeout))) => Ok(got),
        Ok(Either::B((_timeout_error, _get))) => {
            Err(hyper::Error::Io(io::Error::new(
                io::ErrorKind::TimedOut,
                "Client timed out while connecting.",
            )))
        },
        Err(Either::A((get_error, _timeout))) => Err(get_error),
        Err(Either::B((timeout_error, _get))) => Err(From::from(timeout_error)),
    });
    Ok(())
}

pub fn check_host_availability (core: &mut Core, client: &Client<HttpConnector>,
host: &String) -> Result<(), String> {
    let copy_uri = format!("http://{}", host);
    let uri = copy_uri.parse().unwrap();
    let timeout = tokio_core::reactor::Timeout::new(Duration::from_secs(10), &core.handle()).unwrap();

    let request = client.get(uri);

    let work = request.select2(timeout).then(|res| match res {
        Ok(Either::A((got, _timeout))) => Ok(got),
        Ok(Either::B((_timeout_error, _get))) => {
            Err(hyper::Error::Io(io::Error::new(
                io::ErrorKind::TimedOut,
                "Client timed out while connecting.",
            )))
        },
        Err(Either::A((get_error, _timeout))) => Err(get_error),
        Err(Either::B((timeout_error, _get))) => Err(From::from(timeout_error)),
    });

    match core.run(work) {
        Ok(_) => Ok(()),
        Err(e) => { println!("Error checking host availability: {:?}", 
            e.to_string()); return Err(e.to_string()); },
    }
}