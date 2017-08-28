use std::io;
use futures::{Future, Stream};
use futures::future::Either;
use hyper::{self, Client, Method, Request};
use hyper::header::{ContentLength, ContentType};
use tokio_core::reactor::Core;
use tokio_core;
use std::str;
use std::time::Duration;
use serde_json::{self, Value};

pub fn check_host_availability (host: &String) -> Result<(), String> {
    let mut core = match Core::new() {
        Ok(core) => core,
        Err(e) => return Err(e.to_string()),
    };
    let handle = core.handle();
    let client = Client::new(&handle);

    let copy_uri = format!("http://{}", host);
    let uri = copy_uri.parse().unwrap();
    let timeout = tokio_core::reactor::Timeout::new(Duration::from_secs(10), &handle).unwrap();

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

pub fn auth_request (host: String, application: String, appuser: String) -> io::Result<Value> {
    let mut core = Core::new().unwrap();
    let client = Client::new(&core.handle());
    
    let json = format!(r#"{{"application":"{}","user":"{}"}}"#, application, appuser);
    let uri = format!("http://{}/api/v1/auth/request", host)[..].parse().unwrap();

    let mut req = Request::new(Method::Post, uri);
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(json.len() as u64));
    req.set_body(json);

    let post = client.request(req).and_then(|res| {
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

    let post_result = match core.run(post) {
        Ok(result) => result,
        Err(e) => panic!("Error core.run: {}", e),
    };

    Ok(post_result)
}