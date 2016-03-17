/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::StorageEventBinding;
use dom::bindings::codegen::Bindings::StorageEventBinding::{StorageEventMethods};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutNullableHeap, Root, RootedReference};
use dom::bindings::reflector::reflect_dom_object;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::storage::Storage;
use string_cache::Atom;
use util::str::DOMString;

#[dom_struct]
pub struct StorageEvent {
    event: Event,
    key: Option<DOMString>,
    oldValue: Option<DOMString>,
    newValue: Option<DOMString>,
    url: DOMString,
    storageArea: MutNullableHeap<JS<Storage>>
}


impl StorageEvent {
    pub fn new_inherited(key: Option<DOMString>,
                         oldValue: Option<DOMString>,
                         newValue: Option<DOMString>,
                         url: DOMString,
                         storageArea: Option<&Storage>) -> StorageEvent {
        StorageEvent {
            event: Event::new_inherited(),
            key: key,
            oldValue: oldValue,
            newValue: newValue,
            url: url,
            storageArea: MutNullableHeap::new(storageArea)
        }
    }

    pub fn new(global: GlobalRef,
               type_: Atom,
               bubbles: EventBubbles,
               cancelable: EventCancelable,
               key: Option<DOMString>,
               oldValue: Option<DOMString>,
               newValue: Option<DOMString>,
               url: DOMString,
               storageArea: Option<&Storage>) -> Root<StorageEvent> {
        let ev = reflect_dom_object(box StorageEvent::new_inherited(key, oldValue, newValue,
                                                                    url, storageArea),
                                    global,
                                    StorageEventBinding::Wrap);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bool::from(bubbles), bool::from(cancelable));
        }
        ev
    }

    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &StorageEventBinding::StorageEventInit) -> Fallible<Root<StorageEvent>> {
        let key = init.key.clone();
        let oldValue = init.oldValue.clone();
        let newValue = init.newValue.clone();
        let url = init.url.clone();
        let storageArea = init.storageArea.r();
        let bubbles = EventBubbles::from(init.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.cancelable);
        let event = StorageEvent::new(global, Atom::from(type_),
                                      bubbles, cancelable,
                                      key, oldValue, newValue,
                                      url, storageArea);
        Ok(event)
    }
}

impl StorageEventMethods for StorageEvent {
    // https://html.spec.whatwg.org/multipage/#dom-storageevent-key
    fn GetKey(&self) -> Option<DOMString> {
        self.key.clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-storageevent-oldvalue
    fn GetOldValue(&self) -> Option<DOMString> {
        self.oldValue.clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-storageevent-newvalue
    fn GetNewValue(&self) -> Option<DOMString> {
        self.newValue.clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-storageevent-url
    fn Url(&self) -> DOMString {
        self.url.clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-storageevent-storagearea
    fn GetStorageArea(&self) -> Option<Root<Storage>> {
        self.storageArea.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
