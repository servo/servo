/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::Reflectable;
use dom::bindings::str::DOMString;
use dom::bindings::structuredclone::StructuredCloneData;
use js::jsapi::{JSRuntime, JS_RequestInterruptCallback};
use js::rust::Runtime;
use msg::constellation_msg::{PipelineId, ReferrerPolicy};
use net_traits::{LoadOrigin, RequestSource};
use script_runtime::CommonScriptMsg;
use url::Url;

/// Messages used to control the worker event loops
pub enum WorkerScriptMsg {
    /// Common variants associated with the script messages
    Common(CommonScriptMsg),
    /// Message sent through Worker.postMessage
    DOMMessage(StructuredCloneData),
}

#[derive(Clone)]
pub struct WorkerScriptLoadOrigin {
    pub referrer_url: Option<Url>,
    pub referrer_policy: Option<ReferrerPolicy>,
    pub request_source: RequestSource,
    pub pipeline_id: Option<PipelineId>
}

impl LoadOrigin for WorkerScriptLoadOrigin {
    fn referrer_url(&self) -> Option<Url> {
        self.referrer_url.clone()
    }
    fn referrer_policy(&self) -> Option<ReferrerPolicy> {
        self.referrer_policy.clone()
    }
    fn request_source(&self) -> RequestSource {
        self.request_source.clone()
    }
    fn pipeline_id(&self) -> Option<PipelineId> {
        self.pipeline_id.clone()
    }
}

pub struct SimpleWorkerErrorHandler<T: Reflectable> {
    pub addr: Trusted<T>,
}

impl<T: Reflectable> SimpleWorkerErrorHandler<T> {
    pub fn new(addr: Trusted<T>) -> SimpleWorkerErrorHandler<T> {
        SimpleWorkerErrorHandler {
            addr: addr
        }
    }
}

pub struct WorkerErrorHandler<T: Reflectable> {
    pub addr: Trusted<T>,
    pub msg: DOMString,
    pub file_name: DOMString,
    pub line_num: u32,
    pub col_num: u32,
}

#[derive(Copy, Clone)]
pub struct SharedRt {
    rt: *mut JSRuntime
}

impl SharedRt {
    pub fn new(rt: &Runtime) -> SharedRt {
        SharedRt {
            rt: rt.rt()
        }
    }

    #[allow(unsafe_code)]
    pub fn request_interrupt(&self) {
        unsafe {
            JS_RequestInterruptCallback(self.rt);
        }
    }
}
#[allow(unsafe_code)]
unsafe impl Send for SharedRt {}
