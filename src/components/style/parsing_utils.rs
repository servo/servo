/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use std::ascii::StrAsciiExt;
use cssparser::ast::{ComponentValue, Ident, SkipWhitespaceIterable};


pub fn one_component_value<'a>(input: &'a [ComponentValue]) -> Result<&'a ComponentValue, ()> {
    let mut iter = input.skip_whitespace();
    match iter.next() {
        Some(value) => if iter.next().is_none() { Ok(value) } else { Err(()) },
        None => Err(())
    }
}


pub fn get_ident_lower(component_value: &ComponentValue) -> Result<String, ()> {
    match component_value {
        &Ident(ref value) => Ok(value.as_slice().to_ascii_lower()),
        _ => Err(()),
    }
}
