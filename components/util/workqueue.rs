/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A work queue for scheduling units of work across threads in a fork-join fashion.
//!
//! Data associated with queues is simply a pair of unsigned integers. It is expected that a
//! higher-level API on top of this could allow safe fork-join parallelism.

use deque::{Abort, BufferPool, Data, Empty, Stealer, Worker};
use task::spawn_named;
use task_state;

use libc::funcs::posix88::unistd::usleep;
use rand::{Rng, weak_rng, XorShiftRng};
use std::mem;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Sender, Receiver};

/// A unit of work.
///
/// # Type parameters
///
/// - `QueueData`: global custom data for the entire work queue.
/// - `WorkData`: custom data specific to each unit of work.
pub struct WorkUnit<QueueData, WorkData> {
    /// The function to execute.
    pub fun: extern "Rust" fn(WorkData, &mut WorkerProxy<QueueData, WorkData>),
    /// Arbitrary data.
    pub data: WorkData,
}

/// Messages from the supervisor to the worker.
enum WorkerMsg<QueueData: 'static, WorkData: 'static> {
    /// Tells the worker to start work.
    Start(Worker<WorkUnit<QueueData, WorkData>>, *mut AtomicUsize, *const QueueData),
    /// Tells the worker to stop. It can be restarted again with a `WorkerMsg::Start`.
    Stop,
    /// Tells the worker to measure the heap size of its TLS using the supplied function.
    HeapSizeOfTLS(fn() -> usize),
    /// Tells the worker thread to terminate.
    Exit,
}

unsafe impl<QueueData: 'static, WorkData: 'static> Send for WorkerMsg<QueueData, WorkData> {}

/// Messages to the supervisor.
enum SupervisorMsg<QueueData: 'static, WorkData: 'static> {
    Finished,
    HeapSizeOfTLS(usize),
    ReturnDeque(usize, Worker<WorkUnit<QueueData, WorkData>>),
}

unsafe impl<QueueData: 'static, WorkData: 'static> Send for SupervisorMsg<QueueData, WorkData> {}

/// Information that the supervisor thread keeps about the worker threads.
struct WorkerInfo<QueueData: 'static, WorkData: 'static> {
    /// The communication channel to the workers.
    chan: Sender<WorkerMsg<QueueData, WorkData>>,
    /// The worker end of the deque, if we have it.
    deque: Option<Worker<WorkUnit<QueueData, WorkData>>>,
    /// The thief end of the work-stealing deque.
    thief: Stealer<WorkUnit<QueueData, WorkData>>,
}

/// Information specific to each worker thread that the thread keeps.
struct WorkerThread<QueueData: 'static, WorkData: 'static> {
    /// The index of this worker.
    index: usize,
    /// The communication port from the supervisor.
    port: Receiver<WorkerMsg<QueueData, WorkData>>,
    /// The communication channel on which messages are sent to the supervisor.
    chan: Sender<SupervisorMsg<QueueData, WorkData>>,
    /// The thief end of the work-stealing deque for all other workers.
    other_deques: Vec<Stealer<WorkUnit<QueueData, WorkData>>>,
    /// The random number generator for this worker.
    rng: XorShiftRng,
}

unsafe impl<QueueData: 'static, WorkData: 'static> Send for WorkerThread<QueueData, WorkData> {}

const SPINS_UNTIL_BACKOFF: u32 = 128;
const BACKOFF_INCREMENT_IN_US: u32 = 5;
const BACKOFFS_UNTIL_CONTROL_CHECK: u32 = 6;

fn next_power_of_two(mut v: u32) -> u32 {
    v -= 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v += 1;
    v
}

