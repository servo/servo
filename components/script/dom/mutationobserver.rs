/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::LazyCell;
use std::rc::Rc;

use dom_struct::dom_struct;
use html5ever::{namespace_url, ns, LocalName, Namespace};
use js::rust::HandleObject;

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::MutationObserverBinding::MutationObserver_Binding::MutationObserverMethods;
use crate::dom::bindings::codegen::Bindings::MutationObserverBinding::{
    MutationCallback, MutationObserverInit,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::mutationrecord::MutationRecord;
use crate::dom::node::{Node, ShadowIncluding};
use crate::dom::window::Window;
use crate::microtask::Microtask;
use crate::script_runtime::CanGc;
use crate::script_thread::ScriptThread;

#[dom_struct]
pub(crate) struct MutationObserver {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "can't measure Rc values"]
    callback: Rc<MutationCallback>,
    record_queue: DomRefCell<Vec<DomRoot<MutationRecord>>>,
    node_list: DomRefCell<Vec<DomRoot<Node>>>,
}

pub(crate) enum Mutation<'a> {
    Attribute {
        name: LocalName,
        namespace: Namespace,
        old_value: Option<DOMString>,
    },
    CharacterData {
        old_value: DOMString,
    },
    ChildList {
        added: Option<&'a [&'a Node]>,
        removed: Option<&'a [&'a Node]>,
        prev: Option<&'a Node>,
        next: Option<&'a Node>,
    },
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct RegisteredObserver {
    pub(crate) observer: DomRoot<MutationObserver>,
    options: ObserverOptions,
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct ObserverOptions {
    attribute_old_value: bool,
    attributes: bool,
    character_data: bool,
    character_data_old_value: bool,
    child_list: bool,
    subtree: bool,
    attribute_filter: Vec<DOMString>,
}

impl MutationObserver {
    fn new_with_proto(
        global: &Window,
        proto: Option<HandleObject>,
        callback: Rc<MutationCallback>,
        can_gc: CanGc,
    ) -> DomRoot<MutationObserver> {
        let boxed_observer = Box::new(MutationObserver::new_inherited(callback));
        reflect_dom_object_with_proto(boxed_observer, global, proto, can_gc)
    }

    fn new_inherited(callback: Rc<MutationCallback>) -> MutationObserver {
        MutationObserver {
            reflector_: Reflector::new(),
            callback,
            record_queue: DomRefCell::new(vec![]),
            node_list: DomRefCell::new(vec![]),
        }
    }

    /// <https://dom.spec.whatwg.org/#queue-a-mutation-observer-compound-microtask>
    pub(crate) fn queue_mutation_observer_microtask() {
        // Step 1. If the surrounding agent’s mutation observer microtask queued is true, then return.
        if ScriptThread::is_mutation_observer_microtask_queued() {
            return;
        }

        // Step 2. Set the surrounding agent’s mutation observer microtask queued to true.
        ScriptThread::set_mutation_observer_microtask_queued(true);

        // Step 3. Queue a microtask to notify mutation observers.
        ScriptThread::enqueue_microtask(Microtask::NotifyMutationObservers);
    }

    /// <https://dom.spec.whatwg.org/#notify-mutation-observers>
    pub(crate) fn notify_mutation_observers(can_gc: CanGc) {
        // Step 1. Set the surrounding agent’s mutation observer microtask queued to false.
        ScriptThread::set_mutation_observer_microtask_queued(false);

        // Step 2. Let notifySet be a clone of the surrounding agent’s pending mutation observers.
        // TODO Step 3. Empty the surrounding agent’s pending mutation observers.
        let notify_list = ScriptThread::get_mutation_observers();

        // Step 4. Let signalSet be a clone of the surrounding agent’s signal slots.
        // Step 5. Empty the surrounding agent’s signal slots.
        let signal_set = ScriptThread::take_signal_slots();

        // Step 6. For each mo of notifySet:
        for mo in &notify_list {
            // Step 6.1 Let records be a clone of mo’s record queue.
            let queue: Vec<DomRoot<MutationRecord>> = mo.record_queue.borrow().clone();

            // Step 6.2 Empty mo’s record queue.
            mo.record_queue.borrow_mut().clear();

            // TODO Step 6.3 For each node of mo’s node list, remove all transient registered observers
            // whose observer is mo from node’s registered observer list.

            // Step 6.4 If records is not empty, then invoke mo’s callback with « records,
            // mo » and "report", and with callback this value mo.
            if !queue.is_empty() {
                let _ = mo
                    .callback
                    .Call_(&**mo, queue, mo, ExceptionHandling::Report);
            }
        }

        // Step 6. For each slot of signalSet, fire an event named slotchange,
        // with its bubbles attribute set to true, at slot.
        for slot in signal_set {
            slot.upcast::<EventTarget>()
                .fire_event(atom!("slotchange"), can_gc);
        }
    }

    /// <https://dom.spec.whatwg.org/#queueing-a-mutation-record>
    pub(crate) fn queue_a_mutation_record<'a, F>(
        target: &Node,
        attr_type: LazyCell<Mutation<'a>, F>,
    ) where
        F: FnOnce() -> Mutation<'a>,
    {
        if !target.global().as_window().get_exists_mut_observer() {
            return;
        }
        // Step 1
        let mut interested_observers: Vec<(DomRoot<MutationObserver>, Option<DOMString>)> = vec![];

        // Step 2 & 3
        for node in target.inclusive_ancestors(ShadowIncluding::No) {
            let registered = node.registered_mutation_observers();
            if registered.is_none() {
                continue;
            }

            for registered in &*registered.unwrap() {
                if &*node != target && !registered.options.subtree {
                    continue;
                }

                match *attr_type {
                    Mutation::Attribute {
                        ref name,
                        ref namespace,
                        ref old_value,
                    } => {
                        // Step 3.1
                        if !registered.options.attributes {
                            continue;
                        }
                        if !registered.options.attribute_filter.is_empty() {
                            if *namespace != ns!() {
                                continue;
                            }
                            if !registered
                                .options
                                .attribute_filter
                                .iter()
                                .any(|s| **s == **name)
                            {
                                continue;
                            }
                        }
                        // Step 3.1.2
                        let paired_string = if registered.options.attribute_old_value {
                            old_value.clone()
                        } else {
                            None
                        };
                        // Step 3.1.1
                        let idx = interested_observers
                            .iter()
                            .position(|(o, _)| std::ptr::eq(&**o, &*registered.observer));
                        if let Some(idx) = idx {
                            interested_observers[idx].1 = paired_string;
                        } else {
                            interested_observers
                                .push((DomRoot::from_ref(&*registered.observer), paired_string));
                        }
                    },
                    Mutation::CharacterData { ref old_value } => {
                        if !registered.options.character_data {
                            continue;
                        }
                        // Step 3.1.2
                        let paired_string = if registered.options.character_data_old_value {
                            Some(old_value.clone())
                        } else {
                            None
                        };
                        // Step 3.1.1
                        let idx = interested_observers
                            .iter()
                            .position(|(o, _)| std::ptr::eq(&**o, &*registered.observer));
                        if let Some(idx) = idx {
                            interested_observers[idx].1 = paired_string;
                        } else {
                            interested_observers
                                .push((DomRoot::from_ref(&*registered.observer), paired_string));
                        }
                    },
                    Mutation::ChildList { .. } => {
                        if !registered.options.child_list {
                            continue;
                        }
                        interested_observers.push((DomRoot::from_ref(&*registered.observer), None));
                    },
                }
            }
        }

        // Step 4
        for (observer, paired_string) in interested_observers {
            // Steps 4.1-4.7
            let record = match *attr_type {
                Mutation::Attribute {
                    ref name,
                    ref namespace,
                    ..
                } => {
                    let namespace = if *namespace != ns!() {
                        Some(namespace)
                    } else {
                        None
                    };
                    MutationRecord::attribute_mutated(target, name, namespace, paired_string)
                },
                Mutation::CharacterData { .. } => {
                    MutationRecord::character_data_mutated(target, paired_string)
                },
                Mutation::ChildList {
                    ref added,
                    ref removed,
                    ref next,
                    ref prev,
                } => MutationRecord::child_list_mutated(target, *added, *removed, *next, *prev),
            };
            // Step 4.8
            observer.record_queue.borrow_mut().push(record);
        }

        // Step 5
        MutationObserver::queue_mutation_observer_microtask();
    }
}

