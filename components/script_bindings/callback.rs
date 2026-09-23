/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Base classes to work with IDL callbacks.

use std::default::Default;
use std::ffi::CStr;
use std::rc::Rc;

use js::context::JSContext;
use js::jsapi::{Heap, IsCallable, JSObject};
use js::jsval::{JSVal, NullValue, ObjectValue, UndefinedValue};
use js::rust::wrappers2::{EnterRealm, JS_GetProperty, JS_WrapObject, LeaveRealm};
use js::rust::{HandleObject, MutableHandleValue};

use crate::codegen::GenericBindings::WindowBinding::Window_Binding::WindowMethods;
use crate::error::{Error, Fallible};
use crate::interfaces::{DocumentHelpers, DomHelpers, GlobalScopeHelpers};
use crate::permanent_root::PermanentRoot;
use crate::realms::enter_auto_realm;
use crate::reflector::DomObject;
use crate::root::Dom;
use crate::settings_stack::{run_a_callback, run_a_script};
use crate::{DomTypes, cformat};

pub trait ThisReflector {
    fn jsobject(&self) -> *mut JSObject;
}

/// Try to obtain a Window object from a callback target.
/// This Window may be different than the callback's associated global if the
/// owner has been adopted into a different realm than it was created in.
/// As such, the default implementation should be used for any callback target
/// that cannot be adopted (i.e. is not a descendant of Node).
pub trait OwnerWindow<D: DomTypes> {
    fn owner_window(&self) -> Option<crate::root::DomRoot<D::Window>> {
        None
    }
}

impl<T: DomObject> ThisReflector for T {
    fn jsobject(&self) -> *mut JSObject {
        self.reflector().get_jsobject().get()
    }
}

impl ThisReflector for HandleObject<'_> {
    fn jsobject(&self) -> *mut JSObject {
        self.get()
    }
}

impl<D: DomTypes> OwnerWindow<D> for HandleObject<'_> {}

/// The exception handling used for a call.
#[derive(Clone, Copy, PartialEq)]
pub enum ExceptionHandling {
    /// Report any exception and don't throw it to the caller code.
    Report,
    /// Throw any exception to the caller code.
    Rethrow,
}

/// A WebIDL callback that is treated as a GC root.
pub struct RootedCallback<T>(Rc<(T, PermanentRoot)>);

impl<D: DomTypes, T: HasCallbackHolder<D = D>> RootedCallback<T> {
    /// Create a new [TracedCallback] value from this rooted callback.
    pub fn to_traced(&self) -> TracedCallback<T>
    where
        T: for<'a> From<&'a CallbackObject<D>>,
    {
        let mut duplicate = Rc::new(T::from(self.callback_holder()));
        // Note: callback cannot be moved after calling init.
        match Rc::get_mut(&mut duplicate) {
            Some(ref mut callback) => unsafe {
                callback
                    .callback_holder_mut()
                    .init_callback(self.callback())
            },
            None => unreachable!(),
        };
        TracedCallback(duplicate)
    }
}

impl<T> Clone for RootedCallback<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> std::ops::Deref for RootedCallback<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0.0
    }
}

impl<T: js::conversions::ToJSValConvertible> js::conversions::ToJSValConvertible
    for RootedCallback<T>
{
    fn to_jsval(&self, cx: &mut JSContext, rval: MutableHandleValue<'_>) {
        self.0.0.to_jsval(cx, rval)
    }
}

