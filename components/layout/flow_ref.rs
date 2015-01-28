/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Reference-counted pointers to flows.
///
/// Eventually, with dynamically sized types in Rust, much of this code will be superfluous.

use flow::Flow;
use flow;

use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::raw;
use std::sync::atomic::Ordering;

#[unsafe_no_drop_flag]
pub struct FlowRef {
    object: raw::TraitObject,
}

unsafe impl Send for FlowRef {}
unsafe impl Sync for FlowRef {}

impl FlowRef {
    pub fn new(mut flow: Box<Flow>) -> FlowRef {
        unsafe {
            let result = {
                let flow_ref: &mut Flow = &mut *flow;
                let object = mem::transmute::<&mut Flow, raw::TraitObject>(flow_ref);
                FlowRef { object: object }
            };
            mem::forget(flow);
            result
        }
    }
}

impl<'a> Deref for FlowRef {
    type Target = Flow + 'a;
    fn deref(&self) -> &(Flow + 'a) {
        unsafe {
            mem::transmute_copy::<raw::TraitObject, &(Flow + 'a)>(&self.object)
        }
    }
}

impl<'a> DerefMut for FlowRef {
    fn deref_mut<'a>(&mut self) -> &mut (Flow + 'a) {
        unsafe {
            mem::transmute_copy::<raw::TraitObject, &mut (Flow + 'a)>(&self.object)
        }
    }
}

impl Drop for FlowRef {
    fn drop(&mut self) {
        unsafe {
            if self.object.vtable.is_null() {
                return
            }
            if flow::base(&**self).ref_count().fetch_sub(1, Ordering::SeqCst) > 1 {
                return
            }
            let flow_ref: FlowRef = mem::replace(self, FlowRef {
                object: raw::TraitObject {
                    vtable: ptr::null_mut(),
                    data: ptr::null_mut(),
                }
            });
            drop(mem::transmute::<raw::TraitObject, Box<Flow>>(flow_ref.object));
            mem::forget(flow_ref);
            self.object.vtable = ptr::null_mut();
            self.object.data = ptr::null_mut();
        }
    }
}

impl Clone for FlowRef {
    fn clone(&self) -> FlowRef {
        unsafe {
            drop(flow::base(&**self).ref_count().fetch_add(1, Ordering::SeqCst));
            FlowRef {
                object: raw::TraitObject {
                    vtable: self.object.vtable,
                    data: self.object.data,
                }
            }
        }
    }
}
