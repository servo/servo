/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Supports writing a trace file created during each layout scope
//! that can be viewed by an external tool to make layout debugging easier.

#![macro_escape]

use flow_ref::FlowRef;
use serialize::json;
use std::cell::RefCell;
use std::io::File;
use std::sync::atomics::{AtomicUint, SeqCst, INIT_ATOMIC_UINT};

local_data_key!(state_key: RefCell<State>)

static mut DEBUG_ID_COUNTER: AtomicUint = INIT_ATOMIC_UINT;

pub struct Scope;

#[macro_export]
macro_rules! layout_debug_scope(
    ($($arg:tt)*) => (
        if cfg!(not(ndebug)) {
            layout_debug::Scope::new(format!($($arg)*))
        } else {
            layout_debug::Scope
        }
    )
)

#[deriving(Encodable)]
struct ScopeData {
    name: String,
    pre: String,
    post: String,
    children: Vec<Box<ScopeData>>,
}

impl ScopeData {
    fn new(name: String, pre: String) -> ScopeData {
        ScopeData {
            name: name,
            pre: pre,
            post: String::new(),
            children: vec!(),
        }
    }
}

struct State {
    flow_root: FlowRef,
    scope_stack: Vec<Box<ScopeData>>,
}

/// A layout debugging scope. The entire state of the flow tree
/// will be output at the beginning and end of this scope.
impl Scope {
    pub fn new(name: String) -> Scope {
        let maybe_refcell = state_key.get();
        match maybe_refcell {
            Some(refcell) => {
                let mut state = refcell.borrow_mut();
                let flow_trace = json::encode(&state.flow_root.get());
                let data = box ScopeData::new(name, flow_trace);
                state.scope_stack.push(data);
            }
            None => {}
        }
        Scope
    }
}

#[cfg(not(ndebug))]
impl Drop for Scope {
    fn drop(&mut self) {
        let maybe_refcell = state_key.get();
        match maybe_refcell {
            Some(refcell) => {
                let mut state = refcell.borrow_mut();
                let mut current_scope = state.scope_stack.pop().unwrap();
                current_scope.post = json::encode(&state.flow_root.get());
                let previous_scope = state.scope_stack.last_mut().unwrap();
                previous_scope.children.push(current_scope);
            }
            None => {}
        }
    }
}

/// Generate a unique ID. This is used for items such as Fragment
/// which are often reallocated but represent essentially the
/// same data.
pub fn generate_unique_debug_id() -> uint {
    unsafe { DEBUG_ID_COUNTER.fetch_add(1, SeqCst) }
}

/// Begin a layout debug trace. If this has not been called,
/// creating debug scopes has no effect.
pub fn begin_trace(flow_root: FlowRef) {
    assert!(state_key.get().is_none());

    let flow_trace = json::encode(&flow_root.get());
    let state = State {
        scope_stack: vec![box ScopeData::new("root".to_string(), flow_trace)],
        flow_root: flow_root,
    };
    state_key.replace(Some(RefCell::new(state)));
}

/// End the debug layout trace. This will write the layout
/// trace to disk in the current directory. The output
/// file can then be viewed with an external tool.
pub fn end_trace() {
    let task_state_cell = state_key.replace(None).unwrap();
    let mut task_state = task_state_cell.borrow_mut();
    assert!(task_state.scope_stack.len() == 1);
    let mut root_scope = task_state.scope_stack.pop().unwrap();
    root_scope.post = json::encode(&task_state.flow_root.get());

    let result = json::encode(&root_scope);
    let path = Path::new("layout_trace.json");
    let mut file = File::create(&path).unwrap();
    file.write_str(result.as_slice()).unwrap();
}
