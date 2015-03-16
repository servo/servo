/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::StorageBinding;
use dom::bindings::codegen::Bindings::StorageBinding::StorageMethods;
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use util::str::DOMString;
use net::storage_task::StorageTask;
use net::storage_task::StorageType;
use net::storage_task::StorageTaskMsg;
use std::sync::mpsc::channel;
use url::Url;

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

        self.get_storage_task().send(StorageTaskMsg::SetItem(sender, self.get_url(), self.storage_type, name, value)).unwrap();
        if receiver.recv().unwrap() {
            //TODO send notification
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

        self.get_storage_task().send(StorageTaskMsg::RemoveItem(sender, self.get_url(), self.storage_type, name)).unwrap();
        if receiver.recv().unwrap() {
            //TODO send notification
        }
    }

    fn NamedDeleter(self, name: DOMString) {
        self.RemoveItem(name);
    }

    fn Clear(self) {
        let (sender, receiver) = channel();

        self.get_storage_task().send(StorageTaskMsg::Clear(sender, self.get_url(), self.storage_type)).unwrap();
        if receiver.recv().unwrap() {
            //TODO send notification
        }
    }
}
