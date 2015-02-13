/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use resource_task::{Metadata, LoadData, TargetedLoadResponse, start_sending, ResponseSenders};
use resource_task::ProgressMsg::{Payload, Done};

use serialize::base64::FromBase64;

use hyper::mime::Mime;
use url::{percent_decode, SchemeData};

use std::sync::mpsc::Sender;

pub fn factory(load_data: LoadData, start_chan: Sender<TargetedLoadResponse>) {
    // NB: we don't spawn a new task.
    // Hypothesis: data URLs are too small for parallel base64 etc. to be worth it.
    // Should be tested at some point.
    // Left in separate function to allow easy moving to a task, if desired.
    load(load_data, start_chan)
}

fn load(load_data: LoadData, start_chan: Sender<TargetedLoadResponse>) {
    let url = load_data.url;
    assert!("data" == url.scheme.as_slice());

    let mut metadata = Metadata::default(url.clone());

    let senders = ResponseSenders {
        immediate_consumer: start_chan,
        eventual_consumer: load_data.consumer,
    };

    // Split out content type and data.
    let mut scheme_data = match url.scheme_data {
        SchemeData::NonRelative(scheme_data) => scheme_data,
        _ => panic!("Expected a non-relative scheme URL.")
    };
    match url.query {
        Some(query) => {
            scheme_data.push_str("?");
            scheme_data.push_str(query.as_slice());
        },
        None => ()
    }
    let parts: Vec<&str> = scheme_data.as_slice().splitn(1, ',').collect();
    if parts.len() != 2 {
        start_sending(senders, metadata).send(Done(Err("invalid data uri".to_string()))).unwrap();
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

    let progress_chan = start_sending(senders, metadata);
    let bytes = percent_decode(parts[1].as_bytes());

    if is_base64 {
        // FIXME(#2909): It’s unclear what to do with non-alphabet characters,
        // but Acid 3 apparently depends on spaces being ignored.
        let bytes = bytes.into_iter().filter(|&b| b != ' ' as u8).collect::<Vec<u8>>();
        match bytes.as_slice().from_base64() {
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

#[cfg(test)]
fn assert_parse(url:          &'static str,
                content_type: Option<(String, String)>,
                charset:      Option<String>,
                data:         Option<Vec<u8>>) {
    use std::sync::mpsc::channel;
    use url::Url;
    use sniffer_task;

    let (start_chan, start_port) = channel();
    let sniffer_task = sniffer_task::new_sniffer_task();
    load(LoadData::new(Url::parse(url).unwrap(), start_chan), sniffer_task);

    let response = start_port.recv().unwrap();
    assert_eq!(&response.metadata.content_type, &content_type);
    assert_eq!(&response.metadata.charset,      &charset);

    let progress = response.progress_port.recv().unwrap();

    match data {
        None => {
            assert_eq!(progress, Done(Err("invalid data uri".to_string())));
        }
        Some(dat) => {
            assert_eq!(progress, Payload(dat));
            assert_eq!(response.progress_port.recv().unwrap(), Done(Ok(())));
        }
    }
}

#[test]
fn empty_invalid() {
    assert_parse("data:", None, None, None);
}

#[test]
fn plain() {
    assert_parse("data:,hello%20world", None, None, Some(b"hello world".iter().map(|&x| x).collect()));
}

#[test]
fn plain_ct() {
    assert_parse("data:text/plain,hello",
        Some(("text".to_string(), "plain".to_string())), None, Some(b"hello".iter().map(|&x| x).collect()));
}

#[test]
fn plain_charset() {
    assert_parse("data:text/plain;charset=latin1,hello",
        Some(("text".to_string(), "plain".to_string())), Some("latin1".to_string()), Some(b"hello".iter().map(|&x| x).collect()));
}

#[test]
fn base64() {
    assert_parse("data:;base64,C62+7w==", None, None, Some(vec!(0x0B, 0xAD, 0xBE, 0xEF)));
}

#[test]
fn base64_ct() {
    assert_parse("data:application/octet-stream;base64,C62+7w==",
        Some(("application".to_string(), "octet-stream".to_string())), None, Some(vec!(0x0B, 0xAD, 0xBE, 0xEF)));
}

#[test]
fn base64_charset() {
    assert_parse("data:text/plain;charset=koi8-r;base64,8PLl9+XkIO3l5Pfl5A==",
        Some(("text".to_string(), "plain".to_string())), Some("koi8-r".to_string()),
        Some(vec!(0xF0, 0xF2, 0xE5, 0xF7, 0xE5, 0xE4, 0x20, 0xED, 0xE5, 0xE4, 0xF7, 0xE5, 0xE4)));
}
