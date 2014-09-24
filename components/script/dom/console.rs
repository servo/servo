/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ConsoleBinding;
use dom::bindings::codegen::Bindings::ConsoleBinding::ConsoleMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use servo_util::str::DOMString;

#[jstraceable]
#[must_root]
pub struct Console {
    pub reflector_: Reflector
}

impl Console {
    pub fn new_inherited() -> Console {
        Console {
            reflector_: Reflector::new()
        }
    }

    pub fn new(global: &GlobalRef) -> Temporary<Console> {
        reflect_dom_object(box Console::new_inherited(), global, ConsoleBinding::Wrap)
    }
}

impl<'a> ConsoleMethods for JSRef<'a, Console> {
    fn Log(self, message: DOMString) {
        println!("{:s}", message);
    }

    fn Debug(self, message: DOMString) {
        println!("{:s}", message);
    }

    fn Info(self, message: DOMString) {
        println!("{:s}", message);
    }

    fn Warn(self, message: DOMString) {
        println!("{:s}", message);
    }

    fn Error(self, message: DOMString) {
        println!("{:s}", message);
    }

    fn Assert(self, condition: bool, message: Option<DOMString>) {
        if !condition {
            let message = match message {
                Some(ref message) => message.as_slice(),
                None => "no message",
            };
            println!("Assertion failed: {:s}", message);
        }
    }
}

impl Reflectable for Console {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
