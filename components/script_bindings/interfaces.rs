/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::thread::LocalKey;

use js::context::JSContext;
use js::glue::JSPrincipalsCallbacks;
use js::jsapi::{CallArgs, JSObject};
use js::realm::CurrentRealm;
use js::rust::{HandleObject, MutableHandleObject};
use servo_url::{MutableOrigin, ServoUrl};

use crate::DomTypes;
use crate::codegen::PrototypeList;
use crate::conversions::DerivedFrom;
use crate::error::Error;
use crate::realms::InRealm;
use crate::reflector::{DomObject, DomObjectWrap};
use crate::root::DomRoot;
use crate::script_runtime::JSContext as SafeJSContext;
use crate::settings_stack::StackEntry;
use crate::utils::ProtoOrIfaceArray;

/// Operations that can be invoked for a WebIDL interface against
/// a global object.
///
/// <https://github.com/mozilla/gecko-dev/blob/3fd619f47/dom/bindings/WebIDLGlobalNameHash.h#L24>
pub struct Interface {
    /// Define the JS object for this interface on the given global.
    pub define: fn(&mut JSContext, HandleObject),
    /// Returns true if this interface's conditions are met for the given global.
    pub enabled: fn(&mut JSContext, HandleObject) -> bool,
}

/// Operations that must be invoked from the generated bindings.
pub trait DomHelpers<D: DomTypes> {
    fn throw_dom_exception(cx: &mut JSContext, global: &D::GlobalScope, result: Error);

    fn call_html_constructor<T: DerivedFrom<D::Element> + DomObject>(
        cx: &mut JSContext,
        args: &CallArgs,
        global: &D::GlobalScope,
        proto_id: PrototypeList::ID,
        creator: unsafe fn(&mut JSContext, HandleObject, *mut ProtoOrIfaceArray),
    ) -> bool;

    fn settings_stack() -> &'static LocalKey<RefCell<Vec<StackEntry<D>>>>;

    fn principals_callbacks() -> &'static JSPrincipalsCallbacks;

    fn interface_map() -> &'static phf::Map<&'static [u8], Interface>;

    fn push_new_element_queue();
    fn pop_current_element_queue(cx: &mut JSContext);

    fn reflect_dom_object_with_cx<T, U>(cx: &mut JSContext, obj: Box<T>, global: &U) -> DomRoot<T>
    where
        T: DomObject + DomObjectWrap<D>,
        U: DerivedFrom<D::GlobalScope>;

    fn report_pending_exception(cx: &mut CurrentRealm);
}

/// Operations that must be invoked from the generated bindings.
#[expect(unsafe_code)]
pub trait GlobalScopeHelpers<D: DomTypes> {
    fn from_current_realm(realm: &'_ CurrentRealm) -> DomRoot<D::GlobalScope>;
    fn get_cx() -> SafeJSContext;
    /// # Safety
    /// `obj` must point to a valid, non-null JSObject.
    unsafe fn from_object(obj: *mut JSObject) -> DomRoot<D::GlobalScope>;
    fn from_reflector(reflector: &impl DomObject, realm: InRealm) -> DomRoot<D::GlobalScope>;

    fn origin(&self) -> MutableOrigin;

    fn incumbent() -> Option<DomRoot<D::GlobalScope>>;

    fn perform_a_microtask_checkpoint(&self, cx: &mut JSContext);

    fn get_url(&self) -> ServoUrl;

    fn is_secure_context(&self) -> bool;
}

pub trait DocumentHelpers {
    fn ensure_safe_to_run_script_or_layout(&self);
}

pub trait ServoInternalsHelpers {
    fn is_servo_internal(cx: &mut JSContext, global: HandleObject) -> bool;
}

pub trait TestBindingHelpers {
    fn condition_satisfied(cx: &mut JSContext, global: HandleObject) -> bool;
    fn condition_unsatisfied(cx: &mut JSContext, global: HandleObject) -> bool;
}

pub trait WebGL2RenderingContextHelpers {
    fn is_webgl2_enabled(cx: &mut JSContext, global: HandleObject) -> bool;
}

pub trait WindowHelpers {
    fn create_named_properties_object(
        cx: &mut JSContext,
        proto: HandleObject,
        object: MutableHandleObject,
    );
}

pub trait HasOrigin {
    fn origin(&self) -> MutableOrigin;
}
