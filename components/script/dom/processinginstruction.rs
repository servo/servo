/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ProcessingInstructionBinding;
use dom::bindings::codegen::Bindings::ProcessingInstructionBinding::ProcessingInstructionMethods;
use dom::bindings::codegen::InheritTypes::ProcessingInstructionDerived;
use dom::bindings::js::Root;
use dom::characterdata::{CharacterData, CharacterDataTypeId};
use dom::document::Document;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::node::{Node, NodeTypeId};
use util::str::DOMString;

/// An HTML processing instruction node.
#[dom_struct]
pub struct ProcessingInstruction {
    characterdata: CharacterData,
    target: DOMString,
}

impl ProcessingInstructionDerived for EventTarget {
    fn is_processinginstruction(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction))
    }
}

impl ProcessingInstruction {
    fn new_inherited(target: DOMString, data: DOMString, document: &Document) -> ProcessingInstruction {
        ProcessingInstruction {
            characterdata: CharacterData::new_inherited(CharacterDataTypeId::ProcessingInstruction, data, document),
            target: target
        }
    }

    pub fn new(target: DOMString, data: DOMString, document: &Document) -> Root<ProcessingInstruction> {
        Node::reflect_node(box ProcessingInstruction::new_inherited(target, data, document),
                           document, ProcessingInstructionBinding::Wrap)
    }
}


impl ProcessingInstruction {
    pub fn target(&self) -> &DOMString {
        &self.target
    }
}

impl<'a> ProcessingInstructionMethods for &'a ProcessingInstruction {
    // https://dom.spec.whatwg.org/#dom-processinginstruction-target
    fn Target(self) -> DOMString {
        self.target.clone()
    }
}
