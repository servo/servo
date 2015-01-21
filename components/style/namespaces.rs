/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use std::collections::HashMap;
use string_cache::{Atom, Namespace};
use parser::ParserContext;


#[deriving(Clone)]
pub struct NamespaceMap {
    pub default: Option<Namespace>,
    pub prefix_map: HashMap<String, Namespace>,
}


impl NamespaceMap {
    pub fn new() -> NamespaceMap {
        NamespaceMap { default: None, prefix_map: HashMap::new() }
    }
}


pub fn parse_namespace_rule(context: &mut ParserContext, input: &mut Parser)
                            -> Result<(Option<String>, Namespace), ()> {
    let prefix = input.try(|input| input.expect_ident()).ok().map(|p| p.into_owned());
    let url = try!(input.expect_url_or_string());
    try!(input.expect_exhausted());

    let namespace = Namespace(Atom::from_slice(url.as_slice()));
    let is_duplicate = match prefix {
        Some(ref prefix) => {
            context.namespaces.prefix_map.insert(prefix.clone(), namespace.clone()).is_some()
        }
        None => {
            let has_default = context.namespaces.default.is_some();
            if !has_default {
                context.namespaces.default = Some(namespace.clone());
            }
            has_default
        }
    };
    if is_duplicate {
        Err(())  // "Duplicate @namespace rule"
    } else {
        Ok((prefix, namespace))
    }
}
