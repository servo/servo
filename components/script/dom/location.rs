/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::LocationBinding;
use dom::bindings::codegen::Bindings::LocationBinding::LocationMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::urlhelper::UrlHelper;
use dom::window::Window;
use dom::window::WindowHelpers;
use page::Page;

use util::str::DOMString;

use std::rc::Rc;

#[dom_struct]
pub struct Location {
    reflector_: Reflector,
    page: Rc<Page>,
}

impl Location {
    fn new_inherited(page: Rc<Page>) -> Location {
        Location {
            reflector_: Reflector::new(),
            page: page
        }
    }

    pub fn new(window: JSRef<Window>, page: Rc<Page>) -> Temporary<Location> {
        reflect_dom_object(box Location::new_inherited(page),
                           GlobalRef::Window(window),
                           LocationBinding::Wrap)
    }
}

impl<'a> LocationMethods for JSRef<'a, Location> {
    // https://html.spec.whatwg.org/multipage/browsers.html#dom-location-assign
    fn Assign(self, url: DOMString) {
        self.page.frame().as_ref().unwrap().window.root().r().load_url(url);
    }

    fn Href(self) -> DOMString {
        UrlHelper::Href(&self.page.get_url())
    }

    fn Stringify(self) -> DOMString {
        self.Href()
    }

    fn Search(self) -> DOMString {
        UrlHelper::Search(&self.page.get_url())
    }

    fn Hash(self) -> DOMString {
        UrlHelper::Hash(&self.page.get_url())
    }
}

