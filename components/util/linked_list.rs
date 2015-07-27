/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Utility functions for doubly-linked lists.

use mem::HeapSizeOf;

use serde::de::{Error, SeqVisitor, Visitor};
use serde::ser::impls::SeqIteratorVisitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::LinkedList;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};

pub struct SerializableLinkedList<T>(LinkedList<T>);

impl<T> SerializableLinkedList<T> {
    pub fn new(linked_list: LinkedList<T>) -> SerializableLinkedList<T> {
        SerializableLinkedList(linked_list)
    }
}

impl<T> Deref for SerializableLinkedList<T> {
    type Target = LinkedList<T>;

    fn deref(&self) -> &LinkedList<T> {
        &self.0
    }
}

impl<T> DerefMut for SerializableLinkedList<T> {
    fn deref_mut(&mut self) -> &mut LinkedList<T> {
        &mut self.0
    }
}

impl<T: HeapSizeOf> HeapSizeOf for SerializableLinkedList<T> {
    fn heap_size_of_children(&self) -> usize {
        self.0.heap_size_of_children()
    }
}

impl<T> Serialize for SerializableLinkedList<T> where T: Serialize {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        serializer.visit_seq(SeqIteratorVisitor::new(self.0.iter(), Some(self.0.len())))
    }
}

impl<T> Deserialize for SerializableLinkedList<T> where T: Deserialize {
    fn deserialize<D>(deserializer: &mut D) -> Result<SerializableLinkedList<T>, D::Error>
                      where D: Deserializer {
        struct SerializableLinkedListVisitor<T> {
            marker: PhantomData<T>,
        }

        impl<T> Visitor for SerializableLinkedListVisitor<T> where T: Deserialize {
            type Value = SerializableLinkedList<T>;

            #[inline]
            fn visit_seq<V>(&mut self, mut visitor: V)
                            -> Result<SerializableLinkedList<T>, V::Error>
                            where V: SeqVisitor {
                let mut list = LinkedList::new();
                for _ in 0..visitor.size_hint().0 {
                    match try!(visitor.visit()) {
                        Some(element) => list.push_back(element),
                        None => return Err(Error::end_of_stream_error()),
                    }
                }
                try!(visitor.end());
                Ok(SerializableLinkedList(list))
            }
        }

        deserializer.visit_seq(SerializableLinkedListVisitor {
            marker: PhantomData,
        })
    }
}

/// Splits the head off a list in O(1) time, and returns the head.
pub fn split_off_head<T>(list: &mut LinkedList<T>) -> LinkedList<T> {
    let tail = list.split_off(1);
    mem::replace(list, tail)
}

/// Prepends the items in the other list to this one, leaving the other list empty.
#[inline]
pub fn prepend_from<T>(this: &mut LinkedList<T>, other: &mut LinkedList<T>) {
    other.append(this);
    mem::swap(this, other);
}
