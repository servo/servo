/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::LocationBinding;
use dom::bindings::codegen::Bindings::LocationBinding::LocationMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::str::USVString;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::urlhelper::UrlHelper;
use dom::window::Window;
use dom::window::WindowHelpers;

use util::str::DOMString;
use url::Url;

#[dom_struct]
pub struct Location {
    reflector_: Reflector,
    window: JS<Window>,
}

impl Location {
    fn new_inherited(window: JSRef<Window>) -> Location {
        Location {
            reflector_: Reflector::new(),
            window: JS::from_rooted(window)
        }
    }

    pub fn new(window: JSRef<Window>) -> Temporary<Location> {
        reflect_dom_object(box Location::new_inherited(window),
                           GlobalRef::Window(window),
                           LocationBinding::Wrap)
    }
}

impl<'a> LocationMethods for JSRef<'a, Location> {
    // https://html.spec.whatwg.org/multipage/#dom-location-assign
    fn Assign(self, url: DOMString) {
        self.window.root().r().load_url(url);
    }

    fn Href(self) -> USVString {
        UrlHelper::Href(&self.get_url())
    }

    fn Pathname(self) -> USVString {
        UrlHelper::Pathname(&self.get_url())
    }

    fn Stringify(self) -> DOMString {
        self.Href().0
    }

    fn Search(self) -> USVString {
        UrlHelper::Search(&self.get_url())
    }

    fn Hash(self) -> USVString {
        UrlHelper::Hash(&self.get_url())
    }
}

trait PrivateLocationHelpers {
    fn get_url(self) -> Url;
}

impl<'a> PrivateLocationHelpers for JSRef<'a, Location> {
    fn get_url(self) -> Url {
        let window = self.window.root();
        window.r().get_url()
    }
}
