/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::{ScriptToDevtoolsControlMsg, WorkerId};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::reflector::Reflectable;
use dom::bindings::str::DOMString;
use dom::crypto::Crypto;
use dom::eventtarget::EventTarget;
use ipc_channel::ipc::IpcSender;
use js::jsapi::{JS_GetContext, JS_GetObjectRuntime, JSContext};
use profile_traits::{mem, time};
use script_traits::{ScriptMsg as ConstellationMsg, TimerEventRequest};
use std::cell::Cell;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use time::{Timespec, get_time};

#[dom_struct]
pub struct GlobalScope {
    eventtarget: EventTarget,
    crypto: MutNullableHeap<JS<Crypto>>,
    next_worker_id: Cell<WorkerId>,

    /// A flag to indicate whether the developer tools has requested
    /// live updates from the worker.
    devtools_wants_updates: Cell<bool>,

    /// Timers used by the Console API.
    console_timers: DOMRefCell<HashMap<DOMString, u64>>,

    /// For providing instructions to an optional devtools server.
    #[ignore_heap_size_of = "channels are hard"]
    devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,

    /// For sending messages to the memory profiler.
    #[ignore_heap_size_of = "channels are hard"]
    mem_profiler_chan: mem::ProfilerChan,

    /// For sending messages to the time profiler.
    #[ignore_heap_size_of = "channels are hard"]
    time_profiler_chan: time::ProfilerChan,

    /// A handle for communicating messages to the constellation thread.
    #[ignore_heap_size_of = "channels are hard"]
    constellation_chan: IpcSender<ConstellationMsg>,

    #[ignore_heap_size_of = "channels are hard"]
    scheduler_chan: IpcSender<TimerEventRequest>,
}

impl GlobalScope {
    pub fn new_inherited(
            devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
            mem_profiler_chan: mem::ProfilerChan,
            time_profiler_chan: time::ProfilerChan,
            constellation_chan: IpcSender<ConstellationMsg>,
            scheduler_chan: IpcSender<TimerEventRequest>)
            -> Self {
        GlobalScope {
            eventtarget: EventTarget::new_inherited(),
            crypto: Default::default(),
            next_worker_id: Cell::new(WorkerId(0)),
            devtools_wants_updates: Default::default(),
            console_timers: DOMRefCell::new(Default::default()),
            devtools_chan: devtools_chan,
            mem_profiler_chan: mem_profiler_chan,
            time_profiler_chan: time_profiler_chan,
            constellation_chan: constellation_chan,
            scheduler_chan: scheduler_chan,
        }
    }

    #[allow(unsafe_code)]
    pub fn get_cx(&self) -> *mut JSContext {
        unsafe {
            let runtime = JS_GetObjectRuntime(
                self.reflector().get_jsobject().get());
            assert!(!runtime.is_null());
            let context = JS_GetContext(runtime);
            assert!(!context.is_null());
            context
        }
    }

    pub fn crypto(&self) -> Root<Crypto> {
        self.crypto.or_init(|| Crypto::new(self))
    }

    /// Get next worker id.
    pub fn get_next_worker_id(&self) -> WorkerId {
        let worker_id = self.next_worker_id.get();
        let WorkerId(id_num) = worker_id;
        self.next_worker_id.set(WorkerId(id_num + 1));
        worker_id
    }

    pub fn live_devtools_updates(&self) -> bool {
        self.devtools_wants_updates.get()
    }

    pub fn set_devtools_wants_updates(&self, value: bool) {
        self.devtools_wants_updates.set(value);
    }

    pub fn time(&self, label: DOMString) -> Result<(), ()> {
        let mut timers = self.console_timers.borrow_mut();
        if timers.len() >= 10000 {
            return Err(());
        }
        match timers.entry(label) {
            Entry::Vacant(entry) => {
                entry.insert(timestamp_in_ms(get_time()));
                Ok(())
            },
            Entry::Occupied(_) => Err(()),
        }
    }

    pub fn time_end(&self, label: &str) -> Result<u64, ()> {
        self.console_timers.borrow_mut().remove(label).ok_or(()).map(|start| {
            timestamp_in_ms(get_time()) - start
        })
    }

    /// Get an `&IpcSender<ScriptToDevtoolsControlMsg>` to send messages
    /// to the devtools thread when available.
    pub fn devtools_chan(&self) -> Option<&IpcSender<ScriptToDevtoolsControlMsg>> {
        self.devtools_chan.as_ref()
    }

    /// Get a sender to the memory profiler thread.
    pub fn mem_profiler_chan(&self) -> &mem::ProfilerChan {
        &self.mem_profiler_chan
    }

    /// Get a sender to the time profiler thread.
    pub fn time_profiler_chan(&self) -> &time::ProfilerChan {
        &self.time_profiler_chan
    }

    /// Get a sender to the constellation thread.
    pub fn constellation_chan(&self) -> &IpcSender<ConstellationMsg> {
        &self.constellation_chan
    }

    pub fn scheduler_chan(&self) -> &IpcSender<TimerEventRequest> {
        &self.scheduler_chan
    }
}

fn timestamp_in_ms(time: Timespec) -> u64 {
    (time.sec * 1000 + (time.nsec / 1000000) as i64) as u64
}
