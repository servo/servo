/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implements parallel traversals over the flow tree.

use layout::context::{LayoutContext, SharedLayoutInfo};
use layout::flow::{Flow, PostorderFlowTraversal};
use layout::flow;
use layout::layout_task::{AssignHeightsAndStoreOverflowTraversal, BubbleWidthsTraversal};

use gfx::font_context::{FontContext, FontContextInfo};
use native;
use servo_util::time::{ProfilerChan, profile};
use servo_util::time;
use std::cast;
use std::comm::SharedChan;
use std::libc::c_void;
use std::ptr;
use std::sync::atomics::{AtomicInt, Relaxed, SeqCst};
use std::sync::deque::{Abort, BufferPool, Data, Empty, Stealer, Worker};
use std::util;

enum WorkerMsg {
    /// Tells the worker to start a traversal.
    StartMsg(Worker<UnsafeFlow>, TraversalKind, SharedLayoutInfo),

    /// Tells the worker to stop. It can be restarted again with a `StartMsg`.
    StopMsg,

    /// Tells the worker thread to terminate.
    ExitMsg,
}

enum SupervisorMsg {
    /// Sent once the last flow is processed.
    FinishedMsg,

    /// Returns the deque to the supervisor.
    ReturnDequeMsg(uint, Worker<UnsafeFlow>),
}

pub type UnsafeFlow = (*c_void, *c_void);

pub fn owned_flow_to_unsafe_flow(flow: *~Flow) -> UnsafeFlow {
    unsafe {
        cast::transmute_copy(&*flow)
    }
}

