/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo_util::str::DOMString;
use url::Url;

pub struct UrlHelper;

impl UrlHelper {
    pub fn Href(url: &Url) -> DOMString {
        url.serialize()
    }

    pub fn Search(url: &Url) -> DOMString {
        match url.query {
            None => "".to_string(),
            Some(ref query) if query.as_slice() == "" => "".to_string(),
            Some(ref query) => format!("?{}", query)
        }
    }

    pub fn Hash(url: &Url) -> DOMString {
        match url.fragment {
            None => "".to_string(),
            Some(ref hash) if hash.as_slice() == "" => "".to_string(),
            Some(ref hash) => format!("#{}", hash)
        }
    }
}
