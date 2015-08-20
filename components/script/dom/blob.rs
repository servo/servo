/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::BlobBinding;
use dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use dom::bindings::codegen::InheritTypes::FileDerived;
use dom::bindings::error::Fallible;
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::Root;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use num::ToPrimitive;
use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::cell::{Cell};
use std::cmp::{min, max};
use std::sync::mpsc::Sender;
use util::str::DOMString;

#[derive(JSTraceable, HeapSizeOf)]
pub enum BlobTypeId {
    Blob,
    File,
}

// http://dev.w3.org/2006/webapi/FileAPI/#blob
#[dom_struct]
#[derive(HeapSizeOf)]
pub struct Blob {
    reflector_: Reflector,
    type_: BlobTypeId,
    bytes: Option<Vec<u8>>,
    typeString: DOMString,
    global: GlobalField,
    isClosed_: Cell<bool>
}

fn is_ascii_printable(string: &DOMString) -> bool {
    // Step 5.1 in Sec 5.1 of File API spec
    // http://dev.w3.org/2006/webapi/FileAPI/#constructorBlob
    return string.chars().all(|c| { c >= '\x20' && c <= '\x7E' })
}

impl Blob {
    pub fn new_inherited(global: GlobalRef, type_: BlobTypeId,
                         bytes: Option<Vec<u8>>, typeString: &str) -> Blob {
        Blob {
            reflector_: Reflector::new(),
            type_: type_,
            bytes: bytes,
            typeString: typeString.to_owned(),
            global: GlobalField::from_rooted(&global),
            isClosed_: Cell::new(false)
        }
    }

    pub fn new(global: GlobalRef, bytes: Option<Vec<u8>>,
               typeString: &str) -> Root<Blob> {
        reflect_dom_object(box Blob::new_inherited(global, BlobTypeId::Blob, bytes, typeString),
                           global,
                           BlobBinding::Wrap)
    }

    // http://dev.w3.org/2006/webapi/FileAPI/#constructorBlob
    pub fn Constructor(global: GlobalRef) -> Fallible<Root<Blob>> {
        Ok(Blob::new(global, None, ""))
    }

    // http://dev.w3.org/2006/webapi/FileAPI/#constructorBlob
    pub fn Constructor_(global: GlobalRef, blobParts: DOMString,
                        blobPropertyBag: &BlobBinding::BlobPropertyBag) -> Fallible<Root<Blob>> {
        //TODO: accept other blobParts types - ArrayBuffer or ArrayBufferView or Blob
        let bytes: Option<Vec<u8>> = Some(blobParts.into_bytes());
        let typeString = if is_ascii_printable(&blobPropertyBag.type_) {
            &*blobPropertyBag.type_
        } else {
            ""
        };
        Ok(Blob::new(global, bytes, &typeString.to_ascii_lowercase()))
    }
}

pub trait BlobHelpers {
    fn read_out_buffer(self, send: Sender<Vec<u8>>);
}

impl<'a> BlobHelpers for &'a Blob {
    fn read_out_buffer(self, send: Sender<Vec<u8>>) {
        send.send(self.bytes.clone().unwrap_or(vec![])).unwrap();
    }
}

impl<'a> BlobMethods for &'a Blob {
    // https://dev.w3.org/2006/webapi/FileAPI/#dfn-size
    fn Size(self) -> u64 {
        match self.bytes {
            None => 0,
            Some(ref bytes) => bytes.len() as u64
        }
    }

    // https://dev.w3.org/2006/webapi/FileAPI/#dfn-type
    fn Type(self) -> DOMString {
        self.typeString.clone()
    }

    // https://dev.w3.org/2006/webapi/FileAPI/#slice-method-algo
    fn Slice(self, start: Option<i64>, end: Option<i64>,
             contentType: Option<DOMString>) -> Root<Blob> {
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
            None => "".to_owned(),
            Some(str) => {
                if is_ascii_printable(&str) {
                    str.to_ascii_lowercase()
                } else {
                    "".to_owned()
                }
            }
        };
        let span: i64 = max(relativeEnd - relativeStart, 0);
        let global = self.global.root();
        match self.bytes {
            None => Blob::new(global.r(), None, &relativeContentType),
            Some(ref vec) => {
                let start = relativeStart.to_usize().unwrap();
                let end = (relativeStart + span).to_usize().unwrap();
                let mut bytes: Vec<u8> = Vec::new();
                bytes.push_all(&vec[start..end]);
                Blob::new(global.r(), Some(bytes), &relativeContentType)
            }
        }
    }

    // https://dev.w3.org/2006/webapi/FileAPI/#dfn-isClosed
    fn IsClosed(self) -> bool {
        self.isClosed_.get()
    }

    // https://dev.w3.org/2006/webapi/FileAPI/#dfn-close
    fn Close(self) {
        // Step 1
        if self.isClosed_.get() {
            return;
        }

        // Step 2
        self.isClosed_.set(true);

        // TODO Step 3 if Blob URL Store is implemented

    }
}

impl FileDerived for Blob {
    fn is_file(&self) -> bool {
        match self.type_ {
            BlobTypeId::File => true,
            _ => false
        }
    }
}
