/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::LocationBinding;
use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::window::Window;
use dom::urlutils::URLUtils;
use servo_util::str::DOMString;

use script_task::{Page};
use std::rc::Rc;

use serialize::{Encoder, Encodable};


#[deriving(Encodable)]
pub struct Location {
    pub reflector_: Reflector, //XXXjdm cycle: window->Location->window
    pub page: Rc<Page>,
}

impl Location {
    pub fn new_inherited(page: Rc<Page>) -> Location {
        Location {
            reflector_: Reflector::new(),
            page: page
        }
    }

    pub fn new(window: &JS<Window>, page: Rc<Page>) -> JS<Location> {
        reflect_dom_object(~Location::new_inherited(page),
                           window,
                           LocationBinding::Wrap)
    }

    pub fn Assign(&self, _url: DOMString) {

    }

    pub fn Replace(&self, _url: DOMString) {

    }

    pub fn Reload(&self) {

    }
}

impl URLUtils for Location {
    fn Href(&self) -> DOMString {
        self.page.get_url().to_str()
    }
}

impl Reflectable for Location {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}
