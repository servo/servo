/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::os::raw::c_void;

use js::glue::JS_GetReservedSlot;
use js::jsapi::JSObject;
use js::jsval::UndefinedValue;
use js::rust::get_object_class;
use malloc_size_of::MallocSizeOfOps;

use crate::codegen::Globals::Globals;
use crate::codegen::InheritTypes::TopTypeId;
use crate::codegen::PrototypeList::{self, MAX_PROTO_CHAIN_LENGTH, PROTO_OR_IFACE_LENGTH};

/// The struct that holds inheritance information for DOM object reflectors.
#[derive(Clone, Copy)]
pub struct DOMClass {
    /// A list of interfaces that this object implements, in order of decreasing
    /// derivedness.
    pub interface_chain: [PrototypeList::ID; MAX_PROTO_CHAIN_LENGTH],

    /// The last valid index of `interface_chain`.
    pub depth: u8,

    /// The type ID of that interface.
    pub type_id: TopTypeId,

    /// The MallocSizeOf function wrapper for that interface.
    pub malloc_size_of: unsafe fn(ops: &mut MallocSizeOfOps, *const c_void) -> usize,

    /// The `Globals` flag for this global interface, if any.
    pub global: Globals,
}
unsafe impl Sync for DOMClass {}

/// The JSClass used for DOM object reflectors.
#[derive(Copy)]
#[repr(C)]
pub struct DOMJSClass {
    /// The actual JSClass.
    pub base: js::jsapi::JSClass,
    /// Associated data for DOM object reflectors.
    pub dom_class: DOMClass,
}
impl Clone for DOMJSClass {
    fn clone(&self) -> DOMJSClass {
        *self
    }
}
unsafe impl Sync for DOMJSClass {}

/// The index of the slot where the object holder of that interface's
/// unforgeable members are defined.
pub const DOM_PROTO_UNFORGEABLE_HOLDER_SLOT: u32 = 0;

/// The index of the slot that contains a reference to the ProtoOrIfaceArray.
// All DOM globals must have a slot at DOM_PROTOTYPE_SLOT.
pub const DOM_PROTOTYPE_SLOT: u32 = js::JSCLASS_GLOBAL_SLOT_COUNT;

/// The flag set on the `JSClass`es for DOM global objects.
// NOTE: This is baked into the Ion JIT as 0 in codegen for LGetDOMProperty and
// LSetDOMProperty. Those constants need to be changed accordingly if this value
// changes.
pub const JSCLASS_DOM_GLOBAL: u32 = js::JSCLASS_USERBIT1;

/// Returns the ProtoOrIfaceArray for the given global object.
/// Fails if `global` is not a DOM global object.
pub fn get_proto_or_iface_array(global: *mut JSObject) -> *mut ProtoOrIfaceArray {
    unsafe {
        assert_ne!(((*get_object_class(global)).flags & JSCLASS_DOM_GLOBAL), 0);
        let mut slot = UndefinedValue();
        JS_GetReservedSlot(global, DOM_PROTOTYPE_SLOT, &mut slot);
        slot.to_private() as *mut ProtoOrIfaceArray
    }
}

/// An array of *mut JSObject of size PROTO_OR_IFACE_LENGTH.
pub type ProtoOrIfaceArray = [*mut JSObject; PROTO_OR_IFACE_LENGTH];
