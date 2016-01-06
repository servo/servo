use hyper::server::{Server, Request as HyperRequest, Response as HyperResponse};
use net::fetch::request::{Context, fetch, Request};
use net_traits::{ResponseBody};
use std::cell::RefCell;
use std::rc::Rc;
use url::Url;

#[test]
fn test_fetch_response_body_matches_const_message() {
    
    static MESSAGE: &'static [u8] = b"Hello World!";
    fn handler(_: HyperRequest, response: HyperResponse) {
        response.send(MESSAGE).unwrap();
    }

    // this is a Listening server because of handle_threads()
    let server = Server::http("0.0.0.0:0").unwrap().handle_threads(handler, 1).unwrap();
    let port = server.socket.port().to_string();
    let mut url_string = "http://localhost:".to_owned();
    url_string.push_str(&port);
    let url = Url::parse(&url_string).unwrap();
    let request = Rc::new(RefCell::new(Request::new(url, Context::Fetch, false)));

    println!("calling request");
    let fetch_response = fetch(request, false);
    match fetch_response.body {
        ResponseBody::Receiving(body) | ResponseBody::Done(body) => {
            assert_eq!(body, MESSAGE);
        },
        _ => { panic!() }
    };
}