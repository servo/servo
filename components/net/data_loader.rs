/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use mime_classifier::MIMEClassifier;
use net_traits::ProgressMsg::{Payload, Done};
use net_traits::{LoadData, Metadata, LoadConsumer};
use resource_task::start_sending;

use rustc_serialize::base64::FromBase64;

use hyper::mime::Mime;
use std::sync::Arc;
use url::SchemeData;
use url::percent_encoding::percent_decode;

pub fn factory(load_data: LoadData, senders: LoadConsumer, _classifier: Arc<MIMEClassifier>) {
    // NB: we don't spawn a new task.
    // Hypothesis: data URLs are too small for parallel base64 etc. to be worth it.
    // Should be tested at some point.
    // Left in separate function to allow easy moving to a task, if desired.
    load(load_data, senders)
}

pub fn load(load_data: LoadData, start_chan: LoadConsumer) {
    let url = load_data.url;
    assert!(&*url.scheme == "data");

    let mut metadata = Metadata::default(url.clone());

    // Split out content type and data.
    let mut scheme_data = match url.scheme_data {
        SchemeData::NonRelative(scheme_data) => scheme_data,
        _ => panic!("Expected a non-relative scheme URL.")
    };
    match url.query {
        Some(ref query) => {
            scheme_data.push_str("?");
            scheme_data.push_str(query);
        },
        None => ()
    }
    let parts: Vec<&str> = scheme_data.splitn(2, ',').collect();
    if parts.len() != 2 {
        start_sending(start_chan,
                      metadata).send(Done(Err("invalid data uri".to_string()))).unwrap();
        return;
    }

    // ";base64" must come at the end of the content type, per RFC 2397.
    // rust-http will fail to parse it because there's no =value part.
    let mut is_base64 = false;
    let mut ct_str = parts[0];
    if ct_str.ends_with(";base64") {
        is_base64 = true;
        ct_str = &ct_str[..ct_str.as_bytes().len() - 7];
    }

    // Parse the content type using rust-http.
    // FIXME: this can go into an infinite loop! (rust-http #25)
    let content_type: Option<Mime> = ct_str.parse().ok();
    metadata.set_content_type(content_type.as_ref());

    let progress_chan = start_sending(start_chan, metadata);
    let bytes = percent_decode(parts[1].as_bytes());

    if is_base64 {
        // FIXME(#2909): It’s unclear what to do with non-alphabet characters,
        // but Acid 3 apparently depends on spaces being ignored.
        let bytes = bytes.into_iter().filter(|&b| b != ' ' as u8).collect::<Vec<u8>>();
        match bytes.from_base64() {
            Err(..) => {
                progress_chan.send(Done(Err("non-base64 data uri".to_string()))).unwrap();
            }
            Ok(data) => {
                progress_chan.send(Payload(data)).unwrap();
                progress_chan.send(Done(Ok(()))).unwrap();
            }
        }
    } else {
        progress_chan.send(Payload(bytes)).unwrap();
        progress_chan.send(Done(Ok(()))).unwrap();
    }
}
