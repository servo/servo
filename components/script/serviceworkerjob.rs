/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A Job is an abstraction of async operation in service worker lifecycle propagation.
//! Each Job is uniquely identified by its scope_url, and is keyed accordingly under
//! the script thread. The script thread contains a JobQueue, which stores all scheduled Jobs
//! by multiple service worker clients in a Vec.

use dom::bindings::cell::DOMRefCell;
use dom::bindings::error::Error;
use dom::bindings::js::JS;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::DomObject;
use dom::client::Client;
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use dom::serviceworkerregistration::ServiceWorkerRegistration;
use dom::urlhelper::UrlHelper;
use script_thread::{ScriptThread, Runnable};
use servo_url::ServoUrl;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(PartialEq, Clone, Debug, JSTraceable)]
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
    Register,
    Update
}

#[must_root]
#[derive(JSTraceable)]
pub struct Job {
    pub job_type: JobType,
    pub scope_url: ServoUrl,
    pub script_url: ServoUrl,
    pub promise: Rc<Promise>,
    pub equivalent_jobs: Vec<Job>,
    // client can be a window client, worker client so `Client` will be an enum in future
    pub client: JS<Client>,
    pub referrer: ServoUrl
}

impl Job {
    #[allow(unrooted_must_root)]
    // https://w3c.github.io/ServiceWorker/#create-job-algorithm
    pub fn create_job(job_type: JobType,
                      scope_url: ServoUrl,
                      script_url: ServoUrl,
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
    pub scope_url: ServoUrl,
    pub global: Trusted<GlobalScope>,
}

impl FinishJobHandler {
    pub fn new(scope_url: ServoUrl, global: Trusted<GlobalScope>) -> FinishJobHandler {
        FinishJobHandler {
            scope_url: scope_url,
            global: global
        }
    }
}

impl Runnable for FinishJobHandler {
    fn main_thread_handler(self: Box<FinishJobHandler>, script_thread: &ScriptThread) {
        script_thread.invoke_finish_job(self);
    }
}

pub struct AsyncJobHandler {
    pub scope_url: ServoUrl,
    pub invoke_type: InvokeType
}

impl AsyncJobHandler {
    fn new(scope_url: ServoUrl, invoke_type: InvokeType) -> AsyncJobHandler {
        AsyncJobHandler {
            scope_url: scope_url,
            invoke_type: invoke_type
        }
    }
}

impl Runnable for AsyncJobHandler {
    #[allow(unrooted_must_root)]
    fn main_thread_handler(self: Box<AsyncJobHandler>, script_thread: &ScriptThread) {
        script_thread.dispatch_job_queue(self);
    }
}

#[must_root]
#[derive(JSTraceable)]
pub struct JobQueue(pub DOMRefCell<HashMap<ServoUrl, Vec<Job>>>);

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
        let mut queue_ref = self.0.borrow_mut();
        let job_queue = queue_ref.entry(job.scope_url.clone()).or_insert(vec![]);
        // Step 1
        if job_queue.is_empty() {
            let scope_url = job.scope_url.clone();
            job_queue.push(job);
            let run_job_handler = AsyncJobHandler::new(scope_url, InvokeType::Run);
            script_thread.queue_serviceworker_job(box run_job_handler, global);
        } else {
            // Step 2
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
        let handler = *run_job_handler;
        match front_job.job_type {
            JobType::Register => {
                let register_job_handler = AsyncJobHandler::new(handler.scope_url, InvokeType::Register);
                script_thread.queue_serviceworker_job(box register_job_handler, global);
            },
            JobType::Update => {
                let update_job_handler = AsyncJobHandler::new(handler.scope_url, InvokeType::Update);
                script_thread.queue_serviceworker_job(box update_job_handler, global);
            }
            _ => { /* TODO implement Unregister */ }
        }
    }

