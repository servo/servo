/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use html5ever::LocalName;
use style::attr::AttrValue;

use crate::dom::element::Element;

impl Element {
    pub(crate) fn get_int_attribute(&self, local_name: &LocalName, default: i32) -> i32 {
        match self.get_attribute(local_name) {
            Some(ref attribute) => match *attribute.value() {
                AttrValue::Int(_, value) => value,
                _ => unreachable!("Expected an AttrValue::Int: implement parse_plain_attribute"),
            },
            None => default,
        }
    }
}
