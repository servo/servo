/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::reflect_dom_object_with_proto;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::StorageEventBinding;
use crate::dom::bindings::codegen::Bindings::StorageEventBinding::StorageEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::{RootedDomString, USVString};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::storage::Storage;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct StorageEvent {
    event: Event,
    key: DomRefCell<Option<RootedDomString>>,
    old_value: DomRefCell<Option<RootedDomString>>,
    new_value: DomRefCell<Option<RootedDomString>>,
    url: DomRefCell<RootedDomString>,
    storage_area: MutNullableDom<Storage>,
}

#[expect(non_snake_case)]
impl StorageEvent {
    pub(crate) fn new_inherited(
        key: Option<RootedDomString>,
        old_value: Option<RootedDomString>,
        new_value: Option<RootedDomString>,
        url: RootedDomString,
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

    pub(crate) fn new_uninitialized(
        cx: &mut JSContext,
        window: &Window,
        url: RootedDomString,
    ) -> DomRoot<StorageEvent> {
        Self::new_uninitialized_with_proto(cx, window, None, url)
    }

    fn new_uninitialized_with_proto(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        url: RootedDomString,
    ) -> DomRoot<StorageEvent> {
        reflect_dom_object_with_proto(
            cx,
            Box::new(StorageEvent::new_inherited(None, None, None, url, None)),
            window,
            proto,
        )
    }

    #[expect(clippy::too_many_arguments)]
    pub(crate) fn new(
        cx: &mut JSContext,
        global: &Window,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        key: Option<RootedDomString>,
        oldValue: Option<RootedDomString>,
        newValue: Option<RootedDomString>,
        url: RootedDomString,
        storageArea: Option<&Storage>,
    ) -> DomRoot<StorageEvent> {
        Self::new_with_proto(
            cx,
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

    #[expect(clippy::too_many_arguments)]
    fn new_with_proto(
        cx: &mut JSContext,
        global: &Window,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        key: Option<RootedDomString>,
        oldValue: Option<RootedDomString>,
        newValue: Option<RootedDomString>,
        url: RootedDomString,
        storageArea: Option<&Storage>,
    ) -> DomRoot<StorageEvent> {
        let ev = reflect_dom_object_with_proto(
            cx,
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
}

#[expect(non_snake_case)]
impl StorageEventMethods<crate::DomTypeHolder> for StorageEvent {
    /// <https://html.spec.whatwg.org/multipage/#storageevent>
    fn Constructor(
        cx: &mut JSContext,
        global: &Window,
        proto: Option<HandleObject>,
        type_: RootedDomString,
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
            cx,
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

    /// <https://html.spec.whatwg.org/multipage/#dom-storageevent-key>
    fn GetKey(&self) -> Option<RootedDomString> {
        self.key.borrow().clone()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-storageevent-oldvalue>
    fn GetOldValue(&self) -> Option<RootedDomString> {
        self.old_value.borrow().clone()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-storageevent-newvalue>
    fn GetNewValue(&self) -> Option<RootedDomString> {
        self.new_value.borrow().clone()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-storageevent-url>
    fn Url(&self) -> RootedDomString {
        self.url.borrow().clone()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-storageevent-storagearea>
    fn GetStorageArea(&self) -> Option<DomRoot<Storage>> {
        self.storage_area.get()
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-storageevent-initstorageevent>
    fn InitStorageEvent(
        &self,
        type_: RootedDomString,
        bubbles: bool,
        cancelable: bool,
        key: Option<RootedDomString>,
        oldValue: Option<RootedDomString>,
        newValue: Option<RootedDomString>,
        url: USVString,
        storageArea: Option<&Storage>,
    ) {
        self.event
            .init_event(Atom::from(type_), bubbles, cancelable);
        *self.key.borrow_mut() = key;
        *self.old_value.borrow_mut() = oldValue;
        *self.new_value.borrow_mut() = newValue;
        *self.url.borrow_mut() = url.into();
        self.storage_area.set(storageArea);
    }
}
