/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::MimeTypeBinding::MimeTypeMethods;
use dom::bindings::js::Root;
use dom::bindings::reflector::Reflector;
use dom::bindings::str::DOMString;
use dom::plugin::Plugin;
use dom_struct::dom_struct;

#[dom_struct]
pub struct MimeType {
    reflector_: Reflector,
}

impl MimeTypeMethods for MimeType {
    // https://html.spec.whatwg.org/multipage/#dom-mimetype-type
    fn Type(&self) -> DOMString {
        unreachable!()
    }

    // https://html.spec.whatwg.org/multipage/#dom-mimetype-description
    fn Description(&self) -> DOMString {
        unreachable!()
    }

    // https://html.spec.whatwg.org/multipage/#dom-mimetype-suffixes
    fn Suffixes(&self) -> DOMString {
        unreachable!()
    }

    // https://html.spec.whatwg.org/multipage/#dom-mimetype-enabledplugin
    fn EnabledPlugin(&self) -> Root<Plugin> {
        unreachable!()
    }
}
