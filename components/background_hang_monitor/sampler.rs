/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use backtrace;
#[cfg(any(target_os = "android", target_os = "linux"))]
use libc;
#[cfg(target_os = "macos")]
use mach;
use msg::constellation_msg::{HangProfile, HangProfileSymbol};
#[cfg(any(target_os = "android", target_os = "linux"))]
use std::mem;
use std::panic;
use std::process;
use std::ptr;
#[cfg(any(target_os = "android", target_os = "linux"))]
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(any(target_os = "android", target_os = "linux"))]
use std::thread;
#[cfg(any(target_os = "android", target_os = "linux"))]
use std::time::Instant;

const MAX_NATIVE_FRAMES: usize = 1024;

/// The means of communication between monitored and monitor.
#[cfg(any(target_os = "android", target_os = "linux"))]
static mut SAMPLEE_CONTEXT: Option<(MonitoredThreadId, libc::mcontext_t)> = None;

#[cfg(any(target_os = "android", target_os = "linux"))]
lazy_static! {
    /// A state-machine used to coordinate access to SAMPLEE_CONTEXT.
    static ref SAMPLER_STATE: AtomicUsize = AtomicUsize::new(ThreadProfilerState::NotProfiling as usize);
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[derive(Debug)]
#[repr(usize)]
enum ThreadProfilerState {
    NotProfiling = 0,
    SuspendedThread,
    ReadyToObtainBacktrace,
    ObtainedBacktrace,
    ResumedThread,
}

#[cfg(target_os = "macos")]
pub type MonitoredThreadId = mach::mach_types::thread_act_t;

#[cfg(target_os = "windows")]
pub type MonitoredThreadId = usize; // TODO: use winapi

#[cfg(any(target_os = "android", target_os = "linux"))]
pub type MonitoredThreadId = libc::pthread_t;

type Address = *const libc::uint8_t;

/// The registers used for stack unwinding
struct Registers {
    /// Instruction pointer.
    pub instruction_ptr: Address,
    /// Stack pointer.
    pub stack_ptr: Address,
    /// Frame pointer.
    pub frame_ptr: Address,
}

struct NativeStack {
    instruction_ptrs: [*mut std::ffi::c_void; MAX_NATIVE_FRAMES],
    stack_ptrs: [*mut std::ffi::c_void; MAX_NATIVE_FRAMES],
    count: usize,
}

impl NativeStack {
    fn new() -> Self {
        NativeStack {
            instruction_ptrs: [ptr::null_mut(); MAX_NATIVE_FRAMES],
            stack_ptrs: [ptr::null_mut(); MAX_NATIVE_FRAMES],
            count: 0,
        }
    }

    fn process_register(
        &mut self,
        instruction_ptr: *mut std::ffi::c_void,
        stack_ptr: *mut std::ffi::c_void,
    ) -> Result<(), ()> {
        if !(self.count < MAX_NATIVE_FRAMES) {
            return Err(());
        }
        self.instruction_ptrs[self.count] = instruction_ptr;
        self.stack_ptrs[self.count] = stack_ptr;
        self.count = self.count + 1;
        Ok(())
    }
}

#[cfg(target_os = "macos")]
fn check_kern_return(kret: mach::kern_return::kern_return_t) -> Result<(), ()> {
    if kret != mach::kern_return::KERN_SUCCESS {
        return Err(());
    }
    Ok(())
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
unsafe fn suspend_thread(thread_id: MonitoredThreadId) -> Result<(), ()> {
    check_kern_return(mach::thread_act::thread_suspend(thread_id))
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
unsafe fn suspend_thread(_thread_id: MonitoredThreadId) -> Result<(), ()> {
    // TODO: use winapi::um::processthreadsapi::SuspendThread
    Err(())
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[allow(unsafe_code)]
unsafe fn suspend_thread(thread_id: MonitoredThreadId) -> Result<(), ()> {
    // For now, this is just a sanity check.
    // If suspend_and_sample_thread is moved to a separate thread,
    // change this to a loop waiting for that state.
    assert_eq!(
        SAMPLER_STATE.load(Ordering::SeqCst),
        ThreadProfilerState::NotProfiling as usize
    );
    match libc::pthread_kill(thread_id, libc::SIGPROF) {
        0 => {
            SAMPLER_STATE.store(
                ThreadProfilerState::SuspendedThread as usize,
                Ordering::SeqCst,
            );
            Ok(())
        },
        _ => Err(()),
    }
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
unsafe fn get_registers(thread_id: MonitoredThreadId) -> Result<Registers, ()> {
    let mut state = mach::structs::x86_thread_state64_t::new();
    let mut state_count = mach::structs::x86_thread_state64_t::count();
    let kret = mach::thread_act::thread_get_state(
        thread_id,
        mach::thread_status::x86_THREAD_STATE64,
        (&mut state) as *mut _ as *mut _,
        &mut state_count,
    );
    check_kern_return(kret)?;
    Ok(Registers {
        instruction_ptr: state.__rip as Address,
        stack_ptr: state.__rsp as Address,
        frame_ptr: state.__rbp as Address,
    })
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
unsafe fn get_registers(_thread_id: MonitoredThreadId) -> Result<Registers, ()> {
    // TODO: use winapi::um::processthreadsapi::GetThreadContext
    // and populate registers using the context.
    // See https://dxr.mozilla.org/mozilla-central/source/tools/profiler/core/platform-win32.cpp#129
    Err(())
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[allow(unsafe_code)]
unsafe fn get_registers(thread_id: MonitoredThreadId) -> Result<Registers, ()> {
    let now = Instant::now();
    loop {
        if SAMPLER_STATE.load(Ordering::SeqCst) ==
            ThreadProfilerState::ReadyToObtainBacktrace as usize
        {
            break;
        }
        if now.elapsed().as_secs() > 5 {
            // The samplee still hasn't handled the signal, assume failure.
            return Err(());
        }
        thread::yield_now();
    }
    let (t_id, context) = SAMPLEE_CONTEXT.take().unwrap();
    // sanity check, we're sampling the expected thread.
    assert_eq!(thread_id, t_id);
    // Note:: assuming x86_64.
    // see https://dxr.mozilla.org/mozilla-central/source/tools/profiler/core/platform-linux-android.cpp#85
    // Note: this doesn't actually seem to work yet...
    Ok(Registers {
        instruction_ptr: (&context.gregs[libc::RIP as usize]) as *const _ as Address,
        stack_ptr: (&context.gregs[libc::RSP as usize]) as *const _ as Address,
        frame_ptr: (&context.gregs[libc::RBP as usize]) as *const _ as Address,
    })
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
unsafe fn resume_thread(thread_id: MonitoredThreadId) -> Result<(), ()> {
    check_kern_return(mach::thread_act::thread_resume(thread_id))
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
unsafe fn resume_thread(_thread_id: MonitoredThreadId) -> Result<(), ()> {
    // TODO: use winapi::um::processthreadsapi::ResumeThread
    Err(())
}

#[cfg(any(target_os = "android", target_os = "linux"))]
fn resume_thread(_thread_id: MonitoredThreadId) -> Result<(), ()> {
    SAMPLER_STATE.store(
        ThreadProfilerState::ObtainedBacktrace as usize,
        Ordering::SeqCst,
    );
    let now = Instant::now();
    // Wait until the samplee has resumed.
    let result = loop {
        if SAMPLER_STATE.load(Ordering::SeqCst) == ThreadProfilerState::ResumedThread as usize {
            break Ok(());
        }
        if now.elapsed().as_secs() > 5 {
            // The samplee still hasn't resumed, assume failure to resume.
            break Err(());
        }
        thread::yield_now();
    };
    SAMPLER_STATE.store(ThreadProfilerState::NotProfiling as usize, Ordering::SeqCst);
    result
}

#[allow(unsafe_code)]
unsafe fn frame_pointer_stack_walk(regs: Registers) -> NativeStack {
    // Note: this will only work with code build with:
    // --dev,
    // or --with-frame-pointer.
    let mut native_stack = NativeStack::new();
    let pc = regs.instruction_ptr as *mut std::ffi::c_void;
    let stack = regs.stack_ptr as *mut std::ffi::c_void;
    let _ = native_stack.process_register(pc, stack);
    let mut current = regs.frame_ptr as *mut *mut std::ffi::c_void;
    while !current.is_null() {
        let next = *current as *mut *mut std::ffi::c_void;
        let pc = current.add(1);
        let stack = current.add(2);
        if let Err(()) = native_stack.process_register(*pc, *stack) {
            break;
        }
        current = next;
    }
    native_stack
}

fn symbolize_backtrace(native_stack: NativeStack) -> HangProfile {
    let mut profile = HangProfile {
        backtrace: Vec::new(),
    };
    for ip in native_stack.instruction_ptrs.iter().rev() {
        if ip.is_null() {
            continue;
        }
        backtrace::resolve(*ip, |symbol| {
            // TODO: use the demangled or C++ demangled symbols if available.
            let name = symbol
                .name()
                .map(|n| String::from_utf8_lossy(&n.as_bytes()).to_string());
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

#[allow(unsafe_code)]
pub unsafe fn suspend_and_sample_thread(thread_id: MonitoredThreadId) -> Option<HangProfile> {
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
    if let Err(()) = suspend_thread(thread_id) {
        return None;
    };
    let native_stack = match get_registers(thread_id) {
        Ok(regs) => Some(frame_pointer_stack_walk(regs)),
        Err(()) => None,
    };
    if let Err(()) = resume_thread(thread_id) {
        process::abort();
    }
    panic::set_hook(current_hook);
    // NOTE: End of "critical section".
    native_stack.map(symbolize_backtrace)
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
pub unsafe fn get_thread_id() -> MonitoredThreadId {
    mach::mach_init::mach_thread_self()
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub unsafe fn get_thread_id() -> MonitoredThreadId {
    0 // TODO: use winapi::um::processthreadsapi::GetThreadId
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[allow(unsafe_code)]
pub unsafe fn get_thread_id() -> MonitoredThreadId {
    libc::pthread_self()
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[allow(unsafe_code)]
pub unsafe fn install_sigprof_handler() {
    fn handler(_sig: i32, _info: libc::siginfo_t, context: libc::ucontext_t) {
        unsafe {
            SAMPLEE_CONTEXT = Some((libc::pthread_self(), context.uc_mcontext));
        }
        SAMPLER_STATE.store(
            ThreadProfilerState::ReadyToObtainBacktrace as usize,
            Ordering::SeqCst,
        );
        loop {
            if SAMPLER_STATE.load(Ordering::SeqCst) ==
                ThreadProfilerState::ObtainedBacktrace as usize
            {
                break;
            }
            thread::yield_now();
        }
        // Signal to the monitor that the signal handler is finished.
        SAMPLER_STATE.store(
            ThreadProfilerState::ResumedThread as usize,
            Ordering::SeqCst,
        );
    }

    // Note: this is equivalent to the "signal" macro from the "sig" crate,
    // with the addition of the SA_SIGINFO flag,
    // which gives us the ucontext_t in the handler.
    let mut sigset = mem::uninitialized();

    if libc::sigfillset(&mut sigset) != -1 {
        let mut action: libc::sigaction = mem::zeroed();

        action.sa_flags = libc::SA_SIGINFO;
        action.sa_mask = sigset;
        action.sa_sigaction = handler as libc::sighandler_t;

        libc::sigaction(libc::SIGPROF, &action, ptr::null_mut());
    }
}
