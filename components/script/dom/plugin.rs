/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PluginBinding::PluginMethods;
use dom::bindings::reflector::Reflector;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::mimetype::MimeType;
use dom_struct::dom_struct;

#[dom_struct]
pub struct Plugin {
    reflector_: Reflector,
}

impl PluginMethods for Plugin {
    // https://html.spec.whatwg.org/multipage/#dom-plugin-name
    fn Name(&self) -> DOMString {
        unreachable!()
    }

    // https://html.spec.whatwg.org/multipage/#dom-plugin-description
    fn Description(&self) -> DOMString {
        unreachable!()
    }

    // https://html.spec.whatwg.org/multipage/#dom-plugin-filename
    fn Filename(&self) -> DOMString {
        unreachable!()
    }

    // https://html.spec.whatwg.org/multipage/#dom-plugin-length
    fn Length(&self) -> u32 {
        unreachable!()
    }

    // https://html.spec.whatwg.org/multipage/#dom-plugin-item
    fn Item(&self, _index: u32) -> Option<DomRoot<MimeType>> {
        unreachable!()
    }

    // https://html.spec.whatwg.org/multipage/#dom-plugin-nameditem
    fn NamedItem(&self, _name: DOMString) -> Option<DomRoot<MimeType>> {
        unreachable!()
    }

    // https://html.spec.whatwg.org/multipage/#dom-plugin-item
    fn IndexedGetter(&self, _index: u32) -> Option<DomRoot<MimeType>> {
        unreachable!()
    }

    // check-tidy: no specs after this line
    fn NamedGetter(&self, _name: DOMString) -> Option<DomRoot<MimeType>> {
        unreachable!()
    }

    // https://heycam.github.io/webidl/#dfn-supported-property-names
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        unreachable!()
    }
}