impl<QueueData: Send, WorkData: Send> WorkerThread<QueueData, WorkData> {
    /// The main logic. This function starts up the worker and listens for
    /// messages.
    fn start(&mut self) {
        let deque_index_mask = next_power_of_two(self.other_deques.len() as u32) - 1;
        loop {
            // Wait for a start message.
            let (mut deque, ref_count, queue_data) = match self.port.recv().unwrap() {
                WorkerMsg::Start(deque, ref_count, queue_data) => (deque, ref_count, queue_data),
                WorkerMsg::Stop => panic!("unexpected stop message"),
                WorkerMsg::Exit => return,
                WorkerMsg::HeapSizeOfTLS(f) => {
                    self.chan.send(SupervisorMsg::HeapSizeOfTLS(f())).unwrap();
                    continue;
                }
            };

            let mut back_off_sleep = 0 as u32;

            // We're off!
            'outer: loop {
                let work_unit;
                match deque.pop() {
                    Some(work) => work_unit = work,
                    None => {
                        // Become a thief.
                        let mut i = 0;
                        loop {
                            // Don't just use `rand % len` because that's slow on ARM.
                            let mut victim;
                            loop {
                                victim = self.rng.next_u32() & deque_index_mask;
                                if (victim as usize) < self.other_deques.len() {
                                    break
                                }
                            }

                            match self.other_deques[victim as usize].steal() {
                                Empty | Abort => {
                                    // Continue.
                                }
                                Data(work) => {
                                    work_unit = work;
                                    back_off_sleep = 0 as u32;
                                    break
                                }
                            }

                            if i > SPINS_UNTIL_BACKOFF {
                                if back_off_sleep >= BACKOFF_INCREMENT_IN_US *
                                        BACKOFFS_UNTIL_CONTROL_CHECK {
                                    match self.port.try_recv() {
                                        Ok(WorkerMsg::Stop) => break 'outer,
                                        Ok(WorkerMsg::Exit) => return,
                                        Ok(_) => panic!("unexpected message"),
                                        _ => {}
                                    }
                                }

                                unsafe {
                                    usleep(back_off_sleep as u32);
                                }
                                back_off_sleep += BACKOFF_INCREMENT_IN_US;
                                i = 0
                            } else {
                                i += 1
                            }
                        }
                    }
                }

                // At this point, we have some work. Perform it.
                let mut proxy = WorkerProxy {
                    worker: &mut deque,
                    ref_count: ref_count,
                    queue_data: queue_data,
                    worker_index: self.index as u8,
                };
                (work_unit.fun)(work_unit.data, &mut proxy);

                // The work is done. Now decrement the count of outstanding work items. If this was
                // the last work unit in the queue, then send a message on the channel.
                unsafe {
                    if (*ref_count).fetch_sub(1, Ordering::Release) == 1 {
                        self.chan.send(SupervisorMsg::Finished).unwrap()
                    }
                }
            }

            // Give the deque back to the supervisor.
            self.chan.send(SupervisorMsg::ReturnDeque(self.index, deque)).unwrap()
        }
    }
}

/// A handle to the work queue that individual work units have.
pub struct WorkerProxy<'a, QueueData: 'a, WorkData: 'a> {
    worker: &'a mut Worker<WorkUnit<QueueData, WorkData>>,
    ref_count: *mut AtomicUsize,
    queue_data: *const QueueData,
    worker_index: u8,
}

impl<'a, QueueData: 'static, WorkData: Send + 'static> WorkerProxy<'a, QueueData, WorkData> {
    /// Enqueues a block into the work queue.
    #[inline]
    pub fn push(&mut self, work_unit: WorkUnit<QueueData, WorkData>) {
        unsafe {
            drop((*self.ref_count).fetch_add(1, Ordering::Relaxed));
        }
        self.worker.push(work_unit);
    }

    /// Retrieves the queue user data.
    #[inline]
    pub fn user_data<'b>(&'b self) -> &'b QueueData {
        unsafe {
            mem::transmute(self.queue_data)
        }
    }

    /// Retrieves the index of the worker.
    #[inline]
    pub fn worker_index(&self) -> u8 {
        self.worker_index
    }
}

/// A work queue on which units of work can be submitted.
pub struct WorkQueue<QueueData: 'static, WorkData: 'static> {
    /// Information about each of the workers.
    workers: Vec<WorkerInfo<QueueData, WorkData>>,
    /// A port on which deques can be received from the workers.
    port: Receiver<SupervisorMsg<QueueData, WorkData>>,
    /// The amount of work that has been enqueued.
    work_count: usize,
}

