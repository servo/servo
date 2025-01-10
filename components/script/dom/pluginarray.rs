/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::PluginArrayBinding::PluginArrayMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::plugin::Plugin;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct PluginArray {
    reflector_: Reflector,
}

impl PluginArray {
    pub(crate) fn new_inherited() -> PluginArray {
        PluginArray {
            reflector_: Reflector::new(),
        }
    }

    pub(crate) fn new(global: &GlobalScope) -> DomRoot<PluginArray> {
        reflect_dom_object(
            Box::new(PluginArray::new_inherited()),
            global,
            CanGc::note(),
        )
    }
}

impl PluginArrayMethods<crate::DomTypeHolder> for PluginArray {
    // https://html.spec.whatwg.org/multipage/#dom-pluginarray-refresh
    fn Refresh(&self, _reload: bool) {}

    // https://html.spec.whatwg.org/multipage/#dom-pluginarray-length
    fn Length(&self) -> u32 {
        0
    }

    // https://html.spec.whatwg.org/multipage/#dom-pluginarray-item
    fn Item(&self, _index: u32) -> Option<DomRoot<Plugin>> {
        None
    }

    // https://html.spec.whatwg.org/multipage/#dom-pluginarray-nameditem
    fn NamedItem(&self, _name: DOMString) -> Option<DomRoot<Plugin>> {
        None
    }

    // https://html.spec.whatwg.org/multipage/#dom-pluginarray-item
    fn IndexedGetter(&self, _index: u32) -> Option<DomRoot<Plugin>> {
        None
    }

    // check-tidy: no specs after this line
    fn NamedGetter(&self, _name: DOMString) -> Option<DomRoot<Plugin>> {
        None
    }

    // https://heycam.github.io/webidl/#dfn-supported-property-names
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        vec![]
    }
}
