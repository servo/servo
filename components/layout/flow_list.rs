/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A variant of `DList` specialized to store `Flow`s without an extra
//! indirection.

use flow::{Flow, base, mut_base};
use flow_ref::FlowRef;

use std::kinds::marker::ContravariantLifetime;
use std::mem;
use std::ptr;
use std::raw;

pub type Link = Option<FlowRef>;


#[allow(raw_pointer_deriving)]
pub struct Rawlink<'a> {
    object: raw::TraitObject,
    marker: ContravariantLifetime<'a>,
}

/// Doubly-linked list of Flows.
///
/// The forward links are strong references.
/// The backward links are weak references.
pub struct FlowList {
    list_head: Link,
    list_tail: Link,
}

/// Double-ended FlowList iterator
pub struct FlowListIterator<'a> {
    head: &'a Link,
}

/// Double-ended mutable FlowList iterator
pub struct MutFlowListIterator<'a> {
    head: Rawlink<'a>,
}

impl<'a> Rawlink<'a> {
    /// Like Option::None for Rawlink
    pub fn none() -> Rawlink<'static> {
        Rawlink {
            object: raw::TraitObject {
                vtable: ptr::mut_null(),
                data: ptr::mut_null(),
            },
            marker: ContravariantLifetime,
        }
    }

    /// Like Option::Some for Rawlink
    pub fn some(n: &Flow) -> Rawlink {
        unsafe {
            Rawlink {
                object: mem::transmute::<&Flow, raw::TraitObject>(n),
                marker: ContravariantLifetime,
            }
        }
    }

    pub unsafe fn resolve_mut(&self) -> Option<&'a mut Flow> {
        if self.object.data.is_null() {
            None
        } else {
            Some(mem::transmute_copy::<raw::TraitObject, &mut Flow>(&self.object))
        }
    }
}

/// Set the .prev field on `next`, then return `Some(next)`
unsafe fn link_with_prev(mut next: FlowRef, prev: Option<FlowRef>) -> Link {
    mut_base(next.get_mut()).prev_sibling = prev;
    Some(next)
}

impl Collection for FlowList {
    /// O(1)
    #[inline]
    fn is_empty(&self) -> bool {
        self.list_head.is_none()
    }

    /// O(n)
    #[inline]
    fn len(&self) -> uint {
        let mut length = 0;
        for _ in self.iter() {
            length += 1
        }
        length
    }
}

// This doesn't quite fit the Deque trait because of the need to switch between
// &Flow and ~Flow.
impl FlowList {
    /// Provide a reference to the front element, or None if the list is empty
    #[inline]
    pub fn front<'a>(&'a self) -> Option<&'a Flow> {
        self.list_head.as_ref().map(|head| head.get())
    }

    /// Provide a mutable reference to the front element, or None if the list is empty
    #[inline]
    pub unsafe fn front_mut<'a>(&'a mut self) -> Option<&'a mut Flow> {
        self.list_head.as_mut().map(|head| head.get_mut())
    }

    /// Provide a reference to the back element, or None if the list is empty
    #[inline]
    pub fn back<'a>(&'a self) -> Option<&'a Flow> {
        match self.list_tail {
            None => None,
            Some(ref list_tail) => Some(list_tail.get())
        }
    }

    /// Provide a mutable reference to the back element, or None if the list is empty
    #[inline]
    pub unsafe fn back_mut<'a>(&'a mut self) -> Option<&'a mut Flow> {
        // Can't use map() due to error:
        // lifetime of `tail` is too short to guarantee its contents can be safely reborrowed
        match self.list_tail {
            None => None,
            Some(ref mut tail) => {
                let x: &mut Flow = tail.get_mut();
                Some(mem::transmute_copy(&x))
            }
        }
    }

    /// Create an empty list
    #[inline]
    pub fn new() -> FlowList {
        FlowList {
            list_head: None,
            list_tail: None,
        }
    }

    /// Provide a forward iterator
    #[inline]
    pub fn iter<'a>(&'a self) -> FlowListIterator<'a> {
        FlowListIterator {
            head: &self.list_head,
        }
    }

    /// Provide a forward iterator with mutable references
    #[inline]
    pub fn mut_iter<'a>(&'a mut self) -> MutFlowListIterator<'a> {
        let head_raw = match self.list_head {
            Some(ref mut h) => Rawlink::some(h.get()),
            None => Rawlink::none(),
        };
        MutFlowListIterator {
            head: head_raw,
        }
    }
}

