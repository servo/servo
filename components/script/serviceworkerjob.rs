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
use dom::bindings::refcounted::{Trusted, TrustedPromise};
use dom::bindings::reflector::DomObject;
use dom::client::Client;
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use dom::serviceworkerregistration::ServiceWorkerRegistration;
use dom::urlhelper::UrlHelper;
use js::jsapi::JSAutoCompartment;
use script_thread::ScriptThread;
use servo_url::ServoUrl;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::rc::Rc;
use task_source::TaskSource;
use task_source::dom_manipulation::DOMManipulationTaskSource;

#[derive(Clone, Copy, Debug, JSTraceable, PartialEq)]
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

#[must_root]
#[derive(JSTraceable)]
pub struct JobQueue(pub DOMRefCell<HashMap<ServoUrl, Vec<Job>>>);

impl JobQueue {
    pub fn new() -> JobQueue {
        JobQueue(DOMRefCell::new(HashMap::new()))
    }
    #[allow(unrooted_must_root)]
    // https://w3c.github.io/ServiceWorker/#schedule-job-algorithm
    pub fn schedule_job(&self, job: Job, script_thread: &ScriptThread) {
        debug!("scheduling {:?} job", job.job_type);
        let mut queue_ref = self.0.borrow_mut();
        let job_queue = queue_ref.entry(job.scope_url.clone()).or_insert(vec![]);
        // Step 1
        if job_queue.is_empty() {
            let scope_url = job.scope_url.clone();
            job_queue.push(job);
            let _ = script_thread.schedule_job_queue(scope_url);
            debug!("queued task to run newly-queued job");
        } else {
            // Step 2
            let mut last_job = job_queue.pop().unwrap();
            if job == last_job && !last_job.promise.is_settled() {
                last_job.append_equivalent_job(job);
                job_queue.push(last_job);
                debug!("appended equivalent job");
            } else {
                // restore the popped last_job
                job_queue.push(last_job);
                // and push this new job to job queue
                job_queue.push(job);
                debug!("pushed onto job queue job");
            }
        }
    }

    #[allow(unrooted_must_root)]
    // https://w3c.github.io/ServiceWorker/#run-job-algorithm
    pub fn run_job(&self, scope_url: ServoUrl, script_thread: &ScriptThread) {
        debug!("running a job");
        let url = {
            let queue_ref = self.0.borrow();
            let front_job = {
                let job_vec = queue_ref.get(&scope_url);
                job_vec.unwrap().first().unwrap()
            };
            let front_scope_url = front_job.scope_url.clone();
            match front_job.job_type {
                JobType::Register => self.run_register(front_job, scope_url, script_thread),
                JobType::Update => self.update(front_job, script_thread),
                JobType::Unregister => unreachable!(),
            };
            front_scope_url
        };
        self.finish_job(url, script_thread);
    }

