/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//@rustc-env:RUSTC_BOOTSTRAP=1

#![expect(dead_code)]
#![deny(crown::manual_domstring_new)]

struct DOMString {
}

impl DOMString {
    fn new() -> Self {
        Self {}
    }
}

impl From<&str> for DOMString {
    fn from(string: &str) -> Self {
        Self {}
    }
}

fn func(str_: DOMString) {}

fn main() {
    DOMString::from("Static string");
    //~^ ERROR: 27:5: 27:37: use DOMString::from_static("Static string") instead [crown::manual_domstring_new]
    let dom_string: DOMString = "Static string".into();
    //~^ ERROR: 29:33: 29:55: use DOMString::from_static("Static string") instead [crown::manual_domstring_new]
    func("Static string".into());
    //~^ ERROR: 31:10: 31:32: use DOMString::from_static("Static string") instead [crown::manual_domstring_new]
}
