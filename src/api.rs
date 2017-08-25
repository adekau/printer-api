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
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let client = Client::new(&handle);

    let copy_uri = format!("http://{}", host);
    let uri = copy_uri.parse().unwrap();
    let timeout = tokio_core::reactor::Timeout::new(Duration::from_secs(10), &handle).unwrap();

    let request = client.get(uri).map(|res| {
        println!("Connected to host {}: {}", copy_uri, res.status());
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

    match core.run(work) {
        Ok(_) => Ok(()),
        Err(e) => {println!("{:?}", e.to_string()); return Err(e.to_string());},
    }
}

pub fn auth_request () -> Value {
    let mut core = Core::new().unwrap();
    let client = Client::new(&core.handle());
    
    let json = r#"{"application":"printmgr","user":"printmgr"}"#;
    let uri = "http://141.218.24.102/api/v1/auth/request".parse().unwrap();

    let mut req = Request::new(Method::Post, uri);
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(json.len() as u64));
    req.set_body(json);

    let post = client.request(req).and_then(|res| {
        res.body().concat2().and_then(move |body| {
            let v: Value = serde_json::from_slice(&body).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    e
                )
            }).unwrap();
            Ok(v)
        })
    });

    let post_result = core.run(post).unwrap();

    post_result
}