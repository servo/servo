/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::FileDerived;
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::error::Fallible;
use dom::bindings::codegen::Bindings::BlobBinding;
use dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;

use servo_util::str::DOMString;
use std::cmp::{min, max};

#[jstraceable]
pub enum BlobTypeId {
    Blob,
    File,
}

#[dom_struct]
pub struct Blob {
    reflector_: Reflector,
    type_: BlobTypeId,
    bytes: Option<Vec<u8>>,
    typeString: DOMString,
    global: GlobalField
    // isClosed_: bool
}

impl Blob {
    pub fn new_inherited(global: &GlobalRef, type_: BlobTypeId,
                         bytes: Option<Vec<u8>>) -> Blob {
        Blob {
            reflector_: Reflector::new(),
            type_: type_,
            bytes: bytes,
            typeString: "".to_string(),
            global: GlobalField::from_rooted(global)
            //isClosed_: false
        }
    }

    pub fn new(global: &GlobalRef, bytes: Option<Vec<u8>>) -> Temporary<Blob> {
        reflect_dom_object(box Blob::new_inherited(global, BlobTypeId::Blob, bytes),
                           *global,
                           BlobBinding::Wrap)
    }

    pub fn Constructor(global: &GlobalRef) -> Fallible<Temporary<Blob>> {
        Ok(Blob::new(global, None))
    }

    pub fn Constructor_(global: &GlobalRef, blobParts: DOMString) -> Fallible<Temporary<Blob>> {
        //TODO: accept other blobParts types - ArrayBuffer or ArrayBufferView or Blob
        //TODO: accept options parameter
        let bytes: Option<Vec<u8>> = Some(blobParts.into_bytes());
        Ok(Blob::new(global, bytes))
    }
}

impl<'a> BlobMethods for JSRef<'a, Blob> {
    fn Size(self) -> u64{
        match self.bytes {
            None => 0,
            Some(ref bytes) => bytes.len() as u64
        }
    }

    fn Type(self) -> DOMString {
        self.typeString.clone()
    }

    fn Slice(self, start: Option<i64>, end: Option<i64>,
             _contentType: Option<DOMString>) -> Temporary<Blob> {
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
        /*
        let relativeContentType = match contentType {
            None => "".to_string(),
            Some(str) => str
        };
        */
        //TODO: actually use relativeContentType in constructor
        let span: i64 = max(relativeEnd - relativeStart, 0);
        let global = self.global.root();
        match self.bytes {
            None => Blob::new(&global.root_ref(), None),
            Some(ref vec) => {
                let start = relativeStart.to_uint().unwrap();
                let end = (relativeStart + span).to_uint().unwrap();
                let mut bytes: Vec<u8> = Vec::new();
                bytes.push_all(vec.slice(start, end));
                Blob::new(&global.root_ref(), Some(bytes))
            }
        }
    }

    //fn IsClosed(self) -> bool {
    //    self.isClosed_.clone()
    //}

    //fn Close(self) {
    //    TODO
    //}
}
impl Reflectable for Blob {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
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
