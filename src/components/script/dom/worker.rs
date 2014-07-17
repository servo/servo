/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::WorkerBinding;
use dom::bindings::error::{Fallible, Syntax};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Temporary;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::dedicatedworkerglobalscope::DedicatedWorkerGlobalScope;
use dom::eventtarget::{EventTarget, WorkerTypeId};

use servo_util::str::DOMString;
use servo_util::url::try_parse_url;

#[deriving(Encodable)]
pub struct Worker {
    eventtarget: EventTarget,
}

impl Worker {
    pub fn new_inherited() -> Worker {
        Worker {
            eventtarget: EventTarget::new_inherited(WorkerTypeId),
        }
    }

    pub fn new(global: &GlobalRef) -> Temporary<Worker> {
        reflect_dom_object(box Worker::new_inherited(),
                           global,
                           WorkerBinding::Wrap)
    }

    // http://www.whatwg.org/html/#dom-worker
    pub fn Constructor(global: &GlobalRef, scriptURL: DOMString) -> Fallible<Temporary<Worker>> {
        // Step 2-4.
        let worker_url = match try_parse_url(scriptURL.as_slice(), Some(global.get_url())) {
            Ok(url) => url,
            Err(_) => return Err(Syntax),
        };

        let resource_task = global.resource_task();
        DedicatedWorkerGlobalScope::run_worker_scope(worker_url, resource_task, global.script_chan().clone());
        Ok(Worker::new(global))
    }
}

pub trait WorkerMethods {
}

impl Reflectable for Worker {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.eventtarget.reflector()
    }
}
