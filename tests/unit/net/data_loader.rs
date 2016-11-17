/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use fetch_sync;
use hyper::header::ContentType;
use hyper::mime::{Attr, Mime, SubLevel, TopLevel, Value};
use hyper_serde::Serde;
use net_traits::{FetchMetadata, FilteredMetadata, NetworkError};
use net_traits::request::{Origin, Request};
use net_traits::response::ResponseBody;
use servo_url::ServoUrl;
use std::ops::Deref;

#[cfg(test)]
fn assert_parse(url:          &'static str,
                content_type: Option<ContentType>,
                charset:      Option<&str>,
                data:         Option<&[u8]>) {
    let url = ServoUrl::parse(url).unwrap();
    let origin = Origin::Origin(url.origin());
    let request = Request::new(url, Some(origin), false, None);

    let response = fetch_sync(request, None);

    match data {
        Some(data) => {
            assert!(!response.is_network_error());
            assert_eq!(response.headers.len(), 1);

            let header_content_type = response.headers.get::<ContentType>();
            assert_eq!(header_content_type, content_type.as_ref());

            let metadata = match response.metadata() {
                Ok(FetchMetadata::Filtered { filtered: FilteredMetadata::Transparent(m), .. }) => m,
                result => panic!(result),
            };
            assert_eq!(metadata.content_type.map(Serde::into_inner), content_type);
            assert_eq!(metadata.charset.as_ref().map(String::deref), charset);

            let resp_body = response.body.lock().unwrap();
            match *resp_body {
                ResponseBody::Done(ref val) => {
                    assert_eq!(val, &data);
                },
                _ => panic!(),
            }
        },
        None => {
            assert!(response.is_network_error());
            assert_eq!(response.metadata().err(), Some(NetworkError::Internal("Decoding data URL failed".to_owned())));
        },
    }
}

#[test]
fn empty_invalid() {
    assert_parse("data:", None, None, None);
}

#[test]
fn plain() {
    assert_parse(
        "data:,hello%20world",
        Some(ContentType(Mime(TopLevel::Text, SubLevel::Plain,
                              vec!((Attr::Charset, Value::Ext("US-ASCII".to_owned())))))),
        Some("US-ASCII"),
        Some(b"hello world"));
}

#[test]
fn plain_ct() {
    assert_parse(
        "data:text/plain,hello",
        Some(ContentType(Mime(TopLevel::Text, SubLevel::Plain, vec!()))),
        None,
        Some(b"hello"));
}

#[test]
fn plain_html() {
    assert_parse(
        "data:text/html,<p>Servo</p>",
        Some(ContentType(Mime(TopLevel::Text, SubLevel::Html, vec!()))),
        None,
        Some(b"<p>Servo</p>"));
}

#[test]
fn plain_charset() {
    assert_parse(
        "data:text/plain;charset=latin1,hello",
        Some(ContentType(Mime(TopLevel::Text,
                              SubLevel::Plain,
                              vec!((Attr::Charset, Value::Ext("latin1".to_owned())))))),
        Some("latin1"),
        Some(b"hello"));
}

#[test]
fn plain_only_charset() {
    assert_parse(
        "data:;charset=utf-8,hello",
        Some(ContentType(Mime(TopLevel::Text,
                              SubLevel::Plain,
                              vec!((Attr::Charset, Value::Utf8))))),
        Some("utf-8"),
        Some(b"hello"));
}

#[test]
fn base64() {
    assert_parse(
        "data:;base64,C62+7w==",
        Some(ContentType(Mime(TopLevel::Text,
                              SubLevel::Plain,
                              vec!((Attr::Charset, Value::Ext("US-ASCII".to_owned())))))),
        Some("US-ASCII"),
        Some(&[0x0B, 0xAD, 0xBE, 0xEF]));
}

#[test]
fn base64_ct() {
    assert_parse(
        "data:application/octet-stream;base64,C62+7w==",
        Some(ContentType(Mime(TopLevel::Application, SubLevel::Ext("octet-stream".to_owned()), vec!()))),
        None,
        Some(&[0x0B, 0xAD, 0xBE, 0xEF]));
}

#[test]
fn base64_charset() {
    assert_parse(
        "data:text/plain;charset=koi8-r;base64,8PLl9+XkIO3l5Pfl5A==",
        Some(ContentType(Mime(TopLevel::Text, SubLevel::Plain,
                              vec!((Attr::Charset, Value::Ext("koi8-r".to_owned())))))),
        Some("koi8-r"),
        Some(&[0xF0, 0xF2, 0xE5, 0xF7, 0xE5, 0xE4, 0x20, 0xED, 0xE5, 0xE4, 0xF7, 0xE5, 0xE4]));
}
