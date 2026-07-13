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
use script_bindings::reflector::{Reflector, reflect_dom_object_with_proto_and_cx};

use crate::dom::bindings::codegen::Bindings::TestBindingSetlikeWithInterfaceBinding::TestBindingSetlikeWithInterfaceMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::testbinding::TestBinding;
use crate::setlike;

// setlike<TestBinding>
#[dom_struct]
pub(crate) struct TestBindingSetlikeWithInterface {
    reflector: Reflector,
    #[custom_trace]
    internal: DomRefCell<IndexSet<DomRoot<TestBinding>>>,
}

impl TestBindingSetlikeWithInterface {
    fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
    ) -> DomRoot<TestBindingSetlikeWithInterface> {
        reflect_dom_object_with_proto_and_cx(
            Box::new(TestBindingSetlikeWithInterface {
                reflector: Reflector::new(),
                internal: DomRefCell::new(IndexSet::new()),
            }),
            global,
            proto,
            cx,
        )
    }
}

impl TestBindingSetlikeWithInterfaceMethods<crate::DomTypeHolder>
    for TestBindingSetlikeWithInterface
{
    fn Constructor(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
    ) -> Fallible<DomRoot<TestBindingSetlikeWithInterface>> {
        Ok(TestBindingSetlikeWithInterface::new(cx, global, proto))
    }

    fn Size(&self) -> u32 {
        self.internal.borrow().len() as u32
    }
}

impl Setlike for TestBindingSetlikeWithInterface {
    type Key = DomRoot<TestBinding>;

    setlike!(self, internal);
}
