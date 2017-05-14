/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
#![allow(unused_variables)]
use core::ops::Deref;
use dom::bindings::callback::ExceptionHandling;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::MutationObserverBinding;
use dom::bindings::codegen::Bindings::MutationObserverBinding::MutationCallback;
use dom::bindings::codegen::Bindings::MutationObserverBinding::MutationObserverBinding::MutationObserverMethods;
use dom::bindings::codegen::Bindings::MutationObserverBinding::MutationObserverInit;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::mutationrecord::MutationRecord;
use dom::node::Node;
use dom::window::Window;
use dom_struct::dom_struct;
use html5ever::Namespace;
use microtask::Microtask;
use script_thread::ScriptThread;
use servo_atoms::Atom;
use std::cell::{Cell, Ref};
use std::rc::Rc;

#[dom_struct]
pub struct MutationObserver {
    reflector_: Reflector,
    #[ignore_heap_size_of = "can't measure Rc values"]
    callback: Rc<MutationCallback>,
    record_queue: DOMRefCell<Vec<Root<MutationRecord>>>,
    attribute_old_value: Cell<bool>,
    attributes: Cell<bool>,
    character_data: Cell<bool>,
    character_data_old_value: Cell<bool>,
    child_list: Cell<bool>,
    subtree: Cell<bool>,
    attribute_filter: DOMRefCell<Vec<DOMString>>,
}

#[derive(Clone)]
pub enum Mutation {
    Attribute { name: Atom, namespace: Namespace, oldValue: DOMString, newValue: DOMString}
}

impl MutationObserver {
    fn new(global: &Window, callback: Rc<MutationCallback>) -> Root<MutationObserver> {
        let boxed_observer = box MutationObserver::new_inherited(callback);
        reflect_dom_object(boxed_observer, global, MutationObserverBinding::Wrap)
    }

    fn new_inherited(callback: Rc<MutationCallback>) -> MutationObserver {
        MutationObserver {
            reflector_: Reflector::new(),
            callback: callback,
            record_queue: DOMRefCell::new(vec![]),
            attribute_filter: DOMRefCell::new(vec![]),
            attribute_old_value: Cell::from(false),
            attributes: Cell::from(false),
            character_data: Cell::from(false),
            character_data_old_value: Cell::from(false),
            child_list: Cell::from(false),
            subtree: Cell::from(false),
        }
    }

    pub fn Constructor(global: &Window, callback: Rc<MutationCallback>) -> Fallible<Root<MutationObserver>> {
        let observer = MutationObserver::new(global, callback);
        ScriptThread::add_mutation_observer(&*observer);
        Ok(observer)
    }

    /// https://dom.spec.whatwg.org/#queue-a-mutation-observer-compound-microtask
    /// Queue a Mutation Observer compound Microtask.
    pub fn queueMutationObserverCompoundMicrotask() {
        // Step 1
        if ScriptThread::is_mutation_observer_compound_microtask_queued() {
            return;
        }
        // Step 2
        ScriptThread::set_mutation_observer_compound_microtask_queued(true);
        // Step 3
        let compoundMicrotask = Microtask::NotifyMutationObservers();
        ScriptThread::enqueue_microtask(compoundMicrotask);
    }

    /// https://dom.spec.whatwg.org/#notify-mutation-observers
    /// Notify Mutation Observers.
    pub fn notifyMutationObservers() -> Fallible<()> {
        // Step 1
        ScriptThread::set_mutation_observer_compound_microtask_queued(false);
        // Step 2
        let notifyList = ScriptThread::get_mutation_observer();
        // Step 3, Step 4 not needed as Servo doesn't implement anything related to slots yet.
        // Step 5: Ignore the specific text about execute a compound microtask.
        for (_, mo) in notifyList.iter().enumerate() {
            let queue: Vec<Root<MutationRecord>> = mo.record_queue.borrow().clone();
            *mo.record_queue.borrow_mut() = Vec::new();
            // TODO: Step 5.3 Remove all transient registered observers whose observer is mo.
            if !queue.is_empty() {
                let _ = mo.callback.Call_(&**mo, queue, &**mo, ExceptionHandling::Report);
            }
        }
        // Step 6 not needed as Servo doesn't implement anything related to slots yet.
        Ok(())
    }

    //https://dom.spec.whatwg.org/#queueing-a-mutation-record
    //Queuing a mutation record
    pub fn queue_a_mutation_record(target: &Node, attr_type: Mutation) {
        // Step 1
        let mut interestedObservers: Vec<(Root<MutationObserver>, DOMString)> = vec![];
        // Step 2
        let mut nodes: Vec<Root<Node>> = vec![];
        for ancestor in target.inclusive_ancestors() {
            nodes.push(ancestor);
        }
        // Step 3
        for node in &nodes {
            for registered_observer in node.registered_mutation_observers().borrow().iter() {
                match attr_type {
                    // TODO check for Mutations other than Attribute
                    Mutation::Attribute { ref name, ref namespace, ref oldValue, ref newValue } => {
                        let condition1: bool = node != &Root::from_ref(target) &&
                            !registered_observer.subtree.get();
                        let condition2: bool = registered_observer.attributes.get() == false;
                        let condition3: bool = !registered_observer.attribute_filter.borrow().is_empty() &&
                            (registered_observer.attribute_filter.borrow().iter()
                                .find(|s| &**s == &**name).is_none() || namespace != &ns!());
                        if !condition1 && !condition2 && !condition3 {
                            // Step 3.1.2
                            let paired_string = if registered_observer.attribute_old_value.get() {
                                DOMString::from(oldValue.deref())
                            } else {
                                DOMString::from("")
                            };
                            // Step 3.1.1
                            let idx = interestedObservers.iter().position(|&(ref o, _)|
                                &**o as *const MutationObserver == &**registered_observer as *const MutationObserver);
                            if let Some(idx) = idx {
                                interestedObservers[idx].1 = paired_string;
                            } else {
                                interestedObservers.push((Root::from_ref(registered_observer), paired_string));
                            }
                        }
                    }
                }
            }
        }
        // Step 4
        for &(ref observer, ref paired_string) in &interestedObservers {
            // Step 4.1
            let record = &MutationRecord::new(DOMString::from("attributes"), target);
            // Step 4.2
            match attr_type {
                Mutation::Attribute { ref name, ref namespace, ref oldValue, ref newValue } => {
                    record.SetAttributeName(DOMString::from(&**name));
                    if namespace != &ns!() {
                        record.SetAttributeNamespace(DOMString::from(&**namespace));
                    }
                }
            }
            // Step 4.3-4.6- TODO currently not relevant to mutation records for attribute mutations
            // Step 4.7
            if paired_string != "" {
                record.SetoldValue(paired_string.clone());
            }
            // Step 4.8
            observer.record_queue.borrow_mut().push(record.clone());
        }
        // Step 5
        MutationObserver::queueMutationObserverCompoundMicrotask();
    }

}

