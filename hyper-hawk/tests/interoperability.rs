extern crate time;
extern crate hawk;
extern crate hyper;
extern crate hyper_hawk;
extern crate url;

use std::process::{Command, Child};
use hawk::{Request, Credentials, Key, SHA256};
use std::io::Read;
use std::net::TcpListener;
use hyper_hawk::Scheme;
use hyper::Client;
use hyper::header;
use url::Url;

// the port the JS side uses
const PORT: u16 = 62835;
const CALLBACK_PORT: u16 = PORT + 1;

fn start_node_script(script: &str) -> Child {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", CALLBACK_PORT)).unwrap();

    // TODO: run 'npm install' if node_modules do not exist (or maybe skip?)
    let child = Command::new("node")
        .arg(script)
        .arg(format!("{}", PORT))
        .arg(format!("{}", CALLBACK_PORT))
        .current_dir("tests/node")
        .spawn()
        .expect("node command failed to start");

    // wait until the process is ready, signalled by a connect to the callback port
    for stream in listener.incoming() {
        drop(stream);
        break;
    }
    child
}

#[test]
fn client_with_header() {
    let mut child = start_node_script("serve-one.js");

    let credentials = Credentials {
        id: "dh37fgj492je".to_string(),
        key: Key::new("werxhqb98rpaxn39848xrunpaw3489ruxnpa98w4rxn", &SHA256),
    };
    let url = Url::parse(&format!("http://localhost:{}/resource", PORT)).unwrap();
    let request = Request::new()
        .method("GET")
        .url(&url)
        .unwrap()
        .ext(Some("ext-content"));
    let mut headers = hyper::header::Headers::new();
    let header = request.generate_header(&credentials).unwrap();
    headers.set(header::Authorization(Scheme(header)));

    let client = Client::new();
    let mut res = client.get(url.as_str())
        .headers(headers)
        .send()
        .unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();
    assert!(res.status == hyper::Ok);
    assert!(body == "Hello Steve ext-content");

    // TODO: validate server's signature

    drop(res);
    drop(client); // close the kept-alive connection

    child.wait().expect("Failure waiting for child");
}
