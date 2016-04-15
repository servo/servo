/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::abstractworker::WorkerScriptMsg;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::Reflectable;
use script_runtime::{ScriptChan, CommonScriptMsg, ScriptPort};
use std::sync::mpsc::{Receiver, Sender};

/// A ScriptChan that can be cloned freely and will silently send a TrustedWorkerAddress with
/// common event loop messages. While this SendableWorkerScriptChan is alive, the associated
/// Worker object will remain alive.
#[derive(JSTraceable, Clone)]
pub struct SendableWorkerScriptChan<T: Reflectable> {
    pub sender: Sender<(Trusted<T>, CommonScriptMsg)>,
    pub worker: Trusted<T>,
}

impl<T: Reflectable + 'static> ScriptChan for SendableWorkerScriptChan<T> {
    fn send(&self, msg: CommonScriptMsg) -> Result<(), ()> {
        self.sender.send((self.worker.clone(), msg)).map_err(|_| ())
    }

    fn clone(&self) -> Box<ScriptChan + Send> {
        box SendableWorkerScriptChan {
            sender: self.sender.clone(),
            worker: self.worker.clone(),
        }
    }
}

/// A ScriptChan that can be cloned freely and will silently send a TrustedWorkerAddress with
/// worker event loop messages. While this SendableWorkerScriptChan is alive, the associated
/// Worker object will remain alive.
#[derive(JSTraceable, Clone)]
pub struct WorkerThreadWorkerChan<T: Reflectable> {
    pub sender: Sender<(Trusted<T>, WorkerScriptMsg)>,
    pub worker: Trusted<T>,
}

impl<T: Reflectable + 'static> ScriptChan for WorkerThreadWorkerChan<T> {
    fn send(&self, msg: CommonScriptMsg) -> Result<(), ()> {
        self.sender
            .send((self.worker.clone(), WorkerScriptMsg::Common(msg)))
            .map_err(|_| ())
    }

    fn clone(&self) -> Box<ScriptChan + Send> {
        box WorkerThreadWorkerChan {
            sender: self.sender.clone(),
            worker: self.worker.clone(),
        }
    }
}

impl<T: Reflectable> ScriptPort for Receiver<(Trusted<T>, WorkerScriptMsg)> {
    fn recv(&self) -> Result<CommonScriptMsg, ()> {
        match self.recv().map(|(_, msg)| msg) {
            Ok(WorkerScriptMsg::Common(script_msg)) => Ok(script_msg),
            Ok(WorkerScriptMsg::DOMMessage(_)) => panic!("unexpected worker event message!"),
            Err(_) => Err(()),
        }
    }
}
