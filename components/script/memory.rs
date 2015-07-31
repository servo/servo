/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Routines for handling measuring the memory usage of arbitrary DOM nodes.

use dom::bindings::codegen::InheritTypes::{DocumentCast, WindowCast, CharacterDataCast, NodeCast};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::node::NodeTypeId;
use libc;
use util::mem::{HeapSizeOf, heap_size_of};

pub trait MemoryMeasurable {
    fn heap_size_of_self_and_children(&self) -> usize;
}

impl<T: HeapSizeOf> MemoryMeasurable for T {
    fn heap_size_of_self_and_children(&self) -> usize {
        heap_size_of(self as *const T as *const libc::c_void) + self.heap_size_of_children()
    }
}

pub fn measure_memory_for_eventtarget(target: &EventTarget) -> usize {
    //TODO: add more specific matches for concrete element types as derive(HeapSizeOf) is
    //      added to each one.
    match target.type_id() {
        &EventTargetTypeId::Window =>
            WindowCast::to_ref(target).unwrap().heap_size_of_self_and_children(),
        &EventTargetTypeId::Node(NodeTypeId::CharacterData(_)) =>
            CharacterDataCast::to_ref(target).unwrap().heap_size_of_self_and_children(),
        &EventTargetTypeId::Node(NodeTypeId::Document) =>
            DocumentCast::to_ref(target).unwrap().heap_size_of_self_and_children(),
        &EventTargetTypeId::Node(_) =>
            NodeCast::to_ref(target).unwrap().heap_size_of_self_and_children(),
        _ => 0,
    }
}
