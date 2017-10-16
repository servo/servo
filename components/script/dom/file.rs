/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::FileBinding;
use dom::bindings::codegen::Bindings::FileBinding::FileMethods;
use dom::bindings::codegen::UnionTypes::BlobOrString;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::blob::{Blob, BlobImpl, blob_parts_to_bytes};
use dom::globalscope::GlobalScope;
use dom::window::Window;
use dom_struct::dom_struct;
use net_traits::filemanager_thread::SelectedFile;
use time;

#[dom_struct]
pub struct File {
    blob: Blob,
    name: DOMString,
    modified: i64,
}

impl File {
    #[allow(unrooted_must_root)]
    fn new_inherited(blob_impl: BlobImpl, name: DOMString,
                     modified: Option<i64>, type_string: &str) -> File {
        File {
            blob: Blob::new_inherited(blob_impl, type_string.to_owned()),
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

    #[allow(unrooted_must_root)]
    pub fn new(global: &GlobalScope, blob_impl: BlobImpl,
               name: DOMString, modified: Option<i64>, typeString: &str) -> DomRoot<File> {
        reflect_dom_object(Box::new(File::new_inherited(blob_impl, name, modified, typeString)),
                           global,
                           FileBinding::Wrap)
    }

    // Construct from selected file message from file manager thread
    pub fn new_from_selected(window: &Window, selected: SelectedFile) -> DomRoot<File> {
        let name = DOMString::from(selected.filename.to_str().expect("File name encoding error"));

        File::new(window.upcast(), BlobImpl::new_from_file(selected.id, selected.filename, selected.size),
                  name, Some(selected.modified as i64), &selected.type_string)
    }

    // https://w3c.github.io/FileAPI/#file-constructor
    pub fn Constructor(global: &GlobalScope,
                       fileBits: Vec<BlobOrString>,
                       filename: DOMString,
                       filePropertyBag: &FileBinding::FilePropertyBag)
                       -> Fallible<DomRoot<File>> {
        let bytes: Vec<u8> = match blob_parts_to_bytes(fileBits) {
            Ok(bytes) => bytes,
            Err(_) => return Err(Error::InvalidCharacter),
        };

        let ref blobPropertyBag = filePropertyBag.parent;
        let ref typeString = blobPropertyBag.type_;

        let modified = filePropertyBag.lastModified;
        // NOTE: Following behaviour might be removed in future,
        // see https://github.com/w3c/FileAPI/issues/41
        let replaced_filename = DOMString::from_string(filename.replace("/", ":"));
        Ok(File::new(global,
                     BlobImpl::new_from_bytes(bytes),
                     replaced_filename,
                     modified,
                     typeString))
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
