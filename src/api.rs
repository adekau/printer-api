use std::io;
use futures::{Future, Stream};
use hyper::{Client, Method, Request};
use hyper::header::{ContentLength, ContentType};
use tokio_core::reactor::Core;
use std::str;
use serde_json::{self, Value};

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
        println!("POST: {}", res.status());

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