#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(JSTraceable, MallocSizeOf, PartialEq)]
/// A WebIDL callback value that can only be stored in locations that are
/// traced by the GC.
pub struct TracedCallback<T>(#[conditional_malloc_size_of] Rc<T>);

impl<T: crate::JSTraceable> js::gc::Rootable for TracedCallback<T> {}

impl<T: CallbackContainer + HasCallbackHolder> TracedCallback<T> {
    pub fn root(&self, cx: &JSContext) -> RootedCallback<T> {
        // Safety: the callback pointer is valid at this point.
        unsafe { T::new(cx, self.callback()) }
    }
}

impl<T> Clone for TracedCallback<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> std::ops::Deref for TracedCallback<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[expect(unsafe_code)]
pub(crate) unsafe fn create_callback<D: DomTypes, T: HasCallbackHolder<D = D>>(
    cx: &JSContext,
    obj: T,
    callback: *mut JSObject,
) -> Rc<T> {
    let mut ret = Rc::new(obj);
    unsafe {
        Rc::get_mut(&mut ret)
            .unwrap()
            .callback_holder_mut()
            .init(cx, callback)
    };
    ret
}

#[expect(unsafe_code)]
pub(crate) unsafe fn create_callback_rooted<D: DomTypes, T: HasCallbackHolder<D = D>>(
    cx: &JSContext,
    obj: T,
    callback: *mut JSObject,
) -> RootedCallback<T> {
    let mut ret = Rc::new((obj, PermanentRoot::default()));
    let (callback2, permanent_root) = Rc::get_mut(&mut ret).unwrap();
    unsafe {
        callback2.callback_holder_mut().init_callback(callback);
        permanent_root.init(cx, callback2.callback(), c"Callback::root");
    };
    RootedCallback(ret)
}

/// A common base class for representing IDL callback function and
/// callback interface types.
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub struct CallbackObject<D: DomTypes> {
    /// The underlying `JSObject`.
    #[ignore_malloc_size_of = "measured by mozjs"]
    callback: Heap<*mut JSObject>,
    // TODO(47889): Remove this field once no more uses of Rc<Callback> remain.
    permanent_js_root: Option<PermanentRoot>,

    /// The ["callback context"], that is, the global to use as incumbent
    /// global when calling the callback.
    ///
    /// Looking at the WebIDL standard, it appears as though there would always
    /// be a value here, but [sometimes] callback functions are created by
    /// hand-waving without defining the value of the callback context, and
    /// without any JavaScript code on the stack to grab an incumbent global
    /// from.
    ///
    /// ["callback context"]: https://heycam.github.io/webidl/#dfn-callback-context
    /// [sometimes]: https://github.com/whatwg/html/issues/2248
    incumbent: Option<Dom<D::GlobalScope>>,
}

impl<D: DomTypes> CallbackObject<D> {
    fn new_with_interior_root() -> Self {
        Self {
            callback: Heap::default(),
            permanent_js_root: Some(Default::default()),
            incumbent: D::GlobalScope::incumbent().map(|i| Dom::from_ref(&*i)),
        }
    }

    fn new_from_existing(other: &CallbackObject<D>) -> Self {
        Self {
            callback: Heap::default(),
            permanent_js_root: None,
            incumbent: other.incumbent.clone(),
        }
    }

    fn new_with_exterior_root() -> Self {
        Self {
            callback: Heap::default(),
            permanent_js_root: None,
            incumbent: D::GlobalScope::incumbent().map(|i| Dom::from_ref(&*i)),
        }
    }

    pub fn get(&self) -> *mut JSObject {
        self.callback.get()
    }

    #[expect(unsafe_code)]
    unsafe fn init_callback(&mut self, callback: *mut JSObject) {
        self.callback.set(callback);
    }

    #[expect(unsafe_code)]
    unsafe fn init(&mut self, cx: &JSContext, callback: *mut JSObject) {
        unsafe {
            self.init_callback(callback);
        }
        if let Some(ref permanent_root) = self.permanent_js_root {
            unsafe {
                permanent_root.init(cx, self.callback.get(), c"CallbackObject::root");
            }
        }
    }
}

impl<D: DomTypes> PartialEq for CallbackObject<D> {
    fn eq(&self, other: &CallbackObject<D>) -> bool {
        self.callback.get() == other.callback.get()
    }
}

/// A type which can obtain a reference to a CallbackObject member.
pub trait HasCallbackHolder {
    type D: DomTypes;

    /// Returns the underlying `CallbackObject`.
    fn callback_holder(&self) -> &CallbackObject<Self::D>;
    /// Returns the underlying `CallbackObject`.
    fn callback_holder_mut(&mut self) -> &mut CallbackObject<Self::D>;

    /// Returns the underlying `JSObject`.
    fn callback(&self) -> *mut JSObject {
        self.callback_holder().get()
    }
}

/// A trait to be implemented by concrete IDL callback function and
/// callback interface types.
pub trait CallbackContainer {
    /// Create a new rooted CallbackContainer object for the given `JSObject`.
    ///
    /// # Safety
    /// `callback` must point to a valid, non-null JSObject.
    unsafe fn new(cx: &JSContext, callback: *mut JSObject) -> RootedCallback<Self>
    where
        Self: Sized;
}

/// A trait to be implemented by concrete IDL callback function and
/// callback interface types that have not yet been converted to RootedCallback.
pub trait DeprecatedCallbackContainer {
    /// Create a new CallbackContainer object for the given `JSObject`.
    ///
    /// *Deprecated*: Use [CallbackContainer] instead.
    ///
    /// # Safety
    /// `callback` must point to a valid, non-null JSObject.
    unsafe fn new(cx: &JSContext, callback: *mut JSObject) -> Rc<Self>;
}

/// A common base class for representing IDL callback function types.
#[derive(JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub struct CallbackFunction<D: DomTypes> {
    object: CallbackObject<D>,
}

impl<'a, D: DomTypes> From<&'a CallbackObject<D>> for CallbackFunction<D> {
    fn from(object: &'a CallbackObject<D>) -> Self {
        Self {
            object: CallbackObject::new_from_existing(object),
        }
    }
}

impl<D: DomTypes> CallbackFunction<D> {
    /// Create a new `CallbackFunction` for this object.
    pub(crate) fn new_with_interior_root() -> Self {
        Self {
            object: CallbackObject::new_with_interior_root(),
        }
    }

