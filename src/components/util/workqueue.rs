/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A work queue for scheduling units of work across threads in a fork-join fashion.
//!
//! Data associated with queues is simply a pair of unsigned integers. It is expected that a
//! higher-level API on top of this could allow safe fork-join parallelism.

use native;
use rand;
use rand::{Rng, XorShiftRng};
use std::cast;
use std::mem;
use std::sync::atomics::{AtomicUint, SeqCst};
use std::sync::deque::{Abort, BufferPool, Data, Empty, Stealer, Worker};
use std::task::TaskOpts;

/// A unit of work.
///
/// The type parameter `QUD` stands for "queue user data" and represents global custom data for the
/// entire work queue, and the type parameter `WUD` stands for "work user data" and represents
/// custom data specific to each unit of work.
pub struct WorkUnit<QUD,WUD> {
    /// The function to execute.
    pub fun: extern "Rust" fn(WUD, &mut WorkerProxy<QUD,WUD>),
    /// Arbitrary data.
    pub data: WUD,
}

/// Messages from the supervisor to the worker.
enum WorkerMsg<QUD,WUD> {
    /// Tells the worker to start work.
    StartMsg(Worker<WorkUnit<QUD,WUD>>, *mut AtomicUint, *QUD),

    /// Tells the worker to stop. It can be restarted again with a `StartMsg`.
    StopMsg,

    /// Tells the worker thread to terminate.
    ExitMsg,
}

/// Messages to the supervisor.
enum SupervisorMsg<QUD,WUD> {
    FinishedMsg,
    ReturnDequeMsg(uint, Worker<WorkUnit<QUD,WUD>>),
}

/// Information that the supervisor thread keeps about the worker threads.
struct WorkerInfo<QUD,WUD> {
    /// The communication channel to the workers.
    chan: Sender<WorkerMsg<QUD,WUD>>,
    /// The buffer pool for this deque.
    pool: BufferPool<WorkUnit<QUD,WUD>>,
    /// The worker end of the deque, if we have it.
    deque: Option<Worker<WorkUnit<QUD,WUD>>>,
    /// The thief end of the work-stealing deque.
    thief: Stealer<WorkUnit<QUD,WUD>>,
}

/// Information specific to each worker thread that the thread keeps.
struct WorkerThread<QUD,WUD> {
    /// The index of this worker.
    index: uint,
    /// The communication port from the supervisor.
    port: Receiver<WorkerMsg<QUD,WUD>>,
    /// The communication channel on which messages are sent to the supervisor.
    chan: Sender<SupervisorMsg<QUD,WUD>>,
    /// The thief end of the work-stealing deque for all other workers.
    other_deques: Vec<Stealer<WorkUnit<QUD,WUD>>>,
    /// The random number generator for this worker.
    rng: XorShiftRng,
}

static SPIN_COUNT: uint = 1000;

impl<QUD:Send,WUD:Send> WorkerThread<QUD,WUD> {
    /// The main logic. This function starts up the worker and listens for
    /// messages.
    pub fn start(&mut self) {
        loop {
            // Wait for a start message.
            let (mut deque, ref_count, queue_data) = match self.port.recv() {
                StartMsg(deque, ref_count, queue_data) => (deque, ref_count, queue_data),
                StopMsg => fail!("unexpected stop message"),
                ExitMsg => return,
            };

            // We're off!
            //
            // FIXME(pcwalton): Can't use labeled break or continue cross-crate due to a Rust bug.
            loop {
                // FIXME(pcwalton): Nasty workaround for the lack of labeled break/continue
                // cross-crate.
                let mut work_unit = unsafe {
                    mem::uninit()
                };
                match deque.pop() {
                    Some(work) => work_unit = work,
                    None => {
                        // Become a thief.
                        let mut i = 0;
                        let mut should_continue = true;
                        loop {
                            let victim = (self.rng.next_u32() as uint) % self.other_deques.len();
                            match self.other_deques.get_mut(victim).steal() {
                                Empty | Abort => {
                                    // Continue.
                                }
                                Data(work) => {
                                    work_unit = work;
                                    break
                                }
                            }

                            if i == SPIN_COUNT {
                                match self.port.try_recv() {
                                    Ok(StopMsg) => {
                                        should_continue = false;
                                        break
                                    }
                                    Ok(ExitMsg) => return,
                                    Ok(_) => fail!("unexpected message"),
                                    _ => {}
                                }

                                i = 0
                            } else {
                                i += 1
                            }
                        }

                        if !should_continue {
                            break
                        }
                    }
                }

                // At this point, we have some work. Perform it.
                let mut proxy = WorkerProxy {
                    worker: &mut deque,
                    ref_count: ref_count,
                    queue_data: queue_data,
                };
                (work_unit.fun)(work_unit.data, &mut proxy);

                // The work is done. Now decrement the count of outstanding work items. If this was
                // the last work unit in the queue, then send a message on the channel.
                unsafe {
                    if (*ref_count).fetch_sub(1, SeqCst) == 1 {
                        self.chan.send(FinishedMsg)
                    }
                }
            }

            // Give the deque back to the supervisor.
            self.chan.send(ReturnDequeMsg(self.index, deque))
        }
    }
}

