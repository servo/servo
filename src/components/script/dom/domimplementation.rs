/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::DOMImplementationBinding;
use dom::bindings::utils::{DOMString, Reflector, Reflectable, reflect_dom_object};
use dom::bindings::utils::{Fallible, InvalidCharacter, NamespaceError};
use dom::bindings::utils::{QName, Name, InvalidXMLName, xml_name_type};
use dom::documenttype::DocumentType;
use dom::node::AbstractNode;
use dom::window::Window;

pub struct DOMImplementation {
    owner: @mut Window,
    reflector_: Reflector
}

impl DOMImplementation {
    pub fn new_inherited(owner: @mut Window) -> DOMImplementation {
        DOMImplementation {
            owner: owner,
            reflector_: Reflector::new()
        }
    }

    pub fn new(owner: @mut Window) -> @mut DOMImplementation {
        reflect_dom_object(@mut DOMImplementation::new_inherited(owner), owner,
                           DOMImplementationBinding::Wrap)
    }
}

impl Reflectable for DOMImplementation {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}

// http://dom.spec.whatwg.org/#domimplementation
impl DOMImplementation {
    // http://dom.spec.whatwg.org/#dom-domimplementation-createdocumenttype
    pub fn CreateDocumentType(&self, qname: DOMString, pubid: DOMString, sysid: DOMString) -> Fallible<AbstractNode> {
        // FIXME: To be removed in https://github.com/mozilla/servo/issues/1498
        let force_quirks : bool = false;
        match xml_name_type(qname) {
            // Step 1.
            InvalidXMLName => Err(InvalidCharacter),
            // Step 2.
            Name => Err(NamespaceError),
            // Step 3.
            QName => Ok(DocumentType::new(qname, Some(pubid), Some(sysid), force_quirks, self.owner.Document()))
        }
    }
}
