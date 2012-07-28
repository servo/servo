export factory;

import comm::{chan, methods};
import task::spawn;
import resource_task::{ProgressMsg, Payload, Done};
import std::net::url::url;
import http_client::{
    uv_http_request,
    Uri
};
import result::{ok, err};

fn factory(url: url, progress_chan: chan<ProgressMsg>) {
    assert url.scheme == ~"http";

    do spawn {
        #debug("http_loader: requesting via http: %?", url);
        let request = uv_http_request(url_to_http_client_uri(url));
        let errored = @mut false;
        do request.begin |event| {
            alt event {
              http_client::Status(*) { }
              http_client::Payload(data) {
                #debug("http_loader: got data from %?", url);
                let mut crap = none;
                *data <-> crap;
                progress_chan.send(Payload(option::unwrap(crap)));
              }
              http_client::Error(*) {
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

fn url_to_http_client_uri(url: url) -> Uri {
    {
        host: url.host,
        path: url.path
    }
}