/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::abstractworker::WorkerScriptMsg;
use dom::dedicatedworkerglobalscope::DedicatedWorkerScriptMsg;
use dom::worker::TrustedWorkerAddress;
use script_runtime::{ScriptChan, CommonScriptMsg, ScriptPort};
use std::sync::mpsc::{Receiver, Sender};

/// A ScriptChan that can be cloned freely and will silently send a TrustedWorkerAddress with
/// common event loop messages. While this SendableWorkerScriptChan is alive, the associated
/// Worker object will remain alive.
#[derive(Clone, JSTraceable)]
pub struct SendableWorkerScriptChan {
    pub sender: Sender<DedicatedWorkerScriptMsg>,
    pub worker: TrustedWorkerAddress,
}

impl ScriptChan for SendableWorkerScriptChan {
    fn send(&self, msg: CommonScriptMsg) -> Result<(), ()> {
        let msg = DedicatedWorkerScriptMsg::CommonWorker(self.worker.clone(), WorkerScriptMsg::Common(msg));
        self.sender.send(msg).map_err(|_| ())
    }

    fn clone(&self) -> Box<ScriptChan + Send> {
        Box::new(SendableWorkerScriptChan {
            sender: self.sender.clone(),
            worker: self.worker.clone(),
        })
    }
}

/// A ScriptChan that can be cloned freely and will silently send a TrustedWorkerAddress with
/// worker event loop messages. While this SendableWorkerScriptChan is alive, the associated
/// Worker object will remain alive.
#[derive(Clone, JSTraceable)]
pub struct WorkerThreadWorkerChan {
    pub sender: Sender<DedicatedWorkerScriptMsg>,
    pub worker: TrustedWorkerAddress,
}

impl ScriptChan for WorkerThreadWorkerChan {
    fn send(&self, msg: CommonScriptMsg) -> Result<(), ()> {
        let msg = DedicatedWorkerScriptMsg::CommonWorker(self.worker.clone(), WorkerScriptMsg::Common(msg));
        self.sender
            .send(msg)
            .map_err(|_| ())
    }

    fn clone(&self) -> Box<ScriptChan + Send> {
        Box::new(WorkerThreadWorkerChan {
            sender: self.sender.clone(),
            worker: self.worker.clone(),
        })
    }
}

impl ScriptPort for Receiver<DedicatedWorkerScriptMsg> {
    fn recv(&self) -> Result<CommonScriptMsg, ()> {
        let common_msg = match self.recv() {
            Ok(DedicatedWorkerScriptMsg::CommonWorker(_worker, common_msg)) => common_msg,
            Err(_) => return Err(()),
            Ok(DedicatedWorkerScriptMsg::WakeUp) => panic!("unexpected worker event message!")
        };
        match common_msg {
            WorkerScriptMsg::Common(script_msg) => Ok(script_msg),
            WorkerScriptMsg::DOMMessage(_) => panic!("unexpected worker event message!"),
        }
    }
}
