/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::error::Error;
use dom::bindings::js::JS;
use dom::bindings::refcounted::{Trusted, TrustedPromise};
use dom::bindings::reflector::Reflectable;
use dom::client::Client;
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use dom::serviceworkerregistration::ServiceWorkerRegistration;
use dom::urlhelper::UrlHelper;
use js::jsapi::JSAutoCompartment;
use script_thread::{ScriptThread, Runnable};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::rc::Rc;
use url::Url;

/// A Job is an abstraction of async operation in service worker lifecycle propagation.
/// Each Job is uniquely identified by its scope_url, and is keyed by accordingly under
/// the script thread. The script thread contains a JobQueue, which stores all scheduled Jobs
/// by multiple service worker clients in a Vec.

#[derive(PartialEq, Clone, Debug)]
pub enum JobType {
    Register,
    Unregister,
    Update
}


#[derive(Clone)]
pub enum SettleType {
    Resolve(Trusted<ServiceWorkerRegistration>),
    Reject(Error)
}

// This encapsulates what operation to invoke of JobQueue from script thread
pub enum InvokeType {
    Settle(SettleType),
    Run,
    Register
}

#[must_root]
pub struct Job {
    pub job_type: JobType,
    pub scope_url: Url,
    pub script_url: Url,
    pub promise: Rc<Promise>,
    pub equivalent_jobs: Vec<Job>,
    // client can be a window client, worker client so `Client` will be an enum in future
    pub client: JS<Client>,
    pub referrer: Url
}

impl Job {
    #[allow(unrooted_must_root)]
    // https://w3c.github.io/ServiceWorker/#create-job-algorithm
    pub fn create_job(job_type: JobType,
                      scope_url: Url,
                      script_url: Url,
                      promise: Rc<Promise>,
                      client: &Client) -> Job {
        Job {
            job_type: job_type,
            scope_url: scope_url,
            script_url: script_url,
            promise: promise,
            equivalent_jobs: vec![],
            client: JS::from_ref(client),
            referrer: client.creation_url()
        }
    }
    #[allow(unrooted_must_root)]
    pub fn append_equivalent_job(&mut self, job: Job) {
        self.equivalent_jobs.push(job);
    }
}

impl PartialEq for Job {
    // Equality criteria as described in https://w3c.github.io/ServiceWorker/#dfn-job-equivalent
    fn eq(&self, other: &Self) -> bool {
        let same_job = self.job_type == other.job_type;
        if same_job {
            match self.job_type {
                JobType::Register | JobType::Update => {
                    self.scope_url == other.scope_url && self.script_url == other.script_url
                },
                JobType::Unregister => self.scope_url == other.scope_url
            }
        } else {
            false
        }
    }
}

pub struct FinishJobHandler {
    pub scope_url: Url,
    pub global: Trusted<GlobalScope>,
}

impl Runnable for FinishJobHandler {
    fn main_thread_handler(self: Box<FinishJobHandler>, script_thread: &ScriptThread) {
        script_thread.invoke_finish_job(self);
    }
}

pub struct AsyncJobHandler {
    pub scope_url: Url,
    pub invoke_type: InvokeType
}

impl AsyncJobHandler {
    fn new(scope_url: Url, invoke_type: InvokeType) -> AsyncJobHandler {
        AsyncJobHandler {
            scope_url: scope_url,
            invoke_type: invoke_type
        }
    }
}

impl Runnable for AsyncJobHandler {
    #[allow(unrooted_must_root)]
    fn main_thread_handler(self: Box<AsyncJobHandler>, script_thread: &ScriptThread) {
        use self::InvokeType::*;
        match self.invoke_type {
            Run => script_thread.invoke_run_job(self),
            Register => script_thread.invoke_register_job(self),
            _ => script_thread.invoke_promise_job(self)
        }
    }
}

#[must_root]
pub struct JobQueue(pub DOMRefCell<HashMap<Url, Vec<Job>>>);

