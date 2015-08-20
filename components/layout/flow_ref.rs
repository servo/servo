/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Reference-counted pointers to flows.
//!
//! Eventually, with dynamically sized types in Rust, much of this code will
//! be superfluous. This design is largely duplicating logic of Arc<T> and
//! Weak<T>; please see comments there for details.

#![allow(unsafe_code)]

use flow;
use flow::{Flow, BaseFlow};

use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::raw;
use std::rt::heap;
use std::sync::atomic::{self, Ordering};

#[unsafe_no_drop_flag]
pub struct FlowRef {
    object: raw::TraitObject,
}

unsafe impl Send for FlowRef {}
unsafe impl Sync for FlowRef {}

#[unsafe_no_drop_flag]
pub struct WeakFlowRef {
    object: raw::TraitObject,
}

unsafe impl Send for WeakFlowRef {}
unsafe impl Sync for WeakFlowRef {}

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

    /// Downgrades the FlowRef to a WeakFlowRef.
    pub fn downgrade(&self) -> WeakFlowRef {
        unsafe {
            flow::base(&**self).weak_ref_count().fetch_add(1, Ordering::Relaxed);
        }
        WeakFlowRef { object: self.object }
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

impl DerefMut for FlowRef {
    fn deref_mut<'a>(&mut self) -> &mut (Flow + 'a) {
        unsafe {
            mem::transmute_copy::<raw::TraitObject, &mut (Flow + 'a)>(&self.object)
        }
    }
}

impl Drop for FlowRef {
    fn drop(&mut self) {
        unsafe {
            if self.object.vtable.is_null() ||
               self.object.vtable as usize == mem::POST_DROP_USIZE {
                return
            }
            if flow::base(&**self).strong_ref_count().fetch_sub(1, Ordering::Release) != 1 {
                return
            }
            atomic::fence(Ordering::Acquire);

            // Normally we'd call the underlying Drop logic but not free the
            // allocation, but this is almost impossible without DST in
            // Rust. Instead we make a fake trait object to run the drop logic
            // on.
            let flow_ref: FlowRef = mem::replace(self, FlowRef {
                object: raw::TraitObject {
                    vtable: ptr::null_mut(),
                    data: ptr::null_mut(),
                }
            });

            let vtable: &[usize; 3] = mem::transmute::<*mut (), &[usize; 3]>(flow_ref.object.vtable);
            let object_size = vtable[1];
            let object_align = vtable[2];

            let fake_data = heap::allocate(object_size, object_align);
            ptr::copy(flow_ref.object.data as *const u8, fake_data, object_size);

            let fake_box = raw::TraitObject { vtable: flow_ref.object.vtable, data: fake_data as *mut () };
            let fake_flow = mem::transmute::<raw::TraitObject, Box<Flow>>(fake_box);
            drop(fake_flow);

            if flow::base(&*flow_ref).weak_ref_count().fetch_sub(1, Ordering::Release) == 1 {
                atomic::fence(Ordering::Acquire);
                heap::deallocate(flow_ref.object.data as *mut u8, object_size, object_align);
            }

            mem::forget(flow_ref);
        }
    }
}

impl Clone for FlowRef {
    fn clone(&self) -> FlowRef {
        unsafe {
            let _ = flow::base(&**self).strong_ref_count().fetch_add(1, Ordering::Relaxed);
            FlowRef {
                object: raw::TraitObject {
                    vtable: self.object.vtable,
                    data: self.object.data,
                }
            }
        }
    }
}

fn base<'a>(r: &WeakFlowRef) -> &'a BaseFlow {
    let data = r.object.data;
    debug_assert!(!data.is_null());
    unsafe {
        mem::transmute::<*mut (), &'a BaseFlow>(data)
    }
}

impl WeakFlowRef {
    /// Upgrades a WeakFlowRef to a FlowRef.
    pub fn upgrade(&self) -> Option<FlowRef> {
        unsafe {
            let object = base(self);
            // We use a CAS loop to increment the strong count instead of a
            // fetch_add because once the count hits 0 is must never be above
            // 0.
            loop {
                let n = object.strong_ref_count().load(Ordering::SeqCst);
                if n == 0 { return None }
                let old = object.strong_ref_count().compare_and_swap(n, n + 1, Ordering::SeqCst);
                if old == n {
                    return Some(FlowRef { object: self.object })
                }
            }
        }
    }
}

impl Clone for WeakFlowRef {
    fn clone(&self) -> WeakFlowRef {
        unsafe {
            base(self).weak_ref_count().fetch_add(1, Ordering::Relaxed);
        }
        WeakFlowRef { object: self. object }
    }
}

impl Drop for WeakFlowRef {
    fn drop(&mut self) {
        unsafe {
            if self.object.vtable.is_null() ||
               self.object.vtable as usize == mem::POST_DROP_USIZE {
                return
            }

            if base(self).weak_ref_count().fetch_sub(1, Ordering::Release) == 1 {
                atomic::fence(Ordering::Acquire);

                // This dance deallocates the Box<Flow> without running its
                // drop glue. The drop glue is run when the last strong
                // reference is released.
                let weak_ref: WeakFlowRef = mem::replace(self, WeakFlowRef {
                    object: raw::TraitObject {
                        vtable: ptr::null_mut(),
                        data: ptr::null_mut(),
                    }
                });
                let vtable: &[usize; 3] = mem::transmute::<*mut (), &[usize; 3]>(weak_ref.object.vtable);
                let object_size = vtable[1];
                let object_align = vtable[2];
                heap::deallocate(weak_ref.object.data as *mut u8, object_size, object_align);
                mem::forget(weak_ref);
            }
        }
    }
}
