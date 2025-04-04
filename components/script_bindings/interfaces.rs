/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;
use std::thread::LocalKey;

use js::conversions::ToJSValConvertible;
use js::glue::JSPrincipalsCallbacks;
use js::jsapi::{CallArgs, HandleObject as RawHandleObject, JSContext as RawJSContext, JSObject};
use js::rust::{HandleObject, MutableHandleObject};
use servo_url::{MutableOrigin, ServoUrl};

use crate::DomTypes;
use crate::codegen::PrototypeList;
use crate::conversions::DerivedFrom;
use crate::error::Error;
use crate::realms::InRealm;
use crate::reflector::{DomObject, DomObjectWrap};
use crate::root::DomRoot;
use crate::script_runtime::{CanGc, JSContext};
use crate::settings_stack::StackEntry;
use crate::utils::ProtoOrIfaceArray;

/// Operations that must be invoked from the generated bindings.
pub trait DomHelpers<D: DomTypes> {
    fn throw_dom_exception(cx: JSContext, global: &D::GlobalScope, result: Error, can_gc: CanGc);

    fn call_html_constructor<T: DerivedFrom<D::Element> + DomObject>(
        cx: JSContext,
        args: &CallArgs,
        global: &D::GlobalScope,
        proto_id: PrototypeList::ID,
        creator: unsafe fn(JSContext, HandleObject, *mut ProtoOrIfaceArray),
        can_gc: CanGc,
    ) -> bool;

    fn settings_stack() -> &'static LocalKey<RefCell<Vec<StackEntry<D>>>>;

    fn principals_callbacks() -> &'static JSPrincipalsCallbacks;

    fn is_platform_object_same_origin(cx: JSContext, obj: RawHandleObject) -> bool;

    fn interface_map() -> &'static phf::Map<&'static [u8], for<'a> fn(JSContext, HandleObject)>;

    fn push_new_element_queue();
    fn pop_current_element_queue(can_gc: CanGc);

    fn reflect_dom_object<T, U>(obj: Box<T>, global: &U, can_gc: CanGc) -> DomRoot<T>
    where
        T: DomObject + DomObjectWrap<D>,
        U: DerivedFrom<D::GlobalScope>;

    fn report_pending_exception(cx: JSContext, dispatch_event: bool, realm: InRealm, can_gc: CanGc);
}

/// Operations that must be invoked from the generated bindings.
#[allow(unsafe_code)]
pub trait GlobalScopeHelpers<D: DomTypes> {
    /// # Safety
    /// `cx` must point to a valid, non-null RawJSContext.
    unsafe fn from_context(cx: *mut RawJSContext, realm: InRealm) -> DomRoot<D::GlobalScope>;
    fn get_cx() -> JSContext;
    /// # Safety
    /// `obj` must point to a valid, non-null JSObject.
    unsafe fn from_object(obj: *mut JSObject) -> DomRoot<D::GlobalScope>;
    fn from_reflector(reflector: &impl DomObject, realm: InRealm) -> DomRoot<D::GlobalScope>;

    /// # Safety
    /// `obj` must point to a valid, non-null JSObject.
    /// `cx` must point to a valid, non-null RawJSContext.
    unsafe fn from_object_maybe_wrapped(
        obj: *mut JSObject,
        cx: *mut RawJSContext,
    ) -> DomRoot<D::GlobalScope>;

    fn origin(&self) -> &MutableOrigin;

    fn incumbent() -> Option<DomRoot<D::GlobalScope>>;

    fn perform_a_microtask_checkpoint(&self, can_gc: CanGc);

    fn get_url(&self) -> ServoUrl;

    fn is_secure_context(&self) -> bool;
}

pub trait DocumentHelpers {
    fn ensure_safe_to_run_script_or_layout(&self);
}

/// Operations that must be invoked from the generated bindings.
pub trait PromiseHelpers<D: crate::DomTypes> {
    fn new_resolved(
        global: &D::GlobalScope,
        cx: JSContext,
        value: impl ToJSValConvertible,
    ) -> Rc<D::Promise>;
}

pub trait ServoInternalsHelpers {
    fn is_servo_internal(cx: JSContext, global: HandleObject) -> bool;
}

pub trait TestBindingHelpers {
    fn condition_satisfied(cx: JSContext, global: HandleObject) -> bool;
    fn condition_unsatisfied(cx: JSContext, global: HandleObject) -> bool;
}

pub trait WebGL2RenderingContextHelpers {
    fn is_webgl2_enabled(cx: JSContext, global: HandleObject) -> bool;
}

pub trait WindowHelpers {
    fn create_named_properties_object(
        cx: JSContext,
        proto: HandleObject,
        object: MutableHandleObject,
    );
}
