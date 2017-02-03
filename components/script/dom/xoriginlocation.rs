/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::XOriginLocationBinding;
use dom::bindings::codegen::Bindings::XOriginLocationBinding::XOriginLocationMethods;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::DomObject;
use dom::bindings::reflector::Reflector;
use dom::bindings::str::DOMString;
use dom::bindings::str::USVString;
use dom::xoriginwindow::XOriginWindow;

#[dom_struct]
pub struct XOriginLocation {
    reflector: Reflector,
    window: JS<XOriginWindow>,
}

impl XOriginLocation {
    #[allow(unsafe_code)]
    pub fn new(window: &XOriginWindow) -> Root<XOriginLocation> {
        let globalscope = window.global();
        let cx = globalscope.get_cx();
        let loc = box XOriginLocation {
            reflector: Reflector::new(),
            window: JS::from_ref(window),
        };
        unsafe {
            XOriginLocationBinding::Wrap(cx, &globalscope, loc)
        }
    }
}

impl XOriginLocationMethods for XOriginLocation {
    // https://html.spec.whatwg.org/multipage/#dom-location-href
    fn GetHref(&self) -> Fallible<USVString> {
        Err(Error::Security)
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-href
    fn SetHref(&self, _: USVString) -> ErrorResult {
        // TODO: setting href on a cross-origin window should succeed?
        Err(Error::Security)
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-assign
    fn Assign(&self, _: USVString) -> Fallible<()> {
        // TODO: setting href on a cross-origin window should succeed?
        Err(Error::Security)
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-replace
    fn Replace(&self, _: USVString) -> Fallible<()> {
        // TODO: replacing href on a cross-origin window should succeed?
        Err(Error::Security)
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-reload
    fn Reload(&self) -> Fallible<()> {
        Err(Error::Security)
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-href
    fn Stringifier(&self) -> Fallible<DOMString> {
        Err(Error::Security)
    }
}
