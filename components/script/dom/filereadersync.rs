/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use dom::bindings::codegen::Bindings::FileReaderSyncBinding::{FileReaderSyncBinding, FileReaderSyncMethods};
use dom::bindings::error::{Error, Fallible};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::blob::Blob;
use dom::filereader::FileReaderSharedFunctionality;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use js::jsapi::{JSContext, JSObject};
use js::typedarray::{ArrayBuffer, CreateWith};
use std::ptr;
use std::ptr::NonNull;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct FileReaderSync<TH: TypeHolderTrait> {
    reflector: Reflector<TH>,
}

impl<TH: TypeHolderTrait> FileReaderSync<TH> {
    pub fn new_inherited() -> FileReaderSync<TH> {
        FileReaderSync {
            reflector: Reflector::new(),
        }
    }

    pub fn new(global: &GlobalScope<TH>) -> DomRoot<FileReaderSync<TH>> {
        reflect_dom_object(
            Box::new(FileReaderSync::new_inherited()),
            global,
            FileReaderSyncBinding::Wrap,
        )
    }

    pub fn Constructor(global: &GlobalScope<TH>) -> Fallible<DomRoot<FileReaderSync<TH>>> {
        Ok(FileReaderSync::new(global))
    }

    fn get_blob_bytes(blob: &Blob<TH>) -> Result<Vec<u8>, Error> {
        blob.get_bytes().map_err(|_| Error::NotReadable)
    }
}

impl<TH: TypeHolderTrait> FileReaderSyncMethods<TH> for FileReaderSync<TH> {
    // https://w3c.github.io/FileAPI/#readAsBinaryStringSyncSection
    fn ReadAsBinaryString(&self, blob: &Blob<TH>) -> Fallible<DOMString> {
        // step 1
        let blob_contents = FileReaderSync::get_blob_bytes(blob)?;

        // step 2
        Ok(DOMString::from(String::from_utf8_lossy(&blob_contents)))
    }

    // https://w3c.github.io/FileAPI/#readAsTextSync
    fn ReadAsText(&self, blob: &Blob<TH>, label: Option<DOMString>) -> Fallible<DOMString> {
        // step 1
        let blob_contents = FileReaderSync::get_blob_bytes(blob)?;

        // step 2
        let blob_label = label.map(String::from);
        let blob_type = String::from(blob.Type());

        let output =
            FileReaderSharedFunctionality::text_decode(&blob_contents, &blob_type, &blob_label);

        Ok(output)
    }

    // https://w3c.github.io/FileAPI/#readAsDataURLSync-section
    fn ReadAsDataURL(&self, blob: &Blob<TH>) -> Fallible<DOMString> {
        // step 1
        let blob_contents = FileReaderSync::get_blob_bytes(blob)?;

        // step 2
        let output =
            FileReaderSharedFunctionality::dataurl_format(&blob_contents, blob.Type().to_string());

        Ok(output)
    }

    #[allow(unsafe_code)]
    // https://w3c.github.io/FileAPI/#readAsArrayBufferSyncSection
    unsafe fn ReadAsArrayBuffer(
        &self,
        cx: *mut JSContext,
        blob: &Blob<TH>,
    ) -> Fallible<NonNull<JSObject>> {
        // step 1
        let blob_contents = FileReaderSync::get_blob_bytes(blob)?;

        // step 2
        rooted!(in(cx) let mut array_buffer = ptr::null_mut::<JSObject>());
        assert!(
            ArrayBuffer::create(
                cx,
                CreateWith::Slice(&blob_contents),
                array_buffer.handle_mut()
            ).is_ok()
        );

        Ok(NonNull::new_unchecked(array_buffer.get()))
    }
}
