/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use crate::sampler::{NativeStack, Sampler};
use libc;
use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal};
use std::cell::UnsafeCell;
use std::io;
use std::mem;
use std::process;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::thread;
use unwind_sys::{
    unw_cursor_t, unw_get_reg, unw_init_local, unw_step, UNW_ESUCCESS, UNW_REG_IP, UNW_REG_SP,
};

static mut SHARED_STATE: SharedState = SharedState {
    msg2: None,
    msg3: None,
    msg4: None,
    context: AtomicPtr::new(ptr::null_mut()),
};

type MonitoredThreadId = libc::pid_t;

struct SharedState {
    // "msg1" is the signal.
    msg2: Option<PosixSemaphore>,
    msg3: Option<PosixSemaphore>,
    msg4: Option<PosixSemaphore>,
    context: AtomicPtr<libc::ucontext_t>,
}

fn clear_shared_state() {
    unsafe {
        SHARED_STATE.msg2 = None;
        SHARED_STATE.msg3 = None;
        SHARED_STATE.msg4 = None;
        SHARED_STATE.context = AtomicPtr::new(ptr::null_mut());
    }
}

fn reset_shared_state() {
    unsafe {
        SHARED_STATE.msg2 = Some(PosixSemaphore::new(0).expect("valid semaphore"));
        SHARED_STATE.msg3 = Some(PosixSemaphore::new(0).expect("valid semaphore"));
        SHARED_STATE.msg4 = Some(PosixSemaphore::new(0).expect("valid semaphore"));
        SHARED_STATE.context = AtomicPtr::new(ptr::null_mut());
    }
}

struct PosixSemaphore {
    sem: UnsafeCell<libc::sem_t>,
}

impl PosixSemaphore {
    pub fn new(value: u32) -> io::Result<Self> {
        let mut sem: libc::sem_t = unsafe { mem::uninitialized() };
        let r = unsafe {
            libc::sem_init(&mut sem, 0 /* not shared */, value)
        };
        if r == -1 {
            return Err(io::Error::last_os_error());
        }
        Ok(PosixSemaphore {
            sem: UnsafeCell::new(sem),
        })
    }

