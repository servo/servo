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
use wgpu_core::naga::front::wgsl::ImplementedLanguageExtension;

use crate::dom::bindings::codegen::Bindings::WebGPUBinding::WGSLLanguageFeaturesMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub struct WGSLLanguageFeatures {
    reflector: Reflector,
    // internal storage for features
    #[custom_trace]
    internal: DomRefCell<IndexSet<DOMString>>,
}

impl WGSLLanguageFeatures {
    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
    ) -> DomRoot<Self> {
        let set = ImplementedLanguageExtension::all()
            .iter()
            .map(|le| le.to_ident().into())
            .collect();
        reflect_dom_object_with_proto(
            cx,
            Box::new(Self {
                reflector: Reflector::new(),
                internal: DomRefCell::new(set),
            }),
            global,
            proto,
        )
    }
}

impl WGSLLanguageFeaturesMethods<crate::DomTypeHolder> for WGSLLanguageFeatures {
    fn Size(&self) -> u32 {
        self.internal.borrow().len() as u32
    }
}

impl Setlike for WGSLLanguageFeatures {
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
    fn add(&self, _cx: &mut JSContext, _key: Self::Key) {
        unreachable!("readonly");
    }
    #[inline(always)]
    fn has(&self, cx: &mut JSContext, key: Self::Key) -> bool {
        self.internal.has(cx, key)
    }
    #[inline(always)]
    fn clear(&self, _cx: &mut JSContext) {
        unreachable!("readonly");
    }
    #[inline(always)]
    fn delete(&self, _cx: &mut JSContext, _key: Self::Key) -> bool {
        unreachable!("readonly");
    }
}