impl<QueueData: Send, WorkData: Send> WorkQueue<QueueData, WorkData> {
    /// Creates a new work queue and spawns all the threads associated with
    /// it.
    pub fn new(task_name: &'static str,
               state: task_state::TaskState,
               thread_count: usize) -> WorkQueue<QueueData, WorkData> {
        // Set up data structures.
        let (supervisor_chan, supervisor_port) = channel();
        let (mut infos, mut threads) = (vec!(), vec!());
        for i in 0..thread_count {
            let (worker_chan, worker_port) = channel();
            let pool = BufferPool::new();
            let (worker, thief) = pool.deque();
            infos.push(WorkerInfo {
                chan: worker_chan,
                deque: Some(worker),
                thief: thief,
            });
            threads.push(WorkerThread {
                index: i,
                port: worker_port,
                chan: supervisor_chan.clone(),
                other_deques: vec!(),
                rng: weak_rng(),
            });
        }

        // Connect workers to one another.
        for i in 0..thread_count {
            for j in 0..thread_count {
                if i != j {
                    threads[i].other_deques.push(infos[j].thief.clone())
                }
            }
            assert!(threads[i].other_deques.len() == thread_count - 1)
        }

        // Spawn threads.
        for (i, thread) in threads.into_iter().enumerate() {

            spawn_named(
                format!("{} worker {}/{}", task_name, i+1, thread_count),
                move || {
                    task_state::initialize(state | task_state::IN_WORKER);
                    let mut thread = thread;
                    thread.start()
                })
        }

        WorkQueue {
            workers: infos,
            port: supervisor_port,
            work_count: 0,
        }
    }

    /// Enqueues a block into the work queue.
    #[inline]
    pub fn push(&mut self, work_unit: WorkUnit<QueueData, WorkData>) {
        let deque = &mut self.workers[0].deque;
        match *deque {
            None => {
                panic!("tried to push a block but we don't have the deque?!")
            }
            Some(ref mut deque) => deque.push(work_unit),
        }
        self.work_count += 1
    }

    /// Synchronously runs all the enqueued tasks and waits for them to complete.
    pub fn run(&mut self, data: QueueData) {
        // Tell the workers to start.
        let mut work_count = AtomicUsize::new(self.work_count);
        for worker in self.workers.iter_mut() {
            worker.chan.send(WorkerMsg::Start(worker.deque.take().unwrap(),
                                              &mut work_count,
                                              &data)).unwrap()
        }

        // Wait for the work to finish.
        drop(self.port.recv());
        self.work_count = 0;

        // Tell everyone to stop.
        for worker in self.workers.iter() {
            worker.chan.send(WorkerMsg::Stop).unwrap()
        }

        // Get our deques back.
        for _ in 0..self.workers.len() {
            match self.port.recv().unwrap() {
                SupervisorMsg::ReturnDeque(index, deque) => self.workers[index].deque = Some(deque),
                SupervisorMsg::HeapSizeOfTLS(_) => panic!("unexpected HeapSizeOfTLS message"),
                SupervisorMsg::Finished => panic!("unexpected finished message!"),
            }
        }
    }

    /// Synchronously measure memory usage of any thread-local storage.
    pub fn heap_size_of_tls(&self, f: fn() -> usize) -> Vec<usize> {
        // Tell the workers to measure themselves.
        for worker in self.workers.iter() {
            worker.chan.send(WorkerMsg::HeapSizeOfTLS(f)).unwrap()
        }

        // Wait for the workers to finish measuring themselves.
        let mut sizes = vec![];
        for _ in 0..self.workers.len() {
            match self.port.recv().unwrap() {
                SupervisorMsg::HeapSizeOfTLS(size) => {
                    sizes.push(size);
                }
                _ => panic!("unexpected message!"),
            }
        }
        sizes
    }

    pub fn shutdown(&mut self) {
        for worker in self.workers.iter() {
            worker.chan.send(WorkerMsg::Exit).unwrap()
        }
    }
}