    /// Create a new `CallbackFunction` for this object, with rooting provided
    /// by the caller.
    pub(crate) fn new_with_exterior_root() -> Self {
        Self {
            object: CallbackObject::new_with_exterior_root(),
        }
    }
}

impl<D: DomTypes> HasCallbackHolder for CallbackFunction<D> {
    type D = D;
    /// Returns the underlying `CallbackObject`.
    fn callback_holder(&self) -> &CallbackObject<D> {
        &self.object
    }

    fn callback_holder_mut(&mut self) -> &mut CallbackObject<D> {
        &mut self.object
    }
}

/// A common base class for representing IDL callback interface types.
#[derive(JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub struct CallbackInterface<D: DomTypes> {
    object: CallbackObject<D>,
}

impl<'a, D: DomTypes> From<&'a CallbackObject<D>> for CallbackInterface<D> {
    fn from(object: &'a CallbackObject<D>) -> Self {
        Self {
            object: CallbackObject::new_from_existing(object),
        }
    }
}

impl<D: DomTypes> HasCallbackHolder for CallbackInterface<D> {
    type D = D;
    /// Returns the underlying `CallbackObject`.
    fn callback_holder(&self) -> &CallbackObject<D> {
        &self.object
    }

    fn callback_holder_mut(&mut self) -> &mut CallbackObject<D> {
        &mut self.object
    }
}

impl<D: DomTypes> CallbackInterface<D> {
    /// Create a new CallbackInterface object.
    pub(crate) fn new_with_interior_root() -> Self {
        Self {
            object: CallbackObject::new_with_interior_root(),
        }
    }

    /// Create a new CallbackInterface object with rooting provided by the caller.
    pub(crate) fn new_with_exterior_root() -> Self {
        Self {
            object: CallbackObject::new_with_exterior_root(),
        }
    }

    /// Returns the property with the given `name`, if it is a callable object,
    /// or an error otherwise.
    pub fn get_callable_property(&self, cx: &mut JSContext, name: &CStr) -> Fallible<JSVal> {
        rooted!(&in(cx) let mut callable = UndefinedValue());
        rooted!(&in(cx) let obj = self.callback_holder().get());
        unsafe {
            if !JS_GetProperty(cx, obj.handle(), name.as_ptr(), callable.handle_mut()) {
                return Err(Error::JSFailed);
            }

            if !callable.is_object() || !IsCallable(callable.to_object()) {
                return Err(Error::Type(cformat!(
                    "The value of the {} property is not callable",
                    name.to_string_lossy()
                )));
            }
        }
        Ok(callable.get())
    }
}

/// Wraps the reflector for `p` into the realm of `cx`.
pub(crate) fn wrap_call_this_value<T: ThisReflector>(
    cx: &mut JSContext,
    p: &T,
    mut rval: MutableHandleValue,
) -> bool {
    rooted!(&in(cx) let mut obj = p.jsobject());

    if obj.is_null() {
        rval.set(NullValue());
        return true;
    }

    unsafe {
        if !JS_WrapObject(cx, obj.handle_mut()) {
            return false;
        }
    }

    rval.set(ObjectValue(*obj));
    true
}

/// A function wrapper that performs whatever setup we need to safely make a call.
///
/// <https://webidl.spec.whatwg.org/#es-invoking-callback-functions>
pub(crate) fn call_setup<D: DomTypes, T: HasCallbackHolder<D = D>, R>(
    cx: &mut JSContext,
    callback: &T,
    owner_window: Option<&D::Window>,
    handling: ExceptionHandling,
    f: impl FnOnce(&mut JSContext) -> R,
) -> R {
    if let Some(window) = owner_window {
        window.Document().ensure_safe_to_run_script_or_layout();
    }

    // The global for reporting exceptions. This is the global object of the
    // (possibly wrapped) callback object.
    let global = unsafe { D::GlobalScope::from_object(callback.callback()) };
    let global = &global;

    // Step 8: Prepare to run script with relevant settings.
    run_a_script::<D, R, _>(cx, global, move |cx| {
        let actual_callback = || {
            let old_realm = unsafe { EnterRealm(cx, callback.callback()) };
            let result = f(cx);
            unsafe {
                LeaveRealm(cx, old_realm);
            }
            if handling == ExceptionHandling::Report {
                let mut realm = enter_auto_realm::<D>(cx, &**global);
                let cx = &mut realm.current_realm();
                <D as DomHelpers<D>>::report_pending_exception(cx);
            }
            result
        };
        if let Some(incumbent_global) = callback.callback_holder().incumbent.as_deref() {
            // Step 9: Prepare to run a callback with stored settings.
            run_a_callback::<D, R>(incumbent_global, actual_callback)
        } else {
            actual_callback()
        }
    }) // Step 14.2: Clean up after running script with relevant settings.
}
