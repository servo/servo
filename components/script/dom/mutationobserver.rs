/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::ExceptionHandling;
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::MutationObserverBinding;
use dom::bindings::codegen::Bindings::MutationObserverBinding::MutationCallback;
use dom::bindings::codegen::Bindings::MutationObserverBinding::MutationObserverBinding::MutationObserverMethods;
use dom::bindings::codegen::Bindings::MutationObserverBinding::MutationObserverInit;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::mutationrecord::MutationRecord;
use dom::node::Node;
use dom::window::Window;
use dom_struct::dom_struct;
use html5ever::{Namespace, LocalName};
use microtask::Microtask;
use script_thread::ScriptThread;
use std::rc::Rc;

#[dom_struct]
pub struct MutationObserver {
    reflector_: Reflector,
    #[ignore_heap_size_of = "can't measure Rc values"]
    callback: Rc<MutationCallback>,
    record_queue: DomRefCell<Vec<DomRoot<MutationRecord>>>,
}

pub enum Mutation<'a> {
    Attribute { name: LocalName, namespace: Namespace, old_value: DOMString },
    ChildList { added: Option<&'a [&'a Node]>, removed: Option<&'a [&'a Node]>,
                prev: Option<&'a Node>, next: Option<&'a Node> },
}

#[derive(HeapSizeOf, JSTraceable)]
pub struct RegisteredObserver {
    observer: DomRoot<MutationObserver>,
    options: ObserverOptions,
}

#[derive(HeapSizeOf, JSTraceable)]
pub struct ObserverOptions {
    attribute_old_value: bool,
    attributes: bool,
    character_data: bool,
    character_data_old_value: bool,
    child_list: bool,
    subtree: bool,
    attribute_filter: Vec<DOMString>,
}

impl MutationObserver {
    fn new(global: &Window, callback: Rc<MutationCallback>) -> DomRoot<MutationObserver> {
        let boxed_observer = Box::new(MutationObserver::new_inherited(callback));
        reflect_dom_object(boxed_observer, global, MutationObserverBinding::Wrap)
    }

    fn new_inherited(callback: Rc<MutationCallback>) -> MutationObserver {
        MutationObserver {
            reflector_: Reflector::new(),
            callback: callback,
            record_queue: DomRefCell::new(vec![]),
        }
    }

    pub fn Constructor(global: &Window, callback: Rc<MutationCallback>) -> Fallible<DomRoot<MutationObserver>> {
        let observer = MutationObserver::new(global, callback);
        ScriptThread::add_mutation_observer(&*observer);
        Ok(observer)
    }

    /// https://dom.spec.whatwg.org/#queue-a-mutation-observer-compound-microtask
    pub fn queue_mutation_observer_compound_microtask() {
        // Step 1
        if ScriptThread::is_mutation_observer_compound_microtask_queued() {
            return;
        }
        // Step 2
        ScriptThread::set_mutation_observer_compound_microtask_queued(true);
        // Step 3
        ScriptThread::enqueue_microtask(Microtask::NotifyMutationObservers);
    }

    /// https://dom.spec.whatwg.org/#notify-mutation-observers
    pub fn notify_mutation_observers() {
        // Step 1
        ScriptThread::set_mutation_observer_compound_microtask_queued(false);
        // Step 2
        let notify_list = ScriptThread::get_mutation_observers();
        // TODO: steps 3-4 (slots)
        // Step 5
        for mo in &notify_list {
            let queue: Vec<DomRoot<MutationRecord>> = mo.record_queue.borrow().clone();
            mo.record_queue.borrow_mut().clear();
            // TODO: Step 5.3 Remove all transient registered observers whose observer is mo.
            if !queue.is_empty() {
                let _ = mo.callback.Call_(&**mo, queue, &**mo, ExceptionHandling::Report);
            }
        }
        // TODO: Step 6 (slot signals)
    }

