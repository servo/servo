/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::FileBinding;
use dom::bindings::codegen::Bindings::FileBinding::FileMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::blob::Blob;
use std::sync::Arc;
use util::str::DOMString;

#[dom_struct]
pub struct File {
    blob: Blob,
    name: DOMString,
}

impl File {
    fn new_inherited(_file_bits: &Blob, name: DOMString) -> File {
        File {
            //TODO: get type from the underlying filesystem instead of "".to_string()
            blob: Blob::new_inherited(Arc::new(Vec::new()), None, None, ""),
            name: name,
        }
        // XXXManishearth Once Blob is able to store data
        // the relevant subfields of file_bits should be copied over
    }

    pub fn new(global: GlobalRef, file_bits: &Blob, name: DOMString) -> Root<File> {
        reflect_dom_object(box File::new_inherited(file_bits, name),
                           global,
                           FileBinding::Wrap)
    }

    pub fn name(&self) -> &DOMString {
        &self.name
    }
}

impl FileMethods for File {
    // https://w3c.github.io/FileAPI/#dfn-name
    fn Name(&self) -> DOMString {
        self.name.clone()
    }
}
