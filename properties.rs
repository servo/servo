/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ascii::to_ascii_lower;
use cssparser::*;
use errors::{ErrorLoggerIterator, log_css_error};


pub struct PropertyDeclarationBlock {
    important: ~[PropertyDeclaration],
    normal: ~[PropertyDeclaration],
}

pub struct PropertyDeclaration;  // TODO


pub fn parse_property_declaration_list(input: ~[Node]) -> PropertyDeclarationBlock {
    let mut important = ~[];
    let mut normal = ~[];
    for item in ErrorLoggerIterator(parse_declaration_list(input.move_iter())) {
        match item {
            Decl_AtRule(rule) => log_css_error(
                rule.location, fmt!("Unsupported at-rule in declaration list: @%s", rule.name)),
            Declaration(Declaration{ location: l, name: n, value: v, important: i}) => {
                let list = if i { &mut important } else { &mut normal };
                if !parse_one_property_declaration(to_ascii_lower(n), v, list) {
                    log_css_error(l, "Invalid property declaration")
                }
            }
        }
    }
    PropertyDeclarationBlock { important: important, normal: normal }
}


fn parse_one_property_declaration(name: &str, value: ~[ComponentValue],
                                  result_list: &mut ~[PropertyDeclaration]) -> bool {
    false
}