impl MutationObserverMethods for MutationObserver {
    /// https://dom.spec.whatwg.org/#dom-mutationobserver-observe
    /// MutationObserver.observe method
    fn Observe(&self, target: &Node, options: &MutationObserverInit) -> Fallible<()> {
        let mut options_cp: MutationObserverInit = MutationObserverInit {
            attributeFilter: options.attributeFilter.clone(),
            attributeOldValue: options.attributeOldValue,
            attributes: options.attributes,
            characterData: options.characterData,
            characterDataOldValue: options.characterDataOldValue,
            childList: options.childList,
            subtree: options.subtree
        };
        // Step 1: If either options’ attributeOldValue or attributeFilter is present and
        // options’ attributes is omitted, set options’ attributes to true.
        if (options_cp.attributeOldValue.is_some() || options_cp.attributeFilter.is_some()) &&
            options_cp.attributes.is_none() {
            options_cp.attributes = Some(true);
        }
        // Step2: If options’ characterDataOldValue is present and options’ characterData is omitted,
        // set options’ characterData to true.
        if options_cp.characterDataOldValue.is_some() && options_cp.characterData.is_some() {
            options_cp.characterData = Some(true);
        }
        // Step3: If none of options’ childList, attributes, and characterData is true, throw a TypeError.
        if !options_cp.childList && !options_cp.attributes.unwrap_or(false) &&
            !options_cp.characterData.unwrap_or(false) {
            return Err(Error::Type("childList, attributes, and characterData not true".to_owned()));
        }
        // Step4: If options’ attributeOldValue is true and options’ attributes is false, throw a TypeError.
        if options_cp.attributeOldValue.unwrap_or(false) && !options_cp.attributes.unwrap_or(false) {
            return Err(Error::Type("attributeOldValue is true but attributes is false".to_owned()));
        }
        // Step5: If options’ attributeFilter is present and options’ attributes is false, throw a TypeError.
        if options_cp.attributeFilter.is_some() && !options_cp.attributes.unwrap_or(false) {
            return Err(Error::Type("attributeFilter is present but attributes is false".to_owned()));
        }
        // Step6: If options’ characterDataOldValue is true and options’ characterData is false, throw a TypeError.
        if options_cp.characterDataOldValue.unwrap_or(false) && !options_cp.characterData.unwrap_or(false) {
            return Err(Error::Type("characterDataOldValue is true but characterData is false".to_owned()));
        }
        // Step 7
        for registered in target.registered_mutation_observers().borrow().iter() {
            if &**registered as *const MutationObserver == self as *const MutationObserver {
                // TODO: remove matching transient registered observers
                registered.attribute_old_value.set(options_cp.attributeOldValue.unwrap_or(false));
                registered.attributes.set(options_cp.attributes.unwrap_or(false));
                registered.character_data.set(options_cp.characterData.unwrap_or(false));
                registered.character_data_old_value.set(options_cp.characterDataOldValue.unwrap_or(false));
                registered.child_list.set(options_cp.childList);
                registered.subtree.set(options_cp.subtree);
                *registered.attribute_filter.borrow_mut() = options_cp.attributeFilter.clone().unwrap();
            }
        }
        // Step 8
        let observers = target.registered_mutation_observers();
        let idx = observers.borrow().iter().position(|registered| {
            &**registered as *const MutationObserver == self as *const MutationObserver
        });
        let registered = match idx {
            Some(idx) => Ref::map(observers.borrow(), |obs| &*obs[idx]),
            None => {
                target.add_registered_mutation_observer(self);
                let idx = observers.borrow().len() - 1;
                Ref::map(observers.borrow(), |obs| &*obs[idx])
            }
        };
        // TODO: remove matching transient registered observers
        registered.attribute_old_value.set(options_cp.attributeOldValue.unwrap_or(false));
        registered.attributes.set(options_cp.attributes.unwrap_or(false));
        registered.character_data.set(options_cp.characterData.unwrap_or(false));
        registered.character_data_old_value.set(options_cp.characterDataOldValue.unwrap_or(false));
        registered.child_list.set(options_cp.childList);
        registered.subtree.set(options_cp.subtree);
        *registered.attribute_filter.borrow_mut() = options_cp.attributeFilter.clone().unwrap_or(vec![]);

        Ok(())
    }
}
