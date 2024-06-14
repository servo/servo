/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
#[cfg(not(windows))]
use std::env;
use std::ffi::OsStr;
use std::process;

#[cfg(any(
    target_os = "macos",
    all(
        not(target_os = "windows"),
        not(target_os = "ios"),
        not(target_os = "android"),
        not(target_arch = "arm"),
        not(target_arch = "aarch64")
    )
))]
use gaol::profile::{Operation, PathPattern, Profile};
use ipc_channel::Error;
use serde::{Deserialize, Serialize};
use servo_config::opts::Opts;
use servo_config::prefs::PrefValue;

use crate::pipeline::UnprivilegedPipelineContent;
use crate::serviceworker::ServiceWorkerUnprivilegedContent;

#[derive(Deserialize, Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum UnprivilegedContent {
    Pipeline(UnprivilegedPipelineContent),
    ServiceWorker(ServiceWorkerUnprivilegedContent),
}

impl UnprivilegedContent {
    pub fn opts(&self) -> Opts {
        match self {
            UnprivilegedContent::Pipeline(content) => content.opts(),
            UnprivilegedContent::ServiceWorker(content) => content.opts(),
        }
    }

    pub fn prefs(&self) -> HashMap<String, PrefValue> {
        match self {
            UnprivilegedContent::Pipeline(content) => content.prefs(),
            UnprivilegedContent::ServiceWorker(content) => content.prefs(),
        }
    }
}

/// Our content process sandbox profile on Mac. As restrictive as possible.
#[cfg(target_os = "macos")]
pub fn content_process_sandbox_profile() -> Profile {
    use std::path::PathBuf;

    use embedder_traits::resources;
    use gaol::platform;

    let mut operations = vec![
        Operation::FileReadAll(PathPattern::Literal(PathBuf::from("/dev/urandom"))),
        Operation::FileReadAll(PathPattern::Subpath(PathBuf::from("/Library/Fonts"))),
        Operation::FileReadAll(PathPattern::Subpath(PathBuf::from("/System/Library/Fonts"))),
        Operation::FileReadAll(PathPattern::Subpath(PathBuf::from(
            "/System/Library/Frameworks/ApplicationServices.framework",
        ))),
        Operation::FileReadAll(PathPattern::Subpath(PathBuf::from(
            "/System/Library/Frameworks/CoreGraphics.framework",
        ))),
        Operation::FileReadMetadata(PathPattern::Literal(PathBuf::from("/"))),
        Operation::FileReadMetadata(PathPattern::Literal(PathBuf::from("/Library"))),
        Operation::FileReadMetadata(PathPattern::Literal(PathBuf::from("/System"))),
        Operation::FileReadMetadata(PathPattern::Literal(PathBuf::from("/etc"))),
        Operation::SystemInfoRead,
        Operation::PlatformSpecific(platform::macos::Operation::MachLookup(
            b"com.apple.FontServer".to_vec(),
        )),
    ];

    operations.extend(
        resources::sandbox_access_files()
            .into_iter()
            .map(|p| Operation::FileReadAll(PathPattern::Literal(p))),
    );
    operations.extend(
        resources::sandbox_access_files_dirs()
            .into_iter()
            .map(|p| Operation::FileReadAll(PathPattern::Subpath(p))),
    );

    Profile::new(operations).expect("Failed to create sandbox profile!")
}

/// Our content process sandbox profile on Linux. As restrictive as possible.
#[cfg(all(
    not(target_os = "macos"),
    not(target_os = "windows"),
    not(target_os = "ios"),
    not(target_os = "android"),
    not(target_arch = "arm"),
    not(target_arch = "aarch64")
))]
pub fn content_process_sandbox_profile() -> Profile {
    use std::path::PathBuf;

    use embedder_traits::resources;

    let mut operations = vec![Operation::FileReadAll(PathPattern::Literal(PathBuf::from(
        "/dev/urandom",
    )))];

    operations.extend(
        resources::sandbox_access_files()
            .into_iter()
            .map(|p| Operation::FileReadAll(PathPattern::Literal(p))),
    );
    operations.extend(
        resources::sandbox_access_files_dirs()
            .into_iter()
            .map(|p| Operation::FileReadAll(PathPattern::Subpath(p))),
    );

    Profile::new(operations).expect("Failed to create sandbox profile!")
}

#[cfg(any(
    target_os = "windows",
    target_os = "ios",
    target_os = "android",
    target_arch = "arm",

    // exclude apple arm devices
    all(target_arch = "aarch64", not(target_os = "macos"))
))]
pub fn content_process_sandbox_profile() {
    log::error!("Sandboxed multiprocess is not supported on this platform.");
    process::exit(1);
}

