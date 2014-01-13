/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use resource_task::{Done, Payload, Metadata, LoadResponse, LoaderTask, start_sending};

use extra::url::Url;
use extra::base64::FromBase64;

use http::headers::test_utils::from_stream_with_str;
use http::headers::content_type::MediaType;

pub fn factory() -> LoaderTask {
    proc(url, start_chan) {
        // NB: we don't spawn a new task.
        // Hypothesis: data URLs are too small for parallel base64 etc. to be worth it.
        // Should be tested at some point.
        load(url, start_chan)
    }
}

fn load(url: Url, start_chan: Chan<LoadResponse>) {
    assert!("data" == url.scheme);

    let mut metadata = Metadata::default(url.clone());

    // Split out content type and data.
    let parts: ~[&str] = url.path.splitn(',', 1).to_owned_vec();
    if parts.len() != 2 {
        start_sending(start_chan, metadata).send(Done(Err(())));
        return;
    }

    // ";base64" must come at the end of the content type, per RFC 2397.
    // rust-http will fail to parse it because there's no =value part.
    let mut is_base64 = false;
    let mut ct_str = parts[0];
    if ct_str.ends_with(";base64") {
        is_base64 = true;
        ct_str = ct_str.slice_to(ct_str.as_bytes().len() - 7);
    }

    // Parse the content type using rust-http.
    // FIXME: this can go into an infinite loop! (rust-http #25)
    let content_type: Option<MediaType> = from_stream_with_str(ct_str);
    metadata.set_content_type(&content_type);

    let progress_chan = start_sending(start_chan, metadata);

    if is_base64 {
        match parts[1].from_base64() {
            Err(..) => {
                progress_chan.send(Done(Err(())));
            }
            Ok(data) => {
                progress_chan.send(Payload(data));
                progress_chan.send(Done(Ok(())));
            }
        }
    } else {
        // FIXME: Since the %-decoded URL is already a str, we can't
        // handle UTF8-incompatible encodings.
        progress_chan.send(Payload(parts[1].as_bytes().into_owned()));
        progress_chan.send(Done(Ok(())));
    }
}

#[cfg(test)]
fn assert_parse(url:          &'static str,
                content_type: Option<(~str, ~str)>,
                charset:      Option<~str>,
                data:         Option<~[u8]>) {
    use std::from_str::FromStr;

    let (start_port, start_chan) = Chan::new();
    load(FromStr::from_str(url).unwrap(), start_chan);

    let response = start_port.recv();
    assert_eq!(&response.metadata.content_type, &content_type);
    assert_eq!(&response.metadata.charset,      &charset);

    let progress = response.progress_port.recv();

    match data {
        None => {
            assert_eq!(progress, Done(Err(())));
        }
        Some(dat) => {
            assert_eq!(progress, Payload(dat));
            assert_eq!(response.progress_port.recv(), Done(Ok(())));
        }
    }
}

#[test]
fn empty_invalid() {
    assert_parse("data:", None, None, None);
}

#[test]
fn plain() {
    assert_parse("data:,hello%20world", None, None, Some(bytes!("hello world").into_owned()));
}

#[test]
fn plain_ct() {
    assert_parse("data:text/plain,hello",
        Some((~"text", ~"plain")), None, Some(bytes!("hello").into_owned()));
}

#[test]
fn plain_charset() {
    assert_parse("data:text/plain;charset=latin1,hello",
        Some((~"text", ~"plain")), Some(~"latin1"), Some(bytes!("hello").into_owned()));
}

#[test]
fn base64() {
    assert_parse("data:;base64,C62+7w==", None, None, Some(~[0x0B, 0xAD, 0xBE, 0xEF]));
}

#[test]
fn base64_ct() {
    assert_parse("data:application/octet-stream;base64,C62+7w==",
        Some((~"application", ~"octet-stream")), None, Some(~[0x0B, 0xAD, 0xBE, 0xEF]));
}

#[test]
fn base64_charset() {
    assert_parse("data:text/plain;charset=koi8-r;base64,8PLl9+XkIO3l5Pfl5A==",
        Some((~"text", ~"plain")), Some(~"koi8-r"),
        Some(~[0xF0, 0xF2, 0xE5, 0xF7, 0xE5, 0xE4, 0x20, 0xED, 0xE5, 0xE4, 0xF7, 0xE5, 0xE4]));
}
