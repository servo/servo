/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::hashmap::HashMap;
use std::os;
use rust_url;
use rust_url::{Url, UrlParser};

/**
Create a URL object from a string. Does various helpful browsery things like

* If there's no current url and the path looks like a file then it will
  create a file url based of the current working directory
* If there's a current url and the new path is relative then the new url
  is based off the current url

*/
// TODO: about:failure->
pub fn try_parse_url(str_url: &str, base_url: Option<Url>) -> Result<Url, &'static str> {
    let mut parser = UrlParser::new();
    let parser = match base_url {
        Some(ref base) => &*parser.base_url(base),
        None => &parser,
    };
    let str_url = match parser.parse(str_url) {
        Err(err) => {
            if base_url.is_none() {
                // Assume we've been given a file path. If it's absolute just return
                // it, otherwise make it absolute with the cwd.
                if str_url.as_slice().starts_with("/") {
                    format!("file://{}", str_url)
                } else {
                    let mut path = os::getcwd();
                    path.push(str_url);
                    // FIXME (#1094): not the right way to transform a path
                    format!("file://{}", path.display().to_str())
                }
            } else {
                return Err(err)
            }
        },
        Ok(url) => return Ok(url)
    };
    parser.parse(str_url.as_slice())
}

pub fn parse_url(str_url: &str, base_url: Option<rust_url::Url>) -> rust_url::Url {
    // FIXME: Need to handle errors
    try_parse_url(str_url, base_url).ok().expect("URL parsing failed")
}


pub type UrlMap<T> = HashMap<rust_url::Url, T>;

pub fn url_map<T: Clone + 'static>() -> UrlMap<T> {
    HashMap::new()
}


pub fn is_image_data(uri: &str) -> bool {
    static types: &'static [&'static str] = &["data:image/png", "data:image/gif", "data:image/jpeg"];
    types.iter().any(|&type_| uri.starts_with(type_))
}


