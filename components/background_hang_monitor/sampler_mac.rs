/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{panic, process};

use crate::sampler::{Address, NativeStack, Registers, Sampler};

type MonitoredThreadId = mach2::mach_types::thread_act_t;

pub struct MacOsSampler {
    thread_id: MonitoredThreadId,
}

impl MacOsSampler {
    #[allow(unsafe_code)]
    pub fn new_boxed() -> Box<dyn Sampler> {
        let thread_id = unsafe { mach2::mach_init::mach_thread_self() };
        Box::new(MacOsSampler { thread_id })
    }
}

impl Sampler for MacOsSampler {
    #[allow(unsafe_code)]
    fn suspend_and_sample_thread(&self) -> Result<NativeStack, ()> {
        // Warning: The "critical section" begins here.
        // In the critical section:
        // we must not do any dynamic memory allocation,
        // nor try to acquire any lock
        // or any other unshareable resource.
        let current_hook = panic::take_hook();
        panic::set_hook(Box::new(|_| {
            // Avoiding any allocation or locking as part of standard panicking.
            process::abort();
        }));
        let native_stack = unsafe {
            if let Err(()) = suspend_thread(self.thread_id) {
                panic::set_hook(current_hook);
                return Err(());
            };
            let native_stack = match get_registers(self.thread_id) {
                Ok(regs) => Ok(frame_pointer_stack_walk(regs)),
                Err(()) => Err(()),
            };
            if let Err(()) = resume_thread(self.thread_id) {
                process::abort();
            }
            native_stack
        };
        panic::set_hook(current_hook);
        // NOTE: End of "critical section".
        native_stack
    }
}

fn check_kern_return(kret: mach2::kern_return::kern_return_t) -> Result<(), ()> {
    if kret != mach2::kern_return::KERN_SUCCESS {
        return Err(());
    }
    Ok(())
}

#[allow(unsafe_code)]
unsafe fn suspend_thread(thread_id: MonitoredThreadId) -> Result<(), ()> {
    check_kern_return(unsafe { mach2::thread_act::thread_suspend(thread_id) })
}

#[allow(unsafe_code)]
unsafe fn get_registers(thread_id: MonitoredThreadId) -> Result<Registers, ()> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let mut state = mach2::structs::x86_thread_state64_t::new();
        let mut state_count = mach2::structs::x86_thread_state64_t::count();
        let kret = unsafe {
            mach2::thread_act::thread_get_state(
                thread_id,
                mach2::thread_status::x86_THREAD_STATE64,
                (&mut state) as *mut _ as *mut _,
                &mut state_count,
            )
        };
        check_kern_return(kret)?;
        Ok(Registers {
            instruction_ptr: state.__rip as Address,
            stack_ptr: state.__rsp as Address,
            frame_ptr: state.__rbp as Address,
        })
    }
    #[cfg(target_arch = "aarch64")]
    {
        let mut state = mach2::structs::arm_thread_state64_t::new();
        let mut state_count = mach2::structs::arm_thread_state64_t::count();
        let kret = unsafe {
            mach2::thread_act::thread_get_state(
                thread_id,
                mach2::thread_status::ARM_THREAD_STATE64,
                (&mut state) as *mut _ as *mut _,
                &mut state_count,
            )
        };
        check_kern_return(kret)?;
        Ok(Registers {
            instruction_ptr: state.__pc as Address,
            stack_ptr: state.__sp as Address,
            frame_ptr: state.__fp as Address,
        })
    }
}
#[allow(unsafe_code)]
unsafe fn resume_thread(thread_id: MonitoredThreadId) -> Result<(), ()> {
    check_kern_return(unsafe { mach2::thread_act::thread_resume(thread_id) })
}

#[allow(unsafe_code)]
unsafe fn frame_pointer_stack_walk(regs: Registers) -> NativeStack {
    // Note: this function will only work with code build with:
    // --dev,
    // or --with-frame-pointer.
    let mut native_stack = NativeStack::new();
    unsafe {
        let pthread_t = libc::pthread_self();
        let stackaddr = libc::pthread_get_stackaddr_np(pthread_t);
        let stacksize = libc::pthread_get_stacksize_np(pthread_t);
        let pc = regs.instruction_ptr as *mut std::ffi::c_void;
        let stack = regs.stack_ptr as *mut std::ffi::c_void;
        let _ = native_stack.process_register(pc, stack);
        let mut current = regs.frame_ptr as *mut *mut std::ffi::c_void;
        while !current.is_null() {
            if (current as usize) < stackaddr as usize {
                // Reached the end of the stack.
                break;
            }
            if current as usize >= stackaddr.add(stacksize * 8) as usize {
                // Reached the beginning of the stack.
                // Assumining 64 bit mac(see the stacksize * 8).
                break;
            }
            let next = *current as *mut *mut std::ffi::c_void;
            let pc = current.add(1);
            let stack = current.add(2);
            if let Err(()) = native_stack.process_register(*pc, *stack) {
                break;
            }
            if (next <= current) || (next as usize & 3 != 0) {
                break;
            }
            current = next;
        }
    }
    native_stack
}
