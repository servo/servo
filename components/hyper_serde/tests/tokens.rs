// Copyright 2023 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use cookie::{Cookie, CookieBuilder};
use headers::ContentType;
use http::header::{self, HeaderMap, HeaderValue};
use http::StatusCode;
use hyper::{Method, Uri};
use hyper_serde::{De, Ser};
use serde_test::{assert_de_tokens, assert_ser_tokens, Token};
use time_03::Duration;

#[test]
fn test_content_type() {
    let content_type = ContentType::from("Application/Json".parse::<mime::Mime>().unwrap());
    let tokens = &[Token::Str("application/json")];

    assert_ser_tokens(&Ser::new(&content_type), tokens);
    assert_de_tokens(&De::new(content_type), tokens);
}

#[test]
fn test_cookie() {
    // Unfortunately we have to do the to_string().parse() dance here to avoid the object being a
    // string with a bunch of indices in it which apparently is different from the exact same
    // cookie but parsed as a bunch of strings.
    let cookie: Cookie = CookieBuilder::new("Hello", "World!")
        .max_age(Duration::seconds(42))
        .domain("servo.org")
        .path("/")
        .secure(true)
        .http_only(false)
        .build()
        .to_string()
        .parse()
        .unwrap();

    let tokens = &[Token::Str(
        "Hello=World!; Secure; Path=/; Domain=servo.org; Max-Age=42",
    )];

    assert_ser_tokens(&Ser::new(&cookie), tokens);
    assert_de_tokens(&De::new(cookie), tokens);
}

#[test]
fn test_headers_empty() {
    let headers = HeaderMap::new();

    let tokens = &[Token::Map { len: Some(0) }, Token::MapEnd];

    assert_ser_tokens(&Ser::new(&headers), tokens);
    assert_de_tokens(&De::new(headers), tokens);
}

#[test]
fn test_headers_not_empty() {
    let mut headers = HeaderMap::new();
    headers.insert(header::HOST, HeaderValue::from_static("baguette"));

    // In http, HeaderMap is internally a HashMap and thus testing this
    // with multiple headers is non-deterministic.

    let tokens = &[
        Token::Map { len: Some(1) },
        Token::Str("host"),
        Token::Seq { len: Some(1) },
        Token::Bytes(b"baguette"),
        Token::SeqEnd,
        Token::MapEnd,
    ];

    assert_ser_tokens(&Ser::new(&headers), tokens);
    assert_de_tokens(&De::new(headers.clone()), tokens);

    let pretty = &[
        Token::Map { len: Some(1) },
        Token::Str("host"),
        Token::Seq { len: Some(1) },
        Token::Str("baguette"),
        Token::SeqEnd,
        Token::MapEnd,
    ];

    assert_ser_tokens(&Ser::new_pretty(&headers), pretty);
    assert_de_tokens(&De::new(headers), pretty);
}

#[test]
fn test_method() {
    let method = Method::PUT;
    let tokens = &[Token::Str("PUT")];

    assert_ser_tokens(&Ser::new(&method), tokens);
    assert_de_tokens(&De::new(method), tokens);
}

#[test]
fn test_raw_status() {
    let raw_status = StatusCode::from_u16(200).unwrap();
    let tokens = &[Token::U16(200)];

    assert_ser_tokens(&Ser::new(&raw_status), tokens);
    assert_de_tokens(&De::new(raw_status), tokens);
}

#[test]
fn test_tm() {
    use time::strptime;

    let time = strptime("2017-02-22T12:03:31Z", "%Y-%m-%dT%H:%M:%SZ").unwrap();
    let tokens = &[Token::Str("2017-02-22T12:03:31Z")];

    assert_ser_tokens(&Ser::new(&time), tokens);
    assert_de_tokens(&De::new(time), tokens);
}

#[test]
fn test_uri() {
    use std::str::FromStr;

    // Note that fragment is not serialized / deserialized
    let uri_string = "abc://username:password@example.com:123/path/data?key=value&key2=value2";
    let uri = Uri::from_str(uri_string).unwrap();
    let tokens = &[Token::Str(uri_string)];

    assert_ser_tokens(&Ser::new(&uri), tokens);
    assert_de_tokens(&De::new(uri), tokens);
}
