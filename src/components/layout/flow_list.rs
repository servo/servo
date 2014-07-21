/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A variant of `DList` specialized to store `Flow`s without an extra
//! indirection.

use flow::{Flow, base, mut_base};
use flow_ref::FlowRef;

use std::mem;
use std::ptr;

pub type Link = Option<FlowRef>;

#[allow(raw_pointer_deriving)]
#[deriving(Clone)]
pub struct Rawlink {
    vtable: *const (),
    obj: *mut (),
}

/// Doubly-linked list of Flows.
///
/// The forward links are strong references.
/// The backward links are weak references.
pub struct FlowList {
    length: uint,
    list_head: Link,
    list_tail: Link,
}

/// Double-ended FlowList iterator
pub struct FlowListIterator<'a> {
    head: &'a Link,
    nelem: uint,
}

/// Double-ended mutable FlowList iterator
pub struct MutFlowListIterator<'a> {
    _list: &'a mut FlowList,
    head: Rawlink,
    nelem: uint,
}

impl Rawlink {
    /// Like Option::None for Rawlink
    pub fn none() -> Rawlink {
        Rawlink {
            vtable: ptr::null(),
            obj: ptr::mut_null(),
        }
    }

    /// Like Option::Some for Rawlink
    pub fn some(n: &Flow) -> Rawlink {
        unsafe { mem::transmute(n) }
    }

    pub unsafe fn resolve_mut(&self) -> Option<&mut Flow> {
        if self.obj.is_null() {
            None
        } else {
            let me: &mut Flow = mem::transmute_copy(self);
            Some(me)
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
    /// O(1)
    #[inline]
    fn len(&self) -> uint {
        self.length
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

    /// Add an element first in the list
    ///
    /// O(1)
    pub fn push_front(&mut self, mut new_head: FlowRef) {
        unsafe {
            match self.list_head {
                None => {
                    self.list_tail = Some(new_head.clone());
                    self.list_head = link_with_prev(new_head, None);
                }
                Some(ref mut head) => {
                    mut_base(new_head.get_mut()).prev_sibling = None;
                    mut_base(head.get_mut()).prev_sibling = Some(new_head.clone());
                    mem::swap(head, &mut new_head);
                    mut_base(head.get_mut()).next_sibling = Some(new_head);
                }
            }
            self.length += 1;
        }
    }

    /// Remove the first element and return it, or None if the list is empty
    ///
    /// O(1)
    pub fn pop_front(&mut self) -> Option<FlowRef> {
        self.list_head.take().map(|mut front_node| {
            self.length -= 1;
            unsafe {
                match mut_base(front_node.get_mut()).next_sibling.take() {
                    Some(node) => self.list_head = link_with_prev(node, None),
                    None => self.list_tail = None,
                }
            }
            front_node
        })
    }

    /// Add an element last in the list
    ///
    /// O(1)
    pub fn push_back(&mut self, new_tail: FlowRef) {
        if self.list_tail.is_none() {
            return self.push_front(new_tail);
        }

        let old_tail = self.list_tail.clone();
        self.list_tail = Some(new_tail.clone());
        let mut tail = (*old_tail.as_ref().unwrap()).clone();
        let tail_clone = Some(tail.clone());
        unsafe {
            mut_base(tail.get_mut()).next_sibling = link_with_prev(new_tail, tail_clone);
        }
        self.length += 1;
    }

    /// Create an empty list
    #[inline]
    pub fn new() -> FlowList {
        FlowList {
            list_head: None,
            list_tail: None,
            length: 0,
        }
    }

    /// Provide a forward iterator
    #[inline]
    pub fn iter<'a>(&'a self) -> FlowListIterator<'a> {
        FlowListIterator {
            nelem: self.len(),
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
            nelem: self.len(),
            head: head_raw,
            _list: self
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
        self.length = 0;
        self.list_head = None;
    }
}

impl<'a> Iterator<&'a Flow> for FlowListIterator<'a> {
    #[inline]
    fn next(&mut self) -> Option<&'a Flow> {
        if self.nelem == 0 {
            return None;
        }
        self.head.as_ref().map(|head| {
            let head_base = base(head.get());
            self.nelem -= 1;
            self.head = &head_base.next_sibling;
            let ret: &Flow = head.get();
            ret
        })
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        (self.nelem, Some(self.nelem))
    }
}

impl<'a> Iterator<&'a mut Flow> for MutFlowListIterator<'a> {
    #[inline]
    fn next(&mut self) -> Option<&'a mut Flow> {
        if self.nelem == 0 {
            return None;
        }
        unsafe {
            self.head.resolve_mut().map(|next| {
                self.nelem -= 1;
                self.head = match mut_base(next).next_sibling {
                    Some(ref mut node) => {
                        let x: &mut Flow = node.get_mut();
                        Rawlink::some(x)
                    }
                    None => Rawlink::none(),
                };
                next
            })
        }
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        (self.nelem, Some(self.nelem))
    }
}
