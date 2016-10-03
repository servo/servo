/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::serialize_identifier;
use dom::bindings::error::Fallible;
use dom::bindings::reflector::Reflector;
use dom::bindings::str::DOMString;
use dom::globalscope::GlobalScope;

#[dom_struct]
pub struct CSS {
    reflector_: Reflector,
}

impl CSS {
    // http://dev.w3.org/csswg/cssom/#serialize-an-identifier
    pub fn Escape(_: &GlobalScope, ident: DOMString) -> Fallible<DOMString> {
        let mut escaped = String::new();
        serialize_identifier(&ident, &mut escaped).unwrap();
        Ok(DOMString::from(escaped))
    }
}
