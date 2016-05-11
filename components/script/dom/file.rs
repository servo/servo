/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::FileBinding;
use dom::bindings::codegen::Bindings::FileBinding::FileMethods;
use dom::bindings::codegen::UnionTypes::BlobOrString;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::blob::{Blob, DataSlice, blob_parts_to_bytes};
use std::sync::Arc;
use time;
use util::str::DOMString;

#[dom_struct]
pub struct File {
    blob: Blob,
    name: DOMString,
    modified: i64,
}

impl File {
    fn new_inherited(slice: DataSlice, name: DOMString,
                     modified: Option<i64>, typeString: &str) -> File {
        File {
            blob: Blob::new_inherited(slice, typeString),
            name: name,
            // https://w3c.github.io/FileAPI/#dfn-lastModified
            modified: match modified {
                Some(m) => m,
                None => {
                    let time = time::get_time();
                    time.sec * 1000 + (time.nsec / 1000000) as i64
                }
            },
        }
    }

    pub fn new(global: GlobalRef, slice: DataSlice,
               name: DOMString, modified: Option<i64>, typeString: &str) -> Root<File> {
        reflect_dom_object(box File::new_inherited(slice, name, modified, typeString),
                           global,
                           FileBinding::Wrap)
    }

    // https://w3c.github.io/FileAPI/#file-constructor
    pub fn Constructor(global: GlobalRef,
                       fileBits: Vec<BlobOrString>,
                       filename: DOMString,
                       filePropertyBag: &FileBinding::FilePropertyBag)
                       -> Fallible<Root<File>> {
        let bytes: Vec<u8> = blob_parts_to_bytes(fileBits);

        let ref blobPropertyBag = filePropertyBag.parent;
        let typeString = blobPropertyBag.get_typestring();

        let slice = DataSlice::new(Arc::new(bytes), None, None);
        let modified = filePropertyBag.lastModified;
        Ok(File::new(global, slice, filename, modified, &typeString))
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

    // https://w3c.github.io/FileAPI/#dfn-lastModified
    fn LastModified(&self) -> i64 {
        self.modified
    }
}
