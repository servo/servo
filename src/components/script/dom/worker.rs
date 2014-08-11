/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::WorkerBinding;
use dom::bindings::codegen::Bindings::WorkerBinding::WorkerMethods;
use dom::bindings::error::{Fallible, Syntax};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::dedicatedworkerglobalscope::DedicatedWorkerGlobalScope;
use dom::eventtarget::{EventTarget, WorkerTypeId};
use script_task::{ScriptChan, DOMMessage};

use servo_util::str::DOMString;
use url::UrlParser;

#[deriving(Encodable)]
pub struct Worker {
    eventtarget: EventTarget,
    sender: ScriptChan,
}

impl Worker {
    pub fn new_inherited(sender: ScriptChan) -> Worker {
        Worker {
            eventtarget: EventTarget::new_inherited(WorkerTypeId),
            sender: sender,
        }
    }

    pub fn new(global: &GlobalRef, sender: ScriptChan) -> Temporary<Worker> {
        reflect_dom_object(box Worker::new_inherited(sender),
                           global,
                           WorkerBinding::Wrap)
    }

    // http://www.whatwg.org/html/#dom-worker
    pub fn Constructor(global: &GlobalRef, scriptURL: DOMString) -> Fallible<Temporary<Worker>> {
        // Step 2-4.
        let worker_url = match UrlParser::new().base_url(&global.get_url())
                .parse(scriptURL.as_slice()) {
            Ok(url) => url,
            Err(_) => return Err(Syntax),
        };

        let resource_task = global.resource_task();
        let sender = DedicatedWorkerGlobalScope::run_worker_scope(
            worker_url, resource_task);
        Ok(Worker::new(global, sender))
    }
}

impl<'a> WorkerMethods for JSRef<'a, Worker> {
    fn PostMessage(&self, message: DOMString) {
        let ScriptChan(ref sender) = self.sender;
        sender.send(DOMMessage(message));
    }
}

impl Reflectable for Worker {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.eventtarget.reflector()
    }
}