impl JobQueue {
    pub fn new() -> JobQueue {
        JobQueue(DOMRefCell::new(HashMap::new()))
    }
    #[allow(unrooted_must_root)]
    // https://w3c.github.io/ServiceWorker/#schedule-job-algorithm
    pub fn schedule_job(&self,
                        job: Job,
                        global: &GlobalScope,
                        script_thread: &ScriptThread) {
        let queue_ref = &mut *self.0.borrow_mut();
        let job_queue = queue_ref.entry(job.scope_url.clone()).or_insert(vec![]);
        // Step 1
        if job_queue.is_empty() {
            let scope_url = job.scope_url.clone();
            job_queue.push(job);
            let run_job_handler = AsyncJobHandler::new(scope_url, InvokeType::Run);
            // queue task for https://w3c.github.io/ServiceWorker/#run-job-algorithm
            script_thread.queue_run_job(box run_job_handler, global);
        } else {
            // or Step 2
            // below lines doesn't look very appropriate, but lifetimes forces me to do it this way.
            // are there ways to get mutable ref to last_job of job_queue ?
            let mut last_job = job_queue.pop().unwrap();
            if job == last_job && !last_job.promise.is_settled() {
                last_job.append_equivalent_job(job);
                job_queue.push(last_job);
            } else {
                // restore the popped last_job
                job_queue.push(last_job);
                // and push this new job to job queue
                job_queue.push(job);
            }
        }
    }

    #[allow(unrooted_must_root)]
    // https://w3c.github.io/ServiceWorker/#run-job-algorithm
    pub fn run_job(&self, run_job_handler: Box<AsyncJobHandler>, script_thread: &ScriptThread) {
        let queue_ref = &*self.0.borrow();
        let front_job =  {
            let job_vec = queue_ref.get(&run_job_handler.scope_url);
            job_vec.unwrap().first().unwrap()
        };
        let global = &*front_job.client.global();
        match front_job.job_type {
            JobType::Register => {
                let handler = *run_job_handler;
                let register_job_handler = AsyncJobHandler::new(handler.scope_url, InvokeType::Register);
                script_thread.queue_register_job(box register_job_handler, global);
            },
            _ => {}
        }
    }

    #[allow(unrooted_must_root)]
    pub fn settle_promise_job(&self, async_job_handler: Box<AsyncJobHandler>, script_thread: &ScriptThread) {
        use self::InvokeType::*;
        let queue_ref = &*self.0.borrow();
        let front_job =  {
            let job_vec = queue_ref.get(&async_job_handler.scope_url);
            job_vec.unwrap().first().unwrap()
        };
        let handler = *async_job_handler;
        let scope_url = front_job.scope_url.clone();
        let invoke_type = handler.invoke_type;
        let global = &*front_job.client.global();
        let trusted_global = Trusted::new(global);
        let promise = &front_job.promise;
        let _ac = JSAutoCompartment::new(global.get_cx(), promise.reflector().get_jsobject().get());
        match invoke_type {
            Settle(SettleType::Resolve(reg)) => {
                let reg = reg.root();
                promise.resolve_native(global.get_cx(), &*reg);
                let finish_job_handler = box FinishJobHandler {
                    scope_url: scope_url,
                    global: trusted_global
                };
                script_thread.queue_finish_job(finish_job_handler, global);
            }
            Settle(SettleType::Reject(err_type)) => {
                promise.reject_error(global.get_cx(), err_type);
                let finish_job_handler = box FinishJobHandler {
                    scope_url: scope_url,
                    global: trusted_global
                };
                script_thread.queue_finish_job(finish_job_handler, global);
            }
            _ => warn!("other variants already matched before")
        }
    }

