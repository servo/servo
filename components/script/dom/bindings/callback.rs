/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Base classes to work with IDL callbacks.

use dom::bindings::error::{Error, Fallible, report_pending_exception};
use dom::bindings::reflector::DomObject;
use dom::bindings::root::{Dom, DomRoot};
use dom::bindings::settings_stack::{AutoEntryScript, AutoIncumbentScript};
use dom::bindings::utils::AsCCharPtrPtr;
use dom::globalscope::GlobalScope;
use js::jsapi::{IsCallable, JSContext, JSObject, AddRawValueRoot};
use js::jsapi::{JSCompartment, JS_EnterCompartment, JS_LeaveCompartment, RemoveRawValueRoot};
use js::jsapi::Heap;
use js::jsapi::JSAutoCompartment;
use js::jsval::{JSVal, UndefinedValue, ObjectValue};
use js::rust::{MutableHandleObject, Runtime};
use js::rust::wrappers::{JS_WrapObject, JS_GetProperty};
use std::default::Default;
use std::ffi::CString;
use std::mem::drop;
use std::ops::Deref;
use std::ptr;
use std::rc::Rc;
use typeholder::TypeHolderTrait;

/// The exception handling used for a call.
#[derive(Clone, Copy, PartialEq)]
pub enum ExceptionHandling {
    /// Report any exception and don't throw it to the caller code.
    Report,
    /// Throw any exception to the caller code.
    Rethrow,
}

/// A common base class for representing IDL callback function and
/// callback interface types.
#[derive(JSTraceable)]
#[must_root]
pub struct CallbackObject<TH: TypeHolderTrait> {
    /// The underlying `JSObject`.
    callback: Heap<*mut JSObject>,
    permanent_js_root: Heap<JSVal>,

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
    incumbent: Option<Dom<GlobalScope<TH>>>
}

impl<TH: TypeHolderTrait> Default for CallbackObject<TH> {
    #[allow(unrooted_must_root)]
    fn default() -> CallbackObject<TH> {
        CallbackObject::new()
    }
}

impl<TH: TypeHolderTrait> CallbackObject<TH> {
    #[allow(unrooted_must_root)]
    fn new() -> CallbackObject<TH> {
        CallbackObject {
            callback: Heap::default(),
            permanent_js_root: Heap::default(),
            incumbent: GlobalScope::<TH>::incumbent().map(|i| Dom::from_ref(&*i)),
        }
    }

    pub fn get(&self) -> *mut JSObject {
        self.callback.get()
    }

    #[allow(unsafe_code)]
    unsafe fn init(&mut self, cx: *mut JSContext, callback: *mut JSObject) {
        self.callback.set(callback);
        self.permanent_js_root.set(ObjectValue(callback));
        assert!(AddRawValueRoot(cx, self.permanent_js_root.get_unsafe(),
                                b"CallbackObject::root\n".as_c_char_ptr()));
    }
}

impl<TH: TypeHolderTrait> Drop for CallbackObject<TH> {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        unsafe {
            let cx = Runtime::get();
            RemoveRawValueRoot(cx, self.permanent_js_root.get_unsafe());
        }
    }

}

impl<TH: TypeHolderTrait> PartialEq for CallbackObject<TH> {
    fn eq(&self, other: &CallbackObject<TH>) -> bool {
        self.callback.get() == other.callback.get()
    }
}


/// A trait to be implemented by concrete IDL callback function and
/// callback interface types.
pub trait CallbackContainer<TH: TypeHolderTrait> {
    /// Create a new CallbackContainer object for the given `JSObject`.
    unsafe fn new(cx: *mut JSContext, callback: *mut JSObject) -> Rc<Self>;
    /// Returns the underlying `CallbackObject`.
    fn callback_holder(&self) -> &CallbackObject<TH>;
    /// Returns the underlying `JSObject`.
    fn callback(&self) -> *mut JSObject {
        self.callback_holder().get()
    }
    /// Returns the ["callback context"], that is, the global to use as
    /// incumbent global when calling the callback.
    ///
    /// ["callback context"]: https://heycam.github.io/webidl/#dfn-callback-context
    fn incumbent(&self) -> Option<&GlobalScope<TH>> {
        self.callback_holder().incumbent.as_ref().map(Dom::deref)
    }
}


/// A common base class for representing IDL callback function types.
#[derive(JSTraceable, PartialEq)]
#[must_root]
pub struct CallbackFunction<TH: TypeHolderTrait> {
    object: CallbackObject<TH>,
}

impl<TH: TypeHolderTrait> CallbackFunction<TH> {
    /// Create a new `CallbackFunction` for this object.
    #[allow(unrooted_must_root)]
    pub fn new() -> CallbackFunction<TH> {
        CallbackFunction {
            object: CallbackObject::new(),
        }
    }

