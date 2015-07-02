/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Abstractions for global scopes.
//!
//! This module contains smart pointers to global scopes, to simplify writing
//! code that works in workers as well as window scopes.

use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::conversions::native_from_reflector_jsmanaged;
use dom::bindings::js::{JS, Root};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::DocumentHelpers;
use dom::workerglobalscope::{WorkerGlobalScope, WorkerGlobalScopeHelpers};
use dom::window::{self, WindowHelpers};
use devtools_traits::DevtoolsControlChan;
use script_task::{ScriptChan, ScriptPort, ScriptMsg, ScriptTask};

use msg::constellation_msg::{PipelineId, WorkerId};
use net_traits::ResourceTask;

use js::{JSCLASS_IS_GLOBAL, JSCLASS_IS_DOMJSCLASS};
use js::jsapi::{GetGlobalForObjectCrossCompartment};
use js::jsapi::{JSContext, JSObject};
use js::jsapi::{JS_GetClass};
use url::Url;

/// A freely-copyable reference to a rooted global object.
#[derive(Copy, Clone)]
pub enum GlobalRef<'a> {
    /// A reference to a `Window` object.
    Window(&'a window::Window),
    /// A reference to a `WorkerGlobalScope` object.
    Worker(&'a WorkerGlobalScope),
}

/// A stack-based rooted reference to a global object.
#[no_move]
pub enum GlobalRoot {
    /// A root for a `Window` object.
    Window(Root<window::Window>),
    /// A root for a `WorkerGlobalScope` object.
    Worker(Root<WorkerGlobalScope>),
}

/// A traced reference to a global object, for use in fields of traced Rust
/// structures.
#[derive(JSTraceable)]
#[must_root]
pub enum GlobalField {
    /// A field for a `Window` object.
    Window(JS<window::Window>),
    /// A field for a `WorkerGlobalScope` object.
    Worker(JS<WorkerGlobalScope>),
}

impl<'a> GlobalRef<'a> {
    /// Get the `JSContext` for the `JSRuntime` associated with the thread
    /// this global object is on.
    pub fn get_cx(&self) -> *mut JSContext {
        match *self {
            GlobalRef::Window(ref window) => window.get_cx(),
            GlobalRef::Worker(ref worker) => worker.get_cx(),
        }
    }

    /// Extract a `Window`, causing task failure if the global object is not
    /// a `Window`.
    pub fn as_window<'b>(&'b self) -> &'b window::Window {
        match *self {
            GlobalRef::Window(window) => window,
            GlobalRef::Worker(_) => panic!("expected a Window scope"),
        }
    }

    /// Get the `PipelineId` for this global scope.
    pub fn pipeline(&self) -> PipelineId {
        match *self {
            GlobalRef::Window(window) => window.pipeline(),
            GlobalRef::Worker(worker) => worker.pipeline(),
        }
    }

    /// Get `DevtoolsControlChan` to send messages to Devtools
    /// task when available.
    pub fn devtools_chan(&self) -> Option<DevtoolsControlChan> {
        match *self {
            GlobalRef::Window(window) => window.devtools_chan(),
            GlobalRef::Worker(worker) => worker.devtools_chan(),
        }
    }

    /// Get the `ResourceTask` for this global scope.
    pub fn resource_task(&self) -> ResourceTask {
        match *self {
            GlobalRef::Window(ref window) => {
                let doc = window.Document();
                let doc = doc.r();
                let loader = doc.loader();
                loader.resource_task.clone()
            }
            GlobalRef::Worker(ref worker) => worker.resource_task().clone(),
        }
    }

    /// Get next worker id.
    pub fn get_next_worker_id(&self) -> WorkerId {
        match *self {
            GlobalRef::Window(ref window) => window.get_next_worker_id(),
            GlobalRef::Worker(ref worker) => worker.get_next_worker_id()
        }
    }

    /// Get the URL for this global scope.
    pub fn get_url(&self) -> Url {
        match *self {
            GlobalRef::Window(ref window) => window.get_url(),
            GlobalRef::Worker(ref worker) => worker.get_url().clone(),
        }
    }

    /// `ScriptChan` used to send messages to the event loop of this global's
    /// thread.
    pub fn script_chan(&self) -> Box<ScriptChan+Send> {
        match *self {
            GlobalRef::Window(ref window) => window.script_chan(),
            GlobalRef::Worker(ref worker) => worker.script_chan(),
        }
    }

    /// Create a new sender/receiver pair that can be used to implement an on-demand
    /// event loop. Used for implementing web APIs that require blocking semantics
    /// without resorting to nested event loops.
    pub fn new_script_pair(&self) -> (Box<ScriptChan+Send>, Box<ScriptPort+Send>) {
        match *self {
            GlobalRef::Window(ref window) => window.new_script_pair(),
            GlobalRef::Worker(ref worker) => worker.new_script_pair(),
        }
    }

    /// Process a single event as if it were the next event in the task queue for
    /// this global.
    pub fn process_event(&self, msg: ScriptMsg) {
        match *self {
            GlobalRef::Window(_) => ScriptTask::process_event(msg),
            GlobalRef::Worker(ref worker) => worker.process_event(msg),
        }
    }
}

impl<'a> Reflectable for GlobalRef<'a> {
    fn reflector<'b>(&'b self) -> &'b Reflector {
        match *self {
            GlobalRef::Window(ref window) => window.reflector(),
            GlobalRef::Worker(ref worker) => worker.reflector(),
        }
    }
}

impl GlobalRoot {
    /// Obtain a safe reference to the global object that cannot outlive the
    /// lifetime of this root.
    pub fn r<'c>(&'c self) -> GlobalRef<'c> {
        match *self {
            GlobalRoot::Window(ref window) => GlobalRef::Window(window.r()),
            GlobalRoot::Worker(ref worker) => GlobalRef::Worker(worker.r()),
        }
    }
}

impl GlobalField {
    /// Create a new `GlobalField` from a rooted reference.
    pub fn from_rooted(global: &GlobalRef) -> GlobalField {
        match *global {
            GlobalRef::Window(window) => GlobalField::Window(JS::from_ref(window)),
            GlobalRef::Worker(worker) => GlobalField::Worker(JS::from_ref(worker)),
        }
    }

    /// Create a stack-bounded root for this reference.
    pub fn root(&self) -> GlobalRoot {
        match *self {
            GlobalField::Window(ref window) => GlobalRoot::Window(window.root()),
            GlobalField::Worker(ref worker) => GlobalRoot::Worker(worker.root()),
        }
    }
}

/// Returns the global object of the realm that the given JS object was created in.
#[allow(unrooted_must_root)]
pub fn global_object_for_js_object(obj: *mut JSObject) -> GlobalRoot {
    unsafe {
        let global = GetGlobalForObjectCrossCompartment(obj);
        let clasp = JS_GetClass(global);
        assert!(((*clasp).flags & (JSCLASS_IS_DOMJSCLASS | JSCLASS_IS_GLOBAL)) != 0);
        match native_from_reflector_jsmanaged(global) {
            Ok(window) => return GlobalRoot::Window(window),
            Err(_) => (),
        }

        match native_from_reflector_jsmanaged(global) {
            Ok(worker) => return GlobalRoot::Worker(worker),
            Err(_) => (),
        }

        panic!("found DOM global that doesn't unwrap to Window or WorkerGlobalScope")
    }
}
