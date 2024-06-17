/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr;

use background_hang_monitor_api::{HangProfile, HangProfileSymbol};

const MAX_NATIVE_FRAMES: usize = 1024;

pub trait Sampler: Send {
    fn suspend_and_sample_thread(&self) -> Result<NativeStack, ()>;
}

#[allow(dead_code)]
pub struct DummySampler;

impl DummySampler {
    #[allow(dead_code)]
    pub fn new_boxed() -> Box<dyn Sampler> {
        Box::new(DummySampler)
    }
}

impl Sampler for DummySampler {
    fn suspend_and_sample_thread(&self) -> Result<NativeStack, ()> {
        Err(())
    }
}

// Several types in this file are currently not used in a Linux or Windows build.
#[allow(dead_code)]
pub type Address = *const u8;

/// The registers used for stack unwinding
#[allow(dead_code)]
pub struct Registers {
    /// Instruction pointer.
    pub instruction_ptr: Address,
    /// Stack pointer.
    pub stack_ptr: Address,
    /// Frame pointer.
    pub frame_ptr: Address,
}

#[allow(dead_code)]
pub struct NativeStack {
    instruction_ptrs: [*mut std::ffi::c_void; MAX_NATIVE_FRAMES],
    #[allow(dead_code)]
    stack_ptrs: [*mut std::ffi::c_void; MAX_NATIVE_FRAMES],
    #[allow(dead_code)]
    count: usize,
}

impl NativeStack {
    #[allow(dead_code)]
    pub fn new() -> Self {
        NativeStack {
            instruction_ptrs: [ptr::null_mut(); MAX_NATIVE_FRAMES],
            stack_ptrs: [ptr::null_mut(); MAX_NATIVE_FRAMES],
            count: 0,
        }
    }

    #[allow(dead_code)]
    pub fn process_register(
        &mut self,
        instruction_ptr: *mut std::ffi::c_void,
        stack_ptr: *mut std::ffi::c_void,
    ) -> Result<(), ()> {
        if self.count >= MAX_NATIVE_FRAMES {
            return Err(());
        }
        self.instruction_ptrs[self.count] = instruction_ptr;
        self.stack_ptrs[self.count] = stack_ptr;
        self.count += 1;
        Ok(())
    }

    pub fn to_hangprofile(&self) -> HangProfile {
        let mut profile = HangProfile {
            backtrace: Vec::new(),
        };
        for ip in self.instruction_ptrs.iter().rev() {
            if ip.is_null() {
                continue;
            }
            backtrace::resolve(*ip, |symbol| {
                // TODO: use the demangled or C++ demangled symbols if available.
                let name = symbol
                    .name()
                    .map(|n| String::from_utf8_lossy(n.as_bytes()).to_string());
                let filename = symbol.filename().map(|n| n.to_string_lossy().to_string());
                let lineno = symbol.lineno();
                profile.backtrace.push(HangProfileSymbol {
                    name,
                    filename,
                    lineno,
                });
            });
        }
        profile
    }
}
