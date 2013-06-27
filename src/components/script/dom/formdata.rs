/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{WrapperCache, DOMString, str};
use dom::blob::Blob;
use std::hashmap::HashMap;

enum FormDatum {
    StringData(DOMString),
    BlobData { blob: @mut Blob, name: DOMString }
}

pub struct FormData {
    data: HashMap<~str, FormDatum>,
    wrapper: WrapperCache
}

impl FormData {
    pub fn new() -> @mut FormData {
        @mut FormData {
            data: HashMap::new(),
            wrapper: WrapperCache::new()
        }
    }

    pub fn Append(&mut self, name: DOMString, value: @mut Blob, filename: Option<DOMString>) {
        let blob = BlobData {
            blob: value,
            name: filename.get_or_default(str(~"default"))
        };
        self.data.insert(name.to_str(), blob);
    }

    pub fn Append_(&mut self, name: DOMString, value: DOMString) {
        self.data.insert(name.to_str(), StringData(value));
    }
}