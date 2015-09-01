/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::error::{Error, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::utils::Reflector;
use util::str::DOMString;

use cssparser::serialize_identifier;

#[dom_struct]
pub struct CSS {
    reflector_: Reflector,
}

impl CSS {
    // http://dev.w3.org/csswg/cssom/#serialize-an-identifier
    pub fn Escape(_: GlobalRef, ident: DOMString) -> Fallible<DOMString> {
        if ident.bytes().any(|b| b == b'\0') {
            return Err(Error::InvalidCharacter);
        }
        let mut escaped = DOMString::new();
        serialize_identifier(&ident, &mut escaped).unwrap();
        Ok(escaped)
    }
}
