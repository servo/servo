/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::BlobBinding;
use dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use dom::bindings::codegen::UnionTypes::BlobOrString;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::trace::JSTraceable;
use encoding::all::UTF_8;
use encoding::types::{EncoderTrap, Encoding};
use num::ToPrimitive;
use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::cell::Cell;
use std::cmp::{max, min};
use std::sync::Arc;
use util::str::DOMString;

#[derive(Clone, JSTraceable)]
pub struct DataSlice {
    bytes: Arc<Vec<u8>>,
    bytes_start: usize,
    bytes_end: usize
}

impl DataSlice {
    pub fn new(bytes: Arc<Vec<u8>>, start: Option<i64>, end: Option<i64>) -> DataSlice {
        let size = bytes.len() as i64;
        let relativeStart: i64 = match start {
            None => 0,
            Some(start) => {
                if start < 0 {
                    max(size + start, 0)
                } else {
                    min(start, size)
                }
            }
        };
        let relativeEnd: i64 = match end {
            None => size,
            Some(end) => {
                if end < 0 {
                    max(size + end, 0)
                } else {
                    min(end, size)
                }
            }
        };

        let span: i64 = max(relativeEnd - relativeStart, 0);
        let start = relativeStart.to_usize().unwrap();
        let end = (relativeStart + span).to_usize().unwrap();

        DataSlice {
            bytes: bytes,
            bytes_start: start,
            bytes_end: end
        }
    }

    pub fn get_bytes(&self) -> &[u8] {
        &self.bytes[self.bytes_start..self.bytes_end]
    }

    pub fn size(&self) -> u64 {
        (self.bytes_end as u64) - (self.bytes_start as u64)
    }
}


// https://w3c.github.io/FileAPI/#blob
#[dom_struct]
pub struct Blob {
    reflector_: Reflector,
    #[ignore_heap_size_of = "No clear owner"]
    data: DataSlice,
    typeString: String,
    isClosed_: Cell<bool>,
}

fn is_ascii_printable(string: &str) -> bool {
    // Step 5.1 in Sec 5.1 of File API spec
    // https://w3c.github.io/FileAPI/#constructorBlob
    string.chars().all(|c| c >= '\x20' && c <= '\x7E')
}

impl Blob {
    pub fn new_inherited(bytes: Arc<Vec<u8>>,
                         bytes_start: Option<i64>,
                         bytes_end: Option<i64>,
                         typeString: &str) -> Blob {
        Blob {
            reflector_: Reflector::new(),
            data: DataSlice::new(bytes, bytes_start, bytes_end),
            typeString: typeString.to_owned(),
            isClosed_: Cell::new(false),
        }
    }

    pub fn new(global: GlobalRef, bytes: Vec<u8>, typeString: &str) -> Root<Blob> {
        let boxed_blob = box Blob::new_inherited(Arc::new(bytes), None, None, typeString);
        reflect_dom_object(boxed_blob, global, BlobBinding::Wrap)
    }

    fn new_sliced(global: GlobalRef,
                  bytes: Arc<Vec<u8>>,
                  bytes_start: Option<i64>,
                  bytes_end: Option<i64>,
                  typeString: &str) -> Root<Blob> {

      let boxed_blob = box Blob::new_inherited(bytes, bytes_start, bytes_end, typeString);
      reflect_dom_object(boxed_blob, global, BlobBinding::Wrap)
    }

    // https://w3c.github.io/FileAPI/#constructorBlob
    pub fn Constructor(global: GlobalRef) -> Fallible<Root<Blob>> {
        Ok(Blob::new(global, Vec::new(), ""))
    }

    // https://w3c.github.io/FileAPI/#constructorBlob
    pub fn Constructor_(global: GlobalRef,
                        blobParts: Vec<BlobOrString>,
                        blobPropertyBag: &BlobBinding::BlobPropertyBag)
                        -> Fallible<Root<Blob>> {

        // TODO: accept other blobParts types - ArrayBuffer or ArrayBufferView
        let bytes: Vec<u8> = blobParts.iter()
                                .flat_map(|bPart| {
                                    match bPart {
                                        &BlobOrString::String(ref s) => {
                                            UTF_8.encode(s, EncoderTrap::Replace).unwrap()
                                        },
                                        &BlobOrString::Blob(ref b) => {
                                            b.get_data().get_bytes().to_vec()
                                        },
                                    }
                                })
                                .collect();
        let typeString = if is_ascii_printable(&blobPropertyBag.type_) {
            &*blobPropertyBag.type_
        } else {
            ""
        };
        Ok(Blob::new(global, bytes, &typeString.to_ascii_lowercase()))
    }

    pub fn get_data(&self) -> &DataSlice {
        &self.data
    }
}

impl BlobMethods for Blob {
    // https://w3c.github.io/FileAPI/#dfn-size
    fn Size(&self) -> u64 {
        self.data.size()
    }

    // https://w3c.github.io/FileAPI/#dfn-type
    fn Type(&self) -> DOMString {
        DOMString::from(self.typeString.clone())
    }

    // https://w3c.github.io/FileAPI/#slice-method-algo
    fn Slice(&self,
             start: Option<i64>,
             end: Option<i64>,
             contentType: Option<DOMString>)
             -> Root<Blob> {

        let relativeContentType = match contentType {
            None => DOMString::new(),
            Some(mut str) => {
                if is_ascii_printable(&str) {
                    str.make_ascii_lowercase();
                    str
                } else {
                    DOMString::new()
                }
            }
        };
        let global = self.global();
        let bytes = self.data.bytes.clone();
        Blob::new_sliced(global.r(), bytes, start, end, &relativeContentType)
    }

    // https://w3c.github.io/FileAPI/#dfn-isClosed
    fn IsClosed(&self) -> bool {
        self.isClosed_.get()
    }

    // https://w3c.github.io/FileAPI/#dfn-close
    fn Close(&self) {
        // Step 1
        if self.isClosed_.get() {
            return;
        }

        // Step 2
        self.isClosed_.set(true);

        // TODO Step 3 if Blob URL Store is implemented

    }
}
