/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::MimeTypeBinding;
use dom::bindings::codegen::Bindings::MimeTypeBinding::MimeTypeMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::plugin::Plugin;
use util::str::DOMString;

#[dom_struct]
pub struct MimeType {
    reflector_: Reflector,
}

impl MimeType {
    pub fn new_inherited() -> MimeType {
        MimeType {
            reflector_: Reflector::new()
        }
    }

    pub fn new(global: GlobalRef) -> Root<MimeType> {
        reflect_dom_object(box MimeType::new_inherited(),
                           global,
                           MimeTypeBinding::Wrap)
    }
}

impl MimeTypeMethods for MimeType {
    // https://html.spec.whatwg.org/multipage/#dom-mimetype-type
    fn Type(&self) -> DOMString {
        DOMString::new()
    }

    // https://html.spec.whatwg.org/multipage/#dom-mimetype-description
    fn Description(&self) -> DOMString {
        DOMString::new()
    }

    // https://html.spec.whatwg.org/multipage/#dom-mimetype-suffixes
    fn Suffixes(&self) -> DOMString {
        DOMString::new()
    }

    // https://html.spec.whatwg.org/multipage/#dom-mimetype-enabledplugin
    fn EnabledPlugin(&self) -> Root<Plugin> {
        Root::from_ref(&Plugin::new_inherited())
    }
}
