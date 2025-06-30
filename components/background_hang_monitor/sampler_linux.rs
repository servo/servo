/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use std::cell::UnsafeCell;
use std::{io, mem, process, thread};

use nix::sys::signal::{SaFlags, SigAction, SigHandler, SigSet, Signal, sigaction};

use crate::sampler::{NativeStack, Sampler};

struct UncheckedSyncUnsafeCell<T>(std::cell::UnsafeCell<T>);

/// Safety: dereferencing the pointer from `UnsafeCell::get` must involve external synchronization
unsafe impl<T> Sync for UncheckedSyncUnsafeCell<T> {}

static SHARED_STATE: UncheckedSyncUnsafeCell<SharedState> =
    UncheckedSyncUnsafeCell(std::cell::UnsafeCell::new(SharedState {
        msg2: None,
        msg3: None,
        msg4: None,
    }));

type MonitoredThreadId = libc::pid_t;

struct SharedState {
    // "msg1" is the signal.
    msg2: Option<PosixSemaphore>,
    msg3: Option<PosixSemaphore>,
    msg4: Option<PosixSemaphore>,
}

fn clear_shared_state() {
    // Safety: this is only called from the sampling thread (there’s only one)
    // Sampled threads only access SHARED_STATE in their signal handler.
    // This signal and the semaphores in SHARED_STATE provide the necessary synchronization.
    unsafe {
        let shared_state = &mut *SHARED_STATE.0.get();
        shared_state.msg2 = None;
        shared_state.msg3 = None;
        shared_state.msg4 = None;
    }
}

fn reset_shared_state() {
    // Safety: same as clear_shared_state
    unsafe {
        let shared_state = &mut *SHARED_STATE.0.get();
        shared_state.msg2 = Some(PosixSemaphore::new(0).expect("valid semaphore"));
        shared_state.msg3 = Some(PosixSemaphore::new(0).expect("valid semaphore"));
        shared_state.msg4 = Some(PosixSemaphore::new(0).expect("valid semaphore"));
    }
}

struct PosixSemaphore {
    sem: UnsafeCell<libc::sem_t>,
}

impl PosixSemaphore {
    pub fn new(value: u32) -> io::Result<Self> {
        let mut sem = mem::MaybeUninit::uninit();
        let r = unsafe {
            libc::sem_init(sem.as_mut_ptr(), 0 /* not shared */, value)
        };
        if r == -1 {
            return Err(io::Error::last_os_error());
        }
        Ok(PosixSemaphore {
            sem: UnsafeCell::new(unsafe { sem.assume_init() }),
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
    pub fn new_boxed() -> Box<dyn Sampler> {
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

        // Safety: non-exclusive reference only
        // since sampled threads are accessing this concurrently
        let result;
        {
            let shared_state = unsafe { &*SHARED_STATE.0.get() };
            shared_state
                .msg2
                .as_ref()
                .unwrap()
                .wait_through_intr()
                .expect("msg2 failed");

            let mut native_stack = NativeStack::new();
            unsafe {
                backtrace::trace_unsynchronized(|frame| {
                    let ip = frame.ip();
                    let sp = frame.sp();

                    //This return value here determines whether we proceed to the next stack frame or not.
                    native_stack.process_register(ip, sp).is_ok()
                })
            };
            result = Ok(native_stack);

            // signal the thread to continue.
            shared_state
                .msg3
                .as_ref()
                .unwrap()
                .post()
                .expect("msg3 failed");

            // wait for thread to continue.
            shared_state
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
    _ctx: *mut libc::c_void,
) {
    assert_eq!(sig, libc::SIGPROF);

    // Safety: non-exclusive reference only
    // since the sampling thread is accessing this concurrently
    let shared_state = unsafe { &*SHARED_STATE.0.get() };

    // Tell the sampler we copied the context.
    shared_state.msg2.as_ref().unwrap().post().expect("posted");

    // Wait for sampling to finish.
    shared_state
        .msg3
        .as_ref()
        .unwrap()
        .wait_through_intr()
        .expect("msg3 wait succeeded");

    // OK we are done!
    shared_state.msg4.as_ref().unwrap().post().expect("posted");
    // DO NOT TOUCH shared state here onwards.
}

fn send_sigprof(to: libc::pid_t) {
    unsafe {
        libc::syscall(libc::SYS_tgkill, process::id(), to, libc::SIGPROF);
    }
}
