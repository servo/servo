/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::LocationBinding;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::error::Fallible;
use dom::window::Window;
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

    pub fn new(window: &JSRef<Window>, page: Rc<Page>) -> Temporary<Location> {
        reflect_dom_object(~Location::new_inherited(page),
                           window,
                           LocationBinding::Wrap)
    }
}

pub trait LocationMethods {
    fn Assign(&self, _url: DOMString);
    fn Replace(&self, _url: DOMString);
    fn Reload(&self);
    fn Href(&self) -> DOMString;
    fn SetHref(&self, _href: DOMString) -> Fallible<()>;
    fn Origin(&self) -> DOMString;
    fn Protocol(&self) -> DOMString;
    fn SetProtocol(&self, _protocol: DOMString);
    fn Username(&self) -> DOMString;
    fn SetUsername(&self, _username: DOMString);
    fn Password(&self) -> DOMString;
    fn SetPassword(&self, _password: DOMString);
    fn Host(&self) -> DOMString;
    fn SetHost(&self, _host: DOMString);
    fn Hostname(&self) -> DOMString;
    fn SetHostname(&self, _hostname: DOMString);
    fn Port(&self) -> DOMString;
    fn SetPort(&self, _port: DOMString);
    fn Pathname(&self) -> DOMString;
    fn SetPathname(&self, _pathname: DOMString);
    fn Search(&self) -> DOMString;
    fn SetSearch(&self, _search: DOMString);
    fn Hash(&self) -> DOMString;
    fn SetHash(&self, _hash: DOMString);
}

impl<'a> LocationMethods for JSRef<'a, Location> {
    fn Assign(&self, _url: DOMString) {

    }

    fn Replace(&self, _url: DOMString) {

    }

    fn Reload(&self) {

    }

    fn Href(&self) -> DOMString {
        self.page.get_url().to_str()
    }

    fn SetHref(&self, _href: DOMString) -> Fallible<()> {
        Ok(())
    }

    fn Origin(&self) -> DOMString {
        "".to_owned()
    }

    fn Protocol(&self) -> DOMString {
        "".to_owned()
    }

    fn SetProtocol(&self, _protocol: DOMString) {

    }

    fn Username(&self) -> DOMString {
        "".to_owned()
    }

    fn SetUsername(&self, _username: DOMString) {

    }

    fn Password(&self) -> DOMString {
        "".to_owned()
    }

    fn SetPassword(&self, _password: DOMString) {

    }

    fn Host(&self) -> DOMString {
        "".to_owned()
    }

    fn SetHost(&self, _host: DOMString) {

    }

    fn Hostname(&self) -> DOMString {
        "".to_owned()
    }

    fn SetHostname(&self, _hostname: DOMString) {

    }

    fn Port(&self) -> DOMString {
        "".to_owned()
    }

    fn SetPort(&self, _port: DOMString) {

    }

    fn Pathname(&self) -> DOMString {
        "".to_owned()
    }

    fn SetPathname(&self, _pathname: DOMString) {

    }

    fn Search(&self) -> DOMString {
        "".to_owned()
    }

    fn SetSearch(&self, _search: DOMString) {

    }

    fn Hash(&self) -> DOMString {
        "".to_owned()
    }

    fn SetHash(&self, _hash: DOMString) {

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