pub trait TreeMutationMethods {
    /// Add an element last in the list. O(1).
    ///
    /// NB: Only flow construction may safely call this!
    fn push_back(&mut self, new_tail: FlowRef);
}

impl TreeMutationMethods for FlowRef {
    fn push_back(&mut self, mut new_tail: FlowRef) {
        remove_flow_from_parent(new_tail.get_mut());

        let this = self.clone();
        let base = mut_base(self.get_mut());
        if base.children.list_tail.is_none() {
            unsafe {
                mut_base(new_tail.get_mut()).set_parent(Some(this));
            }
            base.children.list_head = Some(new_tail.clone());
            base.children.list_tail = Some(new_tail);
            return
        }

        let old_tail = base.children.list_tail.clone();
        base.children.list_tail = Some(new_tail.clone());
        let mut tail = (*old_tail.as_ref().unwrap()).clone();
        let tail_clone = Some(tail.clone());
        unsafe {
            let tail_base = mut_base(tail.get_mut());
            mut_base(new_tail.get_mut()).set_parent(Some(this));
            tail_base.next_sibling = link_with_prev(new_tail, tail_clone);
        }
    }
}

#[unsafe_destructor]
impl Drop for FlowList {
    fn drop(&mut self) {
        // Dissolve the list in backwards direction
        // Just dropping the list_head can lead to stack exhaustion
        // when length is >> 1_000_000
        let mut tail = mem::replace(&mut self.list_tail, None);
        loop {
            let new_tail = match tail {
                None => break,
                Some(ref mut prev) => {
                    let prev_base = mut_base(prev.get_mut());
                    prev_base.next_sibling.take();
                    prev_base.prev_sibling.clone()
                }
            };
            tail = new_tail
        }
        self.list_head = None;
    }
}

impl<'a> Iterator<&'a Flow> for FlowListIterator<'a> {
    #[inline]
    fn next(&mut self) -> Option<&'a Flow> {
        self.head.as_ref().map(|head| {
            let head_base = base(head.get());
            self.head = &head_base.next_sibling;
            let ret: &Flow = head.get();
            ret
        })
    }
}

impl<'a> Iterator<&'a mut Flow> for MutFlowListIterator<'a> {
    #[inline]
    fn next(&mut self) -> Option<&'a mut Flow> {
        unsafe {
            self.head.resolve_mut().map(|next| {
                self.head = match mut_base(next).next_sibling {
                    Some(ref mut node) => {
                        let x: &mut Flow = node.get_mut();
                        // NOTE: transmute needed here to break the link
                        // between x and next so that it is no longer
                        // borrowed.
                        mem::transmute(Rawlink::some(x))
                    }
                    None => Rawlink::none(),
                };
                next
            })
        }
    }
}

/// Unlinks a flow from its container, if it's in one.
///
/// NB: Do not make this public!
///
/// FIXME(pcwalton): This should taint the containing flow. It's not safe to perform parallel
/// layout on it since its atomic counters may be messed up.
fn remove_flow_from_parent(flow: &mut Flow) {
    let flow_base = mut_base(flow);
    let prev_sibling = flow_base.prev_sibling.clone();
    let mut prev_sibling_2 = flow_base.prev_sibling.clone();
    let mut next_sibling = flow_base.next_sibling.clone();
    unsafe {
        let flow_parent = flow_base.parent();
        let flow_parent = match *flow_parent {
            None => {
                return
            }
            Some(ref mut parent) => parent,
        };
        match next_sibling {
            None => {
                mut_base(flow_parent.get_mut()).children.list_tail = prev_sibling
            }
            Some(ref mut next_sibling) => {
                mut_base(next_sibling.get_mut()).prev_sibling = prev_sibling
            }
        }
        match prev_sibling_2 {
            None => {
                mut_base(flow_parent.get_mut()).children.list_head = next_sibling
            }
            Some(ref mut prev_sibling) => {
                mut_base(prev_sibling.get_mut()).next_sibling = next_sibling
            }
        }
    }

    unsafe {
        flow_base.next_sibling = None;
        flow_base.prev_sibling = None;
        flow_base.set_parent(None);
    }
}

