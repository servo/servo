/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Supports writing a trace file created during each layout scope
//! that can be viewed by an external tool to make layout debugging easier.

#![macro_escape]

use flow_ref::FlowRef;
use flow;
use serialize::json;

use std::borrow::ToOwned;
use std::cell::RefCell;
use std::io::File;
use std::sync::atomic::{AtomicUint, Ordering, ATOMIC_UINT_INIT};

thread_local!(static STATE_KEY: RefCell<Option<State>> = RefCell::new(None));

static mut DEBUG_ID_COUNTER: AtomicUint = ATOMIC_UINT_INIT;

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
);

#[derive(RustcEncodable)]
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
        STATE_KEY.with(|ref r| {
            match &mut *r.borrow_mut() {
                &mut Some(ref mut state) => {
                    let flow_trace = json::encode(&flow::base(&*state.flow_root));
                    let data = box ScopeData::new(name.clone(), flow_trace);
                    state.scope_stack.push(data);
                }
                &mut None => {}
            }
        });
        Scope
    }
}

#[cfg(not(ndebug))]
impl Drop for Scope {
    fn drop(&mut self) {
        STATE_KEY.with(|ref r| {
            match &mut *r.borrow_mut() {
                &mut Some(ref mut state) => {
                    let mut current_scope = state.scope_stack.pop().unwrap();
                    current_scope.post = json::encode(&flow::base(&*state.flow_root));
                    let previous_scope = state.scope_stack.last_mut().unwrap();
                    previous_scope.children.push(current_scope);
                }
                &mut None => {}
            }
        });
    }
}

/// Generate a unique ID. This is used for items such as Fragment
/// which are often reallocated but represent essentially the
/// same data.
pub fn generate_unique_debug_id() -> u16 {
    unsafe { DEBUG_ID_COUNTER.fetch_add(1, Ordering::SeqCst) as u16 }
}

/// Begin a layout debug trace. If this has not been called,
/// creating debug scopes has no effect.
pub fn begin_trace(flow_root: FlowRef) {
    assert!(STATE_KEY.with(|ref r| r.borrow().is_none()));

    STATE_KEY.with(|ref r| {
        let flow_trace = json::encode(&flow::base(&*flow_root));
        let state = State {
            scope_stack: vec![box ScopeData::new("root".to_owned(), flow_trace)],
            flow_root: flow_root.clone(),
        };
        *r.borrow_mut() = Some(state);
    });
}

/// End the debug layout trace. This will write the layout
/// trace to disk in the current directory. The output
/// file can then be viewed with an external tool.
pub fn end_trace() {
    let mut task_state = STATE_KEY.with(|ref r| r.borrow_mut().take().unwrap());
    assert!(task_state.scope_stack.len() == 1);
    let mut root_scope = task_state.scope_stack.pop().unwrap();
    root_scope.post = json::encode(&flow::base(&*task_state.flow_root));

    let result = json::encode(&root_scope);
    let path = Path::new("layout_trace.json");
    let mut file = File::create(&path).unwrap();
    file.write_str(result.as_slice()).unwrap();
}
