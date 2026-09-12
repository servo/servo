/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//@rustc-env:RUSTC_BOOTSTRAP=1

#![expect(dead_code)]

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

impl From<std::string::String> for DOMString {
    fn from(_string: String) -> Self {
        Self {}
    }
}

fn func(str_: DOMString) {}

fn func_with_str(str_: &str) {
    DOMString::from(str_);
    let dom_string: DOMString = str_.into();
    String::from("");
    func(str_.into());
}

fn func_with_string(string_: String) {
    DOMString::from(string_);
}

fn main() {
    func_with_str("");
    func_with_string(String::new())
}
