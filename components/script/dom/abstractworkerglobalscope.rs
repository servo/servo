/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::abstractworker::WorkerScriptMsg;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::DomObject;
use dom::bindings::trace::JSTraceable;
use script_runtime::{ScriptChan, CommonScriptMsg, ScriptPort};
use std::sync::mpsc::{Receiver, Sender};

/// A ScriptChan that can be cloned freely and will silently send a TrustedWorkerAddress with
/// common event loop messages. While this SendableWorkerScriptChan is alive, the associated
/// Worker object will remain alive.
#[derive(Clone, JSTraceable)]
pub struct SendableWorkerScriptChan<T: DomObject> {
    pub sender: Sender<(Trusted<T>, CommonScriptMsg)>,
    pub worker: Trusted<T>,
}

impl<T: JSTraceable + DomObject + 'static> ScriptChan for SendableWorkerScriptChan<T> {
    fn send(&self, msg: CommonScriptMsg) -> Result<(), ()> {
        self.sender.send((self.worker.clone(), msg)).map_err(|_| ())
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
pub struct WorkerThreadWorkerChan<T: DomObject> {
    pub sender: Sender<(Trusted<T>, WorkerScriptMsg)>,
    pub worker: Trusted<T>,
}

impl<T: JSTraceable + DomObject + 'static> ScriptChan for WorkerThreadWorkerChan<T> {
    fn send(&self, msg: CommonScriptMsg) -> Result<(), ()> {
        self.sender
            .send((self.worker.clone(), WorkerScriptMsg::Common(msg)))
            .map_err(|_| ())
    }

    fn clone(&self) -> Box<ScriptChan + Send> {
        Box::new(WorkerThreadWorkerChan {
            sender: self.sender.clone(),
            worker: self.worker.clone(),
        })
    }
}

impl<T: DomObject> ScriptPort for Receiver<(Trusted<T>, WorkerScriptMsg)> {
    fn recv(&self) -> Result<CommonScriptMsg, ()> {
        match self.recv().map(|(_, msg)| msg) {
            Ok(WorkerScriptMsg::Common(script_msg)) => Ok(script_msg),
            Ok(WorkerScriptMsg::DOMMessage(_)) => panic!("unexpected worker event message!"),
            Err(_) => Err(()),
        }
    }
}
