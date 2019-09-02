/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::FileBinding;
use crate::dom::bindings::codegen::Bindings::FileBinding::FileMethods;
use crate::dom::bindings::codegen::UnionTypes::ArrayBufferOrArrayBufferViewOrBlobOrString;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::blob::{blob_parts_to_bytes, Blob};
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use dom_struct::dom_struct;
use net_traits::filemanager_thread::SelectedFile;
use script_traits::serializable::BlobImpl;

#[dom_struct]
pub struct File {
    blob: Blob,
    name: DOMString,
    modified: i64,
}

impl File {
    #[allow(unrooted_must_root)]
    fn new_inherited(blob_impl: &BlobImpl, name: DOMString, modified: Option<i64>) -> File {
        File {
            blob: Blob::new_inherited(blob_impl),
            name: name,
            // https://w3c.github.io/FileAPI/#dfn-lastModified
            modified: match modified {
                Some(m) => m,
                None => {
                    let time = time::get_time();
                    time.sec * 1000 + (time.nsec / 1000000) as i64
                },
            },
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        global: &GlobalScope,
        blob_impl: BlobImpl,
        name: DOMString,
        modified: Option<i64>,
    ) -> DomRoot<File> {
        let file = reflect_dom_object(
            Box::new(File::new_inherited(&blob_impl, name, modified)),
            global,
            FileBinding::Wrap,
        );
        global.track_file(&file, blob_impl);
        file
    }

    // Construct from selected file message from file manager thread
    pub fn new_from_selected(window: &Window, selected: SelectedFile) -> DomRoot<File> {
        let name = DOMString::from(
            selected
                .filename
                .to_str()
                .expect("File name encoding error"),
        );

        File::new(
            window.upcast(),
            BlobImpl::new_from_file(
                selected.id,
                selected.filename,
                selected.size,
                selected.type_string.to_owned(),
            ),
            name,
            Some(selected.modified as i64),
        )
    }

    // https://w3c.github.io/FileAPI/#file-constructor
    pub fn Constructor(
        global: &GlobalScope,
        fileBits: Vec<ArrayBufferOrArrayBufferViewOrBlobOrString>,
        filename: DOMString,
        filePropertyBag: &FileBinding::FilePropertyBag,
    ) -> Fallible<DomRoot<File>> {
        let bytes: Vec<u8> = match blob_parts_to_bytes(fileBits) {
            Ok(bytes) => bytes,
            Err(_) => return Err(Error::InvalidCharacter),
        };

        let ref blobPropertyBag = filePropertyBag.parent;

        let modified = filePropertyBag.lastModified;
        // NOTE: Following behaviour might be removed in future,
        // see https://github.com/w3c/FileAPI/issues/41
        let replaced_filename = DOMString::from_string(filename.replace("/", ":"));
        Ok(File::new(
            global,
            BlobImpl::new_from_bytes(bytes, blobPropertyBag.type_.to_owned().to_string()),
            replaced_filename,
            modified,
        ))
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
