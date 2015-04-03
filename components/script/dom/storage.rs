/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::StorageBinding;
use dom::bindings::codegen::Bindings::StorageBinding::StorageMethods;
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{JSRef, Temporary, RootedReference};
use dom::bindings::refcounted::Trusted;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::bindings::codegen::InheritTypes::{EventCast, EventTargetCast};
use dom::event::{Event, EventHelpers, EventBubbles, EventCancelable};
use dom::eventtarget::{EventTarget};
use dom::storageevent::StorageEvent;
use dom::urlhelper::UrlHelper;
use dom::window::WindowHelpers;
use util::str::DOMString;
use page::IterablePage;
use net_traits::storage_task::{StorageTask, StorageTaskMsg, StorageType};
use std::sync::mpsc::channel;
use url::Url;

use script_task::{ScriptTask, ScriptMsg, MainThreadRunnable};

use collections::borrow::ToOwned;

#[dom_struct]
pub struct Storage {
    reflector_: Reflector,
    global: GlobalField,
    storage_type: StorageType
}

impl Storage {
    fn new_inherited(global: &GlobalRef, storage_type: StorageType) -> Storage {
        Storage {
            reflector_: Reflector::new(),
            global: GlobalField::from_rooted(global),
            storage_type: storage_type
        }
    }

    pub fn new(global: &GlobalRef, storage_type: StorageType) -> Temporary<Storage> {
        reflect_dom_object(box Storage::new_inherited(global, storage_type), *global, StorageBinding::Wrap)
    }

    fn get_url(&self) -> Url {
        let global_root = self.global.root();
        let global_ref = global_root.r();
        global_ref.get_url()
    }

    fn get_storage_task(&self) -> StorageTask {
        let global_root = self.global.root();
        let global_ref = global_root.r();
        global_ref.as_window().storage_task()
    }

}

impl<'a> StorageMethods for JSRef<'a, Storage> {
    fn Length(self) -> u32 {
        let (sender, receiver) = channel();

        self.get_storage_task().send(StorageTaskMsg::Length(sender, self.get_url(), self.storage_type)).unwrap();
        receiver.recv().unwrap()
    }

    fn Key(self, index: u32) -> Option<DOMString> {
        let (sender, receiver) = channel();

        self.get_storage_task().send(StorageTaskMsg::Key(sender, self.get_url(), self.storage_type, index)).unwrap();
        receiver.recv().unwrap()
    }

    fn GetItem(self, name: DOMString) -> Option<DOMString> {
        let (sender, receiver) = channel();

        self.get_storage_task().send(StorageTaskMsg::GetItem(sender, self.get_url(), self.storage_type, name)).unwrap();
        receiver.recv().unwrap()
    }

    fn NamedGetter(self, name: DOMString, found: &mut bool) -> Option<DOMString> {
        let item = self.GetItem(name);
        *found = item.is_some();
        item
    }

    fn SetItem(self, name: DOMString, value: DOMString) {
        let (sender, receiver) = channel();

        self.get_storage_task().send(StorageTaskMsg::SetItem(sender, self.get_url(), self.storage_type, name.clone(), value.clone())).unwrap();
        let (changed, old_value) = receiver.recv().unwrap();
        if changed {
            self.broadcast_change_notification(Some(name), old_value, Some(value));
        }
    }

    fn NamedSetter(self, name: DOMString, value: DOMString) {
        self.SetItem(name, value);
    }

    fn NamedCreator(self, name: DOMString, value: DOMString) {
        self.SetItem(name, value);
    }

    fn RemoveItem(self, name: DOMString) {
        let (sender, receiver) = channel();

        self.get_storage_task().send(StorageTaskMsg::RemoveItem(sender, self.get_url(), self.storage_type, name.clone())).unwrap();
        if let Some(old_value) = receiver.recv().unwrap() {
            self.broadcast_change_notification(Some(name), Some(old_value), None);
        }
    }

    fn NamedDeleter(self, name: DOMString) {
        self.RemoveItem(name);
    }

    fn Clear(self) {
        let (sender, receiver) = channel();

        self.get_storage_task().send(StorageTaskMsg::Clear(sender, self.get_url(), self.storage_type)).unwrap();
        if receiver.recv().unwrap() {
            self.broadcast_change_notification(None, None, None);
        }
    }
}

trait PrivateStorageHelpers {
    fn broadcast_change_notification(self, key: Option<DOMString>, old_value: Option<DOMString>,
                                     new_value: Option<DOMString>);
}

impl<'a> PrivateStorageHelpers for JSRef<'a, Storage> {
    /// https://html.spec.whatwg.org/multipage/webstorage.html#send-a-storage-notification
    fn broadcast_change_notification(self, key: Option<DOMString>, old_value: Option<DOMString>,
                                     new_value: Option<DOMString>){
        let global_root = self.global.root();
        let global_ref = global_root.r();
        let script_chan = global_ref.script_chan();
        let trusted_storage = Trusted::new(global_ref.get_cx(), self,
                                           script_chan.clone());
        script_chan.send(ScriptMsg::MainThreadRunnableMsg(
            box StorageEventRunnable::new(trusted_storage, key,
                                          old_value, new_value))).unwrap();
    }
}

pub struct StorageEventRunnable {
    element: Trusted<Storage>,
    key: Option<DOMString>,
    old_value: Option<DOMString>,
    new_value: Option<DOMString>
}

impl StorageEventRunnable {
    fn new(storage: Trusted<Storage>, key: Option<DOMString>, old_value: Option<DOMString>,
           new_value: Option<DOMString>) -> StorageEventRunnable {
        StorageEventRunnable { element: storage, key: key, old_value: old_value, new_value: new_value }
    }
}

impl MainThreadRunnable for StorageEventRunnable {
    fn handler(self: Box<StorageEventRunnable>, script_task: &ScriptTask) {
        let this = *self;
        let storage_root = this.element.to_temporary().root();
        let storage = storage_root.r();
        let global_root = storage.global.root();
        let global_ref = global_root.r();
        let ev_window = global_ref.as_window();
        let ev_url = storage.get_url();

        let storage_event = StorageEvent::new(
            global_ref,
            "storage".to_owned(),
            EventBubbles::DoesNotBubble, EventCancelable::NotCancelable,
            this.key, this.old_value, this.new_value,
            ev_url.to_string(),
            Some(storage)
        ).root();
        let event: JSRef<Event> = EventCast::from_ref(storage_event.r());

        let root_page = script_task.root_page();
        for it_page in root_page.iter() {
            let it_window_root = it_page.window().root();
            let it_window = it_window_root.r();
            assert!(UrlHelper::SameOrigin(&ev_url, &it_window.get_url()));
            // TODO: Such a Document object is not necessarily fully active, but events fired on such
            // objects are ignored by the event loop until the Document becomes fully active again.
            if ev_window.pipeline() != it_window.pipeline() {
                let target: JSRef<EventTarget> = EventTargetCast::from_ref(it_window);
                event.fire(target);
            }
        }
    }
}
