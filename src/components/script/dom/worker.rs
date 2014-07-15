/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::error::{Fallible, Security, Syntax};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{Temporary, RootCollection};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::dedicatedworkerglobalscope::DedicatedWorkerGlobalScope;
use dom::eventtarget::EventTarget;
use script_task::StackRootTLS;

use servo_net::resource_task::load_whole_resource;
use servo_util::str::DOMString;
use servo_util::url::try_parse_url;

use std::task::TaskBuilder;

#[deriving(Encodable)]
pub struct Worker {
    eventtarget: EventTarget,
}

impl Worker {
    pub fn Constructor(global: &GlobalRef, scriptURL: DOMString) -> Fallible<Temporary<Worker>> {
        // Step 2-4.
        let worker_url = match try_parse_url(scriptURL.as_slice(), Some(global.get_url())) {
            Ok(url) => url,
            Err(_) => return Err(Syntax),
        };

        let name = format!("Web Worker at {}", worker_url);
        let resource_task = global.page().resource_task.deref().clone();
        TaskBuilder::new().named(name).spawn(proc() {
            let roots = RootCollection::new();
            let _stack_roots_tls = StackRootTLS::new(&roots);

            let (filename, source) = match load_whole_resource(&resource_task, worker_url.clone()) {
                Err(_) => {
                    println!("error loading script {}", worker_url);
                    return;
                }
                Ok((metadata, bytes)) => {
                    (metadata.final_url, String::from_utf8(bytes).unwrap())
                }
            };

            let global = DedicatedWorkerGlobalScope::init().root();
            match global.get_rust_cx().evaluate_script(
                global.reflector().get_jsobject(), source, filename.to_str(), 1) {
                Ok(_) => (),
                Err(_) => println!("evaluate_script failed")
            }
        });
        Err(Security)
    }
}

pub trait WorkerMethods {
}

impl Reflectable for Worker {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.eventtarget.reflector()
    }
}
