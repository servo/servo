/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::error::Error;
use dom::bindings::refcounted::{Trusted, TrustedPromise};
use dom::bindings::reflector::Reflectable;
use dom::bindings::trace::JSTraceable;
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

#[must_root]
pub struct Job {
    pub job_type: JobType,
    pub scope_url: Url,
    pub script_url: Url,
    pub promise: Rc<Promise>,
    pub equivalent_jobs: Vec<Job>,
    // client can be a window client, worker client so `Client` will be an enum in future
    pub client: Trusted<Client>,
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
            client: Trusted::new(client),
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
            return match self.job_type {
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

pub struct RunJobHandler {
    pub reg: Trusted<ServiceWorkerRegistration>,
    pub promise: TrustedPromise,
    pub scope_url: Url
}

impl Runnable for RunJobHandler {
    #[allow(unrooted_must_root)]
    fn main_thread_handler(self: Box<RunJobHandler>, script_thread: &ScriptThread) {
        script_thread.invoke_run_job(self);
    }
}

pub struct AsyncJobHandler {
    pub promise: TrustedPromise,
    pub scope_url: Url,
    pub settle_type: SettleType,
    pub global: Trusted<GlobalScope>
}

impl Runnable for AsyncJobHandler {
    #[allow(unrooted_must_root)]
    fn main_thread_handler(self: Box<AsyncJobHandler>, script_thread: &ScriptThread) {
        let settle_type = self.settle_type.clone();
        let global = self.global.root();
        let promise = self.promise.root();
        let _ac = JSAutoCompartment::new((&*global).get_cx(), promise.reflector().get_jsobject().get());
        match settle_type {
            SettleType::Resolve(reg) => {
                let reg = reg.root();
                promise.resolve_native((&*global).get_cx(), &*reg);
            }
            SettleType::Reject(err_type) => {
                promise.reject_error((&*global).get_cx(), err_type);
            }
        }
    }
}

#[must_root]
pub struct JobQueue(pub HashMap<Url, Vec<Job>>);

impl JobQueue {
    pub fn new() -> JobQueue {
        JobQueue(HashMap::new())
    }
    #[allow(unrooted_must_root)]
    // https://w3c.github.io/ServiceWorker/#schedule-job-algorithm
    pub fn schedule_job(&mut self, job: Job,
                           reg: &ServiceWorkerRegistration,
                           script_thread: &ScriptThread) {
        let promise = job.promise.clone();
        if !self.0.contains_key(&job.scope_url) {
            self.0.insert(job.scope_url.clone(), vec![]);
        }
        let ref mut job_queue = match self.0.get_mut(&job.scope_url) {
            Some(r) => r,
            None => return
        };
        // Step 1
        if job_queue.is_empty() {
            let scope_url = job.scope_url.clone();
            job_queue.push(job);
            let run_job_handler = RunJobHandler {
                reg: Trusted::new(reg),
                promise: TrustedPromise::new(promise),
                scope_url: scope_url
            };
            // queue task for https://w3c.github.io/ServiceWorker/#run-job-algorithm
            script_thread.queue_run_job(box run_job_handler, &*reg.global());
        } else {
            // or Step 2
            let mut last_job = job_queue.pop().unwrap();
            if job == last_job && !last_job.promise.is_settled() {
                last_job.append_equivalent_job(job);
                job_queue.push(last_job);
            } else {
                job_queue.push(last_job);
                job_queue.push(job);
            }
        }
    }

    #[allow(unrooted_must_root)]
    // https://w3c.github.io/ServiceWorker/#run-job-algorithm
    pub fn run_job(&mut self, run_job_handler: Box<RunJobHandler>, script_thread: &ScriptThread) {
        let scope_url = run_job_handler.scope_url.clone();
        let front_job =  {
            let job_vec = self.0.get_mut(&scope_url);
            job_vec.unwrap().remove(0)
        };

        match front_job.job_type {
            JobType::Register => {
                self.run_register(&front_job, run_job_handler, script_thread);
            },
            _ => {}
        }
    }

    #[allow(unrooted_must_root)]
    // https://w3c.github.io/ServiceWorker/#register-algorithm
    pub fn run_register(&mut self, job: &Job, run_job_handler: Box<RunJobHandler>, script_thread: &ScriptThread) {
        // Step 1
        if !UrlHelper::is_origin_trustworthy(&job.script_url) {
            let reg = run_job_handler.reg.root();
            let global = reg.global();
            let scope_url = run_job_handler.scope_url.clone();
            let async_job_handler = AsyncJobHandler {
                promise: TrustedPromise::new(run_job_handler.promise.root()),
                scope_url: scope_url,
                settle_type: SettleType::Reject(Error::Type("Invalid script URL".to_owned())),
                global: Trusted::new(&*global)
            };
            return script_thread.queue_async_job(box async_job_handler);
        }
        // Step 2-3 TODO
        // Step 4
        let reg = ScriptThread::get_registration(&job.scope_url);
        let new_reg = &*run_job_handler.reg.root();
        let global = new_reg.global();
        if reg.is_some() {
            let reg = reg.unwrap();
            if reg.get_uninstalling() {
                reg.set_uninstalling(false);
            }
            let newest_worker = reg.get_newest_worker();
            if let Some(ref newest_worker) = reg.get_newest_worker() {
                if (&*newest_worker).get_script_url() == job.script_url {
                    let reg = run_job_handler.reg.clone();
                    let scope_url = run_job_handler.scope_url.clone();
                    let promise = run_job_handler.promise.root();
                    let async_job_handler = AsyncJobHandler {
                        promise: TrustedPromise::new(promise),
                        scope_url: scope_url,
                        settle_type: SettleType::Resolve(reg),
                        global: Trusted::new(&*global)
                    };
                    script_thread.queue_async_job(box async_job_handler);
                }
            }
        } else {
            let pipeline = global.pipeline_id();
            ScriptThread::set_registration(job.scope_url.clone(), &*new_reg, pipeline);
            let scope_url = run_job_handler.scope_url.clone();
            let promise = run_job_handler.promise.root();
            let async_job_handler = AsyncJobHandler {
                        promise: TrustedPromise::new(promise),
                        scope_url: scope_url,
                        settle_type: SettleType::Resolve(Trusted::new(&*new_reg)),
                        global: Trusted::new(&*global)
            };
            script_thread.queue_async_job(box async_job_handler);
        }
    }
}
