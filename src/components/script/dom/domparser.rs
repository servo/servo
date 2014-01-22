/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::DOMParserBinding;
use dom::bindings::codegen::DOMParserBinding::SupportedTypeValues::{Text_html, Text_xml};
use dom::bindings::codegen::InheritTypes::DocumentCast;
use dom::bindings::jsmanaged::JSManaged;
use dom::bindings::utils::{DOMString, Fallible, Reflector, Reflectable, reflect_dom_object2};
use dom::bindings::utils::FailureUnknown;
use dom::document::{Document, XML};
use dom::htmldocument::HTMLDocument;
use dom::window::Window;

pub struct DOMParser {
    owner: JSManaged<Window>, //XXXjdm Document instead?
    reflector_: Reflector,
    force_box_layout: @int
}

impl DOMParser {
    pub fn new_inherited(owner: JSManaged<Window>) -> DOMParser {
        DOMParser {
            owner: owner,
            reflector_: Reflector::new(),
            force_box_layout: @1
        }
    }

    pub fn new(owner: JSManaged<Window>) -> JSManaged<DOMParser> {
        reflect_dom_object2(~DOMParser::new_inherited(owner), owner.value(),
                            DOMParserBinding::Wrap)
    }

    pub fn Constructor(owner: JSManaged<Window>) -> Fallible<JSManaged<DOMParser>> {
        Ok(DOMParser::new(owner))
    }

    pub fn ParseFromString(&self,
                           _s: DOMString,
                           ty: DOMParserBinding::SupportedType)
                           -> Fallible<JSManaged<Document>> {
        match ty {
            Text_html => {
                Ok(DocumentCast::from(HTMLDocument::new(self.owner)))
            }
            Text_xml => {
                Ok(Document::new(self.owner, XML))
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