    /// https://dom.spec.whatwg.org/#queueing-a-mutation-record
    pub fn queue_a_mutation_record(target: &Node, attr_type: Mutation) {
        // Step 1
        let mut interestedObservers: Vec<(DomRoot<MutationObserver>, Option<DOMString>)> = vec![];
        // Step 2 & 3
        for node in target.inclusive_ancestors() {
            for registered in &*node.registered_mutation_observers() {
                if &*node != target && !registered.options.subtree {
                    continue;
                }

                match attr_type {
                    Mutation::Attribute { ref name, ref namespace, ref old_value } => {
                        // Step 3.1
                        if !registered.options.attributes {
                            continue;
                        }
                        if !registered.options.attribute_filter.is_empty() {
                            if *namespace != ns!() {
                                continue;
                            }
                            if registered.options.attribute_filter.iter()
                                .find(|s| &**s == &**name).is_some() {
                                continue;
                            }
                        }
                        // Step 3.1.2
                        let paired_string = if registered.options.attribute_old_value {
                            Some(old_value.clone())
                        } else {
                            None
                        };
                        // Step 3.1.1
                        let idx = interestedObservers.iter().position(|&(ref o, _)|
                            &**o as *const _ == &*registered.observer as *const _);
                        if let Some(idx) = idx {
                            interestedObservers[idx].1 = paired_string;
                        } else {
                            interestedObservers.push((DomRoot::from_ref(&*registered.observer),
                                                      paired_string));
                        }
                    },
                    Mutation::ChildList { .. } => {
                        if !registered.options.child_list {
                            continue;
                        }
                        interestedObservers.push((DomRoot::from_ref(&*registered.observer), None));
                    }
                }
            }
        }

        // Step 4
        for &(ref observer, ref paired_string) in &interestedObservers {
            // Steps 4.1-4.7
            let record = match attr_type {
                Mutation::Attribute { ref name, ref namespace, .. } => {
                    let namespace = if *namespace != ns!() {
                        Some(namespace)
                    } else {
                        None
                    };
                    MutationRecord::attribute_mutated(target, name, namespace, paired_string.clone())
                },
                Mutation::ChildList { ref added, ref removed, ref next, ref prev } => {
                    MutationRecord::child_list_mutated(target, *added, *removed, *next, *prev)
                }
            };
            // Step 4.8
            observer.record_queue.borrow_mut().push(record);
        }

        // Step 5
        MutationObserver::queue_mutation_observer_compound_microtask();
    }

}

impl MutationObserverMethods for MutationObserver {
    /// https://dom.spec.whatwg.org/#dom-mutationobserver-observe
    fn Observe(&self, target: &Node, options: &MutationObserverInit) -> Fallible<()> {
        let attribute_filter = options.attributeFilter.clone().unwrap_or(vec![]);
        let attribute_old_value = options.attributeOldValue.unwrap_or(false);
        let mut attributes = options.attributes.unwrap_or(false);
        let mut character_data = options.characterData.unwrap_or(false);
        let character_data_old_value = options.characterDataOldValue.unwrap_or(false);
        let child_list = options.childList;
        let subtree = options.subtree;

        // Step 1
        if (options.attributeOldValue.is_some() || options.attributeFilter.is_some()) &&
            options.attributes.is_none() {
            attributes = true;
        }

        // Step 2
        if options.characterDataOldValue.is_some() && options.characterData.is_none() {
            character_data = true;
        }

        // Step 3
        if !child_list && !attributes && !character_data {
            return Err(Error::Type("One of childList, attributes, or characterData must be true".into()));
        }

        // Step 4
        if attribute_old_value && !attributes {
            return Err(Error::Type("attributeOldValue is true but attributes is false".into()));
        }

        // Step 5
        if options.attributeFilter.is_some() && !attributes {
            return Err(Error::Type("attributeFilter is present but attributes is false".into()));
        }

        // Step 6
        if character_data_old_value && !character_data {
            return Err(Error::Type("characterDataOldValue is true but characterData is false".into()));
        }

        // Step 7
        let add_new_observer = {
            let mut replaced = false;
            for registered in &mut *target.registered_mutation_observers() {
                if &*registered.observer as *const MutationObserver != self as *const MutationObserver {
                    continue;
                }
                // TODO: remove matching transient registered observers
                registered.options.attribute_old_value = attribute_old_value;
                registered.options.attributes = attributes;
                registered.options.character_data = character_data;
                registered.options.character_data_old_value = character_data_old_value;
                registered.options.child_list = child_list;
                registered.options.subtree = subtree;
                registered.options.attribute_filter = attribute_filter.clone();
                replaced = true;
            }
            !replaced
        };

        // Step 8
        if add_new_observer {
            target.registered_mutation_observers().push(RegisteredObserver {
                observer: DomRoot::from_ref(self),
                options: ObserverOptions {
                    attributes,
                    attribute_old_value,
                    character_data,
                    character_data_old_value,
                    subtree,
                    attribute_filter,
                    child_list
                },
            });
        }

        Ok(())
    }
}
