/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use net_traits::filemanager_thread::SelectedFile;
use script_traits::serializable::BlobImpl;

use crate::dom::bindings::codegen::Bindings::FileBinding;
use crate::dom::bindings::codegen::Bindings::FileBinding::FileMethods;
use crate::dom::bindings::codegen::UnionTypes::ArrayBufferOrArrayBufferViewOrBlobOrString;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::blob::{blob_parts_to_bytes, normalize_type_string, Blob};
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;

#[dom_struct]
pub struct File {
    blob: Blob,
    name: DOMString,
    modified: i64,
}

impl File {
    #[allow(crown::unrooted_must_root)]
    fn new_inherited(blob_impl: &BlobImpl, name: DOMString, modified: Option<i64>) -> File {
        File {
            blob: Blob::new_inherited(blob_impl),
            name,
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

    pub fn new(
        global: &GlobalScope,
        blob_impl: BlobImpl,
        name: DOMString,
        modified: Option<i64>,
    ) -> DomRoot<File> {
        Self::new_with_proto(global, None, blob_impl, name, modified)
    }

    #[allow(crown::unrooted_must_root)]
    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        blob_impl: BlobImpl,
        name: DOMString,
        modified: Option<i64>,
    ) -> DomRoot<File> {
        let file = reflect_dom_object_with_proto(
            Box::new(File::new_inherited(&blob_impl, name, modified)),
            global,
            proto,
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
                normalize_type_string(&selected.type_string.to_string()),
            ),
            name,
            Some(selected.modified as i64),
        )
    }

    // https://w3c.github.io/FileAPI/#file-constructor
    #[allow(non_snake_case)]
    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        fileBits: Vec<ArrayBufferOrArrayBufferViewOrBlobOrString>,
        filename: DOMString,
        filePropertyBag: &FileBinding::FilePropertyBag,
    ) -> Fallible<DomRoot<File>> {
        let bytes: Vec<u8> = match blob_parts_to_bytes(fileBits) {
            Ok(bytes) => bytes,
            Err(_) => return Err(Error::InvalidCharacter),
        };

        let blobPropertyBag = &filePropertyBag.parent;

        let modified = filePropertyBag.lastModified;
        // NOTE: Following behaviour might be removed in future,
        // see https://github.com/w3c/FileAPI/issues/41
        let replaced_filename = DOMString::from_string(filename.replace('/', ":"));
        let type_string = normalize_type_string(blobPropertyBag.type_.as_ref());
        Ok(File::new_with_proto(
            global,
            proto,
            BlobImpl::new_from_bytes(bytes, type_string),
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
