/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use content_blocker_parser::{RuleList, parse_list};
use std::str;
use std::sync::Arc;
use util::resource_files::read_resource_file;

lazy_static! {
    pub static ref BLOCKED_CONTENT_RULES: Arc<Option<RuleList>> = Arc::new(create_rule_list());
}

fn create_rule_list() -> Option<RuleList> {
    let contents = match read_resource_file("blocked-content.json") {
        Ok(c) => c,
        Err(_) => return None,
    };

    let str_contents = match str::from_utf8(&contents) {
        Ok(c) => c,
        Err(_) => return None,
    };

    let list = match parse_list(&str_contents) {
        Ok(l) => l,
        Err(_) => return None,
    };

    Some(list)
}
