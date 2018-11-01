/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use backtrace;
use constellation_msg::{HangAlert, HangAnnotation, HangProfile};
use constellation_msg::{MonitoredComponentId, MonitoredComponentMsg};
use ipc_channel::ipc::IpcSender;
use libc;
#[cfg(target_os = "macos")]
use mach;
use servo_channel::{Receiver, base_channel};
use std::collections::HashMap;
use std::ptr;
#[cfg(any(target_os = "android", target_os = "linux"))]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(any(target_os = "android", target_os = "linux"))]
use std::thread;
use std::time::{Duration, Instant};


const MAX_NATIVE_FRAMES: usize = 1024;

/// The means of communication between monitored and monitor.
#[cfg(any(target_os = "android", target_os = "linux"))]
pub static mut SAMPLEE_CONTEXT: Option<(MonitoredThreadId, libc::ucontext_t)> = None;

/// This is an attempt to implement a protocol similar to:
/// https://github.com/mozilla/gecko-dev/blob/aaac9a77dd456360551dd764ffba4ca4899dcb56/
/// tools/profiler/core/platform-linux-android.cpp#L156
#[cfg(any(target_os = "android", target_os = "linux"))]
lazy_static! {
    /// A flag used to coordinate access to SAMPLEE_CONTEXT.
    pub static ref SAMPLEE_CONTEXT_READY: AtomicBool = AtomicBool::new(false);
    /// A flag used to signal when the samplee may resume executing.
    pub static ref SAMPLEE_MAY_RESUME: AtomicBool = AtomicBool::new(false);
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
    count: usize
}

impl NativeStack {
    fn new() -> Self {
        NativeStack {
            instruction_ptrs: [ptr::null_mut(); MAX_NATIVE_FRAMES],
            stack_ptrs: [ptr::null_mut(); MAX_NATIVE_FRAMES],
            count: 0
        }
    }

    fn process_register(&mut self, instruction_ptr: *mut std::ffi::c_void, stack_ptr: *mut std::ffi::c_void) {
        self.instruction_ptrs[self.count] = instruction_ptr;
        self.stack_ptrs[self.count] = stack_ptr;
        self.count = self.count + 1;
    }
}

