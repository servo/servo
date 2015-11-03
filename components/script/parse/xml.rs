/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unrooted_must_root)]

use dom::document::Document;
use url::Url;

pub enum ParseContext {
    Owner(Option<i32>)
}


pub fn parse_xml(document: &Document,
                  input: String,
                  url: Url,
                  context: ParseContext) {
}
