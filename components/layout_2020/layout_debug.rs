/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Supports writing a trace file created during each layout scope
//! that can be viewed by an external tool to make layout debugging easier.

use std::cell::RefCell;
use std::fs;
#[cfg(debug_assertions)]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[cfg(not(debug_assertions))]
use serde::ser::Serializer;
use serde::Serialize;
use serde_json::{to_string, to_value, Value};

use crate::flow::BoxTree;
use crate::fragment_tree::FragmentTree;

thread_local!(static STATE_KEY: RefCell<Option<State>> = const { RefCell::new(None) });

#[cfg(debug_assertions)]
static DEBUG_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct Scope;

#[macro_export]
macro_rules! layout_debug_scope(
    ($($arg:tt)*) => (
        if cfg!(debug_assertions) {
            layout_debug::Scope::new(format!($($arg)*))
        } else {
            layout_debug::Scope
        }
    )
);

#[derive(Serialize)]
struct TreeValues {
    pub box_tree: Value,
    pub fragment_tree: Value,
}

#[derive(Serialize)]
struct ScopeData {
    name: String,
    pre: TreeValues,
    post: TreeValues,
    children: Vec<ScopeData>,
}

impl ScopeData {
    fn new(name: String, box_tree: Value, fragment_tree: Value) -> ScopeData {
        ScopeData {
            name,
            pre: TreeValues {
                box_tree,
                fragment_tree,
            },
            post: TreeValues {
                box_tree: Value::Null,
                fragment_tree: Value::Null,
            },
            children: vec![],
        }
    }
}

struct State {
    fragment_tree: Arc<FragmentTree>,
    box_tree: Arc<BoxTree>,
    scope_stack: Vec<ScopeData>,
}

/// A layout debugging scope. The entire state of the box and fragment trees
/// will be output at the beginning and end of this scope.
impl Scope {
    pub fn new(name: String) -> Scope {
        STATE_KEY.with(|r| {
            if let Some(ref mut state) = *r.borrow_mut() {
                let box_tree = to_value(&state.box_tree).unwrap();
                let fragment_tree = to_value(&state.fragment_tree).unwrap();
                let data = Box::new(ScopeData::new(name.clone(), box_tree, fragment_tree));
                state.scope_stack.push(*data);
            }
        });
        Scope
    }
}

#[cfg(debug_assertions)]
impl Drop for Scope {
    fn drop(&mut self) {
        STATE_KEY.with(|r| {
            if let Some(ref mut state) = *r.borrow_mut() {
                let mut current_scope = state.scope_stack.pop().unwrap();
                current_scope.post = TreeValues {
                    box_tree: to_value(&state.box_tree).unwrap(),
                    fragment_tree: to_value(&state.fragment_tree).unwrap(),
                };
                let previous_scope = state.scope_stack.last_mut().unwrap();
                previous_scope.children.push(current_scope);
            }
        });
    }
}

/// Begin a layout debug trace. If this has not been called,
/// creating debug scopes has no effect.
pub fn begin_trace(box_tree: Arc<BoxTree>, fragment_tree: Arc<FragmentTree>) {
    assert!(STATE_KEY.with(|r| r.borrow().is_none()));

    STATE_KEY.with(|r| {
        let box_tree_value = to_value(&box_tree).unwrap();
        let fragment_tree_value = to_value(&fragment_tree).unwrap();
        let state = State {
            scope_stack: vec![*Box::new(ScopeData::new(
                "root".to_owned(),
                box_tree_value,
                fragment_tree_value,
            ))],
            box_tree,
            fragment_tree,
        };
        *r.borrow_mut() = Some(state);
    });
}

/// End the debug layout trace. This will write the layout
/// trace to disk in the current directory. The output
/// file can then be viewed with an external tool.
pub fn end_trace(generation: u32) {
    let mut thread_state = STATE_KEY.with(|r| r.borrow_mut().take().unwrap());
    assert_eq!(thread_state.scope_stack.len(), 1);
    let mut root_scope = thread_state.scope_stack.pop().unwrap();
    root_scope.post = TreeValues {
        box_tree: to_value(&thread_state.box_tree).unwrap_or(Value::Null),
        fragment_tree: to_value(&thread_state.fragment_tree).unwrap_or(Value::Null),
    };
    let result = to_string(&root_scope).unwrap();
    fs::write(
        format!("layout_trace-{}.json", generation),
        result.as_bytes(),
    )
    .unwrap();
}

#[cfg(not(debug_assertions))]
#[derive(Clone, Debug)]
pub struct DebugId;

#[cfg(debug_assertions)]
#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct DebugId(u16);

#[cfg(not(debug_assertions))]
impl DebugId {
    pub fn new() -> DebugId {
        DebugId
    }
}

#[cfg(debug_assertions)]
impl DebugId {
    pub fn new() -> DebugId {
        DebugId(DEBUG_ID_COUNTER.fetch_add(1, Ordering::SeqCst) as u16)
    }
}

impl Default for DebugId {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(debug_assertions))]
impl Serialize for DebugId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("{:p}", &self))
    }
}
