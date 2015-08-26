/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Abstractions for global scopes.
//!
//! This module contains smart pointers to global scopes, to simplify writing
//! code that works in workers as well as window scopes.

use devtools_traits::ScriptToDevtoolsControlMsg;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::conversions::native_from_reflector_jsmanaged;
use dom::bindings::js::{JS, Root};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::window::{self, ScriptHelpers};
use dom::workerglobalscope::WorkerGlobalScope;
use script_task::{ScriptChan, ScriptPort, CommonScriptMsg, ScriptTask};

use msg::constellation_msg::{ConstellationChan, PipelineId, WorkerId};
use net_traits::ResourceTask;
use profile_traits::mem;

use ipc_channel::ipc::IpcSender;
use js::jsapi::{GetGlobalForObjectCrossCompartment};
use js::jsapi::{JSContext, JSObject, JS_GetClass, MutableHandleValue};
use js::{JSCLASS_IS_GLOBAL, JSCLASS_IS_DOMJSCLASS};
use url::Url;

use util::mem::HeapSizeOf;

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
#[derive(JSTraceable, HeapSizeOf)]
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

    /// Get a `mem::ProfilerChan` to send messages to the memory profiler task.
    pub fn mem_profiler_chan(&self) -> mem::ProfilerChan {
        match *self {
            GlobalRef::Window(window) => window.mem_profiler_chan(),
            GlobalRef::Worker(worker) => worker.mem_profiler_chan(),
        }
    }

    /// Get a `ConstellationChan` to send messages to the constellation channel when available.
    pub fn constellation_chan(&self) -> ConstellationChan {
        match *self {
            GlobalRef::Window(window) => window.constellation_chan(),
            GlobalRef::Worker(worker) => worker.constellation_chan(),
        }
    }

    /// Get an `IpcSender<ScriptToDevtoolsControlMsg>` to send messages to Devtools
    /// task when available.
    pub fn devtools_chan(&self) -> Option<IpcSender<ScriptToDevtoolsControlMsg>> {
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
                (*loader.resource_task).clone()
            }
            GlobalRef::Worker(ref worker) => worker.resource_task().clone(),
        }
    }

    /// Get the worker's id.
    pub fn get_worker_id(&self) -> Option<WorkerId> {
        match *self {
            GlobalRef::Window(_) => None,
            GlobalRef::Worker(ref worker) => Some(worker.get_worker_id()),
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
    pub fn script_chan(&self) -> Box<ScriptChan + Send> {
        match *self {
            GlobalRef::Window(ref window) => window.script_chan(),
            GlobalRef::Worker(ref worker) => worker.script_chan(),
        }
    }

    /// Create a new sender/receiver pair that can be used to implement an on-demand
    /// event loop. Used for implementing web APIs that require blocking semantics
    /// without resorting to nested event loops.
    pub fn new_script_pair(&self) -> (Box<ScriptChan + Send>, Box<ScriptPort + Send>) {
        match *self {
            GlobalRef::Window(ref window) => window.new_script_pair(),
            GlobalRef::Worker(ref worker) => worker.new_script_pair(),
        }
    }

    /// Process a single event as if it were the next event in the task queue for
    /// this global.
    pub fn process_event(&self, msg: CommonScriptMsg) {
        match *self {
            GlobalRef::Window(_) => ScriptTask::process_event(msg),
            GlobalRef::Worker(ref worker) => worker.process_event(msg),
        }
    }

    /// Evaluate the JS messages on the `RootedValue` of this global
    pub fn evaluate_js_on_global_with_result(&self, code: &str, rval: MutableHandleValue) {
        match *self {
            GlobalRef::Window(window) => window.evaluate_js_on_global_with_result(code, rval),
            GlobalRef::Worker(worker) => worker.evaluate_js_on_global_with_result(code, rval),
        }
    }

    /// Set the `bool` value to indicate whether developer tools has requested
    /// updates from the global
    pub fn set_devtools_wants_updates(&self, send_updates: bool) {
        match *self {
            GlobalRef::Window(window) => window.set_devtools_wants_updates(send_updates),
            GlobalRef::Worker(worker) => worker.set_devtools_wants_updates(send_updates),
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
