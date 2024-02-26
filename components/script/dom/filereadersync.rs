/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr;

use dom_struct::dom_struct;
use js::jsapi::JSObject;
use js::rust::HandleObject;
use js::typedarray::{ArrayBuffer, ArrayBufferU8};

use crate::dom::bindings::buffer_source::create_buffer_source;
use crate::dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use crate::dom::bindings::codegen::Bindings::FileReaderSyncBinding::FileReaderSyncMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::blob::Blob;
use crate::dom::filereader::FileReaderSharedFunctionality;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::JSContext;

#[dom_struct]
pub struct FileReaderSync {
    reflector: Reflector,
}

impl FileReaderSync {
    pub fn new_inherited() -> FileReaderSync {
        FileReaderSync {
            reflector: Reflector::new(),
        }
    }

    fn new(global: &GlobalScope, proto: Option<HandleObject>) -> DomRoot<FileReaderSync> {
        reflect_dom_object_with_proto(Box::new(FileReaderSync::new_inherited()), global, proto)
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
    ) -> Fallible<DomRoot<FileReaderSync>> {
        Ok(FileReaderSync::new(global, proto))
    }

    fn get_blob_bytes(blob: &Blob) -> Result<Vec<u8>, Error> {
        blob.get_bytes().map_err(|_| Error::NotReadable)
    }
}

impl FileReaderSyncMethods for FileReaderSync {
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
    fn ReadAsArrayBuffer(&self, cx: JSContext, blob: &Blob) -> Fallible<ArrayBuffer> {
        // step 1
        let blob_contents = FileReaderSync::get_blob_bytes(blob)?;

        // step 2
        rooted!(in(*cx) let mut array_buffer = ptr::null_mut::<JSObject>());

        create_buffer_source::<ArrayBufferU8>(cx, &blob_contents, array_buffer.handle_mut())
            .map_err(|_| Error::JSFailed)
    }
}
