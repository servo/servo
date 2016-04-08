/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PluginBinding;
use dom::bindings::codegen::Bindings::PluginBinding::PluginMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::mimetype::MimeType;
use util::str::DOMString;

#[dom_struct]
pub struct Plugin {
    reflector_: Reflector,
}

impl Plugin {
    pub fn new_inherited() -> Plugin {
        Plugin {
            reflector_: Reflector::new()
        }
    }

    pub fn new(global: GlobalRef) -> Root<Plugin> {
        reflect_dom_object(box Plugin::new_inherited(),
                           global,
                           PluginBinding::Wrap)
    }
}

impl PluginMethods for Plugin {
    // https://html.spec.whatwg.org/multipage/webappapis.html#dom-plugin-name
    fn Name(&self) -> DOMString {
        DOMString::new()
    }

    // https://html.spec.whatwg.org/multipage/webappapis.html#dom-plugin-description
    fn Description(&self) -> DOMString {
        DOMString::new()
    }

    // https://html.spec.whatwg.org/multipage/webappapis.html#dom-plugin-filename
    fn Filename(&self) -> DOMString {
        DOMString::new()
    }

    // https://html.spec.whatwg.org/multipage/webappapis.html#dom-plugin-length
    fn Length(&self) -> u32 {
        0
    }

    // https://html.spec.whatwg.org/multipage/webappapis.html#dom-plugin-item
    fn Item(&self, _index: u32) -> Option<Root<MimeType>> {
        None
    }

    // https://html.spec.whatwg.org/multipage/webappapis.html#dom-plugin-nameditem
    fn NamedItem(&self, _name: DOMString) -> Option<Root<MimeType>> {
        None
    }

    // https://html.spec.whatwg.org/multipage/webappapis.html#dom-plugin-item
    fn IndexedGetter(&self, _index: u32, _found: &mut bool) -> Option<Root<MimeType>> {
        None
    }

    // check-tidy: no specs after this line
    fn NamedGetter(&self, _name: DOMString, _found: &mut bool) -> Option<Root<MimeType>> {
        None
    }

    // https://heycam.github.io/webidl/#dfn-supported-property-names
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        vec![]
    }

}