/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Reference-counted pointers to flows.
///
/// Eventually, with dynamically sized types in Rust, much of this code will be superfluous.

use flow::Flow;
use flow;

use std::mem;
use std::ptr;
use std::sync::atomics::SeqCst;

#[unsafe_no_drop_flag]
pub struct FlowRef {
    vtable: *const u8,
    ptr: *const u8,
}

impl FlowRef {
    pub fn new(mut flow: Box<Flow>) -> FlowRef {
        unsafe {
            let result = {
                let flow_ref: &mut Flow = flow;
                mem::transmute(flow_ref)
            };
            mem::forget(flow);
            result
        }
    }

    pub fn get<'a>(&'a self) -> &'a Flow {
        unsafe {
            mem::transmute_copy(self)
        }
    }

    pub fn get_mut<'a>(&'a mut self) -> &'a mut Flow {
        unsafe {
            mem::transmute_copy(self)
        }
    }
}

impl Drop for FlowRef {
    fn drop(&mut self) {
        unsafe {
            if self.vtable == ptr::null() {
                return
            }
            if flow::base(self.get()).ref_count().fetch_sub(1, SeqCst) > 1 {
                return
            }
            let flow_ref: FlowRef = mem::replace(self, FlowRef {
                vtable: ptr::null(),
                ptr: ptr::null(),
            });
            drop(mem::transmute::<FlowRef,Box<Flow>>(flow_ref));
            self.vtable = ptr::null();
            self.ptr = ptr::null();
        }
    }
}

impl Clone for FlowRef {
    fn clone(&self) -> FlowRef {
        unsafe {
            drop(flow::base(self.get()).ref_count().fetch_add(1, SeqCst));
            FlowRef {
                vtable: self.vtable,
                ptr: self.ptr,
            }
        }
    }
}

