/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::ast::*;
use std::collections::hashmap::HashMap;
use servo_util::namespace;
use errors::log_css_error;
use string_cache::Namespace;

pub struct NamespaceMap {
    pub default: Option<Namespace>,
    pub prefix_map: HashMap<String, Namespace>,
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
    let mut prefix: Option<String> = None;
    let mut ns: Option<Namespace> = None;
    let mut iter = rule.prelude.move_skip_whitespace();
    for component_value in iter {
        match component_value {
            Ident(value) => {
                if prefix.is_some() { syntax_error!() }
                prefix = Some(value);
            },
            URL(value) | QuotedString(value) => {
                if ns.is_some() { syntax_error!() }
                ns = Some(namespace::from_domstring(Some(value)));
                break
            },
            _ => syntax_error!(),
        }
    }
    if iter.next().is_some() { syntax_error!() }
    match (prefix, ns) {
        (Some(prefix), Some(ns)) => {
            if namespaces.prefix_map.swap(prefix, ns).is_some() {
                log_css_error(location, "Duplicate @namespace rule");
            }
        },
        (None, Some(ns)) => {
            if namespaces.default.is_some() {
                log_css_error(location, "Duplicate @namespace rule");
            }
            namespaces.default = Some(ns);
        },
        _ => syntax_error!()
    }
}
