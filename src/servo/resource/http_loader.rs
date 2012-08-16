export factory;

import comm::Chan;
import task::spawn;
import resource_task::{ProgressMsg, Payload, Done};
import std::net::url::url;
import http_client::{uv_http_request};
import result::{ok, err};

fn factory(+url: url, progress_chan: Chan<ProgressMsg>) {
    assert url.scheme == ~"http";

    do spawn {
        let url = copy url;

        #debug("http_loader: requesting via http: %?", copy url);
        let request = uv_http_request(copy url);
        let errored = @mut false;
        do request.begin |event| {
            let url = copy url;
            match event {
              http_client::Status(*) => { }
              http_client::Payload(data) => {
                #debug("http_loader: got data from %?", url);
                let mut crap = none;
                *data <-> crap;
                progress_chan.send(Payload(option::unwrap(crap)));
              }
              http_client::Error(*) => {
                #debug("http_loader: error loading %?", url);
                *errored = true;
                progress_chan.send(Done(err(())));
              }
            }
        }

        if !*errored {
            progress_chan.send(Done(ok(())));
        }
    }
}
