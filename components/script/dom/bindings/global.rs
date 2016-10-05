/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Abstractions for global scopes.
//!
//! This module contains smart pointers to global scopes, to simplify writing
//! code that works in workers as well as window scopes.

use dom::bindings::conversions::root_from_object;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflectable, Reflector};
use dom::globalscope::GlobalScope;
use dom::window;
use dom::workerglobalscope::WorkerGlobalScope;
use js::{JSCLASS_IS_DOMJSCLASS, JSCLASS_IS_GLOBAL};
use js::glue::{IsWrapper, UnwrapObject};
use js::jsapi::{CurrentGlobalOrNull, GetGlobalForObjectCrossCompartment};
use js::jsapi::{JSContext, JSObject, JS_GetClass};
use task_source::file_reading::FileReadingTaskSource;

/// A freely-copyable reference to a rooted global object.
#[derive(Copy, Clone)]
pub enum GlobalRef<'a> {
    /// A reference to a `Window` object.
    Window(&'a window::Window),
    /// A reference to a `WorkerGlobalScope` object.
    Worker(&'a WorkerGlobalScope),
}

/// A stack-based rooted reference to a global object.
pub enum GlobalRoot {
    /// A root for a `Window` object.
    Window(Root<window::Window>),
    /// A root for a `WorkerGlobalScope` object.
    Worker(Root<WorkerGlobalScope>),
}

impl<'a> GlobalRef<'a> {
    /// Returns that `GlobalRef` as a `GlobalScope` referengce.
    pub fn as_global_scope(&self) -> &GlobalScope {
        match *self {
            GlobalRef::Window(window) => window.upcast(),
            GlobalRef::Worker(worker) => worker.upcast(),
        }
    }

    /// Get the `JSContext` for the `JSRuntime` associated with the thread
    /// this global object is on.
    pub fn get_cx(&self) -> *mut JSContext {
        match *self {
            GlobalRef::Window(ref window) => window.get_cx(),
            GlobalRef::Worker(ref worker) => worker.get_cx(),
        }
    }

    /// `ScriptChan` used to send messages to the event loop of this global's
    /// thread.
    pub fn file_reading_task_source(&self) -> FileReadingTaskSource {
        match *self {
            GlobalRef::Window(ref window) => window.file_reading_task_source(),
            GlobalRef::Worker(ref worker) => worker.file_reading_task_source(),
        }
    }
}

impl<'a> Reflectable for GlobalRef<'a> {
    fn reflector(&self) -> &Reflector {
        match *self {
            GlobalRef::Window(ref window) => window.reflector(),
            GlobalRef::Worker(ref worker) => worker.reflector(),
        }
    }
}

impl GlobalRoot {
    /// Obtain a safe reference to the global object that cannot outlive the
    /// lifetime of this root.
    pub fn r(&self) -> GlobalRef {
        match *self {
            GlobalRoot::Window(ref window) => GlobalRef::Window(window.r()),
            GlobalRoot::Worker(ref worker) => GlobalRef::Worker(worker.r()),
        }
    }
}

/// Returns the global scope of the realm that the given DOM object's reflector was created in.
pub fn global_scope_from_reflector<T: Reflectable>(reflector: &T) -> Root<GlobalScope> {
    unsafe { global_scope_from_object(*reflector.reflector().get_jsobject()) }
}

/// Returns the Rust global scope from a JS global object.
unsafe fn global_scope_from_global(global: *mut JSObject) -> Root<GlobalScope> {
    assert!(!global.is_null());
    let clasp = JS_GetClass(global);
    assert!(((*clasp).flags & (JSCLASS_IS_DOMJSCLASS | JSCLASS_IS_GLOBAL)) != 0);
    root_from_object(global).unwrap()
}

/// Returns the Rust global object from a JS global object.
#[allow(unrooted_must_root)]
unsafe fn global_root_from_global(global: *mut JSObject) -> GlobalRoot {
    let global_scope = global_scope_from_global(global);
    if let Some(window) = global_scope.downcast::<window::Window>() {
        return GlobalRoot::Window(Root::from_ref(window));
    }
    if let Some(worker) = Root::downcast(global_scope) {
        return GlobalRoot::Worker(worker);
    }
    panic!("found DOM global that doesn't unwrap to Window or WorkerGlobalScope")
}

/// Returns the global scope of the realm that the given JS object was created in.
pub unsafe fn global_scope_from_object(obj: *mut JSObject) -> Root<GlobalScope> {
    assert!(!obj.is_null());
    let global = GetGlobalForObjectCrossCompartment(obj);
    global_scope_from_global(global)
}

/// Returns the global object of the realm that the given JS object was created in.
#[allow(unrooted_must_root)]
pub unsafe fn global_root_from_object(obj: *mut JSObject) -> GlobalRoot {
    assert!(!obj.is_null());
    let global = GetGlobalForObjectCrossCompartment(obj);
    global_root_from_global(global)
}

/// Returns the global scope for the given JSContext
#[allow(unrooted_must_root)]
pub unsafe fn global_scope_from_context(cx: *mut JSContext) -> Root<GlobalScope> {
    let global = CurrentGlobalOrNull(cx);
    global_scope_from_global(global)
}

/// Returns the global object for the given JSContext
#[allow(unrooted_must_root)]
pub unsafe fn global_root_from_context(cx: *mut JSContext) -> GlobalRoot {
    let global = CurrentGlobalOrNull(cx);
    global_root_from_global(global)
}

/// Returns the global object of the realm that the given JS object was created in,
/// after unwrapping any wrappers.
pub unsafe fn global_scope_from_object_maybe_wrapped(
        mut obj: *mut JSObject)
        -> Root<GlobalScope> {
    if IsWrapper(obj) {
        obj = UnwrapObject(obj, /* stopAtWindowProxy = */ 0);
        assert!(!obj.is_null());
    }
    global_scope_from_object(obj)
}
