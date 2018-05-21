/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use base64;
use dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use dom::bindings::codegen::Bindings::FileReaderSyncBinding::{FileReaderSyncBinding, FileReaderSyncMethods};
use dom::bindings::error::{Error, Fallible};
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::blob::Blob;
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use encoding_rs::{Encoding, UTF_8};
use hyper::mime::{Attr, Mime};
use js::jsapi::{JSContext, JSObject};
use js::typedarray::{ArrayBuffer, CreateWith};
use std::ptr;
use std::ptr::NonNull;

#[dom_struct]
pub struct FileReaderSync {
    eventtarget: EventTarget,
}

impl FileReaderSync {
    pub fn new_inherited() -> FileReaderSync {
        FileReaderSync {
            eventtarget: EventTarget::new_inherited(),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<FileReaderSync> {
        reflect_dom_object(Box::new(FileReaderSync::new_inherited()),
                           global, FileReaderSyncBinding::Wrap)
    }

    pub fn Constructor(global: &GlobalScope) -> Fallible<DomRoot<FileReaderSync>> {
        Ok(FileReaderSync::new(global))
    }
}

impl FileReaderSyncMethods for FileReaderSync {
    // https://w3c.github.io/FileAPI/#readAsBinaryStringSyncSection
    fn ReadAsBinaryString(&self, blob: &Blob) -> Fallible<DOMString>{
        // step 1
        let blob_contents = blob.get_bytes().unwrap_or(vec![]);
        if blob_contents.is_empty() {
            return Err(Error::NotReadable)
        }

        // step 2
        Ok(DOMString::from(String::from_utf8_lossy(&blob_contents)))
    }

    // https://w3c.github.io/FileAPI/#readAsTextSync
    fn ReadAsText(&self, blob: &Blob, label: Option<DOMString>) -> Fallible<DOMString> {
        // step 1
        let blob_contents = blob.get_bytes().unwrap_or(vec![]);
        if blob_contents.is_empty() {
            return Err(Error::NotReadable)
        }

        // step 2
        let blob_label = label.map(String::from);
        let blob_type = String::from(blob.Type());

        //https://w3c.github.io/FileAPI/#encoding-determination
        let mut encoding = blob_label.as_ref()
            .map(|string| string.as_bytes())
            .and_then(Encoding::for_label);

        encoding = encoding.or_else(|| {
            let resultmime = blob_type.parse::<Mime>().ok();
            resultmime.and_then(|Mime(_, _, ref parameters)| {
                parameters.iter()
                    .find(|&&(ref k, _)| &Attr::Charset == k)
                    .and_then(|&(_, ref v)| Encoding::for_label(v.as_str().as_bytes()))
            })
        });

        let enc = encoding.unwrap_or(UTF_8);
        let convert = blob_contents;

        let (output, _, _) = enc.decode(&convert);

        Ok(DOMString::from(output))
    }

    // https://w3c.github.io/FileAPI/#readAsDataURLSync-section
    fn ReadAsDataURL(&self, blob: &Blob) -> Fallible<DOMString> {
        // step 1
        let blob_contents = blob.get_bytes().unwrap_or(vec![]);
        if blob_contents.is_empty() {
            return Err(Error::NotReadable)
        }

        // step 2
        let base64 = base64::encode(&blob_contents);
        let blob_type = String::from(blob.Type());

        let output = if blob_type.is_empty() {
            format!("data:base64,{}", base64)
        } else {
            format!("data:{};base64,{}", blob_type, base64)
        };

        Ok(DOMString::from(output))
    }

     // https://w3c.github.io/FileAPI/#readAsArrayBufferSyncSection
     #[allow(unsafe_code)]
    unsafe fn ReadAsArrayBuffer(&self, cx: *mut JSContext, blob: &Blob) -> Fallible<NonNull<JSObject>> {
        // step 1
        let blob_contents = blob.get_bytes().unwrap_or(vec![]);
        if blob_contents.is_empty() {
            return Err(Error::NotReadable)
        }

        // step 2
        rooted!(in(cx) let mut array_buffer = ptr::null_mut::<JSObject>());
        assert!(ArrayBuffer::create(cx, CreateWith::Slice(&blob_contents), array_buffer.handle_mut()).is_ok());

        Ok(NonNull::new_unchecked(array_buffer.get()))
    }
}
