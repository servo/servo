/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventBinding::{EventMethods};
use dom::bindings::codegen::Bindings::StorageEventBinding;
use dom::bindings::codegen::Bindings::StorageEventBinding::{StorageEventMethods};

use dom::bindings::codegen::InheritTypes::{EventCast};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, JSRef, MutNullableHeap, Rootable, RootedReference};
use dom::bindings::js::Temporary;
use dom::bindings::utils::{reflect_dom_object};
use dom::event::{Event, EventTypeId, EventBubbles, EventCancelable};
use dom::storage::Storage;
use util::str::DOMString;

#[dom_struct]
pub struct StorageEvent {
    event: Event,
    key: DOMRefCell<Option<DOMString>>,
    oldValue: DOMRefCell<Option<DOMString>>,
    newValue: DOMRefCell<Option<DOMString>>,
    url: DOMRefCell<DOMString>,
    storageArea: MutNullableHeap<JS<Storage>>
}


impl StorageEvent {
    pub fn new_inherited(type_id: EventTypeId,
                         key: Option<DOMString>,
                         oldValue: Option<DOMString>,
                         newValue: Option<DOMString>,
                         url: DOMString,
                         storageArea: Option<JSRef<Storage>>) -> StorageEvent {
        StorageEvent {
            event: Event::new_inherited(type_id),
            key: DOMRefCell::new(key),
            oldValue: DOMRefCell::new(oldValue),
            newValue: DOMRefCell::new(newValue),
            url: DOMRefCell::new(url),
            storageArea: MutNullableHeap::new(storageArea.map(JS::from_rooted))
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
               storageArea: Option<JSRef<Storage>>) -> Temporary<StorageEvent> {
        let ev = reflect_dom_object(box StorageEvent::new_inherited(EventTypeId::StorageEvent,
                                                                    key, oldValue, newValue,
                                                                    url, storageArea),
                                    global,
                                    StorageEventBinding::Wrap).root();
        let event: JSRef<Event> = EventCast::from_ref(ev.r());
        event.InitEvent(type_, bubbles == EventBubbles::Bubbles, cancelable == EventCancelable::Cancelable);
        Temporary::from_rooted(ev.r())
    }

    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &StorageEventBinding::StorageEventInit) -> Fallible<Temporary<StorageEvent>> {
        let key = init.key.clone();
        let oldValue = init.oldValue.clone();
        let newValue = init.newValue.clone();
        let url = init.url.clone();
        let storageArea = init.storageArea.r();
        let bubbles = if init.parent.bubbles { EventBubbles::Bubbles } else { EventBubbles::DoesNotBubble };
        let cancelable = if init.parent.cancelable { EventCancelable::Cancelable } else { EventCancelable::NotCancelable };
        let event = StorageEvent::new(global, type_,
                                      bubbles, cancelable,
                                      key, oldValue, newValue,
                                      url, storageArea);
        Ok(event)
    }
}

impl<'a> StorageEventMethods for JSRef<'a, StorageEvent> {
    fn GetKey(self) -> Option<DOMString> {
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let key = self.key.borrow();
        key.clone()
    }

    fn GetOldValue(self) -> Option<DOMString> {
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let oldValue = self.oldValue.borrow();
        oldValue.clone()
    }

    fn GetNewValue(self) -> Option<DOMString> {
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let newValue = self.newValue.borrow();
        newValue.clone()
    }

    fn Url(self) -> DOMString {
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let url = self.url.borrow();
        url.clone()
    }

    fn GetStorageArea(self) -> Option<Temporary<Storage>> {
        self.storageArea.get().map(Temporary::from_rooted)
    }

}
