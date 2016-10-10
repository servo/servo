/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::refcounted::{Trusted, TrustedPromise};
use dom::bindings::reflector::Reflectable;
use dom::bindings::trace::JSTraceable;
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use dom::serviceworkerregistration::ServiceWorkerRegistration;
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

#[must_root]
pub struct Job {
    pub job_type: JobType,
    pub scope_url: Url,
    pub script_url: Url,
    pub promise: Rc<Promise>,
    pub equivalent_jobs: Vec<Job>,
}

impl Job {
    #[allow(unrooted_must_root)]
    pub fn create_job(job_type: JobType,
                      scope_url: Url,
                      script_url: Url,
                      promise: Rc<Promise>) -> Job {
        Job {
            job_type: job_type,
            scope_url: scope_url,
            script_url: script_url,
            promise: promise,
            equivalent_jobs: vec![]
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

pub struct ResolveJobHandler {
    pub reg: Trusted<ServiceWorkerRegistration>,
    pub promise: TrustedPromise,
    pub scope_url: Url
}

impl Runnable for ResolveJobHandler {
    #[allow(unrooted_must_root)]
    fn main_thread_handler(self: Box<ResolveJobHandler>, script_thread: &ScriptThread) {
        let reg = self.reg.root();
        let promise = self.promise.root();
        let global = reg.global();
        let _ac = JSAutoCompartment::new((&*global).get_cx(), promise.reflector().get_jsobject().get());
        promise.resolve_native((&*global).get_cx(), &*reg);
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
        // Initialize to an empty vec
        self.0.insert(job.scope_url.clone(), vec![]);
        let ref mut job_queue = match self.0.get_mut(&job.scope_url) {
            Some(r) => r,
            None => return
        };
        if job_queue.is_empty() {
            let scope_url = job.scope_url.clone();
            job_queue.push(job);
            let run_job_handler = RunJobHandler {
                reg: Trusted::new(reg),
                promise: TrustedPromise::new(promise),
                scope_url: scope_url
            };

            // queue task for https://w3c.github.io/ServiceWorker/#run-job-algorithm
            let global = reg.global();
            script_thread.queue_run_job(box run_job_handler, &*global);
        } else {
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
                self.run_register(front_job, run_job_handler, script_thread);
            },
            _ => {}
        }
    }

    #[allow(unrooted_must_root)]
    // https://w3c.github.io/ServiceWorker/#register-algorithm
    pub fn run_register(&mut self, job: Job, run_job_handler: Box<RunJobHandler>, script_thread: &ScriptThread) {
        // Step 1-3 TODO
        // Step 4
        let reg = ScriptThread::get_registration(&job.scope_url);
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
                    let resolve_job_handler = ResolveJobHandler {
                        reg: reg,
                        promise: TrustedPromise::new(promise),
                        scope_url: scope_url
                    };
                    script_thread.queue_resolve_job(box resolve_job_handler);
                }
            }
        } else {
            let reg = run_job_handler.reg.root();
            let global = reg.global();
            let pipeline = global.pipeline_id();
            ScriptThread::set_registration(job.scope_url.clone(), &*reg, pipeline);
            let scope_url = run_job_handler.scope_url.clone();
            let promise = run_job_handler.promise.root();
            let resolve_job_handler = ResolveJobHandler {
                        reg: Trusted::new(&*reg),
                        promise: TrustedPromise::new(promise),
                        scope_url: scope_url
            };
            script_thread.queue_resolve_job(box resolve_job_handler);
        }
    }
}
