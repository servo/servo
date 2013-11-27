/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implements parallel traversals over the flow tree.

use layout::context::LayoutContext;
use layout::flow::{FlowContext, ImmutableFlowUtils, PostorderFlowTraversal};
use layout::flow;
use layout::layout_task::BubbleWidthsTraversal;

use servo_util::deque::{Abort, BufferPool, Data, Empty, Stealer, Worker};
use servo_util::time::{ProfilerChan, profile};
use servo_util::time;
use std::cast;
use std::cell::Cell;
use std::comm::SharedChan;
use std::libc::c_void;
use std::ptr;
use std::task::{SingleThreaded, spawn_sched};
use std::unstable::atomics::{AtomicInt, SeqCst};
use std::util;

static WORKER_COUNT: uint = 4;

enum WorkerMsg {
    StartMsg(Worker<UnsafeFlow>),
    StopMsg,
}

enum SupervisorMsg {
    FinishedMsg,
}

type UnsafeFlow = (*c_void, *c_void);

fn to_unsafe_flow(flow: *mut ~FlowContext:) -> UnsafeFlow {
    unsafe {
        cast::transmute_copy(&(*flow))
    }
}

fn null_unsafe_flow() -> UnsafeFlow {
    (ptr::null(), ptr::null())
}

/// Information that we need stored in each flow.
pub struct FlowParallelInfo {
    /// The number of children that still need work done.
    children_count: AtomicInt,
    /// The address of the parent flow.
    parent: UnsafeFlow,
}

impl FlowParallelInfo {
    pub fn init() -> FlowParallelInfo {
        FlowParallelInfo {
            children_count: AtomicInt::new(0),
            parent: null_unsafe_flow(),
        }
    }
}

/// Information that the supervisor thread keeps about the worker threads.
struct WorkerInfo {
    /// The communication channel to the workers.
    chan: Chan<WorkerMsg>,
    /// A port on which we receive the worker end of the deque from the worker.
    return_deque_port: Port<Worker<UnsafeFlow>>,
    /// The buffer pool for this deque.
    pool: BufferPool<UnsafeFlow>,
    /// The worker end of the deque, if we have it.
    deque: Option<Worker<UnsafeFlow>>,
    /// The thief end of the work-stealing deque.
    thief: Stealer<UnsafeFlow>,
}

/// Information that each worker needs to do its job.
///
/// TODO(pcwalton): Have an atomic count of thieves so we know when we're done.
struct PostorderWorker {
    /// The traversal.
    traversal: LayoutContext,
    /// The communication port from the supervisor.
    port: Port<WorkerMsg>,
    /// The communication channel to the supervisor.
    chan: SharedChan<SupervisorMsg>,
    /// A channel used to return the worker end of the deque back to the supervisor when done.
    return_deque_chan: Chan<Worker<UnsafeFlow>>,
    /// The thief end of the work-stealing deque for all other workers.
    other_deques: ~[Stealer<UnsafeFlow>],
}

impl PostorderWorker {
    /// Starts up the worker and listens for messages.
    pub fn start(&mut self) {
        loop {
            // Wait for a start message.
            let mut deque = match self.port.recv() {
                StopMsg => fail!("unexpected stop message"),
                StartMsg(deque) => deque,
            };

            // And we're off!
            'outer: loop {
                let unsafe_flow;
                match deque.pop() {
                    Some(the_flow) => unsafe_flow = the_flow,
                    None => {
                        // Become a thief.
                        //
                        // FIXME(pcwalton): This won't work if `WORKER_COUNT == 1`.
                        let mut i = 0;
                        loop {
                            if self.port.peek() {
                                match self.port.recv() {
                                    StopMsg => break 'outer,
                                    _ => fail!("unexpected message!"),
                                }
                            }

                            match self.other_deques[i].steal() {
                                Empty => {
                                    // Try the next one.
                                    i += 1;
                                    if i >= WORKER_COUNT - 1 {
                                        i = 0
                                    }
                                }
                                Abort => {
                                    // Continue.
                                }
                                Data(the_flow) => {
                                    unsafe_flow = the_flow;
                                    break
                                }
                            }
                        }
                    }
                }

                // OK, we've got some data. The rest of this is unsafe code.
                unsafe {
                    // Get a real flow.
                    let flow: &mut ~FlowContext: = cast::transmute(&unsafe_flow);
                    let mut traversal = BubbleWidthsTraversal(&mut self.traversal);
                    traversal.process(*flow);

                    let base = flow::mut_base(*flow);
                    let unsafe_parent = base.parallel.parent;
                    if unsafe_parent == null_unsafe_flow() {
                        // We're done!
                        self.chan.send(FinishedMsg);
                    } else {
                        // No, we're not at the root yet. Then are we the last sibling of our
                        // parent? If so, we can enqueue our parent; otherwise, we've gotta wait.
                        let parent: &mut ~FlowContext: = cast::transmute(&unsafe_parent);
                        let parent_base = flow::mut_base(*parent);
                        if parent_base.parallel.children_count.fetch_sub(1, SeqCst) == 1 {
                            // We were the last child of our parent. Enqueue the parent.
                            deque.push(unsafe_parent)
                        }
                    }
                }
            }

            // Give the deque back to the supervisor.
            self.return_deque_chan.send(deque)
        }
    }
}

