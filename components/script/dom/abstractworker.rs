/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use constellation_traits::StructuredSerializedData;
use servo_url::ImmutableOrigin;

use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomObject;
use crate::messaging::CommonScriptMsg;

/// Messages used to control the worker event loops
pub(crate) enum WorkerScriptMsg {
    /// Common variants associated with the script messages
    Common(CommonScriptMsg),
    /// Message sent through Worker.postMessage
    DOMMessage {
        origin: ImmutableOrigin,
        data: Box<StructuredSerializedData>,
    },
}

pub(crate) struct SimpleWorkerErrorHandler<T: DomObject> {
    pub(crate) addr: Trusted<T>,
}

impl<T: DomObject> SimpleWorkerErrorHandler<T> {
    pub(crate) fn new(addr: Trusted<T>) -> SimpleWorkerErrorHandler<T> {
        SimpleWorkerErrorHandler { addr }
    }
}