    /// Returns the underlying `CallbackObject`.
    pub fn callback_holder(&self) -> &CallbackObject<TH> {
        &self.object
    }

    /// Initialize the callback function with a value.
    /// Should be called once this object is done moving.
    pub unsafe fn init(&mut self, cx: *mut JSContext, callback: *mut JSObject) {
        self.object.init(cx, callback);
    }
}




/// A common base class for representing IDL callback interface types.
#[derive(JSTraceable, PartialEq)]
#[must_root]
pub struct CallbackInterface<TH: TypeHolderTrait> {
    object: CallbackObject<TH>,
}

impl<TH: TypeHolderTrait> CallbackInterface<TH> {
    /// Create a new CallbackInterface object for the given `JSObject`.
    pub fn new() -> CallbackInterface<TH> {
        CallbackInterface {
            object: CallbackObject::new(),
        }
    }

    /// Returns the underlying `CallbackObject`.
    pub fn callback_holder(&self) -> &CallbackObject<TH> {
        &self.object
    }

    /// Initialize the callback function with a value.
    /// Should be called once this object is done moving.
    pub unsafe fn init(&mut self, cx: *mut JSContext, callback: *mut JSObject) {
        self.object.init(cx, callback);
    }

    /// Returns the property with the given `name`, if it is a callable object,
    /// or an error otherwise.
    pub fn get_callable_property(&self, cx: *mut JSContext, name: &str) -> Fallible<JSVal> {
        rooted!(in(cx) let mut callable = UndefinedValue());
        rooted!(in(cx) let obj = self.callback_holder().get());
        unsafe {
            let c_name = CString::new(name).unwrap();
            if !JS_GetProperty(cx, obj.handle(), c_name.as_ptr(), callable.handle_mut()) {
                return Err(Error::JSFailed);
            }

            if !callable.is_object() || !IsCallable(callable.to_object()) {
                return Err(Error::Type(format!("The value of the {} property is not callable",
                                               name)));
            }
        }
        Ok(callable.get())
    }
}


/// Wraps the reflector for `p` into the compartment of `cx`.
pub fn wrap_call_this_object<T: DomObject>(cx: *mut JSContext,
                                           p: &T,
                                           mut rval: MutableHandleObject) {
    rval.set(p.reflector().get_jsobject().get());
    assert!(!rval.get().is_null());

    unsafe {
        if !JS_WrapObject(cx, rval) {
            rval.set(ptr::null_mut());
        }
    }
}


/// A class that performs whatever setup we need to safely make a call while
/// this class is on the stack. After `new` returns, the call is safe to make.
pub struct CallSetup<TH: TypeHolderTrait> {
    /// The global for reporting exceptions. This is the global object of the
    /// (possibly wrapped) callback object.
    exception_global: DomRoot<GlobalScope<TH>>,
    /// The `JSContext` used for the call.
    cx: *mut JSContext,
    /// The compartment we were in before the call.
    old_compartment: *mut JSCompartment,
    /// The exception handling used for the call.
    handling: ExceptionHandling,
    /// <https://heycam.github.io/webidl/#es-invoking-callback-functions>
    /// steps 8 and 18.2.
    entry_script: Option<AutoEntryScript<TH>>,
    /// <https://heycam.github.io/webidl/#es-invoking-callback-functions>
    /// steps 9 and 18.1.
    incumbent_script: Option<AutoIncumbentScript<TH>>,
}

impl<TH: TypeHolderTrait> CallSetup<TH> {
    /// Performs the setup needed to make a call.
    #[allow(unrooted_must_root)]
    pub fn new<T: CallbackContainer<TH>>(callback: &T,
                                     handling: ExceptionHandling)
                                     -> CallSetup<TH> {
        let global = unsafe { GlobalScope::<TH>::from_object(callback.callback()) };
        let cx = global.get_cx();

        let aes = AutoEntryScript::new(&global);
        let ais = callback.incumbent().map(AutoIncumbentScript::new);
        CallSetup {
            exception_global: global,
            cx: cx,
            old_compartment: unsafe { JS_EnterCompartment(cx, callback.callback()) },
            handling: handling,
            entry_script: Some(aes),
            incumbent_script: ais,
        }
    }

    /// Returns the `JSContext` used for the call.
    pub fn get_context(&self) -> *mut JSContext {
        self.cx
    }
}

impl<TH: TypeHolderTrait> Drop for CallSetup<TH> {
    fn drop(&mut self) {
        unsafe {
            JS_LeaveCompartment(self.cx, self.old_compartment);
            if self.handling == ExceptionHandling::Report {
                let _ac = JSAutoCompartment::new(self.cx,
                                                 self.exception_global.reflector().get_jsobject().get());
                report_pending_exception::<TH>(self.cx, true);
            }
            drop(self.incumbent_script.take());
            drop(self.entry_script.take().unwrap());
        }
    }
}
