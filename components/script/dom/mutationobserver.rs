/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::MutationObserverBinding;
use dom::bindings::codegen::Bindings::MutationObserverBinding::MutationCallback;
use dom::bindings::codegen::Bindings::MutationObserverBinding::MutationObserverBinding::MutationObserverMethods;
use dom::bindings::codegen::Bindings::MutationObserverBinding::MutationObserverInit;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::node::Node;
use dom::window::Window;
use dom_struct::dom_struct;
use script_thread::ScriptThread;
use std::default::Default;
use std::option::Option;
use std::rc::Rc;

#[dom_struct]
pub struct MutationObserver {
    reflector_: Reflector,
    #[ignore_heap_size_of = "can't measure Rc values"]
    callback: Rc<MutationCallback>,
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
        }
    }

    pub fn Constructor(global: &Window, callback: Rc<MutationCallback>) -> Fallible<Root<MutationObserver>> {
        let observer = MutationObserver::new(global, callback);
        ScriptThread::add_mutation_observer(&*observer);
        Ok(observer)
    }
}

impl MutationObserverMethods for MutationObserver {
    /// https://dom.spec.whatwg.org/#dom-mutationobserver-observe
    /// MutationObserver.observe method
    fn Observe(&self, target: &Node, options: &MutationObserverInit) -> Fallible<()> {
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
        // TODO: Step 8
        Ok(())
    }
}
