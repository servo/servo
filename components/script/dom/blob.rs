/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::BlobBinding;
use dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use dom::bindings::error::Fallible;
use dom::bindings::global::{GlobalField, GlobalRef};
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use num::ToPrimitive;
use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::cell::Cell;
use std::cmp::{max, min};
use std::sync::Arc;
use util::str::DOMString;

// http://dev.w3.org/2006/webapi/FileAPI/#blob
#[dom_struct]
pub struct Blob {
    reflector_: Reflector,
    bytes: Arc<Vec<u8>>,
    bytes_start: Option<usize>,
    bytes_end: Option<usize>,
    typeString: String,
    global: GlobalField,
    isClosed_: Cell<bool>,
}

fn is_ascii_printable(string: &str) -> bool {
    // Step 5.1 in Sec 5.1 of File API spec
    // http://dev.w3.org/2006/webapi/FileAPI/#constructorBlob
    string.chars().all(|c| c >= '\x20' && c <= '\x7E')
}

impl Blob {
    pub fn new_inherited(global: GlobalRef, bytes: Vec<u8>, typeString: &str) -> Blob {
        Blob {
            reflector_: Reflector::new(),
            bytes: Arc::new(bytes),
            bytes_start: None,
            bytes_end: None,
            typeString: typeString.to_owned(),
            global: GlobalField::from_rooted(&global),
            isClosed_: Cell::new(false),
        }
    }

    pub fn new(global: GlobalRef, bytes: Vec<u8>, typeString: &str) -> Root<Blob> {
        reflect_dom_object(box Blob::new_inherited(global, bytes, typeString),
                           global,
                           BlobBinding::Wrap)
    }

    pub fn new_sliced(global: GlobalRef,
                      bytes: Arc<Vec<u8>>,
                      bytes_start: Option<usize>,
                      bytes_end: Option<usize>,
                      typeString: &str) -> Root<Blob> {

      let boxed_blob =
        box Blob {
               reflector_: Reflector::new(),
               bytes: bytes,
               bytes_start: bytes_start,
               bytes_end: bytes_end,
               typeString: typeString.to_owned(),
               global: GlobalField::from_rooted(&global),
               isClosed_: Cell::new(false),
           };

      reflect_dom_object(boxed_blob, global, BlobBinding::Wrap)
    }

    // http://dev.w3.org/2006/webapi/FileAPI/#constructorBlob
    pub fn Constructor(global: GlobalRef) -> Fallible<Root<Blob>> {
        Ok(Blob::new(global, Vec::new(), ""))
    }

    // http://dev.w3.org/2006/webapi/FileAPI/#constructorBlob
    pub fn Constructor_(global: GlobalRef,
                        blobParts: DOMString,
                        blobPropertyBag: &BlobBinding::BlobPropertyBag)
                        -> Fallible<Root<Blob>> {
        // TODO: accept other blobParts types - ArrayBuffer or ArrayBufferView or Blob
        // FIXME(ajeffrey): convert directly from a DOMString to a Vec<u8>
        let bytes: Vec<u8> = String::from(blobParts).into_bytes();
        let typeString = if is_ascii_printable(&blobPropertyBag.type_) {
            &*blobPropertyBag.type_
        } else {
            ""
        };
        Ok(Blob::new(global, bytes, &typeString.to_ascii_lowercase()))
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        let start = self.bytes_start.unwrap_or(0);
        let end = self.bytes_end.unwrap_or(self.bytes.len());

        self.bytes[start..end].to_vec()
    }
}

impl BlobMethods for Blob {
    // https://dev.w3.org/2006/webapi/FileAPI/#dfn-size
    fn Size(&self) -> u64 {
        let start = self.bytes_start.unwrap_or(0) as u64;
        let end = self.bytes_end.unwrap_or(self.bytes.len()) as u64;
        end - start
    }

    // https://dev.w3.org/2006/webapi/FileAPI/#dfn-type
    fn Type(&self) -> DOMString {
        DOMString::from(self.typeString.clone())
    }

    // https://dev.w3.org/2006/webapi/FileAPI/#slice-method-algo
    fn Slice(&self,
             start: Option<i64>,
             end: Option<i64>,
             contentType: Option<DOMString>)
             -> Root<Blob> {
        let size: i64 = self.Size().to_i64().unwrap();
        let relativeStart: i64 = match start {
            None => 0,
            Some(start) => {
                if start < 0 {
                    max(size.to_i64().unwrap() + start, 0)
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
        let span: i64 = max(relativeEnd - relativeStart, 0);
        let global = self.global.root();
        let start = relativeStart.to_usize().unwrap();
        let end = (relativeStart + span).to_usize().unwrap();
        let bytes = self.bytes.clone();

        Blob::new_sliced(global.r(), bytes, Some(start), Some(end), &relativeContentType)
    }

    // https://dev.w3.org/2006/webapi/FileAPI/#dfn-isClosed
    fn IsClosed(&self) -> bool {
        self.isClosed_.get()
    }

    // https://dev.w3.org/2006/webapi/FileAPI/#dfn-close
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