#[cfg(any(
    target_os = "android",
    target_arch = "arm",
    all(target_arch = "aarch64", not(target_os = "windows"))
))]
pub fn spawn_multiprocess(content: UnprivilegedContent) -> Result<(), Error> {
    use ipc_channel::ipc::{IpcOneShotServer, IpcSender};
    // Note that this function can panic, due to process creation,
    // avoiding this panic would require a mechanism for dealing
    // with low-resource scenarios.
    let (server, token) = IpcOneShotServer::<IpcSender<UnprivilegedContent>>::new()
        .expect("Failed to create IPC one-shot server.");

    let path_to_self = env::current_exe().expect("Failed to get current executor.");
    let mut child_process = process::Command::new(path_to_self);
    setup_common(&mut child_process, token);
    let _ = child_process
        .spawn()
        .expect("Failed to start unsandboxed child process!");

    let (_receiver, sender) = server.accept().expect("Server failed to accept.");
    sender.send(content)?;

    Ok(())
}

#[cfg(all(
    not(target_os = "windows"),
    not(target_os = "ios"),
    not(target_os = "android"),
    not(target_arch = "arm"),
    not(target_arch = "aarch64")
))]
pub fn spawn_multiprocess(content: UnprivilegedContent) -> Result<(), Error> {
    use gaol::sandbox::{self, Sandbox, SandboxMethods};
    use ipc_channel::ipc::{IpcOneShotServer, IpcSender};

    impl CommandMethods for sandbox::Command {
        fn arg<T>(&mut self, arg: T)
        where
            T: AsRef<OsStr>,
        {
            self.arg(arg);
        }

        fn env<T, U>(&mut self, key: T, val: U)
        where
            T: AsRef<OsStr>,
            U: AsRef<OsStr>,
        {
            self.env(key, val);
        }
    }

    // Note that this function can panic, due to process creation,
    // avoiding this panic would require a mechanism for dealing
    // with low-resource scenarios.
    let (server, token) = IpcOneShotServer::<IpcSender<UnprivilegedContent>>::new()
        .expect("Failed to create IPC one-shot server.");

    // If there is a sandbox, use the `gaol` API to create the child process.
    if content.opts().sandbox {
        let mut command = sandbox::Command::me().expect("Failed to get current sandbox.");
        setup_common(&mut command, token);

        let profile = content_process_sandbox_profile();
        let _ = Sandbox::new(profile)
            .start(&mut command)
            .expect("Failed to start sandboxed child process!");
    } else {
        let path_to_self = env::current_exe().expect("Failed to get current executor.");
        let mut child_process = process::Command::new(path_to_self);
        setup_common(&mut child_process, token);
        let _ = child_process
            .spawn()
            .expect("Failed to start unsandboxed child process!");
    }

    let (_receiver, sender) = server.accept().expect("Server failed to accept.");
    sender.send(content)?;

    Ok(())
}

#[cfg(any(target_os = "windows", target_os = "ios"))]
pub fn spawn_multiprocess(_content: UnprivilegedContent) -> Result<(), Error> {
    log::error!("Multiprocess is not supported on Windows or iOS.");
    process::exit(1);
}

#[cfg(not(windows))]
fn setup_common<C: CommandMethods>(command: &mut C, token: String) {
    C::arg(command, "--content-process");
    C::arg(command, token);

    if let Ok(value) = env::var("RUST_BACKTRACE") {
        C::env(command, "RUST_BACKTRACE", value);
    }

    if let Ok(value) = env::var("RUST_LOG") {
        C::env(command, "RUST_LOG", value);
    }
}

/// A trait to unify commands launched as multiprocess with or without a sandbox.
#[allow(dead_code)]
trait CommandMethods {
    /// A command line argument.
    fn arg<T>(&mut self, arg: T)
    where
        T: AsRef<OsStr>;

    /// An environment variable.
    fn env<T, U>(&mut self, key: T, val: U)
    where
        T: AsRef<OsStr>,
        U: AsRef<OsStr>;
}

impl CommandMethods for process::Command {
    fn arg<T>(&mut self, arg: T)
    where
        T: AsRef<OsStr>,
    {
        self.arg(arg);
    }

    fn env<T, U>(&mut self, key: T, val: U)
    where
        T: AsRef<OsStr>,
        U: AsRef<OsStr>,
    {
        self.env(key, val);
    }
}
