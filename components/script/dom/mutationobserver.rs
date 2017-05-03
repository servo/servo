/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::MutationObserverBinding;
use dom::bindings::codegen::Bindings::MutationObserverBinding::MutationCallback;
use dom::bindings::codegen::Bindings::MutationObserverBinding::MutationObserverBinding::MutationObserverMethods;
use dom::bindings::codegen::Bindings::MutationObserverBinding::MutationObserverInit;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::mutationrecord::MutationRecord;
use dom::node::Node;
use dom::window::Window;
use dom_struct::dom_struct;
//use html5ever_atoms::Namespace;
use js::jsapi::{JSContext, JSObject};
use microtask::Microtask;
use script_thread::ScriptThread;
use servo_atoms::Atom;
use std::cell::Cell;
use std::rc::Rc;
use style::Namespace;

#[dom_struct]
pub struct MutationObserver {
    reflector_: Reflector,
    #[ignore_heap_size_of = "can't measure Rc values"]
    callback: Rc<MutationCallback>,
    record_queue: Vec<Root<MutationRecord>>,
//    #[ignore_heap_size_of = "can't measure DomRefCell values"]
//    options: Cell<MutationObserverInit>,
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
            record_queue: vec![],
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
    pub fn notifyMutationObservers() {
        // Step 1
        ScriptThread::set_mutation_observer_compound_microtask_queued(false);
        // Step 2
        let notifyList = ScriptThread::get_mutation_observer();
        // Step 3, Step 4 not needed as Servo doesn't implement anything related to slots yet.
        // Step 5: Ignore the specific text about execute a compound microtask.
        // Step 6 not needed as Servo doesn't implement anything related to slots yet.
    }

    //https://dom.spec.whatwg.org/#queueing-a-mutation-record
    //Queuing a mutation record
    pub fn queue_a_mutation_record(target: &Node, attr_type: Mutation) {
        let mut given_name = "";
        let mut given_namespace = "";
        let mut old_value = "";
        // Step 1
        let mut interestedObservers: Vec<Root<MutationObserver>> = vec![];
        let mut pairedStrings: Vec<DOMString> = vec![];
        // Step 2
        let mut nodes: Vec<Root<Node>> = vec![];
        for ancestor in target.inclusive_ancestors() {
            nodes.push(ancestor);
        }
        // Step 3
        for node in &nodes {
            for registered_observer in node.registered_mutation_observers().borrow().iter() {
                match attr_type {
                    Mutation::Attribute { ref name, ref namespace, ref oldValue, ref newValue } => {
                        let given_name = &*name.clone();
                        let given_namespace = &*namespace.clone();
                        let old_value = &*oldValue.clone();
                    },
                }
                let condition1: bool = node != &Root::from_ref(target) &&
                    !registered_observer.subtree.get();
                let condition2: bool = given_name == "attribute" &&
                    registered_observer.attributes.get() == false;
                let condition3: bool = (given_name == "attribute" &&
                    registered_observer.attribute_filter != DOMRefCell::new(vec![])) &&
                    (given_namespace != "");
                let condition4: bool = given_name == "characterData" &&
                    registered_observer.character_data.get() == false;
                let condition5: bool = given_name == "childList" &&
                    !registered_observer.child_list.get();
                if !condition1 && !condition2 && !condition3 && !condition4 && !condition5 {
                    // Step 3.1
                    if !interestedObservers.contains(registered_observer) {
                        interestedObservers.push(Root::from_ref(registered_observer));
                    }
                    // Step 3.2
                    let condition: bool = (given_name == "attribute" &&
                        registered_observer.attribute_old_value.get() == true) ||
                            (given_name == "characterData" &&
                                registered_observer.character_data_old_value.get() == true);
                    if condition {
                        pairedStrings.push(DOMString::from(old_value));
                    }
                }
            }
        }
        // Step 4
        let mut i = 0;
        for observer in &interestedObservers {
            //Step 4.1
            let mut record = &MutationRecord::new(DOMString::from(&*given_name), target);
            //Step 4.2
            if given_name != "" && given_namespace != "" {
                record.SetAttributeName(DOMString::from(&*given_name));
                record.SetAttributeNamespace(DOMString::from(&*given_namespace));
            }
            //Step 4.3-4.6- TODO currently not relevant to mutation records for attribute mutations
            //Step 4.7
            if pairedStrings[i] != "" {
                record.SetoldValue(pairedStrings[i].clone());
            }
            i = i + 1;
        }
        // Step 5
        MutationObserver::queueMutationObserverCompoundMicrotask();
    }

}

impl MutationObserverMethods for MutationObserver {
    /// https://dom.spec.whatwg.org/#dom-mutationobserver-observe
    /// MutationObserver.observe method
    fn Observe(&self, target: &Node, options: &MutationObserverInit) -> Fallible<()> {
        // Step 1: If either options’ attributeOldValue or attributeFilter is present and
        // options’ attributes is omitted, set options’ attributes to true.
        if (options.attributeOldValue.is_some() || options.attributeFilter.is_some()) && options.attributes.is_none() {
//            options.attributes.set(Some(true));
        }
        // Step2: If options’ characterDataOldValue is present and options’ characterData is omitted,
        // set options’ characterData to true.
        if options.characterDataOldValue.is_some() && options.characterData.is_some() {
//            options.characterData.set(Some(true));
        }
        // Step3: If none of options’ childList, attributes, and characterData is true, throw a TypeError.
        if !options.childList && !options.attributes.unwrap_or(false) && !options.characterData.unwrap_or(false) {
            return Err(Error::Type("childList, attributes, and characterData not true".to_owned()));
        }
        // Step4: If options’ attributeOldValue is true and options’ attributes is false, throw a TypeError.
        if options.attributeOldValue.unwrap_or(false) && !options.attributes.unwrap_or(false) {
            return Err(Error::Type("attributeOldValue is true but attributes is false".to_owned()));
        }
        // Step5: If options’ attributeFilter is present and options’ attributes is false, throw a TypeError.
        if options.attributeFilter.is_some() && !options.attributes.unwrap_or(false) {
            return Err(Error::Type("attributeFilter is present but attributes is false".to_owned()));
        }
        // Step6: If options’ characterDataOldValue is true and options’ characterData is false, throw a TypeError.
        if options.characterDataOldValue.unwrap_or(false) && !options.characterData.unwrap_or(false) {
            return Err(Error::Type("characterDataOldValue is true but characterData is false".to_owned()));
        }
        // TODO: Step 7
        for registered in target.registered_mutation_observers().borrow().iter() {
//            if &*registered as *const Root<MutationObserver> == self as *const MutationObserver {
                // TODO: remove matching transient registered observers
                if let Some(value) = options.attributeOldValue {
                    registered.attribute_old_value.set(value);
                }
                if let Some(value) = options.attributes {
                    registered.attributes.set(value);
                }
                if let Some(value) = options.characterData {
                    registered.character_data.set(value);
                }
                if let Some(value) = options.characterDataOldValue {
                    registered.character_data_old_value.set(value);
                }
                registered.child_list.set(options.childList);
                registered.subtree.set(options.subtree);
                *registered.attribute_filter.borrow_mut() = options.attributeFilter.clone().unwrap();
//            }
            // TODO: Step 8
            target.add_registered_mutation_observer(&self);
        }
        Ok(())
    }
}