pub fn mut_owned_flow_to_unsafe_flow(flow: *mut ~Flow) -> UnsafeFlow {
    unsafe {
        cast::transmute_copy(&*flow)
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
    pub fn new() -> FlowParallelInfo {
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
    /// The buffer pool for this deque.
    pool: BufferPool<UnsafeFlow>,
    /// The worker end of the deque, if we have it.
    deque: Option<Worker<UnsafeFlow>>,
    /// The thief end of the work-stealing deque.
    thief: Stealer<UnsafeFlow>,
}

/// Information that each worker needs to do its job.
struct PostorderWorker {
    /// The font context.
    font_context: Option<~FontContext>,
    /// Communications for the worker.
    comm: PostorderWorkerComm,
}

/// Communication channels for postorder workers.
struct PostorderWorkerComm {
    /// The index of this worker.
    index: uint,
    /// The communication port from the supervisor.
    port: Port<WorkerMsg>,
    /// The communication channel to the supervisor.
    chan: SharedChan<SupervisorMsg>,
    /// The thief end of the work-stealing deque for all other workers.
    other_deques: ~[Stealer<UnsafeFlow>],
}

/// The type of traversal we're performing.
pub enum TraversalKind {
    BubbleWidthsTraversalKind,
    AssignHeightsAndStoreOverflowTraversalKind,
}

impl PostorderWorker {
    /// Starts up the worker and listens for messages.
    pub fn start(&mut self) {
        loop {
            // Wait for a start message.
            let (mut deque, kind, shared_layout_info) = match self.comm.port.recv() {
                StopMsg => fail!("unexpected stop message"),
                StartMsg(deque, kind, shared_layout_info) => (deque, kind, shared_layout_info),
                ExitMsg => return,
            };

            // Set up our traversal context.
            let mut traversal = LayoutContext {
                shared: shared_layout_info,
                font_ctx: self.font_context.take_unwrap(),
            };

            // And we're off!
            'outer: loop {
                let unsafe_flow;
                match deque.pop() {
                    Some(the_flow) => unsafe_flow = the_flow,
                    None => {
                        // Become a thief.
                        let mut i = 0;
                        loop {
                            if self.comm.other_deques.len() != 0 {
                                match self.comm.other_deques[i].steal() {
                                    Empty => {
                                        // Try the next one.
                                        i += 1;
                                        if i >= self.comm.other_deques.len() {
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

                            if i == 0 {
                                match self.comm.port.try_recv() {
                                    Some(StopMsg) => break 'outer,
                                    Some(ExitMsg) => return,
                                    Some(_) => fail!("unexpected message!"),
                                    None => {}
                                }
                            }
                        }
                    }
                }

                // OK, we've got some data. The rest of this is unsafe code.
                unsafe {
                    // Get a real flow.
                    let flow: &mut ~Flow = cast::transmute(&unsafe_flow);

                    // Perform the appropriate traversal.
                    match kind {
                        BubbleWidthsTraversalKind => {
                            let mut traversal = BubbleWidthsTraversal(&mut traversal);
                            if traversal.should_process(*flow) {
                                traversal.process(*flow);
                            }
                        }
                        AssignHeightsAndStoreOverflowTraversalKind => {
                            let mut traversal =
                                AssignHeightsAndStoreOverflowTraversal(&mut traversal);
                            if traversal.should_process(*flow) {
                                traversal.process(*flow);
                            }
                        }
                    }

                    let base = flow::mut_base(*flow);

                    // Reset the count of children for the next layout traversal.
                    base.parallel.children_count.store(base.children.len() as int, Relaxed);

                    // Possibly enqueue the parent.
                    let unsafe_parent = base.parallel.parent;
                    if unsafe_parent == null_unsafe_flow() {
                        // We're done!
                        self.comm.chan.send(FinishedMsg);
                    } else {
                        // No, we're not at the root yet. Then are we the last sibling of our
                        // parent? If so, we can enqueue our parent; otherwise, we've gotta wait.
                        let parent: &mut ~Flow = cast::transmute(&unsafe_parent);
                        let parent_base = flow::mut_base(*parent);
                        if parent_base.parallel.children_count.fetch_sub(1, SeqCst) == 1 {
                            // We were the last child of our parent. Enqueue the parent.
                            deque.push(unsafe_parent)
                        }
                    }
                }
            }

            // Destroy the traversal and save the font context.
            let LayoutContext {
                font_ctx: font_context,
                ..
            } = traversal;
            self.font_context = Some(font_context);

            // Give the deque back to the supervisor.
            self.comm.chan.send(ReturnDequeMsg(self.comm.index, deque))
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
    pub fn new(font_context_info: FontContextInfo, thread_count: uint)
               -> ParallelPostorderFlowTraversal {
        let (supervisor_port, supervisor_chan) = SharedChan::new();
        let (mut infos, mut comms) = (~[], ~[]);
        for i in range(0, thread_count) {
            let (worker_port, worker_chan) = Chan::new();
            let mut pool = BufferPool::new();
            let (worker, thief) = pool.deque();
            infos.push(WorkerInfo {
                chan: worker_chan,
                pool: pool,
                deque: Some(worker),
                thief: thief,
            });
            comms.push(PostorderWorkerComm {
                index: i,
                port: worker_port,
                chan: supervisor_chan.clone(),
                other_deques: ~[],
            });
        }

        for i in range(0, thread_count) {
            for j in range(0, thread_count) {
                if i != j {
                    comms[i].other_deques.push(infos[j].thief.clone())
                }
            }
            assert!(comms[i].other_deques.len() == thread_count - 1)
        }

        for comm in comms.move_iter() {
            let font_context_info = font_context_info.clone();
            native::task::spawn(proc() {
                let mut worker = PostorderWorker {
                    font_context: Some(~FontContext::new(font_context_info)),
                    comm: comm,
                };
                worker.start()
            })
        }

        ParallelPostorderFlowTraversal {
            workers: infos,
            port: supervisor_port,
        }
    }

    /// TODO(pcwalton): This could be parallelized.
    fn warmup(&mut self, layout_context: &mut LayoutContext) {
        layout_context.shared.leaf_set.access(|leaf_set| {
            for &flow in leaf_set.iter() {
                match self.workers[0].deque {
                    None => fail!("no deque!"),
                    Some(ref mut deque) => {
                        deque.push(flow);
                    }
                }
            }
        });
    }

    /// Traverses the given flow tree in parallel.
    pub fn start(&mut self,
                 kind: TraversalKind,
                 layout_context: &mut LayoutContext,
                 profiler_chan: ProfilerChan) {
        profile(time::LayoutParallelWarmupCategory, profiler_chan, || self.warmup(layout_context));

        for worker in self.workers.mut_iter() {
            worker.chan.send(StartMsg(util::replace(&mut worker.deque, None).unwrap(),
                                      kind,
                                      layout_context.shared.clone()))
        }

        // Wait for them to finish.
        let _ = self.port.recv();

        // Tell everyone to stop.
        for worker in self.workers.iter() {
            worker.chan.send(StopMsg);
        }

        // Get our deques back.
        //
        // TODO(pcwalton): Might be able to get a little parallelism over multiple traversals by
        // doing this lazily.
        for _ in range(0, self.workers.len()) {
            match self.port.recv() {
                ReturnDequeMsg(returned_deque_index, returned_deque) => {
                    self.workers[returned_deque_index].deque = Some(returned_deque)
                }
                _ => fail!("unexpected message received during return queue phase"),
            }
        }
    }

    /// Shuts down all the worker threads.
    pub fn shutdown(&mut self) {
        for worker in self.workers.iter() {
            worker.chan.send(ExitMsg)
        }
    }
}

