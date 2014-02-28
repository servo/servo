/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::LocationBinding;
use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::error::Fallible;
use dom::window::Window;
use servo_util::str::DOMString;

use script_task::{Page};
use std::rc::Rc;

use extra::serialize::{Encoder, Encodable};


#[deriving(Encodable)]
pub struct Location {
    reflector_: Reflector, //XXXjdm cycle: window->Location->window
    extra: Untraceable,
}

struct Untraceable {
    page: Rc<Page>,
}

impl<S: Encoder> Encodable<S> for Untraceable {
    fn encode(&self, s: &mut S) {
        self.page.borrow().encode(s);
    }
}

impl Location {
    pub fn new_inherited(page: Rc<Page>) -> Location {
        Location {
            reflector_: Reflector::new(),
            extra: Untraceable {
                page: page
            }
        }
    }

    pub fn new(window: &Window, page: Rc<Page>) -> JS<Location> {
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

    pub fn Href(&self) -> DOMString {
        self.extra.page.borrow().get_url().to_str()
    }

    pub fn SetHref(&self, _href: DOMString) -> Fallible<()> {
        Ok(())
    }

    pub fn Origin(&self) -> DOMString {
        ~""
    }

    pub fn Protocol(&self) -> DOMString {
        ~""
    }

    pub fn SetProtocol(&self, _protocol: DOMString) {

    }

    pub fn Username(&self) -> DOMString {
        ~""
    }

    pub fn SetUsername(&self, _username: DOMString) {

    }

    pub fn Password(&self) -> DOMString {
        ~""
    }

    pub fn SetPassword(&self, _password: DOMString) {

    }

    pub fn Host(&self) -> DOMString {
        ~""
    }

    pub fn SetHost(&self, _host: DOMString) {

    }

    pub fn Hostname(&self) -> DOMString {
        ~""
    }

    pub fn SetHostname(&self, _hostname: DOMString) {

    }

    pub fn Port(&self) -> DOMString {
        ~""
    }

    pub fn SetPort(&self, _port: DOMString) {

    }

    pub fn Pathname(&self) -> DOMString {
        ~""
    }

    pub fn SetPathname(&self, _pathname: DOMString) {

    }

    pub fn Search(&self) -> DOMString {
        ~""
    }

    pub fn SetSearch(&self, _search: DOMString) {

    }

    pub fn Hash(&self) -> DOMString {
        ~""
    }

    pub fn SetHash(&self, _hash: DOMString) {

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
