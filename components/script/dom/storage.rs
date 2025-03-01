/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;
use net_traits::storage_thread::{StorageThreadMsg, StorageType};
use net_traits::IpcSend;
use profile_traits::ipc;
use script_traits::ScriptMsg;
use servo_url::ServoUrl;

use crate::dom::bindings::codegen::Bindings::StorageBinding::StorageMethods;
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::storageevent::StorageEvent;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct Storage {
    reflector_: Reflector,
    #[no_trace]
    storage_type: StorageType,
}

impl Storage {
    fn new_inherited(storage_type: StorageType) -> Storage {
        Storage {
            reflector_: Reflector::new(),
            storage_type,
        }
    }

    pub(crate) fn new(
        global: &Window,
        storage_type: StorageType,
        can_gc: CanGc,
    ) -> DomRoot<Storage> {
        reflect_dom_object(
            Box::new(Storage::new_inherited(storage_type)),
            global,
            can_gc,
        )
    }

    fn get_url(&self) -> ServoUrl {
        self.global().get_url()
    }

    fn get_storage_thread(&self) -> IpcSender<StorageThreadMsg> {
        self.global().resource_threads().sender()
    }
}

impl StorageMethods<crate::DomTypeHolder> for Storage {
    // https://html.spec.whatwg.org/multipage/#dom-storage-length
    fn Length(&self) -> u32 {
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();

        self.get_storage_thread()
            .send(StorageThreadMsg::Length(
                sender,
                self.get_url(),
                self.storage_type,
            ))
            .unwrap();
        receiver.recv().unwrap() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-storage-key
    fn Key(&self, index: u32) -> Option<DOMString> {
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();

        self.get_storage_thread()
            .send(StorageThreadMsg::Key(
                sender,
                self.get_url(),
                self.storage_type,
                index,
            ))
            .unwrap();
        receiver.recv().unwrap().map(DOMString::from)
    }

    // https://html.spec.whatwg.org/multipage/#dom-storage-getitem
    fn GetItem(&self, name: DOMString) -> Option<DOMString> {
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
        let name = String::from(name);

        let msg = StorageThreadMsg::GetItem(sender, self.get_url(), self.storage_type, name);
        self.get_storage_thread().send(msg).unwrap();
        receiver.recv().unwrap().map(DOMString::from)
    }

    // https://html.spec.whatwg.org/multipage/#dom-storage-setitem
    fn SetItem(&self, name: DOMString, value: DOMString) -> ErrorResult {
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
        let name = String::from(name);
        let value = String::from(value);

        let msg = StorageThreadMsg::SetItem(
            sender,
            self.get_url(),
            self.storage_type,
            name.clone(),
            value.clone(),
        );
        self.get_storage_thread().send(msg).unwrap();
        match receiver.recv().unwrap() {
            Err(_) => Err(Error::QuotaExceeded),
            Ok((changed, old_value)) => {
                if changed {
                    self.broadcast_change_notification(Some(name), old_value, Some(value));
                }
                Ok(())
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-storage-removeitem
    fn RemoveItem(&self, name: DOMString) {
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
        let name = String::from(name);

        let msg =
            StorageThreadMsg::RemoveItem(sender, self.get_url(), self.storage_type, name.clone());
        self.get_storage_thread().send(msg).unwrap();
        if let Some(old_value) = receiver.recv().unwrap() {
            self.broadcast_change_notification(Some(name), Some(old_value), None);
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-storage-clear
    fn Clear(&self) {
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();

        self.get_storage_thread()
            .send(StorageThreadMsg::Clear(
                sender,
                self.get_url(),
                self.storage_type,
            ))
            .unwrap();
        if receiver.recv().unwrap() {
            self.broadcast_change_notification(None, None, None);
        }
    }

    // https://html.spec.whatwg.org/multipage/#the-storage-interface:supported-property-names
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();

        self.get_storage_thread()
            .send(StorageThreadMsg::Keys(
                sender,
                self.get_url(),
                self.storage_type,
            ))
            .unwrap();
        receiver
            .recv()
            .unwrap()
            .into_iter()
            .map(DOMString::from)
            .collect()
    }

    // check-tidy: no specs after this line
    fn NamedGetter(&self, name: DOMString) -> Option<DOMString> {
        self.GetItem(name)
    }

    fn NamedSetter(&self, name: DOMString, value: DOMString) -> ErrorResult {
        self.SetItem(name, value)
    }

    fn NamedDeleter(&self, name: DOMString) {
        self.RemoveItem(name);
    }
}

impl Storage {
    /// <https://html.spec.whatwg.org/multipage/#send-a-storage-notification>
    fn broadcast_change_notification(
        &self,
        key: Option<String>,
        old_value: Option<String>,
        new_value: Option<String>,
    ) {
        let storage = self.storage_type;
        let url = self.get_url();
        let msg = ScriptMsg::BroadcastStorageEvent(storage, url, key, old_value, new_value);
        self.global()
            .script_to_constellation_chan()
            .send(msg)
            .unwrap();
    }

    /// <https://html.spec.whatwg.org/multipage/#send-a-storage-notification>
    pub(crate) fn queue_storage_event(
        &self,
        url: ServoUrl,
        key: Option<String>,
        old_value: Option<String>,
        new_value: Option<String>,
    ) {
        let global = self.global();
        let this = Trusted::new(self);
        global.task_manager().dom_manipulation_task_source().queue(
            task!(send_storage_notification: move || {
                let this = this.root();
                let global = this.global();
                let event = StorageEvent::new(
                    global.as_window(),
                    atom!("storage"),
                    EventBubbles::DoesNotBubble,
                    EventCancelable::NotCancelable,
                    key.map(DOMString::from),
                    old_value.map(DOMString::from),
                    new_value.map(DOMString::from),
                    DOMString::from(url.into_string()),
                    Some(&this),
                    CanGc::note()
                );
                event.upcast::<Event>().fire(global.upcast(), CanGc::note());
            }),
        );
    }
}
