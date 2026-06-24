/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::gc::Traceable as JSTraceable;
use js::realm::CurrentRealm;
use js::rust::HandleValue;
use jstraceable_derive::JSTraceable;
use malloc_size_of::MallocSizeOf;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx_and_wrap};
use script_bindings::root::DomRoot;

/// Types that implement the `Callback` trait follow the same rooting requirements
/// as types that use the `#[dom_struct]` attribute.
/// Prefer storing `Dom<T>` members inside them instead of `DomRoot<T>`
/// to minimize redundant work by the garbage collector.
pub trait Callback: JSTraceable + MallocSizeOf {
    fn callback(&self, cx: &mut CurrentRealm, v: HandleValue);
}

#[dom_struct]
pub struct PromiseNativeHandler<D: DomTypes> {
    reflector: Reflector,
    resolve: Option<Box<dyn Callback>>,
    reject: Option<Box<dyn Callback>>,
    phantom: PhantomData<D>,
}

impl<D: DomTypes> PromiseNativeHandler<D>
where
    D: DomTypes<PromiseNativeHandler = PromiseNativeHandler<D>>,
{
    pub fn new(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        resolve: Option<Box<dyn Callback>>,
        reject: Option<Box<dyn Callback>>,
    ) -> DomRoot<PromiseNativeHandler<D>> {
        reflect_dom_object_with_cx_and_wrap::<D,_,_>(
            Box::new(PromiseNativeHandler {
                reflector: Reflector::new(),
                resolve,
                reject,
                phantom: PhantomData
            }),
            global,
            cx,
            crate::dom::bindings::codegen::GenericBindings::PromiseNativeHandlerBinding::PromiseNativeHandler_Binding::Wrap::<D>
        )
    }

    fn callback(callback: &Option<Box<dyn Callback>>, cx: &mut CurrentRealm, v: HandleValue) {
        if let Some(ref callback) = *callback {
            callback.callback(cx, v)
        }
    }

    pub(crate) fn resolved_callback(&self, cx: &mut CurrentRealm, v: HandleValue) {
        PromiseNativeHandler::<D>::callback(&self.resolve, cx, v)
    }

    pub(crate) fn rejected_callback(&self, cx: &mut CurrentRealm, v: HandleValue) {
        PromiseNativeHandler::<D>::callback(&self.reject, cx, v)
    }
}