impl MutationObserverMethods<crate::DomTypeHolder> for MutationObserver {
    /// <https://dom.spec.whatwg.org/#dom-mutationobserver-mutationobserver>
    fn Constructor(
        global: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        callback: Rc<MutationCallback>,
    ) -> Fallible<DomRoot<MutationObserver>> {
        global.set_exists_mut_observer();
        let observer = MutationObserver::new_with_proto(global, proto, callback, can_gc);
        ScriptThread::add_mutation_observer(&observer);
        Ok(observer)
    }

    /// <https://dom.spec.whatwg.org/#dom-mutationobserver-observe>
    fn Observe(&self, target: &Node, options: &MutationObserverInit) -> Fallible<()> {
        let attribute_filter = options.attributeFilter.clone().unwrap_or_default();
        let attribute_old_value = options.attributeOldValue.unwrap_or(false);
        let mut attributes = options.attributes.unwrap_or(false);
        let mut character_data = options.characterData.unwrap_or(false);
        let character_data_old_value = options.characterDataOldValue.unwrap_or(false);
        let child_list = options.childList;
        let subtree = options.subtree;

        // Step 1
        if (options.attributeOldValue.is_some() || options.attributeFilter.is_some()) &&
            options.attributes.is_none()
        {
            attributes = true;
        }

        // Step 2
        if options.characterDataOldValue.is_some() && options.characterData.is_none() {
            character_data = true;
        }

        // Step 3
        if !child_list && !attributes && !character_data {
            return Err(Error::Type(
                "One of childList, attributes, or characterData must be true".into(),
            ));
        }

        // Step 4
        if attribute_old_value && !attributes {
            return Err(Error::Type(
                "attributeOldValue is true but attributes is false".into(),
            ));
        }

        // Step 5
        if options.attributeFilter.is_some() && !attributes {
            return Err(Error::Type(
                "attributeFilter is present but attributes is false".into(),
            ));
        }

        // Step 6
        if character_data_old_value && !character_data {
            return Err(Error::Type(
                "characterDataOldValue is true but characterData is false".into(),
            ));
        }

        // Step 7
        let add_new_observer = {
            let mut replaced = false;
            for registered in &mut *target.registered_mutation_observers_mut() {
                if &*registered.observer as *const MutationObserver !=
                    self as *const MutationObserver
                {
                    continue;
                }
                // TODO: remove matching transient registered observers
                registered.options.attribute_old_value = attribute_old_value;
                registered.options.attributes = attributes;
                registered.options.character_data = character_data;
                registered.options.character_data_old_value = character_data_old_value;
                registered.options.child_list = child_list;
                registered.options.subtree = subtree;
                registered
                    .options
                    .attribute_filter
                    .clone_from(&attribute_filter);
                replaced = true;
            }
            !replaced
        };

        // Step 8
        if add_new_observer {
            target.add_mutation_observer(RegisteredObserver {
                observer: DomRoot::from_ref(self),
                options: ObserverOptions {
                    attributes,
                    attribute_old_value,
                    character_data,
                    character_data_old_value,
                    subtree,
                    attribute_filter,
                    child_list,
                },
            });

            self.node_list.borrow_mut().push(DomRoot::from_ref(target));
        }

        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-mutationobserver-takerecords>
    fn TakeRecords(&self) -> Vec<DomRoot<MutationRecord>> {
        let records: Vec<DomRoot<MutationRecord>> = self.record_queue.borrow().clone();
        self.record_queue.borrow_mut().clear();
        records
    }

    /// <https://dom.spec.whatwg.org/#dom-mutationobserver-disconnect>
    fn Disconnect(&self) {
        // Step 1
        let mut nodes = self.node_list.borrow_mut();
        for node in nodes.drain(..) {
            node.remove_mutation_observer(self);
        }

        // Step 2
        self.record_queue.borrow_mut().clear();
    }
}
