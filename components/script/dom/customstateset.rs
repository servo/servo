/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use indexmap::IndexSet;
use script_bindings::codegen::GenericBindings::ElementInternalsBinding::CustomStateSetMethods;
use script_bindings::like::Setlike;
use script_bindings::root::{Dom, DomRoot};
use script_bindings::script_runtime::CanGc;
use script_bindings::str::DOMString;
use script_bindings::trace::CustomTraceable;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::node::{Node, NodeDamage};
use crate::dom::window::Window;

/// <https://html.spec.whatwg.org/multipage/#customstateset>
#[dom_struct]
pub(crate) struct CustomStateSet {
    reflector: Reflector,
    internal: DomRefCell<IndexSet<DOMString>>,
    owner_element: Dom<HTMLElement>,
}

impl CustomStateSet {
    fn new_inherited(element: &HTMLElement) -> Self {
        Self {
            reflector: Reflector::new(),
            internal: DomRefCell::new(Default::default()),
            owner_element: Dom::from_ref(element),
        }
    }

    pub(crate) fn new(window: &Window, element: &HTMLElement, can_gc: CanGc) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited(element)), window, can_gc)
    }

    /// Returns a borrowed version of the set without the usual Ref wrapper.
    /// Mutating the underlying refcell while this value is active is
    /// undefined behaviour.
    #[expect(unsafe_code)]
    pub(crate) unsafe fn set_for_layout(&self) -> &IndexSet<DOMString> {
        unsafe { self.internal.borrow_for_layout() }
    }

    fn states_did_change(&self) {
        self.owner_element.upcast::<Node>().dirty(NodeDamage::Other);
    }
}

impl Setlike for CustomStateSet {
    type Key = DOMString;

    #[inline(always)]
    fn get_index(&self, index: u32) -> Option<Self::Key> {
        self.internal.get_index(index)
    }

    #[inline(always)]
    fn size(&self) -> u32 {
        self.internal.size()
    }

    #[inline(always)]
    fn add(&self, key: Self::Key) {
        self.internal.add(key);
        self.states_did_change();
    }

    #[inline(always)]
    fn has(&self, key: Self::Key) -> bool {
        self.internal.has(key)
    }

    #[inline(always)]
    fn clear(&self) {
        let old_size = self.internal.size();
        self.internal.clear();
        if old_size != 0 {
            self.states_did_change();
        }
    }

    #[inline(always)]
    fn delete(&self, key: Self::Key) -> bool {
        if self.internal.delete(key) {
            self.states_did_change();
            true
        } else {
            false
        }
    }
}

impl CustomStateSetMethods<crate::DomTypeHolder> for CustomStateSet {
    /// <https://html.spec.whatwg.org/multipage/#customstateset>
    fn Size(&self) -> u32 {
        self.internal.size()
    }
}
