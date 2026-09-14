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
    fn from(_string: &str) -> Self {
        Self {}
    }
}

impl From<std::string::String> for DOMString {
    fn from(_string: String) -> Self {
        Self {}
    }
}

fn func(_str: DOMString) {}

fn main() {
    let _ = DOMString::from("");
    //~^ 33:13: 33:32: use DOMString::new() instead [crown::manual_domstring_new]
    let _ = DOMString::from(String::new());
    //~^ 35:13: 35:43: use DOMString::new() instead [crown::manual_domstring_new]
    let _: DOMString = "".into();
    //~^ ERROR: 37:24: 37:33: use DOMString::new() instead [crown::manual_domstring_new]
    let _ = func("".into());
    //~^ 39:18: 39:27: use DOMString::new() instead [crown::manual_domstring_new]
    let t = "";
    let _ = DOMString::from(t);
}