    #[allow(unrooted_must_root)]
    // https://w3c.github.io/ServiceWorker/#register-algorithm
    pub fn run_register(&self, job: &Job, register_job_handler: Box<AsyncJobHandler>, script_thread: &ScriptThread) {
        let global = &*job.client.global();
        let scope_url = register_job_handler.scope_url.clone();
        // Step 1-3
        if !UrlHelper::is_origin_trustworthy(&job.script_url) {
            let settle_type = SettleType::Reject(Error::Type("Invalid script URL".to_owned()));
            let async_job_handler = AsyncJobHandler::new(scope_url, InvokeType::Settle(settle_type));
            return script_thread.queue_async_job(box async_job_handler, global);
        } else if job.script_url.origin() != job.referrer.origin() || job.scope_url.origin() != job.referrer.origin() {
            let settle_type = SettleType::Reject(Error::Security);
            let async_job_handler = AsyncJobHandler::new(scope_url, InvokeType::Settle(settle_type));
            return script_thread.queue_async_job(box async_job_handler, global);
        }
        // Step 4
        if let Some(reg) = script_thread.handle_get_registration(&job.scope_url) {
            // Step 5.1
            if reg.get_uninstalling() {
                reg.set_uninstalling(false);
            }
            // Step 5.3
            if let Some(ref newest_worker) = reg.get_newest_worker() {
                if (&*newest_worker).get_script_url() == job.script_url {
                    let handler: AsyncJobHandler = *register_job_handler;
                    let settle_type = SettleType::Resolve(Trusted::new(&*reg));
                    let async_job_handler = AsyncJobHandler::new(handler.scope_url, InvokeType::Settle(settle_type));
                    script_thread.queue_async_job(box async_job_handler, global);
                    let finish_job_handler = box FinishJobHandler {
                        scope_url: job.scope_url.clone(),
                        global: Trusted::new(&*global)
                    };
                    script_thread.queue_finish_job(finish_job_handler, &*global);
                }
            }
        } else {
            // Step 6.1
            let pipeline = global.pipeline_id();
            let handler: AsyncJobHandler = *register_job_handler;
            let new_reg = ServiceWorkerRegistration::new(&*global,
                                                         job.script_url.clone(),
                                                         handler.scope_url.clone());
            script_thread.handle_serviceworker_registration(job.scope_url.clone(), &*new_reg, pipeline);
            // TODO Do not resolve the promise here but do it in update algorithm
            let settle_type = SettleType::Resolve(Trusted::new(&*new_reg));
            let async_job_handler = AsyncJobHandler::new(handler.scope_url, InvokeType::Settle(settle_type));
            script_thread.queue_async_job(box async_job_handler, &*global);
        }
        script_thread.invoke_job_update(job, &*global);
    }

    #[allow(unrooted_must_root)]
    // https://w3c.github.io/ServiceWorker/#finish-job-algorithm
    pub fn finish_job(&self, scope_url: Url, global: &GlobalScope, script_thread: &ScriptThread) {
        if let Some(job_vec) = (*self.0.borrow_mut()).get_mut(&scope_url) {
            // first conditional causes an early return avoiding the unwrap panic in second conditional
            if job_vec.first().is_some() && job_vec.first().unwrap().scope_url == scope_url {
                let _  = job_vec.remove(0);
            }
            if !job_vec.is_empty() {
                let run_job_handler = AsyncJobHandler::new(scope_url.clone(), InvokeType::Run);
                script_thread.queue_run_job(box run_job_handler, global);
            }
        } else {
            warn!("non-existent job vector for url: {:?}", scope_url);
        }
    }
    // https://w3c.github.io/ServiceWorker/#update-algorithm
    pub fn update(&self, job: &Job, global: &GlobalScope, script_thread: &ScriptThread) {
        if let Some(reg) = script_thread.handle_get_registration(&job.scope_url) {
            // Step 1
            if reg.get_uninstalling() {
                let err_type = Error::Type("Update called on an uninstalling registration".to_owned());
                let settle_type = SettleType::Reject(err_type);
                let async_job_handler = AsyncJobHandler::new(job.scope_url.clone(), InvokeType::Settle(settle_type));
                return script_thread.queue_async_job(box async_job_handler, global);
            }
            // Step 2
            if let Some(ref newest_worker) = reg.get_newest_worker() {
                if (&*newest_worker).get_script_url() == job.script_url /* && job.job_type == JobType::Update*/{
                    job.client.set_controller(&*newest_worker);
                    let settle_type = SettleType::Resolve(Trusted::new(&*reg));
                    let scope_url = job.scope_url.clone();
                    let async_job_handler = AsyncJobHandler::new(scope_url, InvokeType::Settle(settle_type));
                    script_thread.queue_async_job(box async_job_handler, global);
                    let finish_job_handler = box FinishJobHandler {
                        scope_url: job.scope_url.clone(),
                        global: Trusted::new(global)
                    };
                    script_thread.queue_finish_job(finish_job_handler, global);
                } else {
                    warn!("Update: Workers script url and job's script_url does not match");
                }
            }
            // TODO improve this update method.
        }
    }
}
