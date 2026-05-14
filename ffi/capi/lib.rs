/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

extern crate servo as servo_api;

// cbindgen:opaque
#[derive(Default)]
pub struct ServoBuilder {
    event_loop_waker: ServoEventLoopWaker,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ServoEventLoopWaker {
    wake_callback: extern "C" fn(),
}

pub extern "C" fn default_event_loop_waker_wake_callback() {}

impl Default for ServoEventLoopWaker {
    fn default() -> Self {
        Self {
            wake_callback: default_event_loop_waker_wake_callback,
        }
    }
}

impl servo_api::EventLoopWaker for ServoEventLoopWaker {
    fn clone_box(&self) -> Box<dyn servo_api::EventLoopWaker> {
        Box::new(*self)
    }

    fn wake(&self) {
        (self.wake_callback)()
    }
}

/// Creates a new `ServoBuilder` populated with default values.
///
/// The returned builder must be freed with `servo_builder_free` or
/// consumed by `servo_builder_build`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_builder_create() -> *mut ServoBuilder {
    Box::into_raw(Box::new(ServoBuilder::default()))
}

/// Sets the callback used to wake the embedder's event loop when Servo
/// has new work to process (e.g, rendering updates).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_builder_set_event_loop_waker(
    builder: *mut ServoBuilder,
    event_loop_waker: ServoEventLoopWaker,
) {
    assert!(!builder.is_null(), "builder pointer must not be null");
    assert!(!builder.is_null(), "builder pointer must not be null");
    unsafe {
        (*builder).event_loop_waker = event_loop_waker;
    }
}

/// Consumes the builder and creates a new `Servo` instance.
///
/// The returned `Servo` handle must be freed with `servo_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_builder_build(builder: *mut ServoBuilder) -> *mut servo_api::Servo {
    assert!(!builder.is_null(), "builder pointer must not be null");
    let mut rust_builder = servo_api::ServoBuilder::default();

    // SAFETY: `builder` is non-null, as verified above.
    let builder = unsafe { &mut *builder };

    rust_builder = rust_builder.event_loop_waker(Box::new(builder.event_loop_waker));

    let servo = rust_builder.build();
    Box::into_raw(Box::new(servo))
}

/// Destroys `servo_builder` and frees its memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_builder_free(builder: *mut ServoBuilder) {
    assert!(!builder.is_null(), "builder pointer must not be null");
    unsafe {
        let _ = Box::from_raw(builder);
    }
}
