/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::jsapi::JSObject;
use servo_config::pref;

use crate::dom::bindings::codegen::RegisterBindings;
use crate::dom::bindings::conversions::is_dom_proxy;
use crate::dom::bindings::proxyhandler;
use crate::dom::bindings::utils::is_platform_object_static;
use crate::script_runtime::JSEngineSetup;

#[cfg(target_os = "linux")]
#[allow(unsafe_code)]
fn perform_platform_specific_initialization() {
    // 4096 is default max on many linux systems
    const MAX_FILE_LIMIT: libc::rlim_t = 4096;

    // Bump up our number of file descriptors to save us from impending doom caused by an onslaught
    // of iframes.
    unsafe {
        let mut rlim = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        match libc::getrlimit(libc::RLIMIT_NOFILE, &mut rlim) {
            0 => {
                if rlim.rlim_cur >= MAX_FILE_LIMIT {
                    // we have more than enough
                    return;
                }

                rlim.rlim_cur = match rlim.rlim_max {
                    libc::RLIM_INFINITY => MAX_FILE_LIMIT,
                    _ => {
                        if rlim.rlim_max < MAX_FILE_LIMIT {
                            rlim.rlim_max
                        } else {
                            MAX_FILE_LIMIT
                        }
                    },
                };
                match libc::setrlimit(libc::RLIMIT_NOFILE, &rlim) {
                    0 => (),
                    _ => warn!("Failed to set file count limit"),
                };
            },
            _ => warn!("Failed to get file count limit"),
        };
    }
}

#[cfg(not(target_os = "linux"))]
fn perform_platform_specific_initialization() {}

#[allow(unsafe_code)]
unsafe extern "C" fn is_dom_object(obj: *mut JSObject) -> bool {
    !obj.is_null() && (is_platform_object_static(obj) || is_dom_proxy(obj))
}

/// Returns true if JIT is forbidden
///
/// Spidermonkey will crash if JIT is not allowed on a system, so we do a short detection
/// if jit is allowed or not.
#[cfg(target_os = "linux")]
#[allow(unsafe_code)]
fn jit_forbidden() -> bool {
    let flags = libc::MAP_NORESERVE | libc::MAP_PRIVATE | libc::MAP_ANON;
    let first_mmap =
        unsafe { libc::mmap(core::ptr::null_mut(), 4096, libc::PROT_NONE, flags, -1, 0) };
    assert_ne!(first_mmap, libc::MAP_FAILED, "mmap not allowed?");
    let remap_flags =
        libc::MAP_ANONYMOUS | libc::MAP_FIXED | libc::MAP_PRIVATE | libc::MAP_EXECUTABLE;
    // remap the page with PROT_EXEC. If this fails, JIT is not possible.
    let second_mmap = unsafe {
        libc::mmap(
            first_mmap,
            4096,
            libc::PROT_READ | libc::PROT_EXEC,
            remap_flags,
            -1,
            0,
        )
    };
    let remap_failed = second_mmap == libc::MAP_FAILED;
    // SAFETY: For the second mmap we used `MAP_FIXED` so in both success and failure case, the
    // address will be the same as the first mmap.
    unsafe { libc::munmap(first_mmap, 4096) };
    remap_failed
}

#[cfg(not(target_os = "linux"))]
fn jit_forbidden() -> bool {
    false
}

#[allow(unsafe_code)]
pub fn init() -> JSEngineSetup {
    if pref!(js_disable_jit) || jit_forbidden() {
        let reason = if pref!(js_disable_jit) {
            "preference `js_disable_jit` is set to true"
        } else {
            "runtime test determined JIT is forbidden on this system"
        };
        warn!("Disabling JIT for Javascript, since {reason}. This may cause subpar performance");
        // SAFETY: This function has no particular preconditions.
        unsafe {
            js::jsapi::DisableJitBackend();
        }
    }
    proxyhandler::init();

    // Create the global vtables used by the (generated) DOM
    // bindings to implement JS proxies.
    RegisterBindings::RegisterProxyHandlers::<crate::DomTypeHolder>();
    RegisterBindings::InitAllStatics::<crate::DomTypeHolder>();

    unsafe {
        js::glue::InitializeMemoryReporter(Some(is_dom_object));
    }

    perform_platform_specific_initialization();

    JSEngineSetup::default()
}
