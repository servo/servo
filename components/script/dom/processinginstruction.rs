/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ProcessingInstructionBinding;
use dom::bindings::codegen::Bindings::ProcessingInstructionBinding::ProcessingInstructionMethods;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::characterdata::CharacterData;
use dom::document::Document;
use dom::node::Node;
use dom_struct::dom_struct;
use typeholder::TypeHolderTrait;

/// An HTML processing instruction node.
#[dom_struct]
pub struct ProcessingInstruction<TH: TypeHolderTrait> {
    characterdata: CharacterData<TH>,
    target: DOMString,
}

impl<TH: TypeHolderTrait> ProcessingInstruction<TH> {
    fn new_inherited(target: DOMString, data: DOMString, document: &Document<TH>) -> ProcessingInstruction<TH> {
        ProcessingInstruction {
            characterdata: CharacterData::new_inherited(data, document),
            target: target
        }
    }

    pub fn new(target: DOMString, data: DOMString, document: &Document<TH>) -> DomRoot<ProcessingInstruction<TH>> {
        Node::<TH>::reflect_node(Box::new(ProcessingInstruction::new_inherited(target, data, document)),
                           document, ProcessingInstructionBinding::Wrap)
    }
}


impl<TH: TypeHolderTrait> ProcessingInstruction<TH> {
    pub fn target(&self) -> &DOMString {
        &self.target
    }
}

impl<TH: TypeHolderTrait> ProcessingInstructionMethods for ProcessingInstruction<TH> {
    // https://dom.spec.whatwg.org/#dom-processinginstruction-target
    fn Target(&self) -> DOMString {
        self.target.clone()
    }
}
