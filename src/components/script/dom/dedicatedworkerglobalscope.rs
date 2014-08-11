/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DedicatedWorkerGlobalScopeBinding;
use dom::bindings::codegen::InheritTypes::DedicatedWorkerGlobalScopeDerived;
use dom::bindings::codegen::InheritTypes::{EventTargetCast, WorkerGlobalScopeCast};
use dom::bindings::global::Worker;
use dom::bindings::js::{JSRef, Temporary, RootCollection};
use dom::bindings::trace::Untraceable;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::eventtarget::EventTarget;
use dom::eventtarget::WorkerGlobalScopeTypeId;
use dom::messageevent::MessageEvent;
use dom::workerglobalscope::DedicatedGlobalScope;
use dom::workerglobalscope::WorkerGlobalScope;
use dom::xmlhttprequest::XMLHttpRequest;
use script_task::{ScriptTask, ScriptChan, ScriptMsg, DOMMessage, XHRProgressMsg};
use script_task::StackRootTLS;

use servo_net::resource_task::{ResourceTask, load_whole_resource};

use js::rust::Cx;

use std::rc::Rc;
use std::task::TaskBuilder;
use native::task::NativeTaskBuilder;
use url::Url;

#[deriving(Encodable)]
pub struct DedicatedWorkerGlobalScope {
    workerglobalscope: WorkerGlobalScope,
    receiver: Untraceable<Receiver<ScriptMsg>>,
}

impl DedicatedWorkerGlobalScope {
    pub fn new_inherited(worker_url: Url,
                         cx: Rc<Cx>,
                         receiver: Receiver<ScriptMsg>,
                         resource_task: ResourceTask,
                         script_chan: ScriptChan)
                         -> DedicatedWorkerGlobalScope {
        DedicatedWorkerGlobalScope {
            workerglobalscope: WorkerGlobalScope::new_inherited(
                DedicatedGlobalScope, worker_url, cx, resource_task,
                script_chan),
            receiver: Untraceable::new(receiver),
        }
    }

    pub fn new(worker_url: Url,
               cx: Rc<Cx>,
               receiver: Receiver<ScriptMsg>,
               resource_task: ResourceTask,
               script_chan: ScriptChan)
               -> Temporary<DedicatedWorkerGlobalScope> {
        let scope = box DedicatedWorkerGlobalScope::new_inherited(
            worker_url, cx.clone(), receiver, resource_task, script_chan);
        DedicatedWorkerGlobalScopeBinding::Wrap(cx.ptr, scope)
    }
}

impl DedicatedWorkerGlobalScope {
    pub fn run_worker_scope(worker_url: Url,
                            resource_task: ResourceTask) -> ScriptChan {
        let (receiver, sender) = ScriptChan::new();
        let sender_clone = sender.clone();
        TaskBuilder::new()
            .native()
            .named(format!("Web Worker at {}", worker_url.serialize()))
            .spawn(proc() {
            let roots = RootCollection::new();
            let _stack_roots_tls = StackRootTLS::new(&roots);

            let (url, source) = match load_whole_resource(&resource_task, worker_url.clone()) {
                Err(_) => {
                    println!("error loading script {}", worker_url.serialize());
                    return;
                }
                Ok((metadata, bytes)) => {
                    (metadata.final_url, String::from_utf8(bytes).unwrap())
                }
            };

            let (_js_runtime, js_context) = ScriptTask::new_rt_and_cx();
            let global = DedicatedWorkerGlobalScope::new(
                worker_url, js_context.clone(), receiver, resource_task,
                sender).root();
            match js_context.evaluate_script(
                global.reflector().get_jsobject(), source, url.serialize(), 1) {
                Ok(_) => (),
                Err(_) => println!("evaluate_script failed")
            }

            let scope: &JSRef<WorkerGlobalScope> =
                WorkerGlobalScopeCast::from_ref(&*global);
            let target: &JSRef<EventTarget> =
                EventTargetCast::from_ref(&*global);
            loop {
                match global.receiver.recv_opt() {
                    Ok(DOMMessage(message)) => {
                        MessageEvent::dispatch(target, &Worker(*scope), message)
                    },
                    Ok(XHRProgressMsg(addr, progress)) => {
                        XMLHttpRequest::handle_xhr_progress(addr, progress)
                    },
                    Ok(_) => fail!("Unexpected message"),
                    Err(_) => break,
                }
            }
        });
        return sender_clone;
    }
}

pub trait DedicatedWorkerGlobalScopeMethods {
}

impl Reflectable for DedicatedWorkerGlobalScope {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.workerglobalscope.reflector()
    }
}

impl DedicatedWorkerGlobalScopeDerived for EventTarget {
    fn is_dedicatedworkerglobalscope(&self) -> bool {
        match self.type_id {
            WorkerGlobalScopeTypeId(DedicatedGlobalScope) => true,
            _ => false
        }
    }
}
