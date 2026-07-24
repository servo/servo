/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use atomic_refcell::{AtomicRef, AtomicRefCell};
use dom_struct::dom_struct;
use indexmap::IndexSet;
use js::context::JSContext;
use script_bindings::codegen::GenericBindings::ElementInternalsBinding::CustomStateSetMethods;
use script_bindings::like::Setlike;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use script_bindings::root::{Dom, DomRoot};
use script_bindings::str::DOMString;

use crate::dom::bindings::inheritance::Castable;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::node::{Node, NodeDamage};
use crate::dom::window::Window;

/// <https://html.spec.whatwg.org/multipage/#customstateset>
#[dom_struct]
pub(crate) struct CustomStateSet {
    reflector: Reflector,
    #[no_trace]
    internal: AtomicRefCell<IndexSet<DOMString>>,
    owner_element: Dom<HTMLElement>,
}

impl CustomStateSet {
    fn new_inherited(element: &HTMLElement) -> Self {
        Self {
            reflector: Reflector::new(),
            internal: Default::default(),
            owner_element: Dom::from_ref(element),
        }
    }

    pub(crate) fn new(cx: &mut JSContext, window: &Window, element: &HTMLElement) -> DomRoot<Self> {
        reflect_dom_object_with_cx(Box::new(Self::new_inherited(element)), window, cx)
    }

    pub(crate) fn set_for_layout(&self) -> AtomicRef<'_, IndexSet<DOMString>> {
        self.internal.borrow()
    }

    fn states_did_change(&self) {
        self.owner_element.upcast::<Node>().dirty(NodeDamage::Other);
    }
}

impl Setlike for CustomStateSet {
    type Key = DOMString;

    #[inline(always)]
    fn get_index(&self, cx: &mut JSContext, index: u32) -> Option<Self::Key> {
        self.internal.get_index(cx, index)
    }

    #[inline(always)]
    fn size(&self, cx: &mut JSContext) -> u32 {
        self.internal.size(cx)
    }

    #[inline(always)]
    fn add(&self, cx: &mut JSContext, key: Self::Key) {
        self.internal.add(cx, key);
        self.states_did_change();
    }

    #[inline(always)]
    fn has(&self, cx: &mut JSContext, key: Self::Key) -> bool {
        self.internal.has(cx, key)
    }

    #[inline(always)]
    fn clear(&self, cx: &mut JSContext) {
        let old_size = self.internal.size(cx);
        self.internal.clear(cx);
        if old_size != 0 {
            self.states_did_change();
        }
    }

    #[inline(always)]
    fn delete(&self, cx: &mut JSContext, key: Self::Key) -> bool {
        if self.internal.delete(cx, key) {
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
        self.internal.borrow().len() as u32
    }
}
