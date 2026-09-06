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
    DOMString::from("");
    //~^ ERROR: 27:5: 27:24: use DOMString::new() instead [crown::manual_domstring_new]
    let dom_string: DOMString = "".into();
    //~^ ERROR: 29:33: 29:42: use DOMString::new() instead [crown::manual_domstring_new]
    func("".into());
    //~^ ERROR: 31:10: 31:19: use DOMString::new() instead [crown::manual_domstring_new]
    let t = "";
    DOMString::from(t);
}