/// A handle to the work queue that individual work units have.
pub struct WorkerProxy<'a,QUD,WUD> {
    pub worker: &'a mut Worker<WorkUnit<QUD,WUD>>,
    pub ref_count: *mut AtomicUint,
    pub queue_data: *QUD,
}

impl<'a,QUD,WUD:Send> WorkerProxy<'a,QUD,WUD> {
    /// Enqueues a block into the work queue.
    #[inline]
    pub fn push(&mut self, work_unit: WorkUnit<QUD,WUD>) {
        unsafe {
            drop((*self.ref_count).fetch_add(1, SeqCst));
        }
        self.worker.push(work_unit);
    }

    /// Retrieves the queue user data.
    #[inline]
    pub fn user_data<'a>(&'a self) -> &'a QUD {
        unsafe {
            cast::transmute(self.queue_data)
        }
    }
}

/// A work queue on which units of work can be submitted.
pub struct WorkQueue<QUD,WUD> {
    /// Information about each of the workers.
    workers: Vec<WorkerInfo<QUD,WUD>>,
    /// A port on which deques can be received from the workers.
    port: Receiver<SupervisorMsg<QUD,WUD>>,
    /// The amount of work that has been enqueued.
    pub work_count: uint,
    /// Arbitrary user data.
    pub data: QUD,
}

impl<QUD:Send,WUD:Send> WorkQueue<QUD,WUD> {
    /// Creates a new work queue and spawns all the threads associated with
    /// it.
    pub fn new(task_name: &'static str, thread_count: uint, user_data: QUD) -> WorkQueue<QUD,WUD> {
        // Set up data structures.
        let (supervisor_chan, supervisor_port) = channel();
        let (mut infos, mut threads) = (vec!(), vec!());
        for i in range(0, thread_count) {
            let (worker_chan, worker_port) = channel();
            let mut pool = BufferPool::new();
            let (worker, thief) = pool.deque();
            infos.push(WorkerInfo {
                chan: worker_chan,
                pool: pool,
                deque: Some(worker),
                thief: thief,
            });
            threads.push(WorkerThread {
                index: i,
                port: worker_port,
                chan: supervisor_chan.clone(),
                other_deques: vec!(),
                rng: rand::weak_rng(),
            });
        }

        // Connect workers to one another.
        for i in range(0, thread_count) {
            for j in range(0, thread_count) {
                if i != j {
                    threads.get_mut(i).other_deques.push(infos.get(j).thief.clone())
                }
            }
            assert!(threads.get(i).other_deques.len() == thread_count - 1)
        }

        // Spawn threads.
        for thread in threads.move_iter() {
            let mut opts = TaskOpts::new();
            opts.name = Some(task_name.into_maybe_owned());
            native::task::spawn_opts(opts, proc() {
                let mut thread = thread;
                thread.start()
            })
        }

        WorkQueue {
            workers: infos,
            port: supervisor_port,
            work_count: 0,
            data: user_data,
        }
    }

    /// Enqueues a block into the work queue.
    #[inline]
    pub fn push(&mut self, work_unit: WorkUnit<QUD,WUD>) {
        match self.workers.get_mut(0).deque {
            None => {
                fail!("tried to push a block but we don't have the deque?!")
            }
            Some(ref mut deque) => deque.push(work_unit),
        }
        self.work_count += 1
    }

    /// Synchronously runs all the enqueued tasks and waits for them to complete.
    pub fn run(&mut self) {
        // Tell the workers to start.
        let mut work_count = AtomicUint::new(self.work_count);
        for worker in self.workers.mut_iter() {
            worker.chan.send(StartMsg(worker.deque.take_unwrap(), &mut work_count, &self.data))
        }

        // Wait for the work to finish.
        drop(self.port.recv());
        self.work_count = 0;

        // Tell everyone to stop.
        for worker in self.workers.iter() {
            worker.chan.send(StopMsg)
        }

        // Get our deques back.
        for _ in range(0, self.workers.len()) {
            match self.port.recv() {
                ReturnDequeMsg(index, deque) => self.workers.get_mut(index).deque = Some(deque),
                FinishedMsg => fail!("unexpected finished message!"),
            }
        }
    }

    pub fn shutdown(&mut self) {
        for worker in self.workers.iter() {
            worker.chan.send(ExitMsg)
        }
    }
}

