/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr;

use dom_struct::dom_struct;
use js::jsapi::JSObject;
use js::rust::HandleObject;
use js::typedarray::{ArrayBufferU8, HeapArrayBuffer};
use script_bindings::trace::RootedTraceableBox;

use crate::dom::bindings::buffer_source::create_buffer_source;
use crate::dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use crate::dom::bindings::codegen::Bindings::FileReaderSyncBinding::FileReaderSyncMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::blob::Blob;
use crate::dom::filereader::FileReaderSharedFunctionality;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::{CanGc, JSContext};

#[dom_struct]
pub(crate) struct FileReaderSync {
    reflector: Reflector,
}

impl FileReaderSync {
    pub(crate) fn new_inherited() -> FileReaderSync {
        FileReaderSync {
            reflector: Reflector::new(),
        }
    }

    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<FileReaderSync> {
        reflect_dom_object_with_proto(
            Box::new(FileReaderSync::new_inherited()),
            global,
            proto,
            can_gc,
        )
    }

    fn get_blob_bytes(blob: &Blob) -> Result<Vec<u8>, Error> {
        blob.get_bytes().map_err(|_| Error::NotReadable(None))
    }
}

impl FileReaderSyncMethods<crate::DomTypeHolder> for FileReaderSync {
    /// <https://w3c.github.io/FileAPI/#filereadersyncConstrctr>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<FileReaderSync>> {
        Ok(FileReaderSync::new(global, proto, can_gc))
    }

    /// <https://w3c.github.io/FileAPI/#readAsBinaryStringSyncSection>
    fn ReadAsBinaryString(&self, blob: &Blob) -> Fallible<DOMString> {
        // step 1
        let blob_contents = FileReaderSync::get_blob_bytes(blob)?;

        // step 2
        Ok(DOMString::from(String::from_utf8_lossy(&blob_contents)))
    }

    /// <https://w3c.github.io/FileAPI/#readAsTextSync>
    fn ReadAsText(&self, blob: &Blob, label: Option<DOMString>) -> Fallible<DOMString> {
        // step 1
        let blob_contents = FileReaderSync::get_blob_bytes(blob)?;

        // step 2
        let blob_label = label.map(String::from);
        let blob_type = String::from(blob.Type());

        let output =
            FileReaderSharedFunctionality::text_decode(&blob_contents, &blob_type, &blob_label);

        Ok(output)
    }

    /// <https://w3c.github.io/FileAPI/#readAsDataURLSync-section>
    fn ReadAsDataURL(&self, blob: &Blob) -> Fallible<DOMString> {
        // step 1
        let blob_contents = FileReaderSync::get_blob_bytes(blob)?;

        // step 2
        let output =
            FileReaderSharedFunctionality::dataurl_format(&blob_contents, blob.Type().to_string());

        Ok(output)
    }

    /// <https://w3c.github.io/FileAPI/#readAsArrayBufferSyncSection>
    fn ReadAsArrayBuffer(
        &self,
        cx: JSContext,
        blob: &Blob,
        can_gc: CanGc,
    ) -> Fallible<RootedTraceableBox<HeapArrayBuffer>> {
        // step 1
        let blob_contents = FileReaderSync::get_blob_bytes(blob)?;

        // step 2
        rooted!(in(*cx) let mut array_buffer = ptr::null_mut::<JSObject>());

        create_buffer_source::<ArrayBufferU8>(cx, &blob_contents, array_buffer.handle_mut(), can_gc)
            .map_err(|_| Error::JSFailed)
    }
}
