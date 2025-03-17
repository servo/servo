/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::process::Child;

use crossbeam_channel::{Receiver, Select};
use log::{debug, warn};

pub enum Process {
    Unsandboxed(Child),
    Sandboxed(u32),
}

impl Process {
    fn pid(&self) -> u32 {
        match self {
            Self::Unsandboxed(child) => child.id(),
            Self::Sandboxed(pid) => *pid,
        }
    }

    fn wait(&mut self) {
        match self {
            Self::Unsandboxed(child) => {
                let _ = child.wait();
            },
            Self::Sandboxed(_pid) => {
                // TODO: use nix::waitpid() on supported platforms.
                warn!("wait() is not yet implemented for sandboxed processes.");
            },
        }
    }
}

type ProcessReceiver = Receiver<Result<(), ipc_channel::Error>>;

pub(crate) struct ProcessManager {
    processes: Vec<(Process, ProcessReceiver)>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self { processes: vec![] }
    }

    pub fn add(&mut self, receiver: ProcessReceiver, process: Process) {
        debug!("Adding process pid={}", process.pid());
        self.processes.push((process, receiver));
    }

    pub fn register<'a>(&'a self, select: &mut Select<'a>) {
        for (_, receiver) in &self.processes {
            select.recv(receiver);
        }
    }

    pub fn receiver_at(&self, index: usize) -> &ProcessReceiver {
        let (_, receiver) = &self.processes[index];
        receiver
    }

    pub fn remove(&mut self, index: usize) {
        let (mut process, _) = self.processes.swap_remove(index);
        debug!("Removing process pid={}", process.pid());
        process.wait();
    }
}