    pub fn post(&self) -> io::Result<()> {
        if unsafe { libc::sem_post(self.sem.get()) } == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    pub fn wait(&self) -> io::Result<()> {
        if unsafe { libc::sem_wait(self.sem.get()) } == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    /// Retries the wait if it returned due to EINTR.
    /// Returns Ok on success and the error on any other return value.
    pub fn wait_through_intr(&self) -> io::Result<()> {
        loop {
            match self.wait() {
                Err(os_error) => {
                    let err = os_error.raw_os_error().expect("no os error");
                    if err == libc::EINTR {
                        thread::yield_now();
                        continue;
                    }
                    return Err(os_error);
                },
                _ => return Ok(()),
            }
        }
    }
}

unsafe impl Sync for PosixSemaphore {}

impl Drop for PosixSemaphore {
    /// Destroys the semaphore.
    fn drop(&mut self) {
        unsafe { libc::sem_destroy(self.sem.get()) };
    }
}

#[allow(dead_code)]
pub struct LinuxSampler {
    thread_id: MonitoredThreadId,
    old_handler: SigAction,
}

impl LinuxSampler {
    #[allow(unsafe_code, dead_code)]
    pub fn new() -> Box<Sampler> {
        let thread_id = unsafe { libc::syscall(libc::SYS_gettid) as libc::pid_t };
        let handler = SigHandler::SigAction(sigprof_handler);
        let action = SigAction::new(
            handler,
            SaFlags::SA_RESTART | SaFlags::SA_SIGINFO,
            SigSet::empty(),
        );
        let old_handler =
            unsafe { sigaction(Signal::SIGPROF, &action).expect("signal handler set") };
        Box::new(LinuxSampler {
            thread_id,
            old_handler,
        })
    }
}

enum RegNum {
    Ip = UNW_REG_IP as isize,
    Sp = UNW_REG_SP as isize,
}

fn get_register(cursor: &mut unw_cursor_t, num: RegNum) -> Result<u64, i32> {
    unsafe {
        let mut val = 0;
        let ret = unw_get_reg(cursor, num as i32, &mut val);
        if ret == UNW_ESUCCESS {
            Ok(val)
        } else {
            Err(ret)
        }
    }
}

fn step(cursor: &mut unw_cursor_t) -> Result<bool, i32> {
    unsafe {
        // libunwind 1.1 seems to get confused and walks off the end of the stack. The last IP
        // it reports is 0, so we'll stop if we're there.
        if get_register(cursor, RegNum::Ip).unwrap_or(1) == 0 {
            return Ok(false);
        }

        let ret = unw_step(cursor);
        if ret > 0 {
            Ok(true)
        } else if ret == 0 {
            Ok(false)
        } else {
            Err(ret)
        }
    }
}

impl Sampler for LinuxSampler {
    #[allow(unsafe_code)]
    fn suspend_and_sample_thread(&self) -> Result<NativeStack, ()> {
        // Warning: The "critical section" begins here.
        // In the critical section:
        // we must not do any dynamic memory allocation,
        // nor try to acquire any lock
        // or any other unshareable resource.
        // first we reinitialize the semaphores
        reset_shared_state();

        // signal the thread, wait for it to tell us state was copied.
        send_sigprof(self.thread_id);
        unsafe {
            SHARED_STATE
                .msg2
                .as_ref()
                .unwrap()
                .wait_through_intr()
                .expect("msg2 failed");
        }

        let context = unsafe { SHARED_STATE.context.load(Ordering::SeqCst) };
        let mut cursor = unsafe { mem::uninitialized() };
        let ret = unsafe { unw_init_local(&mut cursor, context) };
        let result = if ret == UNW_ESUCCESS {
            let mut native_stack = NativeStack::new();
            loop {
                let ip = match get_register(&mut cursor, RegNum::Ip) {
                    Ok(ip) => ip,
                    Err(_) => break,
                };
                let sp = match get_register(&mut cursor, RegNum::Sp) {
                    Ok(sp) => sp,
                    Err(_) => break,
                };
                if native_stack
                    .process_register(ip as *mut _, sp as *mut _)
                    .is_err() ||
                    !step(&mut cursor).unwrap_or(false)
                {
                    break;
                }
            }
            Ok(native_stack)
        } else {
            Err(())
        };

        // signal the thread to continue.
        unsafe {
            SHARED_STATE
                .msg3
                .as_ref()
                .unwrap()
                .post()
                .expect("msg3 failed");
        }

        // wait for thread to continue.
        unsafe {
            SHARED_STATE
                .msg4
                .as_ref()
                .unwrap()
                .wait_through_intr()
                .expect("msg4 failed");
        }

        clear_shared_state();

        // NOTE: End of "critical section".
        result
    }
}

impl Drop for LinuxSampler {
    fn drop(&mut self) {
        unsafe {
            sigaction(Signal::SIGPROF, &self.old_handler).expect("previous signal handler restored")
        };
    }
}

extern "C" fn sigprof_handler(
    sig: libc::c_int,
    _info: *mut libc::siginfo_t,
    ctx: *mut libc::c_void,
) {
    assert_eq!(sig, libc::SIGPROF);
    unsafe {
        // copy the context.
        SHARED_STATE
            .context
            .store(ctx as *mut libc::ucontext_t, Ordering::SeqCst);
        // Tell the sampler we copied the context.
        SHARED_STATE.msg2.as_ref().unwrap().post().expect("posted");

        // Wait for sampling to finish.
        SHARED_STATE
            .msg3
            .as_ref()
            .unwrap()
            .wait_through_intr()
            .expect("msg3 wait succeeded");

        // OK we are done!
        SHARED_STATE.msg4.as_ref().unwrap().post().expect("posted");
        // DO NOT TOUCH shared state here onwards.
    }
}

fn send_sigprof(to: libc::pid_t) {
    unsafe {
        libc::syscall(libc::SYS_tgkill, process::id(), to, libc::SIGPROF);
    }
}
