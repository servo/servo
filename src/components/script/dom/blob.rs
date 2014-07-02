/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::FileDerived;
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::error::Fallible;
use dom::bindings::codegen::Bindings::BlobBinding;
use dom::window::Window;

#[deriving(Encodable)]
pub enum BlobType {
    BlobTypeId,
    FileTypeId
}

#[deriving(Encodable)]
pub struct Blob {
    reflector_: Reflector,
    window: JS<Window>,
    type_: BlobType
}

impl Blob {
    pub fn new_inherited(window: &JSRef<Window>) -> Blob {
        Blob {
            reflector_: Reflector::new(),
            window: JS::from_rooted(window),
            type_: BlobTypeId
        }
    }

    pub fn new(window: &JSRef<Window>) -> Temporary<Blob> {
        reflect_dom_object(box Blob::new_inherited(window),
                           window,
                           BlobBinding::Wrap)
    }

    pub fn Constructor(window: &JSRef<Window>) -> Fallible<Temporary<Blob>> {
        Ok(Blob::new(window))
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
