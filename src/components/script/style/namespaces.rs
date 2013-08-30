/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::hashmap::HashMap;
use cssparser::*;
use style::errors::log_css_error;

pub struct NamespaceMap {
    default: Option<~str>,  // Optional URL
    prefix_map: HashMap<~str, ~str>,  // prefix -> URL
}


impl NamespaceMap {
    pub fn new() -> NamespaceMap {
        NamespaceMap { default: None, prefix_map: HashMap::new() }
    }
}


pub fn parse_namespace_rule(rule: AtRule, namespaces: &mut NamespaceMap) {
    let location = rule.location;
    macro_rules! syntax_error(
        () => {{
            log_css_error(location, "Invalid @namespace rule");
            return
        }};
    );
    if rule.block.is_some() { syntax_error!() }
    let mut prefix: Option<~str> = None;
    let mut url: Option<~str> = None;
    let mut iter = rule.prelude.move_skip_whitespace();
    for component_value in iter {
        match component_value {
            Ident(value) => {
                if prefix.is_some() { syntax_error!() }
                prefix = Some(value);
            },
            URL(value) | String(value) => {
                if url.is_some() { syntax_error!() }
                url = Some(value);
                break
            },
            _ => syntax_error!(),
        }
    }
    if iter.next().is_some() { syntax_error!() }
    match (prefix, url) {
        (Some(prefix), Some(url)) => {
            if namespaces.prefix_map.swap(prefix, url).is_some() {
                log_css_error(location, "Duplicate @namespace rule");
            }
        },
        (None, Some(url)) => {
            if namespaces.default.is_some() {
                log_css_error(location, "Duplicate @namespace rule");
            }
            namespaces.default = Some(url);
        },
        _ => syntax_error!()
    }
}
