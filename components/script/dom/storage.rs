/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::StorageBinding;
use dom::bindings::codegen::Bindings::StorageBinding::StorageMethods;
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::storageevent::StorageEvent;
use dom::urlhelper::UrlHelper;
use ipc_channel::ipc::{self, IpcSender};
use net_traits::IpcSend;
use net_traits::storage_thread::{StorageThreadMsg, StorageType};
use script_thread::{Runnable, ScriptThread};
use task_source::TaskSource;
use url::Url;

#[dom_struct]
pub struct Storage {
    reflector_: Reflector,
    storage_type: StorageType
}

impl Storage {
    fn new_inherited(storage_type: StorageType) -> Storage {
        Storage {
            reflector_: Reflector::new(),
            storage_type: storage_type
        }
    }

    pub fn new(global: &GlobalRef, storage_type: StorageType) -> Root<Storage> {
        reflect_dom_object(box Storage::new_inherited(storage_type), *global, StorageBinding::Wrap)
    }

    fn get_url(&self) -> Url {
        let global_root = self.global();
        let global_ref = global_root.r();
        global_ref.get_url()
    }

    fn get_storage_thread(&self) -> IpcSender<StorageThreadMsg> {
        let global_root = self.global();
        let global_ref = global_root.r();
        global_ref.as_window().resource_threads().sender()
    }

}

impl StorageMethods for Storage {
    // https://html.spec.whatwg.org/multipage/#dom-storage-length
    fn Length(&self) -> u32 {
        let (sender, receiver) = ipc::channel().unwrap();

        self.get_storage_thread().send(StorageThreadMsg::Length(sender, self.get_url(), self.storage_type)).unwrap();
        receiver.recv().unwrap() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-storage-key
    fn Key(&self, index: u32) -> Option<DOMString> {
        let (sender, receiver) = ipc::channel().unwrap();

        self.get_storage_thread()
            .send(StorageThreadMsg::Key(sender, self.get_url(), self.storage_type, index))
            .unwrap();
        receiver.recv().unwrap().map(DOMString::from)
    }

    // https://html.spec.whatwg.org/multipage/#dom-storage-getitem
    fn GetItem(&self, name: DOMString) -> Option<DOMString> {
        let (sender, receiver) = ipc::channel().unwrap();
        let name = String::from(name);

        let msg = StorageThreadMsg::GetItem(sender, self.get_url(), self.storage_type, name);
        self.get_storage_thread().send(msg).unwrap();
        receiver.recv().unwrap().map(DOMString::from)
    }

    // https://html.spec.whatwg.org/multipage/#dom-storage-setitem
    fn SetItem(&self, name: DOMString, value: DOMString) -> ErrorResult {
        let (sender, receiver) = ipc::channel().unwrap();
        let name = String::from(name);
        let value = String::from(value);

        let msg = StorageThreadMsg::SetItem(sender, self.get_url(), self.storage_type, name.clone(), value.clone());
        self.get_storage_thread().send(msg).unwrap();
        match receiver.recv().unwrap() {
            Err(_) => Err(Error::QuotaExceeded),
            Ok((changed, old_value)) => {
              if changed {
                  self.broadcast_change_notification(Some(name), old_value, Some(value));
              }
              Ok(())
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-storage-removeitem
    fn RemoveItem(&self, name: DOMString) {
        let (sender, receiver) = ipc::channel().unwrap();
        let name = String::from(name);

        let msg = StorageThreadMsg::RemoveItem(sender, self.get_url(), self.storage_type, name.clone());
        self.get_storage_thread().send(msg).unwrap();
        if let Some(old_value) = receiver.recv().unwrap() {
            self.broadcast_change_notification(Some(name), Some(old_value), None);
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-storage-clear
    fn Clear(&self) {
        let (sender, receiver) = ipc::channel().unwrap();

        self.get_storage_thread().send(StorageThreadMsg::Clear(sender, self.get_url(), self.storage_type)).unwrap();
        if receiver.recv().unwrap() {
            self.broadcast_change_notification(None, None, None);
        }
    }

    // https://html.spec.whatwg.org/multipage/#the-storage-interface:supported-property-names
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        let (sender, receiver) = ipc::channel().unwrap();

        self.get_storage_thread().send(StorageThreadMsg::Keys(sender, self.get_url(), self.storage_type)).unwrap();
        receiver.recv()
                .unwrap()
                .into_iter()
                .map(DOMString::from)
                .collect()
    }

    // check-tidy: no specs after this line
    fn NamedGetter(&self, name: DOMString, found: &mut bool) -> Option<DOMString> {
        let item = self.GetItem(name);
        *found = item.is_some();
        item
    }

    fn NamedSetter(&self, name: DOMString, value: DOMString) -> ErrorResult {
        self.SetItem(name, value)
    }

    fn NamedDeleter(&self, name: DOMString) {
        self.RemoveItem(name);
    }
}


impl Storage {
    /// https://html.spec.whatwg.org/multipage/#send-a-storage-notification
    fn broadcast_change_notification(&self, key: Option<String>, old_value: Option<String>,
                                     new_value: Option<String>) {
        let global_root = self.global();
        let global_ref = global_root.r();
        let window = global_ref.as_window();
        let task_source = window.dom_manipulation_task_source();
        let trusted_storage = Trusted::new(self);
        task_source.queue(box StorageEventRunnable::new(trusted_storage, key, old_value, new_value),
                          global_ref).unwrap();
    }
}

pub struct StorageEventRunnable {
    element: Trusted<Storage>,
    key: Option<String>,
    old_value: Option<String>,
    new_value: Option<String>
}

impl StorageEventRunnable {
    fn new(storage: Trusted<Storage>, key: Option<String>, old_value: Option<String>,
           new_value: Option<String>) -> StorageEventRunnable {
        StorageEventRunnable { element: storage, key: key, old_value: old_value, new_value: new_value }
    }
}

impl Runnable for StorageEventRunnable {
    fn name(&self) -> &'static str { "StorageEventRunnable" }

    fn main_thread_handler(self: Box<StorageEventRunnable>, script_thread: &ScriptThread) {
        let this = *self;
        let storage_root = this.element.root();
        let storage = storage_root.r();
        let global_root = storage.global();
        let global_ref = global_root.r();
        let ev_window = global_ref.as_window();
        let ev_url = storage.get_url();

        let storage_event = StorageEvent::new(
            global_ref,
            atom!("storage"),
            EventBubbles::DoesNotBubble, EventCancelable::NotCancelable,
            this.key.map(DOMString::from), this.old_value.map(DOMString::from), this.new_value.map(DOMString::from),
            DOMString::from(ev_url.to_string()),
            Some(storage)
        );

        let root_context = script_thread.root_browsing_context();
        for it_context in root_context.iter() {
            let it_window_root = it_context.active_window();
            let it_window = it_window_root.r();
            assert!(UrlHelper::SameOrigin(&ev_url, &it_window.get_url()));
            // TODO: Such a Document object is not necessarily fully active, but events fired on such
            // objects are ignored by the event loop until the Document becomes fully active again.
            if ev_window.pipeline() != it_window.pipeline() {
                storage_event.upcast::<Event>().fire(it_window.upcast());
            }
        }
    }
}