    #[allow(unrooted_must_root)]
    // https://w3c.github.io/ServiceWorker/#register-algorithm
    pub fn run_register(&self, job: &Job, register_job_handler: Box<AsyncJobHandler>, script_thread: &ScriptThread) {
        let global = &*job.client.global();
        let AsyncJobHandler { scope_url, .. } = *register_job_handler;
        // Step 1-3
        if !UrlHelper::is_origin_trustworthy(&job.script_url) {
            let settle_type = SettleType::Reject(Error::Type("Invalid script ServoURL".to_owned()));
            let async_job_handler = AsyncJobHandler::new(scope_url, InvokeType::Settle(settle_type));
            return script_thread.queue_serviceworker_job(box async_job_handler, global);
        } else if job.script_url.origin() != job.referrer.origin() || job.scope_url.origin() != job.referrer.origin() {
            let settle_type = SettleType::Reject(Error::Security);
            let async_job_handler = AsyncJobHandler::new(scope_url, InvokeType::Settle(settle_type));
            return script_thread.queue_serviceworker_job(box async_job_handler, global);
        }
        // Step 4-5
        if let Some(reg) = script_thread.handle_get_registration(&job.scope_url) {
            // Step 5.1
            if reg.get_uninstalling() {
                reg.set_uninstalling(false);
            }
            // Step 5.3
            if let Some(ref newest_worker) = reg.get_newest_worker() {
                if (&*newest_worker).get_script_url() == job.script_url {
                    let settle_type = SettleType::Resolve(Trusted::new(&*reg));
                    let async_job_handler = AsyncJobHandler::new(scope_url, InvokeType::Settle(settle_type));
                    script_thread.queue_serviceworker_job(box async_job_handler, global);
                    let finish_job_handler = box FinishJobHandler::new(job.scope_url.clone(), Trusted::new(&*global));
                    script_thread.queue_finish_job(finish_job_handler, &*global);
                }
            }
        } else {
            // Step 6.1
            let pipeline = global.pipeline_id();
            let new_reg = ServiceWorkerRegistration::new(&*global, &job.script_url, scope_url);
            script_thread.handle_serviceworker_registration(&job.scope_url, &*new_reg, pipeline);
        }
        // Step 7
        script_thread.invoke_job_update(job, &*global);
    }

    #[allow(unrooted_must_root)]
    // https://w3c.github.io/ServiceWorker/#finish-job-algorithm
    pub fn finish_job(&self, scope_url: ServoUrl, global: &GlobalScope, script_thread: &ScriptThread) {
        if let Some(job_vec) = (*self.0.borrow_mut()).get_mut(&scope_url) {
            if job_vec.first().map_or(false, |job| job.scope_url == scope_url) {
                let _  = job_vec.remove(0);
            }
            if !job_vec.is_empty() {
                let run_job_handler = AsyncJobHandler::new(scope_url, InvokeType::Run);
                script_thread.queue_serviceworker_job(box run_job_handler, global);
            }
        } else {
            warn!("non-existent job vector for Servourl: {:?}", scope_url);
        }
    }

    // https://w3c.github.io/ServiceWorker/#update-algorithm
    pub fn update(&self, job: &Job, global: &GlobalScope, script_thread: &ScriptThread) {
        let reg = match script_thread.handle_get_registration(&job.scope_url) {
            Some(reg) => reg,
            None => return
        };
        // Step 1
        if reg.get_uninstalling() {
            let err_type = Error::Type("Update called on an uninstalling registration".to_owned());
            let settle_type = SettleType::Reject(err_type);
            let async_job_handler = AsyncJobHandler::new(job.scope_url.clone(), InvokeType::Settle(settle_type));
            return script_thread.queue_serviceworker_job(box async_job_handler, global);
        }
        let newest_worker = match reg.get_newest_worker() {
            Some(worker) => worker,
            None => return
        };
        // Step 2
        if (&*newest_worker).get_script_url() == job.script_url  && job.job_type == JobType::Update {
            // Step 4
            let err_type = Error::Type("Invalid script ServoURL".to_owned());
            let settle_type = SettleType::Reject(err_type);
            let async_job_handler = AsyncJobHandler::new(job.scope_url.clone(), InvokeType::Settle(settle_type));
            script_thread.queue_serviceworker_job(box async_job_handler, global);
        } else {
            job.client.set_controller(&*newest_worker);
            let settle_type = SettleType::Resolve(Trusted::new(&*reg));
            let async_job_handler = AsyncJobHandler::new(job.scope_url.clone(), InvokeType::Settle(settle_type));
            script_thread.queue_serviceworker_job(box async_job_handler, global);
        }
        let finish_job_handler = box FinishJobHandler::new(job.scope_url.clone(), Trusted::new(global));
        script_thread.queue_finish_job(finish_job_handler, global);
    }
}
