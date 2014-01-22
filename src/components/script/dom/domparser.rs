/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::DOMParserBinding;
use dom::bindings::codegen::DOMParserBinding::SupportedTypeValues::{Text_html, Text_xml};
use dom::bindings::utils::{DOMString, Fallible, Reflector, Reflectable, reflect_dom_object};
use dom::bindings::utils::FailureUnknown;
use dom::document::{AbstractDocument, Document};
use dom::htmldocument::HTMLDocument;
use dom::window::Window;

pub struct DOMParser {
    owner: @mut Window, //XXXjdm Document instead?
    reflector_: Reflector
}

impl DOMParser {
    pub fn new_inherited(owner: @mut Window) -> DOMParser {
        DOMParser {
            owner: owner,
            reflector_: Reflector::new()
        }
    }

    pub fn new(owner: @mut Window) -> @mut DOMParser {
        reflect_dom_object(@mut DOMParser::new_inherited(owner), owner,
                           DOMParserBinding::Wrap)
    }

    pub fn Constructor(owner: @mut Window) -> Fallible<@mut DOMParser> {
        Ok(DOMParser::new(owner))
    }

    pub fn ParseFromString(&self,
                           _s: DOMString,
                           ty: DOMParserBinding::SupportedType)
                           -> Fallible<AbstractDocument> {
        match ty {
            Text_html => {
                Ok(HTMLDocument::new(self.owner))
            }
            Text_xml => {
                Document::Constructor(self.owner)
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
