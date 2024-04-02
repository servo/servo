/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_atoms::Atom;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::StorageEventBinding;
use crate::dom::bindings::codegen::Bindings::StorageEventBinding::StorageEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::storage::Storage;
use crate::dom::window::Window;

#[dom_struct]
pub struct StorageEvent {
    event: Event,
    key: DomRefCell<Option<DOMString>>,
    old_value: DomRefCell<Option<DOMString>>,
    new_value: DomRefCell<Option<DOMString>>,
    url: DomRefCell<DOMString>,
    storage_area: MutNullableDom<Storage>,
}

#[allow(non_snake_case)]
impl StorageEvent {
    pub fn new_inherited(
        key: Option<DOMString>,
        old_value: Option<DOMString>,
        new_value: Option<DOMString>,
        url: DOMString,
        storage_area: Option<&Storage>,
    ) -> StorageEvent {
        StorageEvent {
            event: Event::new_inherited(),
            key: DomRefCell::new(key),
            old_value: DomRefCell::new(old_value),
            new_value: DomRefCell::new(new_value),
            url: DomRefCell::new(url),
            storage_area: MutNullableDom::new(storage_area),
        }
    }

    pub fn new_uninitialized(window: &Window, url: DOMString) -> DomRoot<StorageEvent> {
        Self::new_uninitialized_with_proto(window, None, url)
    }

    fn new_uninitialized_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        url: DOMString,
    ) -> DomRoot<StorageEvent> {
        reflect_dom_object_with_proto(
            Box::new(StorageEvent::new_inherited(None, None, None, url, None)),
            window,
            proto,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        global: &Window,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        key: Option<DOMString>,
        oldValue: Option<DOMString>,
        newValue: Option<DOMString>,
        url: DOMString,
        storageArea: Option<&Storage>,
    ) -> DomRoot<StorageEvent> {
        Self::new_with_proto(
            global,
            None,
            type_,
            bubbles,
            cancelable,
            key,
            oldValue,
            newValue,
            url,
            storageArea,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_proto(
        global: &Window,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        key: Option<DOMString>,
        oldValue: Option<DOMString>,
        newValue: Option<DOMString>,
        url: DOMString,
        storageArea: Option<&Storage>,
    ) -> DomRoot<StorageEvent> {
        let ev = reflect_dom_object_with_proto(
            Box::new(StorageEvent::new_inherited(
                key,
                oldValue,
                newValue,
                url,
                storageArea,
            )),
            global,
            proto,
        );
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bool::from(bubbles), bool::from(cancelable));
        }
        ev
    }

    pub fn Constructor(
        global: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &StorageEventBinding::StorageEventInit,
    ) -> Fallible<DomRoot<StorageEvent>> {
        let key = init.key.clone();
        let oldValue = init.oldValue.clone();
        let newValue = init.newValue.clone();
        let url = init.url.clone();
        let storageArea = init.storageArea.as_deref();
        let bubbles = EventBubbles::from(init.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.cancelable);
        let event = StorageEvent::new_with_proto(
            global,
            proto,
            Atom::from(type_),
            bubbles,
            cancelable,
            key,
            oldValue,
            newValue,
            url,
            storageArea,
        );
        Ok(event)
    }
}

#[allow(non_snake_case)]
impl StorageEventMethods for StorageEvent {
    // https://html.spec.whatwg.org/multipage/#dom-storageevent-key
    fn GetKey(&self) -> Option<DOMString> {
        self.key.borrow().clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-storageevent-oldvalue
    fn GetOldValue(&self) -> Option<DOMString> {
        self.old_value.borrow().clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-storageevent-newvalue
    fn GetNewValue(&self) -> Option<DOMString> {
        self.new_value.borrow().clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-storageevent-url
    fn Url(&self) -> DOMString {
        self.url.borrow().clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-storageevent-storagearea
    fn GetStorageArea(&self) -> Option<DomRoot<Storage>> {
        self.storage_area.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }

    // https://html.spec.whatwg.org/multipage/#dom-storageevent-initstorageevent
    fn InitStorageEvent(
        &self,
        type_: DOMString,
        bubbles: bool,
        cancelable: bool,
        key: Option<DOMString>,
        oldValue: Option<DOMString>,
        newValue: Option<DOMString>,
        url: USVString,
        storageArea: Option<&Storage>,
    ) {
        self.event
            .init_event(Atom::from(type_), bubbles, cancelable);
        *self.key.borrow_mut() = key;
        *self.old_value.borrow_mut() = oldValue;
        *self.new_value.borrow_mut() = newValue;
        *self.url.borrow_mut() = DOMString::from_string(url.0);
        self.storage_area.set(storageArea);
    }
}
