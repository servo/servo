/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::env;
use std::path::Path;

use log::warn;
use servo::net_traits::pub_domains::is_reg_domain;
use servo::servo_config::pref;
use servo::servo_url::ServoUrl;
use url::{self, Url};

pub fn parse_url_or_filename(cwd: &Path, input: &str) -> Result<ServoUrl, ()> {
    match ServoUrl::parse(input) {
        Ok(url) => Ok(url),
        Err(url::ParseError::RelativeUrlWithoutBase) => {
            Url::from_file_path(&*cwd.join(input)).map(ServoUrl::from_url)
        },
        Err(_) => Err(()),
    }
}

pub fn get_default_url(url_opt: Option<String>) -> ServoUrl {
    // If the url is not provided, we fallback to the homepage in prefs,
    // or a blank page in case the homepage is not set either.
    let cwd = env::current_dir().unwrap();

    let cmdline_url = url_opt.map(|s| s.to_string()).and_then(|url_string| {
        parse_url_or_filename(&cwd, &url_string)
            .map_err(|error| {
                warn!("URL parsing failed ({:?}).", error);
                error
            })
            .ok()
    });

    let pref_url = {
        let homepage_url = pref!(shell.homepage);
        parse_url_or_filename(&cwd, &homepage_url).ok()
    };
    let blank_url = ServoUrl::parse("about:blank").ok();

    cmdline_url.or(pref_url).or(blank_url).unwrap()
}

pub fn sanitize_url(request: &str) -> Option<ServoUrl> {
    let request = request.trim();
    ServoUrl::parse(request)
        .ok()
        .or_else(|| {
            if request.contains('/') || is_reg_domain(request) {
                ServoUrl::parse(&format!("https://{}", request)).ok()
            } else {
                None
            }
        })
        .or_else(|| {
            let url = pref!(shell.searchpage).replace("%s", request);
            ServoUrl::parse(&url).ok()
        })
}
