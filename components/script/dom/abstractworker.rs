/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::structuredclone::StructuredCloneData;
use crate::script_runtime::CommonScriptMsg;

/// Messages used to control the worker event loops
pub enum WorkerScriptMsg {
    /// Common variants associated with the script messages
    Common(CommonScriptMsg),
    /// Message sent through Worker.postMessage
    DOMMessage {
        origin: String,
        data: StructuredCloneData,
    }
}

pub struct SimpleWorkerErrorHandler<T: DomObject> {
    pub addr: Trusted<T>,
}

impl<T: DomObject> SimpleWorkerErrorHandler<T> {
    pub fn new(addr: Trusted<T>) -> SimpleWorkerErrorHandler<T> {
        SimpleWorkerErrorHandler { addr: addr }
    }
}
