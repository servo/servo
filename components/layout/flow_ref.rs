/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Reference-counted pointers to flows.
//!
//! Eventually, with dynamically sized types in Rust, much of this code will
//! be superfluous. This design is largely duplicating logic of Arc<T> and
//! Weak<T>; please see comments there for details.


use flow::Flow;
use std::ops::Deref;
use std::sync::{Arc, Weak};

#[derive(Clone,Debug)]
pub struct FlowRef(Arc<Flow>);

impl Deref for FlowRef {
    type Target = Flow;
    fn deref(&self) -> &Flow {
        self.0.deref()
    }
}

impl FlowRef {
    /// `FlowRef`s can only be made available to the traversal code.
    /// See https://github.com/servo/servo/issues/14014 for more details.
    pub fn new(mut r: Arc<Flow>) -> Self {
        // This assertion checks that this `FlowRef` does not alias normal `Arc`s.
        // If that happens, we're in trouble.
        assert!(Arc::get_mut(&mut r).is_some());
        FlowRef(r)
    }
    pub fn get_mut(this: &mut FlowRef) -> Option<&mut Flow> {
        Arc::get_mut(&mut this.0)
    }
    pub fn downgrade(this: &FlowRef) -> WeakFlowRef {
        WeakFlowRef(Arc::downgrade(&this.0))
    }
    pub fn into_arc(mut this: FlowRef) -> Arc<Flow> {
        // This assertion checks that this `FlowRef` does not alias normal `Arc`s.
        // If that happens, we're in trouble.
        assert!(FlowRef::get_mut(&mut this).is_some());
        this.0
    }
    /// WARNING: This should only be used when there is no aliasing:
    /// when the traversal ensures that no other threads accesses the same flow at the same time.
    /// See https://github.com/servo/servo/issues/6503
    /// Use Arc::get_mut instead when possible (e.g. on an Arc that was just created).
    #[allow(unsafe_code)]
    pub fn deref_mut(this: &mut FlowRef) -> &mut Flow {
        let ptr: *const Flow = &*this.0;
        unsafe { &mut *(ptr as *mut Flow) }
    }
}

#[derive(Clone,Debug)]
pub struct WeakFlowRef(Weak<Flow>);

impl WeakFlowRef {
    pub fn upgrade(&self) -> Option<FlowRef> {
        self.0.upgrade().map(FlowRef)
    }
}

