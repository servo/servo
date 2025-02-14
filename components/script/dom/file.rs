/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::time::SystemTime;

use dom_struct::dom_struct;
use js::rust::HandleObject;
use net_traits::filemanager_thread::SelectedFile;
use script_traits::serializable::BlobImpl;
use time::{Duration, OffsetDateTime};

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
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct File {
    blob: Blob,
    name: DOMString,
    modified: SystemTime,
}

impl File {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited(blob_impl: &BlobImpl, name: DOMString, modified: Option<SystemTime>) -> File {
        File {
            blob: Blob::new_inherited(blob_impl),
            name,
            // https://w3c.github.io/FileAPI/#dfn-lastModified
            modified: modified.unwrap_or_else(SystemTime::now),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        blob_impl: BlobImpl,
        name: DOMString,
        modified: Option<SystemTime>,
        can_gc: CanGc,
    ) -> DomRoot<File> {
        Self::new_with_proto(global, None, blob_impl, name, modified, can_gc)
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        blob_impl: BlobImpl,
        name: DOMString,
        modified: Option<SystemTime>,
        can_gc: CanGc,
    ) -> DomRoot<File> {
        let file = reflect_dom_object_with_proto(
            Box::new(File::new_inherited(&blob_impl, name, modified)),
            global,
            proto,
            can_gc,
        );
        global.track_file(&file, blob_impl);
        file
    }

    // Construct from selected file message from file manager thread
    pub(crate) fn new_from_selected(
        window: &Window,
        selected: SelectedFile,
        can_gc: CanGc,
    ) -> DomRoot<File> {
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
            Some(selected.modified),
            can_gc,
        )
    }

    pub(crate) fn file_bytes(&self) -> Result<Vec<u8>, ()> {
        self.blob.get_bytes()
    }

    pub(crate) fn name(&self) -> &DOMString {
        &self.name
    }

    pub(crate) fn file_type(&self) -> String {
        self.blob.type_string()
    }
}

impl FileMethods<crate::DomTypeHolder> for File {
    // https://w3c.github.io/FileAPI/#file-constructor
    #[allow(non_snake_case)]
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        fileBits: Vec<ArrayBufferOrArrayBufferViewOrBlobOrString>,
        filename: DOMString,
        filePropertyBag: &FileBinding::FilePropertyBag,
    ) -> Fallible<DomRoot<File>> {
        let bytes: Vec<u8> = match blob_parts_to_bytes(fileBits) {
            Ok(bytes) => bytes,
            Err(_) => return Err(Error::InvalidCharacter),
        };

        let blobPropertyBag = &filePropertyBag.parent;
        let modified = filePropertyBag
            .lastModified
            .map(|modified| OffsetDateTime::UNIX_EPOCH + Duration::milliseconds(modified))
            .map(Into::into);

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
            can_gc,
        ))
    }

    // https://w3c.github.io/FileAPI/#dfn-name
    fn Name(&self) -> DOMString {
        self.name.clone()
    }

    // https://w3c.github.io/FileAPI/#dfn-lastModified
    fn LastModified(&self) -> i64 {
        // This is first converted to a `time::OffsetDateTime` because it might be from before the
        // Unix epoch in which case we will need to return a negative duration to script.
        (OffsetDateTime::from(self.modified) - OffsetDateTime::UNIX_EPOCH).whole_milliseconds()
            as i64
    }
}
