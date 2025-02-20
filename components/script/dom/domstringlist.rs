/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::DOMStringListBinding::DOMStringListMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct DOMStringList {
    reflector_: Reflector,
    strings: Vec<DOMString>,
}

impl DOMStringList {
    #[allow(unused)]
    pub(crate) fn new_inherited(strings: Vec<DOMString>) -> DOMStringList {
        DOMStringList {
            reflector_: Reflector::new(),
            strings,
        }
    }

    #[allow(unused)]
    pub(crate) fn new(
        window: &Window,
        strings: Vec<DOMString>,
        can_gc: CanGc,
    ) -> DomRoot<DOMStringList> {
        reflect_dom_object(
            Box::new(DOMStringList::new_inherited(strings)),
            window,
            can_gc,
        )
    }
}

// https://html.spec.whatwg.org/multipage/#domstringlist
impl DOMStringListMethods<crate::DomTypeHolder> for DOMStringList {
    // https://html.spec.whatwg.org/multipage/#dom-domstringlist-length
    fn Length(&self) -> u32 {
        self.strings.len() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-domstringlist-item
    fn Item(&self, index: u32) -> Option<DOMString> {
        self.strings.get(index as usize).cloned()
    }

    // https://html.spec.whatwg.org/multipage/#dom-domstringlist-contains
    fn Contains(&self, string: DOMString) -> bool {
        self.strings.contains(&string)
    }

    // check-tidy: no specs after this line
    fn IndexedGetter(&self, index: u32) -> Option<DOMString> {
        self.Item(index)
    }
}
