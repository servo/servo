/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::server::{Server, Request as HyperRequest, Response as HyperResponse};
use net::fetch::request::{Context, fetch, Referer, Request};
use net_traits::response::{ResponseBody};
use std::rc::Rc;
use url::Url;

#[test]
fn test_fetch_response_body_matches_const_message() {

    static MESSAGE: &'static [u8] = b"Hello World!";
    fn handler(_: HyperRequest, response: HyperResponse) {
        response.send(MESSAGE).unwrap();
    }

    // this is a Listening server because of handle_threads()
    let mut server = Server::http("0.0.0.0:0").unwrap().handle_threads(handler, 1).unwrap();
    let port = server.socket.port().to_string();
    let mut url_string = "http://localhost:".to_owned();
    url_string.push_str(&port);
    let url = Url::parse(&url_string).unwrap();

    let mut request = Request::new(url, Context::Fetch, false);
    request.referer = Referer::NoReferer;
    let wrapped_request = Rc::new(request);

    let _ = fetch(wrapped_request, false);

    // TODO this will be useful, when response body gets set
    // match fetch_response.body {
    //     ResponseBody::Receiving(body) | ResponseBody::Done(body) => {
    //         assert_eq!(body, MESSAGE);
    //     },
    //     _ => { panic!() }
    // };

    let _ = server.close();
}
