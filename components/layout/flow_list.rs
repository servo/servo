/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use flow::Flow;
use flow_ref::{self, FlowRef};
use std::collections::{LinkedList, linked_list};

/// This needs to be reworked now that we have dynamically-sized types in Rust.
/// Until then, it's just a wrapper around LinkedList.
///
/// SECURITY-NOTE(pcwalton): It is very important that `FlowRef` values not leak directly to
/// layout. Layout code must only interact with `&Flow` or `&mut Flow` values. Otherwise, layout
/// could stash `FlowRef` values in random places unknown to the system and thereby cause data
/// races. Those data races can lead to memory safety problems, potentially including arbitrary
/// remote code execution! In general, do not add new methods to this file (e.g. new ways of
/// iterating over flows) unless you are *very* sure of what you are doing.
pub struct FlowList {
    flows: LinkedList<FlowRef>,
}

pub struct MutFlowListIterator<'a> {
    it: linked_list::IterMut<'a, FlowRef>,
}

impl FlowList {
    /// Add an element last in the list
    ///
    /// O(1)
    pub fn push_back(&mut self, new_tail: FlowRef) {
        self.flows.push_back(new_tail);
    }

    pub fn back(&self) -> Option<&Flow> {
        self.flows.back().map(|x| &**x)
    }

    /// Add an element first in the list
    ///
    /// O(1)
    pub fn push_front(&mut self, new_head: FlowRef) {
        self.flows.push_front(new_head);
    }

    pub fn pop_front(&mut self) -> Option<FlowRef> {
        self.flows.pop_front()
    }

    pub fn front(&self) -> Option<&Flow> {
        self.flows.front().map(|x| &**x)
    }

    /// Create an empty list
    #[inline]
    pub fn new() -> FlowList {
        FlowList {
            flows: LinkedList::new(),
        }
    }

    /// Provide a forward iterator.
    ///
    /// SECURITY-NOTE(pcwalton): This does not hand out `FlowRef`s by design. Do not add a method
    /// to do so! See the comment above in `FlowList`.
    #[inline]
    pub fn iter<'a>(&'a self) -> impl DoubleEndedIterator<Item = &'a Flow> {
        self.flows.iter().map(|flow| &**flow)
    }

    /// Provide a forward iterator with mutable references
    ///
    /// SECURITY-NOTE(pcwalton): This does not hand out `FlowRef`s by design. Do not add a method
    /// to do so! See the comment above in `FlowList`.
    #[inline]
    pub fn iter_mut(&mut self) -> MutFlowListIterator {
        MutFlowListIterator {
            it: self.flows.iter_mut(),
        }
    }

    /// Provides a caching random-access iterator that yields mutable references. This is
    /// guaranteed to perform no more than O(n) pointer chases.
    ///
    /// SECURITY-NOTE(pcwalton): This does not hand out `FlowRef`s by design. Do not add a method
    /// to do so! See the comment above in `FlowList`.
    #[inline]
    pub fn random_access_mut(&mut self) -> FlowListRandomAccessMut {
        let length = self.flows.len();
        FlowListRandomAccessMut {
            iterator: self.flows.iter_mut(),
            cache: Vec::with_capacity(length),
        }
    }

    /// O(1)
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.flows.is_empty()
    }

    /// O(1)
    #[inline]
    pub fn len(&self) -> usize {
        self.flows.len()
    }

    #[inline]
    pub fn split_off(&mut self, i: usize) -> Self {
        FlowList {
            flows: self.flows.split_off(i)
        }
    }
}

impl<'a> DoubleEndedIterator for MutFlowListIterator<'a> {
    fn next_back(&mut self) -> Option<&'a mut Flow> {
        self.it.next_back().map(flow_ref::deref_mut)
    }
}

impl<'a> Iterator for MutFlowListIterator<'a> {
    type Item = &'a mut Flow;
    #[inline]
    fn next(&mut self) -> Option<&'a mut Flow> {
        self.it.next().map(flow_ref::deref_mut)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }
}

/// A caching random-access iterator that yields mutable references. This is guaranteed to perform
/// no more than O(n) pointer chases.
pub struct FlowListRandomAccessMut<'a> {
    iterator: linked_list::IterMut<'a, FlowRef>,
    cache: Vec<FlowRef>,
}

impl<'a> FlowListRandomAccessMut<'a> {
    pub fn get<'b>(&'b mut self, index: usize) -> &'b mut Flow {
        while index >= self.cache.len() {
            match self.iterator.next() {
                None => panic!("Flow index out of range!"),
                Some(next_flow) => self.cache.push((*next_flow).clone()),
            }
        }
        flow_ref::deref_mut(&mut self.cache[index])
    }
}
