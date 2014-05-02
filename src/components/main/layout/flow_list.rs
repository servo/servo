/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A variant of `DList` specialized to store `Flow`s without an extra
//! indirection.

use std::cast;
use std::mem;
use std::ptr;

use layout::flow::{Flow, base, mut_base};

pub type Link = Option<~Flow:Share>;

#[deriving(Clone)]
pub struct Rawlink {
    vtable: *(),
    obj: *mut (),
}

/// Doubly-linked list of Flows.
///
/// The forward links are strong references.
/// The backward links are weak references.
pub struct FlowList {
    length: uint,
    list_head: Link,
    list_tail: Rawlink,
}

/// Double-ended FlowList iterator
pub struct FlowListIterator<'a> {
    head: &'a Link,
    tail: Rawlink,
    nelem: uint,
}

/// Double-ended mutable FlowList iterator
pub struct MutFlowListIterator<'a> {
    list: &'a mut FlowList,
    head: Rawlink,
    tail: Rawlink,
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
    pub fn some(n: &mut Flow) -> Rawlink {
        unsafe { cast::transmute(n) }
    }

    /// Convert the `Rawlink` into an Option value
    fn resolve_immut(&self) -> Option<&Flow> {
        if self.obj.is_null() {
            None
        } else {
            let me: &Flow = unsafe { cast::transmute_copy(self) };
            Some(me)
        }
    }

    pub fn resolve(&mut self) -> Option<&mut Flow> {
        if self.obj.is_null() {
            None
        } else {
            let me: &mut Flow = unsafe { cast::transmute_copy(self) };
            Some(me)
        }
    }

    fn is_none(&self) -> bool {
        self.obj.is_null()
    }

    unsafe fn get<'a>(&'a mut self) -> &'a mut Flow {
        assert!(self.obj.is_not_null());
        cast::transmute_copy(self)
    }
}

/// Set the .prev field on `next`, then return `Some(next)`
fn link_with_prev(mut next: ~Flow:Share, prev: Rawlink) -> Link {
    mut_base(next).prev_sibling = prev;
    Some(next)
}

impl Container for FlowList {
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
        self.list_head.as_ref().map(|head| { let x: &Flow = *head; x })
    }

    /// Provide a mutable reference to the front element, or None if the list is empty
    #[inline]
    pub fn front_mut<'a>(&'a mut self) -> Option<&'a mut Flow> {
        self.list_head.as_mut().map(|head| { let x: &mut Flow = *head; x })
    }

    /// Provide a reference to the back element, or None if the list is empty
    #[inline]
    pub fn back<'a>(&'a self) -> Option<&'a Flow> {
        let tmp = self.list_tail.resolve_immut();
        tmp.as_ref().map(|tail| { let x: &Flow = *tail; x })
    }

    /// Provide a mutable reference to the back element, or None if the list is empty
    #[inline]
    pub fn back_mut<'a>(&'a mut self) -> Option<&'a mut Flow> {
        // Can't use map() due to error:
        // lifetime of `tail` is too short to guarantee its contents can be safely reborrowed
        let tmp = self.list_tail.resolve();
        match tmp {
            None => None,
            Some(tail) => {
                let x: &mut Flow = tail;
                Some(x)
            }
        }
    }

    /// Add an element first in the list
    ///
    /// O(1)
    pub fn push_front(&mut self, mut new_head: ~Flow:Share) {
        match self.list_head {
            None => {
                self.list_tail = Rawlink::some(new_head);
                self.list_head = link_with_prev(new_head, Rawlink::none());
            }
            Some(ref mut head) => {
                mut_base(new_head).prev_sibling = Rawlink::none();
                mut_base(*head).prev_sibling = Rawlink::some(new_head);
                mem::swap(head, &mut new_head);
                mut_base(*head).next_sibling = Some(new_head);
            }
        }
        self.length += 1;
    }

    /// Remove the first element and return it, or None if the list is empty
    ///
    /// O(1)
    pub fn pop_front(&mut self) -> Option<~Flow:Share> {
        self.list_head.take().map(|mut front_node| {
            self.length -= 1;
            match mut_base(front_node).next_sibling.take() {
                Some(node) => self.list_head = link_with_prev(node, Rawlink::none()),
                None => self.list_tail = Rawlink::none()
            }
            front_node
        })
    }

    /// Add an element last in the list
    ///
    /// O(1)
    pub fn push_back(&mut self, mut new_tail: ~Flow:Share) {
        if self.list_tail.is_none() {
            return self.push_front(new_tail);
        } else {
            let mut old_tail = self.list_tail;
            self.list_tail = Rawlink::some(new_tail);
            let tail = unsafe { old_tail.get() };
            mut_base(tail).next_sibling = link_with_prev(new_tail, Rawlink::some(tail));
        }
        self.length += 1;
    }

    /// Remove the last element and return it, or None if the list is empty
    ///
    /// O(1)
    pub fn pop_back(&mut self) -> Option<~Flow:Share> {
        if self.list_tail.is_none() {
            None
        } else {
            self.length -= 1;

            self.list_tail = base(unsafe { self.list_tail.get() }).prev_sibling;
            if self.list_tail.is_none() {
                self.list_head.take()
            } else {
                mut_base(unsafe { self.list_tail.get() }).next_sibling.take()
            }
        }
    }

    /// Create an empty list
    #[inline]
    pub fn new() -> FlowList {
        FlowList {
            list_head: None,
            list_tail: Rawlink::none(),
            length: 0,
        }
    }

    /// Provide a forward iterator
    #[inline]
    pub fn iter<'a>(&'a self) -> FlowListIterator<'a> {
        FlowListIterator {
            nelem: self.len(),
            head: &self.list_head,
            tail: self.list_tail
        }
    }

    /// Provide a forward iterator with mutable references
    #[inline]
    pub fn mut_iter<'a>(&'a mut self) -> MutFlowListIterator<'a> {
        let head_raw = match self.list_head {
            Some(ref mut h) => Rawlink::some(*h),
            None => Rawlink::none(),
        };
        MutFlowListIterator {
            nelem: self.len(),
            head: head_raw,
            tail: self.list_tail,
            list: self
        }
    }
}

#[unsafe_destructor]
impl Drop for FlowList {
    fn drop(&mut self) {
        // Dissolve the list in backwards direction
        // Just dropping the list_head can lead to stack exhaustion
        // when length is >> 1_000_000
        let mut tail = self.list_tail;
        loop {
            match tail.resolve() {
                None => break,
                Some(prev) => {
                    let prev_base = mut_base(prev);
                    prev_base.next_sibling.take();
                    tail = prev_base.prev_sibling;
                }
            }
        }
        self.length = 0;
        self.list_head = None;
        self.list_tail = Rawlink::none();
    }
}

impl<'a> Iterator<&'a Flow> for FlowListIterator<'a> {
    #[inline]
    fn next(&mut self) -> Option<&'a Flow> {
        if self.nelem == 0 {
            return None;
        }
        self.head.as_ref().map(|head| {
            let head_base = base(*head);
            self.nelem -= 1;
            self.head = &head_base.next_sibling;
            let ret: &Flow = *head;
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
        self.head.resolve().map(|next| {
            self.nelem -= 1;
            self.head = match mut_base(next).next_sibling {
                Some(ref mut node) => {
                    let x: &mut Flow = *node;
                    Rawlink::some(x)
                }
                None => Rawlink::none(),
            };
            next
        })
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        (self.nelem, Some(self.nelem))
    }
}