    #[allow(unrooted_must_root)]
    // https://w3c.github.io/ServiceWorker/#register-algorithm
    fn run_register(&self, job: &Job, scope_url: ServoUrl, script_thread: &ScriptThread) {
        debug!("running register job");
        // Step 1-3
        if !UrlHelper::is_origin_trustworthy(&job.script_url) {
            // Step 1.1
            reject_job_promise(job,
                               Error::Type("Invalid script ServoURL".to_owned()),
                               script_thread.dom_manipulation_task_source());
            // Step 1.2 (see run_job)
            return;
        } else if job.script_url.origin() != job.referrer.origin() || job.scope_url.origin() != job.referrer.origin() {
            // Step 2.1/3.1
            reject_job_promise(job,
                               Error::Security,
                               script_thread.dom_manipulation_task_source());
            // Step 2.2/3.2 (see run_job)
            return;
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
                    // Step 5.3.1
                    resolve_job_promise(job, &*reg, script_thread.dom_manipulation_task_source());
                    // Step 5.3.2 (see run_job)
                    return;
                }
            }
        } else {
            // Step 6.1
            let global = &*job.client.global();
            let pipeline = global.pipeline_id();
            let new_reg = ServiceWorkerRegistration::new(&*global, &job.script_url, scope_url);
            script_thread.handle_serviceworker_registration(&job.scope_url, &*new_reg, pipeline);
        }
        // Step 7
        self.update(job, script_thread)
    }

    #[allow(unrooted_must_root)]
    // https://w3c.github.io/ServiceWorker/#finish-job-algorithm
    pub fn finish_job(&self, scope_url: ServoUrl, script_thread: &ScriptThread) {
        debug!("finishing previous job");
        let run_job = if let Some(job_vec) = (*self.0.borrow_mut()).get_mut(&scope_url) {
            assert_eq!(job_vec.first().as_ref().unwrap().scope_url, scope_url);
            let _  = job_vec.remove(0);
            !job_vec.is_empty()
        } else {
            warn!("non-existent job vector for Servourl: {:?}", scope_url);
            false
        };

        if run_job {
            debug!("further jobs in queue after finishing");
            self.run_job(scope_url, script_thread);
        }
    }

    // https://w3c.github.io/ServiceWorker/#update-algorithm
    fn update(&self, job: &Job, script_thread: &ScriptThread) {
        debug!("running update job");
        // Step 1
        let reg = match script_thread.handle_get_registration(&job.scope_url) {
            Some(reg) => reg,
            None => {
                let err_type = Error::Type("No registration to update".to_owned());
                // Step 2.1
                reject_job_promise(job, err_type, script_thread.dom_manipulation_task_source());
                // Step 2.2 (see run_job)
                return;
            }
        };
        // Step 2
        if reg.get_uninstalling() {
            let err_type = Error::Type("Update called on an uninstalling registration".to_owned());
            // Step 2.1
            reject_job_promise(job, err_type, script_thread.dom_manipulation_task_source());
            // Step 2.2 (see run_job)
            return;
        }
        // Step 3
        let newest_worker = reg.get_newest_worker();
        let newest_worker_url = newest_worker.as_ref().map(|w| w.get_script_url());
        // Step 4
        if newest_worker_url.as_ref() == Some(&job.script_url)  && job.job_type == JobType::Update {
            let err_type = Error::Type("Invalid script ServoURL".to_owned());
            // Step 4.1
            reject_job_promise(job, err_type, script_thread.dom_manipulation_task_source());
            // Step 4.2 (see run_job)
            return;
        }
        // Step 8
        if let Some(newest_worker) = newest_worker {
            job.client.set_controller(&*newest_worker);
            // Step 8.1
            resolve_job_promise(job, &*reg, script_thread.dom_manipulation_task_source());
            // Step 8.2 present in run_job
        }
        // TODO Step 9 (create new service worker)
    }
}

fn settle_job_promise(global: &GlobalScope, promise: &Promise, settle: SettleType) {
    let _ac = JSAutoCompartment::new(global.get_cx(), promise.reflector().get_jsobject().get());
    match settle {
        SettleType::Resolve(reg) => promise.resolve_native(global.get_cx(), &*reg.root()),
        SettleType::Reject(err) => promise.reject_error(global.get_cx(), err),
    };
}

#[allow(unrooted_must_root)]
fn queue_settle_promise_for_job(job: &Job, settle: SettleType, task_source: &DOMManipulationTaskSource) {
    let global = job.client.global();
    let promise = TrustedPromise::new(job.promise.clone());
    // FIXME(nox): Why are errors silenced here?
    let _ = task_source.queue(
        task!(settle_promise_for_job: move || {
            let promise = promise.root();
            settle_job_promise(&promise.global(), &promise, settle)
        }),
        &*global,
    );
}

// https://w3c.github.io/ServiceWorker/#reject-job-promise-algorithm
// https://w3c.github.io/ServiceWorker/#resolve-job-promise-algorithm
fn queue_settle_promise(job: &Job, settle: SettleType, task_source: &DOMManipulationTaskSource) {
    // Step 1
    queue_settle_promise_for_job(job, settle.clone(), task_source);
    // Step 2
    for job in &job.equivalent_jobs {
        queue_settle_promise_for_job(job, settle.clone(), task_source);
    }
}

fn reject_job_promise(job: &Job, err: Error, task_source: &DOMManipulationTaskSource) {
    queue_settle_promise(job, SettleType::Reject(err), task_source)
}

fn resolve_job_promise(job: &Job, reg: &ServiceWorkerRegistration, task_source: &DOMManipulationTaskSource) {
    queue_settle_promise(job, SettleType::Resolve(Trusted::new(reg)), task_source)
}
