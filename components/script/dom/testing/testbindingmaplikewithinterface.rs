/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use dom_struct::dom_struct;
use indexmap::IndexMap;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::cell::DomRefCell;
use script_bindings::cformat;
use script_bindings::like::Maplike;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_proto};

use crate::dom::bindings::codegen::Bindings::TestBindingMaplikeWithInterfaceBinding::TestBindingMaplikeWithInterfaceMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::testbinding::TestBinding;
use crate::maplike;

/// maplike<DOMString, TestBinding>
#[dom_struct]
pub(crate) struct TestBindingMaplikeWithInterface {
    reflector: Reflector,
    #[custom_trace]
    internal: DomRefCell<IndexMap<DOMString, DomRoot<TestBinding>>>,
}

impl TestBindingMaplikeWithInterface {
    fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
    ) -> DomRoot<TestBindingMaplikeWithInterface> {
        reflect_dom_object_with_proto(
            cx,
            Box::new(TestBindingMaplikeWithInterface {
                reflector: Reflector::new(),
                internal: DomRefCell::new(IndexMap::new()),
            }),
            global,
            proto,
        )
    }
}

impl TestBindingMaplikeWithInterfaceMethods<crate::DomTypeHolder>
    for TestBindingMaplikeWithInterface
{
    fn Constructor(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
    ) -> Fallible<DomRoot<TestBindingMaplikeWithInterface>> {
        Ok(TestBindingMaplikeWithInterface::new(cx, global, proto))
    }

    fn SetInternal(&self, cx: &mut JSContext, key: DOMString, value: &TestBinding) {
        let value = DomRoot::from_ref(value);
        self.internal.set(cx, key, value)
    }

    fn ClearInternal(&self, cx: &mut JSContext) {
        self.internal.clear(cx)
    }

    fn DeleteInternal(&self, cx: &mut JSContext, key: DOMString) -> bool {
        self.internal.delete(cx, key)
    }

    fn HasInternal(&self, cx: &mut JSContext, key: DOMString) -> bool {
        self.internal.has(cx, key)
    }

    fn GetInternal(&self, key: DOMString) -> Fallible<DomRoot<TestBinding>> {
        // TODO: error type?
        self.internal
            .borrow()
            .get(&key)
            .ok_or_else(|| Error::Type(cformat!("No entry for key {key}")))
            .cloned()
    }

    fn Size(&self, cx: &mut JSContext) -> u32 {
        self.internal.size(cx)
    }
}

impl Maplike for TestBindingMaplikeWithInterface {
    type Key = DOMString;
    type Value = DomRoot<TestBinding>;

    maplike!(self, internal);
}
