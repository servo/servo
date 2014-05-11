/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::ConsoleBinding;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::window::Window;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct Console {
    pub reflector_: Reflector
}

impl Console {
    pub fn new_inherited() -> Console {
        Console {
            reflector_: Reflector::new()
        }
    }

    pub fn new(window: &JSRef<Window>) -> Temporary<Console> {
        reflect_dom_object(box Console::new_inherited(), window, ConsoleBinding::Wrap)
    }
}

pub trait ConsoleMethods {
    fn Log(&self, message: DOMString);
    fn Debug(&self, message: DOMString);
    fn Info(&self, message: DOMString);
    fn Warn(&self, message: DOMString);
    fn Error(&self, message: DOMString);
}

impl<'a> ConsoleMethods for JSRef<'a, Console> {
    fn Log(&self, message: DOMString) {
        println!("{:s}", message);
    }

    fn Debug(&self, message: DOMString) {
        println!("{:s}", message);
    }

    fn Info(&self, message: DOMString) {
        println!("{:s}", message);
    }

    fn Warn(&self, message: DOMString) {
        println!("{:s}", message);
    }

    fn Error(&self, message: DOMString) {
        println!("{:s}", message);
    }
}

impl Reflectable for Console {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}
