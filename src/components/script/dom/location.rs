/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::LocationBinding;
use dom::bindings::global::Window;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::window::Window;
use page::Page;

use servo_util::str::DOMString;

use serialize::{Encoder, Encodable};
use std::rc::Rc;
use url::query_to_str;

#[deriving(Encodable)]
pub struct Location {
    reflector_: Reflector, //XXXjdm cycle: window->Location->window
    page: Rc<Page>,
}

impl Location {
    pub fn new_inherited(page: Rc<Page>) -> Location {
        Location {
            reflector_: Reflector::new(),
            page: page
        }
    }

    pub fn new(window: &JSRef<Window>, page: Rc<Page>) -> Temporary<Location> {
        reflect_dom_object(box Location::new_inherited(page),
                           &Window(*window),
                           LocationBinding::Wrap)
    }
}

pub trait LocationMethods {
    fn Href(&self) -> DOMString;
    fn Search(&self) -> DOMString;
}

impl<'a> LocationMethods for JSRef<'a, Location> {
    fn Href(&self) -> DOMString {
        self.page.get_url().to_str()
    }

    fn Search(&self) -> DOMString {
        let query = query_to_str(&self.page.get_url().query);
        if query.as_slice() == "" {
            query
        } else {
            "?".to_string().append(query.as_slice())
        }
    }
}

impl Reflectable for Location {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
