/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A persistent, thread-safe singly-linked list.

use std::mem;
use std::sync::Arc;

pub struct PersistentList<T> {
    head: PersistentListLink<T>,
    length: usize,
}

struct PersistentListEntry<T> {
    value: T,
    next: PersistentListLink<T>,
}

type PersistentListLink<T> = Option<Arc<PersistentListEntry<T>>>;

impl<T> PersistentList<T> where T: Send + Sync {
    #[inline]
    pub fn new() -> PersistentList<T> {
        PersistentList {
            head: None,
            length: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.length
    }

    #[inline]
    pub fn front(&self) -> Option<&T> {
        self.head.as_ref().map(|head| &head.value)
    }

    #[inline]
    pub fn prepend_elem(&self, value: T) -> PersistentList<T> {
        PersistentList {
            head: Some(Arc::new(PersistentListEntry {
                value: value,
                next: self.head.clone(),
            })),
            length: self.length + 1,
        }
    }

    #[inline]
    pub fn iter<'a>(&'a self) -> PersistentListIterator<'a, T> {
        // This could clone (and would not need the lifetime if it did), but then it would incur
        // atomic operations on every call to `.next()`. Bad.
        PersistentListIterator {
            entry: self.head.as_ref().map(|head| &**head),
        }
    }
}

impl<T> Clone for PersistentList<T> where T: Send + Sync {
    fn clone(&self) -> PersistentList<T> {
        // This establishes the persistent nature of this list: we can clone a list by just cloning
        // its head.
        PersistentList {
            head: self.head.clone(),
            length: self.length,
        }
    }
}

pub struct PersistentListIterator<'a,T> where T: 'a + Send + Sync {
    entry: Option<&'a PersistentListEntry<T>>,
}

impl<'a, T> Iterator for PersistentListIterator<'a, T> where T: Send + Sync + 'static {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        let entry = match self.entry {
            None => return None,
            Some(entry) => {
                // This `transmute` is necessary to ensure that the lifetimes of the next entry and
                // this entry match up; the compiler doesn't know this, but we do because of the
                // reference counting behavior of `Arc`.
                unsafe {
                    mem::transmute::<&'a PersistentListEntry<T>,
                                     &'static PersistentListEntry<T>>(entry)
                }
            }
        };
        let value = &entry.value;
        self.entry = match entry.next {
            None => None,
            Some(ref entry) => Some(&**entry),
        };
        Some(value)
    }
}

