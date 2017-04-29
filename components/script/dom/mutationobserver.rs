/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::borrow::BorrowMut;
use dom::bindings::codegen::Bindings::MutationObserverBinding;
use dom::bindings::codegen::Bindings::MutationObserverBinding::MutationCallback;
use dom::bindings::codegen::Bindings::MutationObserverBinding::MutationObserverBinding::MutationObserverMethods;
use dom::bindings::codegen::Bindings::MutationObserverBinding::MutationObserverInit;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::mutationrecord::MutationRecord;
//use dom::element::Atom;
use dom::node::Node;
use dom::window::Window;
use dom_struct::dom_struct;
use html5ever_atoms::Namespace;
use microtask::Microtask;
use script_thread::ScriptThread;
use servo_atoms::Atom;
use std::default::Default;
use std::option::Option;
use std::rc::Rc;

#[dom_struct]
pub struct MutationObserver {
    reflector_: Reflector,
    #[ignore_heap_size_of = "can't measure Rc values"]
    callback: Rc<MutationCallback>,
    record_queue: Vec<Root<MutationRecord>>,
}

#[derive(Debug)]
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
        }
    }

    pub fn Constructor(global: &Window, callback: Rc<MutationCallback>) -> Fallible<Root<MutationObserver>> {
        let observer = MutationObserver::new(global, callback);
        ScriptThread::add_mutation_observer(&*observer);
        Ok(observer)
    }

    /// https://dom.spec.whatwg.org/#queue-a-mutation-observer-compound-microtask
    /// Queue a Mutation Observer compound Microtask.
    pub fn queueMutationObserverCompoundMicrotask(&self, compoundMicrotask: Microtask) {
        // Step 1
        if ScriptThread::get_mutation_observer_compound_microtask_queued() {
            return;
        }
        // Step 2
        ScriptThread::set_mutation_observer_compound_microtask_queued(true);
        // Step 3
        ScriptThread::enqueue_microtask(compoundMicrotask);
    }

    /// https://dom.spec.whatwg.org/#notify-mutation-observers
    /// Notify Mutation Observers.
    pub fn notifyMutationObservers() {
        // Step 1
        ScriptThread::set_mutation_observer_compound_microtask_queued(false);
        // Step 2
//        let notifyList = ScriptThread::get_mutation_observer();
        // Step 3, Step 4 not needed as Servo doesn't implement anything related to slots yet.
        // Step 5
//        Ignore the specific text about execute a compound microtask.
        // Step 6 not needed as Servo doesn't implement anything related to slots yet.
    }
}

impl MutationObserverMethods for MutationObserver {
    /// https://dom.spec.whatwg.org/#dom-mutationobserver-observe
    /// MutationObserver.observe method
    fn Observe(&self, target: &Node, options: &MutationObserverInit) -> Fallible<()> {
        let mut options = options;
        // Step 1: If either options’ attributeOldValue or attributeFilter is present and
        // options’ attributes is omitted, set options’ attributes to true.
        if (options.attributeOldValue.is_some() || options.attributeFilter.is_some()) && options.attributes.is_none() {
            options.attributes = Some(true);
        }
        // Step2: If options’ characterDataOldValue is present and options’ characterData is omitted,
        // set options’ characterData to true.
        if options.characterDataOldValue.is_some() && options.characterData.is_some() {
            options.characterData = Some(true);
        }
        // Step3: If none of options’ childList, attributes, and characterData is true, throw a TypeError.
        if !options.childList && !options.attributes.unwrap() && !options.characterData.unwrap() {
            return Err(Error::Type("childList, attributes, and characterData not true".to_owned()));
        }
        // Step4: If options’ attributeOldValue is true and options’ attributes is false, throw a TypeError.
        if options.attributeOldValue.unwrap() && !options.attributes.unwrap() {
            return Err(Error::Type("attributeOldValue is true but attributes is false".to_owned()));
        }
        // Step5: If options’ attributeFilter is present and options’ attributes is false, throw a TypeError.
        if options.attributeFilter.is_some() && !options.attributes.unwrap() {
            return Err(Error::Type("attributeFilter is present but attributes is false".to_owned()));
        }
        // Step6: If options’ characterDataOldValue is true and options’ characterData is false, throw a TypeError.
        if options.characterDataOldValue.unwrap() && !options.characterData.unwrap() {
            return Err(Error::Type("characterDataOldValue is true but characterData is false".to_owned()));
        }
        // TODO: Step 7
        //let mut registeredObservers = &target.registered_mutation_observers_for_type().into_iter();

        for registered in target.registered_mutation_observers_for_type().borrow().iter(){
            if &*registered as *const MutationObserver == self as *const MutationObserver{
                
               //TODO: 
            }
            // TODO: Step 8
            //else {

                
            //}
        }
        Ok(())
    }
}
