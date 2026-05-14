/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

extern crate servo as servo_api;

// cbindgen:opaque
#[derive(Default)]
pub struct ServoBuilder {}

/// Creates a new `ServoBuilder` populated with default values.
///
/// The returned builder must be freed with `servo_builder_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_builder_create() -> *mut ServoBuilder {
    Box::into_raw(Box::new(ServoBuilder::default()))
}

/// Destroys `servo_builder` and frees its memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_builder_free(builder: *mut ServoBuilder) {
    assert!(!builder.is_null(), "builder pointer must not be null");
    unsafe {
        let _ = Box::from_raw(builder);
    }
}
