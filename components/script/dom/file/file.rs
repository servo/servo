/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;
use std::time::SystemTime;

use dom_struct::dom_struct;
use embedder_traits::SelectedFile;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::reflector::reflect_weak_referenceable_dom_object_with_proto;
use servo_base::id::{FileId, FileIndex};
use servo_constellation_traits::{BlobImpl, SerializableFile};
use time::{Duration, OffsetDateTime};

use crate::dom::bindings::codegen::Bindings::FileBinding;
use crate::dom::bindings::codegen::Bindings::FileBinding::FileMethods;
use crate::dom::bindings::codegen::UnionTypes::ArrayBufferOrArrayBufferViewOrBlobOrString;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::serializable::Serializable;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::structuredclone::StructuredData;
use crate::dom::blob::{Blob, normalize_type_string, process_blob_parts};
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct File {
    blob: Blob,
    name: DOMString,
    modified: SystemTime,
    // TODO: This depends on the `webkitdirectory` from `HTMLInputElement`.
    // Then need to change `SelectedFile` in embedder,
    // and filemanager_thread to recursively walk.
    webkit_relative_path: USVString,
}

impl File {
    fn new_inherited(
        blob_impl: &BlobImpl,
        name: DOMString,
        modified: Option<SystemTime>,
        webkit_relative_path: USVString,
    ) -> File {
        File {
            blob: Blob::new_inherited(blob_impl),
            name,
            // https://w3c.github.io/FileAPI/#dfn-lastModified
            modified: modified.unwrap_or_else(SystemTime::now),
            webkit_relative_path,
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        blob_impl: BlobImpl,
        name: DOMString,
        modified: Option<SystemTime>,
    ) -> DomRoot<File> {
        Self::new_with_proto(
            cx,
            global,
            None,
            blob_impl,
            name,
            modified,
            USVString::default(),
        )
    }

    fn new_with_proto(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        blob_impl: BlobImpl,
        name: DOMString,
        modified: Option<SystemTime>,
        webkit_relative_path: USVString,
    ) -> DomRoot<File> {
        let file = reflect_weak_referenceable_dom_object_with_proto(
            cx,
            Rc::new(File::new_inherited(
                &blob_impl,
                name,
                modified,
                webkit_relative_path,
            )),
            global,
            proto,
        );
        global.track_file(&file, blob_impl);
        file
    }

    // Construct from selected file message from file manager thread
    pub(crate) fn new_from_selected(
        cx: &mut JSContext,
        window: &Window,
        selected: SelectedFile,
    ) -> DomRoot<File> {
        let name = DOMString::from(
            selected
                .filename
                .to_str()
                .expect("File name encoding error"),
        );

        File::new(
            cx,
            window.upcast(),
            BlobImpl::new_from_file(
                selected.id,
                selected.filename,
                selected.size,
                normalize_type_string(&selected.type_string.to_string()),
            ),
            name,
            Some(selected.modified),
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

    pub(crate) fn get_modified(&self) -> SystemTime {
        self.modified
    }

    pub(crate) fn serialized_data(&self) -> Result<SerializableFile, ()> {
        let (_, blob_impl) = self.upcast::<Blob>().serialize()?;
        Ok(SerializableFile {
            blob_impl,
            name: self.name.to_string(),
            modified: self.LastModified(),
            webkit_relative_path: self.webkit_relative_path.to_string(),
        })
    }
}

impl Serializable for File {
    type Index = FileIndex;
    type Data = SerializableFile;

    /// <https://html.spec.whatwg.org/multipage/#serialization-steps>
    fn serialize(&self) -> Result<(FileId, SerializableFile), ()> {
        Ok((FileId::new(), self.serialized_data()?))
    }

    /// <https://html.spec.whatwg.org/multipage/#deserialization-steps>
    fn deserialize(
        cx: &mut JSContext,
        owner: &GlobalScope,
        serialized: SerializableFile,
    ) -> Result<DomRoot<Self>, ()> {
        let modified = OffsetDateTime::UNIX_EPOCH + Duration::milliseconds(serialized.modified);
        Ok(File::new_with_proto(
            cx,
            owner,
            None,
            serialized.blob_impl,
            serialized.name.into(),
            Some(modified.into()),
            USVString::from(serialized.webkit_relative_path),
        ))
    }

    fn serialized_storage<'a>(
        reader: StructuredData<'a, '_>,
    ) -> &'a mut Option<rustc_hash::FxHashMap<FileId, Self::Data>> {
        match reader {
            StructuredData::Reader(r) => &mut r.files,
            StructuredData::Writer(w) => &mut w.files,
        }
    }
}

impl FileMethods<crate::DomTypeHolder> for File {
    // https://w3c.github.io/FileAPI/#file-constructor
    #[expect(non_snake_case)]
    fn Constructor(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        fileBits: Vec<ArrayBufferOrArrayBufferViewOrBlobOrString>,
        filename: DOMString,
        filePropertyBag: &FileBinding::FilePropertyBag,
    ) -> Fallible<DomRoot<File>> {
        let bytes: Vec<u8> = match process_blob_parts(fileBits, filePropertyBag.parent.endings) {
            Ok(bytes) => bytes,
            Err(_) => return Err(Error::InvalidCharacter(None)),
        };

        let blobPropertyBag = &filePropertyBag.parent;
        let modified = filePropertyBag
            .lastModified
            .map(|modified| OffsetDateTime::UNIX_EPOCH + Duration::milliseconds(modified))
            .map(Into::into);

        let type_string = normalize_type_string(&blobPropertyBag.type_.str());
        Ok(File::new_with_proto(
            cx,
            global,
            proto,
            BlobImpl::new_from_bytes(bytes, type_string),
            filename,
            modified,
            USVString::default(),
        ))
    }

    /// <https://w3c.github.io/FileAPI/#dfn-name>
    fn Name(&self) -> DOMString {
        self.name.clone()
    }

    /// <https://wicg.github.io/entries-api/#dom-file-webkitrelativepath>
    fn WebkitRelativePath(&self) -> USVString {
        self.webkit_relative_path.clone()
    }

    /// <https://w3c.github.io/FileAPI/#dfn-lastModified>
    fn LastModified(&self) -> i64 {
        // This is first converted to a `time::OffsetDateTime` because it might be from before the
        // Unix epoch in which case we will need to return a negative duration to script.
        (OffsetDateTime::from(self.modified) - OffsetDateTime::UNIX_EPOCH).whole_milliseconds()
            as i64
    }
}
