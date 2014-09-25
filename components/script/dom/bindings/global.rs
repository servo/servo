/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Abstractions for global scopes.
//!
//! This module contains smart pointers to global scopes, to simplify writing
//! code that works in workers as well as window scopes.

use dom::bindings::conversions::FromJSValConvertible;
use dom::bindings::js::{JS, JSRef, Root};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::workerglobalscope::WorkerGlobalScope;
use dom::window;
use script_task::ScriptChan;

use servo_net::resource_task::ResourceTask;

use js::{JSCLASS_IS_GLOBAL, JSCLASS_IS_DOMJSCLASS};
use js::glue::{GetGlobalForObjectCrossCompartment};
use js::jsapi::{JSContext, JSObject};
use js::jsapi::{JS_GetClass};
use js::jsval::ObjectOrNullValue;
use url::Url;

use std::ptr;

/// A freely-copyable reference to a rooted global object.
pub enum GlobalRef<'a> {
    Window(JSRef<'a, window::Window>),
    Worker(JSRef<'a, WorkerGlobalScope>),
}

/// A stack-based rooted reference to a global object.
pub enum GlobalRoot<'a, 'b> {
    WindowRoot(Root<'a, 'b, window::Window>),
    WorkerRoot(Root<'a, 'b, WorkerGlobalScope>),
}

/// A traced reference to a global object, for use in fields of traced Rust
/// structures.
#[jstraceable]
#[must_root]
pub enum GlobalField {
    WindowField(JS<window::Window>),
    WorkerField(JS<WorkerGlobalScope>),
}

impl<'a> GlobalRef<'a> {
    /// Get the `JSContext` for the `JSRuntime` associated with the thread
    /// this global object is on.
    pub fn get_cx(&self) -> *mut JSContext {
        match *self {
            Window(ref window) => window.get_cx(),
            Worker(ref worker) => worker.get_cx(),
        }
    }

    /// Extract a `Window`, causing task failure if the global object is not
    /// a `Window`.
    pub fn as_window<'b>(&'b self) -> JSRef<'b, window::Window> {
        match *self {
            Window(window) => window,
            Worker(_) => fail!("expected a Window scope"),
        }
    }

    pub fn resource_task(&self) -> ResourceTask {
        match *self {
            Window(ref window) => window.page().resource_task.deref().clone(),
            Worker(ref worker) => worker.resource_task().clone(),
        }
    }

    pub fn get_url(&self) -> Url {
        match *self {
            Window(ref window) => window.get_url(),
            Worker(ref worker) => worker.get_url().clone(),
        }
    }

    /// `ScriptChan` used to send messages to the event loop of this global's
    /// thread.
    pub fn script_chan<'b>(&'b self) -> &'b ScriptChan {
        match *self {
            Window(ref window) => &window.script_chan,
            Worker(ref worker) => worker.script_chan(),
        }
    }
}

impl<'a> Reflectable for GlobalRef<'a> {
    fn reflector<'b>(&'b self) -> &'b Reflector {
        match *self {
            Window(ref window) => window.reflector(),
            Worker(ref worker) => worker.reflector(),
        }
    }
}

impl<'a, 'b> GlobalRoot<'a, 'b> {
    /// Obtain a safe reference to the global object that cannot outlive the
    /// lifetime of this root.
    pub fn root_ref<'c>(&'c self) -> GlobalRef<'c> {
        match *self {
            WindowRoot(ref window) => Window(window.root_ref()),
            WorkerRoot(ref worker) => Worker(worker.root_ref()),
        }
    }
}

impl GlobalField {
    /// Create a new `GlobalField` from a rooted reference.
    pub fn from_rooted(global: &GlobalRef) -> GlobalField {
        match *global {
            Window(window) => WindowField(JS::from_rooted(window)),
            Worker(worker) => WorkerField(JS::from_rooted(worker)),
        }
    }

    /// Create a stack-bounded root for this reference.
    pub fn root(&self) -> GlobalRoot {
        match *self {
            WindowField(ref window) => WindowRoot(window.root()),
            WorkerField(ref worker) => WorkerRoot(worker.root()),
        }
    }
}

/// Returns the global object of the realm that the given JS object was created in.
#[allow(unrooted_must_root)]
pub fn global_object_for_js_object(obj: *mut JSObject) -> GlobalField {
    unsafe {
        let global = GetGlobalForObjectCrossCompartment(obj);
        let clasp = JS_GetClass(global);
        assert!(((*clasp).flags & (JSCLASS_IS_DOMJSCLASS | JSCLASS_IS_GLOBAL)) != 0);
        match FromJSValConvertible::from_jsval(ptr::null_mut(), ObjectOrNullValue(global), ()) {
            Ok(window) => return WindowField(window),
            Err(_) => (),
        }

        match FromJSValConvertible::from_jsval(ptr::null_mut(), ObjectOrNullValue(global), ()) {
            Ok(worker) => return WorkerField(worker),
            Err(_) => (),
        }

        fail!("found DOM global that doesn't unwrap to Window or WorkerGlobalScope")
    }
}
