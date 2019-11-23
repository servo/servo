/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::RegisterBindings;
use crate::dom::bindings::proxyhandler;
use crate::script_runtime::setup_js_engine;
use crate::serviceworker_manager::ServiceWorkerManager;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use script_traits::SWManagerSenders;

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

pub fn init_service_workers(sw_senders: SWManagerSenders) {
    // Spawn the service worker manager passing the constellation sender
    ServiceWorkerManager::spawn_manager(sw_senders);
}

/// Initialize the global script state, and listen for a shutdown signal
/// on the provided receiver.
#[allow(unsafe_code)]
pub fn init_with_shutdown_receiver(shutdown_receiver: IpcReceiver<()>) {
    unsafe {
        proxyhandler::init();

        // Create the global vtables used by the (generated) DOM
        // bindings to implement JS proxies.
        RegisterBindings::RegisterProxyHandlers();
    }

    perform_platform_specific_initialization();

    setup_js_engine(shutdown_receiver)
}

/// Initialize the global script state, and return a sender that will
/// initiate shutdown procedures.
pub fn init() -> IpcSender<()> {
    let (shutdown_sender, shutdown_receiver) = ipc::channel().unwrap();
    init_with_shutdown_receiver(shutdown_receiver);
    shutdown_sender
}
