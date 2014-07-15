/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::FileBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::blob::{Blob, BlobType, FileTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct File {
    pub blob: Blob,
    pub name: DOMString,
    pub type_: BlobType
}

impl File {
    pub fn new_inherited(global: &GlobalRef, _file_bits: &JSRef<Blob>, name: DOMString) -> File {
        File {
            blob: Blob::new_inherited(global),
            name: name,
            type_: FileTypeId
        }
        // XXXManishearth Once Blob is able to store data
        // the relevant subfields of file_bits should be copied over
    }

    pub fn new(global: &GlobalRef, file_bits: &JSRef<Blob>, name: DOMString) -> Temporary<File> {
        reflect_dom_object(box File::new_inherited(global, file_bits, name),
                           global,
                           FileBinding::Wrap)
    }
}

pub trait FileMethods {
    fn Name(&self) -> DOMString;
}

impl FileMethods for File {
    fn Name(&self) -> DOMString {
        self.name.clone()
    }
}

impl Reflectable for File {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.blob.reflector()
    }
}
