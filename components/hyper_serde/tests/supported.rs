// Copyright 2023 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use cookie::Cookie;
use headers::ContentType;
use http::header::HeaderMap;
use hyper::{Method, StatusCode, Uri};
use hyper_serde::{De, Ser, Serde};
use mime::Mime;
use serde::{Deserialize, Serialize};
use time::Tm;

fn is_supported<T>()
where
    for<'de> De<T>: Deserialize<'de>,
    for<'a> Ser<'a, T>: Serialize,
    for<'de> Serde<T>: Deserialize<'de> + Serialize,
{
}

#[test]
fn supported() {
    is_supported::<Cookie>();
    is_supported::<ContentType>();
    is_supported::<HeaderMap>();
    is_supported::<Method>();
    is_supported::<Mime>();
    is_supported::<StatusCode>();
    is_supported::<Tm>();
    is_supported::<Uri>();
}
