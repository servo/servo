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
use dom::blob::Blob;
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
    fn new_inherited(file_bits: &Blob, name: DOMString, modified: Option<i64>) -> File {
       let mut bytes = Vec::new();
       bytes.extend_from_slice(file_bits.get_data().get_all_bytes().as_slice());

        File {
            blob: Blob::new_inherited(Arc::new(bytes), None, None, ""),
            name: name,
            modified: match modified {
                Some(m) => m,
                None => {
                    let time = time::get_time();
                    time.sec * 1000 + (time.nsec / 1000000) as i64
                }
            },
        }
    }

    pub fn new(global: GlobalRef, file_bits: &Blob, name: DOMString, modified: Option<i64>) -> Root<File> {
        reflect_dom_object(box File::new_inherited(file_bits, name, modified),
                           global,
                           FileBinding::Wrap)
    }

    // https://w3c.github.io/FileAPI/#file-constructor
    pub fn Constructor(global: GlobalRef,
                       fileBits: Vec<BlobOrString>,
                       filename: DOMString,
                       filePropertyBag: &FileBinding::FilePropertyBag)
                       -> Fallible<Root<File>> {

        match Blob::Constructor(global, Some(fileBits), &filePropertyBag.parent) {
            Ok(b) => Ok(File::new(global, b.r(), filename, filePropertyBag.lastModified)),
            Err(e) => Err(e)
        }
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
