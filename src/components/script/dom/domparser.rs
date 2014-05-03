/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::DOMParserBinding;
use dom::bindings::codegen::BindingDeclarations::DOMParserBinding::SupportedTypeValues::{Text_html, Text_xml};
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflector, Reflectable, reflect_dom_object};
use dom::bindings::error::{Fallible, FailureUnknown};
use dom::document::{Document, HTMLDocument, NonHTMLDocument};
use dom::window::Window;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct DOMParser {
    pub owner: JS<Window>, //XXXjdm Document instead?
    pub reflector_: Reflector
}

impl DOMParser {
    pub fn new_inherited(owner: &JSRef<Window>) -> DOMParser {
        DOMParser {
            owner: owner.unrooted(),
            reflector_: Reflector::new()
        }
    }

    pub fn new(owner: &JSRef<Window>) -> Temporary<DOMParser> {
        reflect_dom_object(~DOMParser::new_inherited(owner), owner,
                           DOMParserBinding::Wrap)
    }

    pub fn Constructor(owner: &JSRef<Window>) -> Fallible<Temporary<DOMParser>> {
        Ok(DOMParser::new(owner))
    }
}

pub trait DOMParserMethods {
    fn ParseFromString(&self, _s: DOMString, ty: DOMParserBinding::SupportedType)
        -> Fallible<Temporary<Document>>;
}

impl<'a> DOMParserMethods for JSRef<'a, DOMParser> {
    fn ParseFromString(&self,
                       _s: DOMString,
                       ty: DOMParserBinding::SupportedType)
                       -> Fallible<Temporary<Document>> {
        let owner = self.owner.root();
        match ty {
            Text_html => {
                Ok(Document::new(&owner.root_ref(), None, HTMLDocument, Some(~"text/html")))
            }
            Text_xml => {
                Ok(Document::new(&owner.root_ref(), None, NonHTMLDocument, Some(~"text/xml")))
            }
            _ => {
                Err(FailureUnknown)
            }
        }
    }
}

impl Reflectable for DOMParser {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}
