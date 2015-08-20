/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate hyper;

use ipc_channel::ipc;
use net_traits::LoadConsumer::Channel;
use net_traits::LoadData;
use net_traits::ProgressMsg::{Payload, Done};
use self::hyper::header::ContentType;
use self::hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};

#[cfg(test)]
fn assert_parse(url:          &'static str,
                content_type: Option<ContentType>,
                charset:      Option<String>,
                data:         Option<Vec<u8>>) {
    use net::data_loader::load;
    use std::sync::mpsc::channel;
    use url::Url;

    let (start_chan, start_port) = ipc::channel().unwrap();
    load(LoadData::new(Url::parse(url).unwrap(), None), Channel(start_chan));

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
    assert_parse(
        "data:text/plain,hello",
        Some(ContentType(Mime(TopLevel::Text, SubLevel::Plain, vec!()))),
        None,
        Some(b"hello".iter().map(|&x| x).collect()));
}

#[test]
fn plain_charset() {
    assert_parse("data:text/plain;charset=latin1,hello",
        Some(ContentType(Mime(TopLevel::Text,
                              SubLevel::Plain,
                              vec!((Attr::Charset, Value::Ext("latin1".to_string())))))),
        Some("latin1".to_string()), Some(b"hello".iter().map(|&x| x).collect()));
}

#[test]
fn base64() {
    assert_parse("data:;base64,C62+7w==", None, None, Some(vec!(0x0B, 0xAD, 0xBE, 0xEF)));
}

#[test]
fn base64_ct() {
    assert_parse("data:application/octet-stream;base64,C62+7w==",
        Some(ContentType(Mime(TopLevel::Application, SubLevel::Ext("octet-stream".to_string()), vec!()))),
        None,
        Some(vec!(0x0B, 0xAD, 0xBE, 0xEF)));
}

#[test]
fn base64_charset() {
    assert_parse("data:text/plain;charset=koi8-r;base64,8PLl9+XkIO3l5Pfl5A==",
        Some(ContentType(Mime(TopLevel::Text, SubLevel::Plain,
                              vec!((Attr::Charset, Value::Ext("koi8-r".to_string())))))),
        Some("koi8-r".to_string()),
        Some(vec!(0xF0, 0xF2, 0xE5, 0xF7, 0xE5, 0xE4, 0x20, 0xED, 0xE5, 0xE4, 0xF7, 0xE5, 0xE4)));
}
