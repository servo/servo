/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::StorageBinding;
use dom::bindings::codegen::Bindings::StorageBinding::StorageMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use servo_util::str::DOMString;

#[dom_struct]
pub struct Storage {
    reflector_: Reflector,
}

impl Storage {
    fn new_inherited() -> Storage {
        Storage {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: &GlobalRef) -> Temporary<Storage> {
        reflect_dom_object(box Storage::new_inherited(), global, StorageBinding::Wrap)
    }
}

impl<'a> StorageMethods for JSRef<'a, Storage> {
    fn Length(self) -> u32 {
        0
    }

    fn Key(self, index: u32) -> Option<DOMString> {

        //Return null for out of range index
        if index >= self.Length() {
            return None;
        }

        return None;
    }

    fn GetItem(self, key: DOMString) -> Option<DOMString> {
        if key.is_empty() {
            return None;
        }

        return None;
    }

    fn NamedGetter(self, key: DOMString, found: &mut bool) -> Option<DOMString> {
        let item = self.GetItem(key);
        *found = item.is_some();
        item
    }

    fn Clear(self) {
    }
}

impl Reflectable for Storage {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
