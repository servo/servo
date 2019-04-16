/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! FFI implementations for types listed in ServoBoxedTypeList.h.

use crate::gecko_bindings::sugar::ownership::{HasBoxFFI, HasFFI, HasSimpleFFI};
use to_shmem::SharedMemoryBuilder;

// TODO(heycam): The FFI impls for most of the types in ServoBoxedTypeList.h are spread across
// various files at the moment, but should probably all move here, and use macros to define
// them more succinctly, like we do in arc_types.rs.

#[cfg(feature = "gecko")]
unsafe impl HasFFI for SharedMemoryBuilder {
    type FFIType = crate::gecko_bindings::bindings::RawServoSharedMemoryBuilder;
}
#[cfg(feature = "gecko")]
unsafe impl HasSimpleFFI for SharedMemoryBuilder {}
#[cfg(feature = "gecko")]
unsafe impl HasBoxFFI for SharedMemoryBuilder {}
