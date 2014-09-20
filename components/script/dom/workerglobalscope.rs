/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::WorkerGlobalScopeBinding::WorkerGlobalScopeMethods;
use dom::bindings::error::{ErrorResult, Fallible, Syntax, Network, FailureUnknown};
use dom::bindings::trace::Untraceable;
use dom::bindings::global;
use dom::bindings::js::{JS, JSRef, Temporary, OptionalSettable};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::console::Console;
use dom::eventtarget::{EventTarget, WorkerGlobalScopeTypeId};
use dom::workerlocation::WorkerLocation;
use dom::workernavigator::WorkerNavigator;
use dom::window::{base64_atob, base64_btoa};
use script_task::ScriptChan;

use servo_net::resource_task::{ResourceTask, load_whole_resource};
use servo_util::str::DOMString;

use js::jsapi::JSContext;
use js::rust::Cx;

use std::cell::Cell;
use std::rc::Rc;
use url::{Url, UrlParser};

#[deriving(PartialEq,Encodable)]
pub enum WorkerGlobalScopeId {
    DedicatedGlobalScope,
}

#[deriving(Encodable)]
#[must_root]
pub struct WorkerGlobalScope {
    pub eventtarget: EventTarget,
    worker_url: Untraceable<Url>,
    js_context: Untraceable<Rc<Cx>>,
    resource_task: Untraceable<ResourceTask>,
    script_chan: ScriptChan,
    location: Cell<Option<JS<WorkerLocation>>>,
    navigator: Cell<Option<JS<WorkerNavigator>>>,
    console: Cell<Option<JS<Console>>>,
}

impl WorkerGlobalScope {
    pub fn new_inherited(type_id: WorkerGlobalScopeId,
                         worker_url: Url,
                         cx: Rc<Cx>,
                         resource_task: ResourceTask,
                         script_chan: ScriptChan) -> WorkerGlobalScope {
        WorkerGlobalScope {
            eventtarget: EventTarget::new_inherited(WorkerGlobalScopeTypeId(type_id)),
            worker_url: Untraceable::new(worker_url),
            js_context: Untraceable::new(cx),
            resource_task: Untraceable::new(resource_task),
            script_chan: script_chan,
            location: Cell::new(None),
            navigator: Cell::new(None),
            console: Cell::new(None),
        }
    }

    pub fn get_cx(&self) -> *mut JSContext {
        self.js_context.ptr
    }

    pub fn resource_task<'a>(&'a self) -> &'a ResourceTask {
        &*self.resource_task
    }

    pub fn get_url<'a>(&'a self) -> &'a Url {
        &*self.worker_url
    }

    pub fn script_chan<'a>(&'a self) -> &'a ScriptChan {
        &self.script_chan
    }
}

impl<'a> WorkerGlobalScopeMethods for JSRef<'a, WorkerGlobalScope> {
    fn Self(&self) -> Temporary<WorkerGlobalScope> {
        Temporary::from_rooted(*self)
    }

    fn Location(&self) -> Temporary<WorkerLocation> {
        if self.location.get().is_none() {
            let location = WorkerLocation::new(*self, self.worker_url.deref().clone());
            self.location.assign(Some(location));
        }
        Temporary::new(self.location.get().get_ref().clone())
    }

    fn ImportScripts(&self, url_strings: Vec<DOMString>) -> ErrorResult {
        let mut urls = Vec::with_capacity(url_strings.len());
        for url in url_strings.move_iter() {
            let url = UrlParser::new().base_url(&*self.worker_url)
                                      .parse(url.as_slice());
            match url {
                Ok(url) => urls.push(url),
                Err(_) => return Err(Syntax),
            };
        }

        for url in urls.move_iter() {
            let (url, source) = match load_whole_resource(&*self.resource_task, url) {
                Err(_) => return Err(Network),
                Ok((metadata, bytes)) => {
                    (metadata.final_url, String::from_utf8(bytes).unwrap())
                }
            };

            match self.js_context.deref().evaluate_script(
                self.reflector().get_jsobject(), source, url.serialize(), 1) {
                Ok(_) => (),
                Err(_) => {
                    println!("evaluate_script failed");
                    return Err(FailureUnknown);
                }
            }
        }

        Ok(())
    }

    fn Navigator(&self) -> Temporary<WorkerNavigator> {
        if self.navigator.get().is_none() {
            let navigator = WorkerNavigator::new(*self);
            self.navigator.assign(Some(navigator));
        }
        Temporary::new(self.navigator.get().get_ref().clone())
    }

    fn Console(&self) -> Temporary<Console> {
        if self.console.get().is_none() {
            let console = Console::new(&global::Worker(*self));
            self.console.assign(Some(console));
        }
        Temporary::new(self.console.get().get_ref().clone())
    }

    fn Btoa(&self, btoa: DOMString) -> Fallible<DOMString> {
        base64_btoa(btoa)
    }

    fn Atob(&self, atob: DOMString) -> Fallible<DOMString> {
        base64_atob(atob)
    }
}

impl Reflectable for WorkerGlobalScope {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.eventtarget.reflector()
    }
}
