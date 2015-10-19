/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::{EventMethods};
use dom::bindings::codegen::Bindings::StorageEventBinding;
use dom::bindings::codegen::Bindings::StorageEventBinding::{StorageEventMethods};
use dom::bindings::conversions::Castable;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutNullableHeap, Root, RootedReference};
use dom::bindings::utils::{reflect_dom_object};
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::storage::Storage;
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
               type_: DOMString,
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
            event.InitEvent(type_, bubbles == EventBubbles::Bubbles, cancelable == EventCancelable::Cancelable);
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
        let bubbles = if init.parent.bubbles { EventBubbles::Bubbles } else { EventBubbles::DoesNotBubble };
        let cancelable = if init.parent.cancelable {
            EventCancelable::Cancelable
        } else {
            EventCancelable::NotCancelable
        };
        let event = StorageEvent::new(global, type_,
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
        self.storageArea.get_rooted()
    }
}
