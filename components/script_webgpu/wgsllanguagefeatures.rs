/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use std::marker::PhantomData;

use dom_struct::dom_struct;
use indexmap::IndexSet;
use js::context::JSContext;
use js::gc::HandleObject;
use jstraceable_derive::JSTraceable;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    WGSLLanguageFeaturesMethods, WGSLLanguageFeaturesWrap,
};
use script_bindings::like::Setlike;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_proto_and_wrap};
use wgpu_core::naga::front::wgsl::ImplementedLanguageExtension;

use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;

#[dom_struct]
pub struct WGSLLanguageFeatures<D: DomTypes> {
    reflector: Reflector,
    // internal storage for features
    #[custom_trace]
    internal: DomRefCell<IndexSet<DOMString>>,
    #[no_trace = "PhantomData does not exist"]
    phantom: PhantomData<D>,
}

impl<D> WGSLLanguageFeatures<D>
where
    D: DomTypes<WGSLLanguageFeatures = WGSLLanguageFeatures<D>>,
{
    pub fn new(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        proto: Option<HandleObject>,
    ) -> DomRoot<Self> {
        let set = ImplementedLanguageExtension::all()
            .iter()
            .map(|le| le.to_ident().into())
            .collect();
        reflect_dom_object_with_proto_and_wrap::<D, _, _>(
            Box::new(Self {
                reflector: Reflector::new(),
                internal: DomRefCell::new(set),
                phantom: PhantomData,
            }),
            global,
            proto,
            cx,
            WGSLLanguageFeaturesWrap::<D>,
        )
    }
}

impl<D: DomTypes> WGSLLanguageFeaturesMethods<D> for WGSLLanguageFeatures<D> {
    fn Size(&self) -> u32 {
        self.internal.borrow().len() as u32
    }
}

impl<D: DomTypes> Setlike for WGSLLanguageFeatures<D> {
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
