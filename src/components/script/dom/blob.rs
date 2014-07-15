/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::FileDerived;
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::Temporary;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::error::Fallible;
use dom::bindings::codegen::Bindings::BlobBinding;

#[deriving(Encodable)]
pub enum BlobType {
    BlobTypeId,
    FileTypeId
}

#[deriving(Encodable)]
pub struct Blob {
    reflector_: Reflector,
    global: GlobalField,
    type_: BlobType
}

impl Blob {
    pub fn new_inherited(global: &GlobalRef) -> Blob {
        Blob {
            reflector_: Reflector::new(),
            global: GlobalField::from_rooted(global),
            type_: BlobTypeId
        }
    }

    pub fn new(global: &GlobalRef) -> Temporary<Blob> {
        reflect_dom_object(box Blob::new_inherited(global),
                           global,
                           BlobBinding::Wrap)
    }

    pub fn Constructor(global: &GlobalRef) -> Fallible<Temporary<Blob>> {
        Ok(Blob::new(global))
    }
}

pub trait BlobMethods {
}

impl Reflectable for Blob {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}

impl FileDerived for Blob {
    fn is_file(&self) -> bool {
        match self.type_ {
            FileTypeId => true,
            _ => false
        }
    }
}
