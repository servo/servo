/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use dom_struct::dom_struct;
use indexmap::IndexSet;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::cell::DomRefCell;
use script_bindings::like::Setlike;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_proto};

use crate::dom::bindings::codegen::Bindings::TestBindingSetlikeWithPrimitiveBinding::TestBindingSetlikeWithPrimitiveMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::setlike;

// setlike<DOMString>
#[dom_struct]
pub(crate) struct TestBindingSetlikeWithPrimitive {
    reflector: Reflector,
    #[custom_trace]
    internal: DomRefCell<IndexSet<DOMString>>,
}

impl TestBindingSetlikeWithPrimitive {
    fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
    ) -> DomRoot<TestBindingSetlikeWithPrimitive> {
        reflect_dom_object_with_proto(
            cx,
            Box::new(TestBindingSetlikeWithPrimitive {
                reflector: Reflector::new(),
                internal: DomRefCell::new(IndexSet::new()),
            }),
            global,
            proto,
        )
    }
}

impl TestBindingSetlikeWithPrimitiveMethods<crate::DomTypeHolder>
    for TestBindingSetlikeWithPrimitive
{
    fn Constructor(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
    ) -> Fallible<DomRoot<TestBindingSetlikeWithPrimitive>> {
        Ok(TestBindingSetlikeWithPrimitive::new(cx, global, proto))
    }

    fn Size(&self) -> u32 {
        self.internal.borrow().len() as u32
    }
}

impl Setlike for TestBindingSetlikeWithPrimitive {
    type Key = DOMString;

    setlike!(self, internal);
}
