/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use script_bindings::callback::ExceptionHandling;
use script_bindings::inheritance::Castable;
use script_bindings::root::{Dom, DomRoot};
use script_bindings::script_runtime::CanGc;

use crate::ScriptThread;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::types::{EventTarget, HTMLSlotElement, MutationObserver, MutationRecord};
use crate::microtask::{Microtask, MicrotaskQueue};

/// A helper struct for mutation observers used in `ScriptThread`
/// Since the Rc is always stored in ScriptThread, it's always reachable by the GC.
#[derive(JSTraceable, Default)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::allow_unrooted_in_rc)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct ScriptMutationObservers {
    /// Microtask Queue for adding support for mutation observer microtasks
    mutation_observer_microtask_queued: Cell<bool>,

    /// The unit of related similar-origin browsing contexts' list of MutationObserver objects
    mutation_observers: DomRefCell<Vec<Dom<MutationObserver>>>,
}

impl ScriptMutationObservers {
    pub(crate) fn add_mutation_observer(&self, observer: &MutationObserver) {
        self.mutation_observers
            .borrow_mut()
            .push(Dom::from_ref(observer));
    }

    /// <https://dom.spec.whatwg.org/#notify-mutation-observers>
    pub(crate) fn notify_mutation_observers(&self, can_gc: CanGc) {
        // Step 1. Set the surrounding agent’s mutation observer microtask queued to false.
        self.mutation_observer_microtask_queued.set(false);

        // Step 2. Let notifySet be a clone of the surrounding agent’s pending mutation observers.
        // TODO Step 3. Empty the surrounding agent’s pending mutation observers.
        let notify_list = self.mutation_observers.borrow();

        // Step 4. Let signalSet be a clone of the surrounding agent’s signal slots.
        // Step 5. Empty the surrounding agent’s signal slots.
        let signal_set: Vec<DomRoot<HTMLSlotElement>> = ScriptThread::take_signal_slots();

        // Step 6. For each mo of notifySet:
        for mo in notify_list.iter() {
            let record_queue = mo.record_queue();

            // Step 6.1 Let records be a clone of mo’s record queue.
            let queue: Vec<DomRoot<MutationRecord>> = record_queue.borrow().clone();

            // Step 6.2 Empty mo’s record queue.
            record_queue.borrow_mut().clear();

            // TODO Step 6.3 For each node of mo’s node list, remove all transient registered observers
            // whose observer is mo from node’s registered observer list.

            // Step 6.4 If records is not empty, then invoke mo’s callback with « records,
            // mo » and "report", and with callback this value mo.
            if !queue.is_empty() {
                let _ = mo
                    .callback()
                    .Call_(&**mo, queue, mo, ExceptionHandling::Report, can_gc);
            }
        }

        // Step 6. For each slot of signalSet, fire an event named slotchange,
        // with its bubbles attribute set to true, at slot.
        for slot in signal_set {
            slot.upcast::<EventTarget>()
                .fire_event(atom!("slotchange"), can_gc);
        }
    }

    /// <https://dom.spec.whatwg.org/#queue-a-mutation-observer-compound-microtask>
    pub(crate) fn queue_mutation_observer_microtask(&self, microtask_queue: Rc<MicrotaskQueue>) {
        // Step 1. If the surrounding agent’s mutation observer microtask queued is true, then return.
        if self.mutation_observer_microtask_queued.get() {
            return;
        }

        // Step 2. Set the surrounding agent’s mutation observer microtask queued to true.
        self.mutation_observer_microtask_queued.set(true);

        // Step 3. Queue a microtask to notify mutation observers.
        crate::script_thread::with_script_thread(|script_thread| {
            microtask_queue.enqueue(Microtask::NotifyMutationObservers, script_thread.get_cx());
        });
    }
}