#[cfg(target_os = "macos")]
fn check_kern_return(kret: mach::kern_return::kern_return_t) -> Result<(), ()> {
    if kret != mach::kern_return::KERN_SUCCESS {
        warn!("Kern return error: {:?}", kret);
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
unsafe fn suspend_thread(thread_id: MonitoredThreadId) -> Result<(), ()> {
    // TODO: use winapi::um::processthreadsapi::SuspendThread
    Err(())
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[allow(unsafe_code)]
unsafe fn suspend_thread(thread_id: MonitoredThreadId) -> Result<(), ()> {
    match libc::pthread_kill(thread_id, libc::SIGPROF) {
        0 => Ok(()),
        _ => Err(()),
    }
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
unsafe fn get_registers(thread_id: MonitoredThreadId) -> Result<Registers, ()> {
    let mut state = mach::structs::x86_thread_state64_t::new();
    let mut state_count = mach::structs::x86_thread_state64_t::count();
    let kret = mach::thread_act::thread_get_state(thread_id,
                                                  mach::thread_status::x86_THREAD_STATE64,
                                                  (&mut state) as *mut _ as *mut _,
                                                  &mut state_count);
    check_kern_return(kret)?;
    Ok(Registers {
        instruction_ptr: state.__rip as Address,
        stack_ptr: state.__rsp as Address,
        frame_ptr: state.__rbp as Address,
    })
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
unsafe fn get_registers(thread_id: MonitoredThreadId) -> Result<Registers, ()> {
    // TODO: use winapi::um::processthreadsapi::GetThreadContext
    // and populate registers using the context.
    // See https://dxr.mozilla.org/mozilla-central/source/tools/profiler/core/platform-win32.cpp#129
    Err(())
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[allow(unsafe_code)]
unsafe fn get_registers(_thread_id: MonitoredThreadId) -> Result<Registers, ()> {
    loop {
        if SAMPLEE_CONTEXT_READY.load(Ordering::SeqCst) {
            break;
        }
        thread::yield_now();
    }
    let (_thread_id, _context) = SAMPLEE_CONTEXT.take().unwrap();
    // TODO: populate registers from context.
    // See https://dxr.mozilla.org/mozilla-central/source/tools/profiler/core/platform-linux-android.cpp#85
    Err(())
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
unsafe fn resume_thread(thread_id: MonitoredThreadId) {
    if let Err(()) = check_kern_return(mach::thread_act::thread_resume(thread_id)) {
        warn!("Background-hang-monitor failed to resume thread: {:?}", thread_id);
    }
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
unsafe fn resume_thread(thread_id: MonitoredThreadId) {
    // TODO: use winapi::um::processthreadsapi::ResumeThread
}

#[cfg(any(target_os = "android", target_os = "linux"))]
fn resume_thread(_thread_id: MonitoredThreadId) {
    SAMPLEE_MAY_RESUME.store(true, Ordering::SeqCst);
}

#[allow(unsafe_code)]
unsafe fn frame_pointer_stack_walk(regs: Registers) -> NativeStack {
    // Note: this will only work with code build with debug flags,
    // or RUSTFLAGS=-Cforce-frame-pointers=yes
    let mut native_stack = NativeStack::new();
    let pc = regs.instruction_ptr as *mut std::ffi::c_void;
    let stack = regs.stack_ptr as *mut std::ffi::c_void;
    native_stack.process_register(pc, stack);
    let mut current = regs.frame_ptr as *mut *mut std::ffi::c_void;
    let mut frame_count = 1;
    while !current.is_null() && frame_count < MAX_NATIVE_FRAMES {
        let next = *current as *mut *mut std::ffi::c_void;
        // NOTE: assuming i386 or powerpc32 linux.
        // TODO: ppc mac or powerpc64 linux require add(2) and add(3) below.
        let pc = current.add(1);
        let stack = current.add(2);
        native_stack.process_register(*pc, *stack);
        frame_count = frame_count + 1;
        current = next;
    }
    native_stack
}

fn symbolize_backtrace(native_stack: NativeStack) -> HangProfile {
    let mut profile = HangProfile {
        backtrace: Vec::new()
    };
    for ip in native_stack.instruction_ptrs.iter().rev() {
        if ip.is_null() {
            continue;
        }
        backtrace::resolve(*ip, |symbol| {
            if let Some(symbol_name) = symbol.name() {
                let bytes = symbol_name.as_bytes();
                profile.backtrace.push(String::from_utf8_lossy(&bytes).to_string());
            }
        });
    }
    profile
}

#[allow(unsafe_code)]
unsafe fn suspend_and_sample_thread(monitored: &MonitoredComponent) -> Option<HangProfile> {
    // Warning: The "critical section" begins here.
    // In the critical section:
    // we must not do any dynamic memory allocation,
    // nor try to acquire any lock
    // or any other unshareable resource.
    if let Err(()) = suspend_thread(monitored.thread_id) {
        return None
    };
    let native_stack = match get_registers(monitored.thread_id) {
        Ok(regs) => frame_pointer_stack_walk(registers),
        Err(()) => None,
    };
    resume_thread(monitored.thread_id);
    // NOTE: End of "critical section".
    match native_stack {
        Some(stack) => Some(symbolize_backtrace(native_stack)),
        None => None
    }
}

struct MonitoredComponent {
    thread_id: MonitoredThreadId,
    last_activity: Instant,
    last_annotation: Option<HangAnnotation>,
    transient_hang_timeout: Duration,
    permanent_hang_timeout: Duration,
    sent_transient_alert: bool,
    sent_permanent_alert: bool,
    is_waiting: bool,
}

pub struct BackgroundHangMonitor {
    monitored_components: HashMap<MonitoredComponentId, MonitoredComponent>,
    constellation_chan: IpcSender<HangAlert>,
    port: Receiver<(MonitoredComponentId, MonitoredComponentMsg)>,
}

impl BackgroundHangMonitor {
    pub fn new(
        constellation_chan: IpcSender<HangAlert>,
        port: Receiver<(MonitoredComponentId, MonitoredComponentMsg)>,
    ) -> Self {
        BackgroundHangMonitor {
            monitored_components: Default::default(),
            constellation_chan,
            port,
        }
    }

    pub fn run(&mut self) -> bool {
        let received = select! {
            recv(self.port.select(), event) => {
                match event {
                    Some(msg) => Some(msg),
                    // Our sender has been dropped, quit.
                    None => return false,
                }
            },
            recv(base_channel::after(Duration::from_millis(100))) => None,
        };
        if let Some(msg) = received {
            self.handle_msg(msg);
            while let Some(another_msg) = self.port.try_recv() {
                // Handle any other incoming messages,
                // before performing a hang checkpoint.
                self.handle_msg(another_msg);
            }
        }
        self.perform_a_hang_monitor_checkpoint();
        true
    }

    fn handle_msg(&mut self, msg: (MonitoredComponentId, MonitoredComponentMsg)) {
        match msg {
            (component_id, MonitoredComponentMsg::Register(
                    thread_id,
                    transient_hang_timeout,
                    permanent_hang_timeout)) => {
                let component = MonitoredComponent {
                    thread_id,
                    last_activity: Instant::now(),
                    last_annotation: None,
                    transient_hang_timeout,
                    permanent_hang_timeout,
                    sent_transient_alert: false,
                    sent_permanent_alert: false,
                    is_waiting: true,
                };
                assert!(
                    self
                        .monitored_components
                        .insert(component_id, component)
                        .is_none(),
                    "This component was already registered for monitoring."
                );
            },
            (component_id, MonitoredComponentMsg::NotifyActivity(annotation)) => {
                let mut component = self
                    .monitored_components
                    .get_mut(&component_id)
                    .expect("Received NotifyActivity for an unknown component");
                component.last_activity = Instant::now();
                component.last_annotation = Some(annotation);
                component.sent_transient_alert = false;
                component.sent_permanent_alert = false;
                component.is_waiting = false;
            },
            (component_id, MonitoredComponentMsg::NotifyWait) => {
                let mut component = self
                    .monitored_components
                    .get_mut(&component_id)
                    .expect("Received NotifyWait for an unknown component");
                component.last_activity = Instant::now();
                component.sent_transient_alert = false;
                component.sent_permanent_alert = false;
                component.is_waiting = true;
            },
        }
    }

    #[allow(unsafe_code)]
    fn perform_a_hang_monitor_checkpoint(&mut self) {
        for (component_id, mut monitored) in self.monitored_components.iter_mut() {
            if monitored.is_waiting {
                continue;
            }
            let last_annotation = monitored.last_annotation.unwrap();
            if monitored.last_activity.elapsed() > monitored.permanent_hang_timeout {
                if monitored.sent_permanent_alert {
                    continue;
                }
                let profile = unsafe {
                    suspend_and_sample_thread(&monitored)
                };
                let _ = self
                    .constellation_chan
                    .send(
                        HangAlert::Permanent(
                            component_id.clone(),
                            last_annotation,
                            profile
                        )
                    );
                monitored.sent_permanent_alert = true;
                continue;
            }
            if monitored.last_activity.elapsed() > monitored.transient_hang_timeout {
                if monitored.sent_transient_alert {
                    continue;
                }
                let _ = self
                    .constellation_chan
                    .send(HangAlert::Transient(component_id.clone(), last_annotation));
                monitored.sent_transient_alert = true;
            }
        }
    }
}