/// A parallel bottom-up traversal.
pub struct ParallelPostorderFlowTraversal {
    /// Information about each of the workers.
    workers: ~[WorkerInfo],
    /// A port on which information can be received from the workers.
    port: Port<SupervisorMsg>,
}

impl ParallelPostorderFlowTraversal {
    pub fn init(traversal: LayoutContext) -> ParallelPostorderFlowTraversal {
        let (supervisor_port, supervisor_chan) = stream();
        let supervisor_chan = SharedChan::new(supervisor_chan);
        let mut infos = ~[];
        let mut workers = ~[];
        for _ in range(0, WORKER_COUNT) {
            let (worker_port, worker_chan) = stream();
            let (return_deque_port, return_deque_chan) = stream();
            let mut pool = BufferPool::new();
            let (worker, thief) = pool.deque();
            infos.push(WorkerInfo {
                chan: worker_chan,
                return_deque_port: return_deque_port,
                pool: pool,
                deque: Some(worker),
                thief: thief,
            });
            workers.push(PostorderWorker {
                traversal: traversal.clone(),
                port: worker_port,
                chan: supervisor_chan.clone(),
                return_deque_chan: return_deque_chan,
                other_deques: ~[],
            });
        }

        for i in range(0, WORKER_COUNT) {
            for j in range(0, WORKER_COUNT) {
                if i != j {
                    workers[i].other_deques.push(infos[j].thief.clone())
                }
            }
            assert!(workers[i].other_deques.len() == WORKER_COUNT - 1)
        }

        for worker in workers.move_iter() {
            let worker = Cell::new(worker);
            do spawn_sched(SingleThreaded) {
                let mut worker = worker.take();
                worker.start()
            }
        }

        ParallelPostorderFlowTraversal {
            workers: infos,
            port: supervisor_port,
        }
    }

    /// TODO(pcwalton): This could be parallelized.
    fn warmup(&mut self,
              flow: &mut ~FlowContext:,
              parent: UnsafeFlow,
              next_queue_index: &mut uint) {
        let child_count = flow.child_count();
        let base = flow::mut_base(*flow);
        assert!(child_count < 0x8000_0000_0000_0000);   // You never know...
        base.parallel.children_count = AtomicInt::new(child_count as int);
        base.parallel.parent = parent;

        // If this is a leaf, enqueue it in a round-robin fashion.
        if child_count == 0 {
            match self.workers[*next_queue_index].deque {
                None => fail!("no deque!"),
                Some(ref mut deque) => deque.push(to_unsafe_flow(flow)),
            }

            *next_queue_index += 1;
            if *next_queue_index >= WORKER_COUNT - 1 {
                *next_queue_index = 0
            }

            return
        }

        for kid in base.child_iter() {
            self.warmup(kid, to_unsafe_flow(flow), next_queue_index)
        }
    }

    /// Traverses the given flow tree in parallel.
    pub fn start(&mut self, root: &mut ~FlowContext:, profiler_chan: ProfilerChan) {
        profile(time::LayoutParallelWarmupCategory, profiler_chan, || {
            let mut index = 0;
            self.warmup(root, null_unsafe_flow(), &mut index);
        });

        for worker in self.workers.mut_iter() {
            worker.chan.send(StartMsg(util::replace(&mut worker.deque, None).unwrap()))
        }

        // Wait for them to finish.
        let _ = self.port.recv();

        // Tell everyone to stop.
        for worker in self.workers.iter() {
            worker.chan.send(StopMsg);
        }

        // Get our deques back.
        //
        // FIXME(pcwalton): Might be able to get a little parallelism over multiple traversals by
        // doing this lazily.
        for worker in self.workers.mut_iter() {
            worker.deque = Some(worker.return_deque_port.recv())
        }
    }
}

