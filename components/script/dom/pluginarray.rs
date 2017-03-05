/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PluginArrayBinding;
use dom::bindings::codegen::Bindings::PluginArrayBinding::PluginArrayMethods;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::globalscope::GlobalScope;
use dom::plugin::Plugin;
use dom_struct::dom_struct;

#[dom_struct]
pub struct PluginArray {
    reflector_: Reflector,
}

impl PluginArray {
    pub fn new_inherited() -> PluginArray {
        PluginArray {
            reflector_: Reflector::new()
        }
    }

    pub fn new(global: &GlobalScope) -> Root<PluginArray> {
        reflect_dom_object(box PluginArray::new_inherited(),
                           global,
                           PluginArrayBinding::Wrap)
    }
}

impl PluginArrayMethods for PluginArray {
    // https://html.spec.whatwg.org/multipage/#dom-pluginarray-refresh
    fn Refresh(&self, _reload: bool) {
    }

    // https://html.spec.whatwg.org/multipage/#dom-pluginarray-length
    fn Length(&self) -> u32 {
        0
    }

    // https://html.spec.whatwg.org/multipage/#dom-pluginarray-item
    fn Item(&self, _index: u32) -> Option<Root<Plugin>> {
        None
    }

    // https://html.spec.whatwg.org/multipage/#dom-pluginarray-nameditem
    fn NamedItem(&self, _name: DOMString) -> Option<Root<Plugin>> {
        None
    }

    // https://html.spec.whatwg.org/multipage/#dom-pluginarray-item
    fn IndexedGetter(&self, _index: u32) -> Option<Root<Plugin>> {
        None
    }

    // check-tidy: no specs after this line
    fn NamedGetter(&self, _name: DOMString) -> Option<Root<Plugin>> {
        None
    }

    // https://heycam.github.io/webidl/#dfn-supported-property-names
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        vec![]
    }
}
