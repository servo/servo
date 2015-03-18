/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Utility functions for doubly-linked lists.

use std::collections::LinkedList;
use std::mem;

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
