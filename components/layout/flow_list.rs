/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use flow::Flow;
use flow_ref::FlowRef;

use std::collections::{dlist, DList};

// This needs to be reworked now that we have dynamically-sized types in Rust.
// Until then, it's just a wrapper around DList.

pub struct FlowList {
    flows: DList<FlowRef>,
}

pub struct FlowListIterator<'a> {
    it: dlist::Iter<'a, FlowRef>,
}

pub struct MutFlowListIterator<'a> {
    it: dlist::IterMut<'a, FlowRef>,
}

impl FlowList {
    /// Provide a reference to the front element, or None if the list is empty
    #[inline]
    pub fn front<'a>(&'a self) -> Option<&'a Flow> {
        self.flows.front().map(|head| &**head)
    }

    /// Provide a mutable reference to the front element, or None if the list is empty
    #[inline]
    pub unsafe fn front_mut<'a>(&'a mut self) -> Option<&'a mut Flow> {
        self.flows.front_mut().map(|head| &mut **head)
    }

    /// Provide a reference to the back element, or None if the list is empty
    #[inline]
    pub fn back<'a>(&'a self) -> Option<&'a Flow> {
        self.flows.back().map(|tail| &**tail)
    }

    /// Provide a mutable reference to the back element, or None if the list is empty
    #[inline]
    pub unsafe fn back_mut<'a>(&'a mut self) -> Option<&'a mut Flow> {
        self.flows.back_mut().map(|tail| &mut **tail)
    }

    /// Add an element first in the list
    ///
    /// O(1)
    pub fn push_front(&mut self, new_head: FlowRef) {
        self.flows.push_front(new_head);
    }

    /// Remove the first element and return it, or None if the list is empty
    ///
    /// O(1)
    pub fn pop_front(&mut self) -> Option<FlowRef> {
        self.flows.pop_front()
    }

    /// Add an element last in the list
    ///
    /// O(1)
    pub fn push_back(&mut self, new_tail: FlowRef) {
        self.flows.push_back(new_tail);
    }

    /// Create an empty list
    #[inline]
    pub fn new() -> FlowList {
        FlowList {
            flows: DList::new(),
        }
    }

    /// Provide a forward iterator
    #[inline]
    pub fn iter<'a>(&'a self) -> FlowListIterator<'a> {
        FlowListIterator {
            it: self.flows.iter(),
        }
    }

    /// Provide a forward iterator with mutable references
    #[inline]
    pub fn iter_mut<'a>(&'a mut self) -> MutFlowListIterator<'a> {
        MutFlowListIterator {
            it: self.flows.iter_mut(),
        }
    }

    /// O(1)
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.flows.is_empty()
    }

    /// O(1)
    #[inline]
    pub fn len(&self) -> uint {
        self.flows.len()
    }
}

impl<'a> Iterator for FlowListIterator<'a> {
    type Item = &'a (Flow + 'a);
    #[inline]
    fn next(&mut self) -> Option<&'a (Flow + 'a)> {
        self.it.next().map(|x| &**x)
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        self.it.size_hint()
    }
}

impl<'a> Iterator for MutFlowListIterator<'a> {
    type Item = &'a mut (Flow + 'a);
    #[inline]
    fn next(&mut self) -> Option<&'a mut (Flow + 'a)> {
        self.it.next().map(|x| &mut **x)
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        self.it.size_hint()
    }
}
