/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::ProcessingInstructionBinding;
use dom::bindings::codegen::InheritTypes::ProcessingInstructionDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::characterdata::CharacterData;
use dom::document::Document;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::node::{Node, ProcessingInstructionNodeTypeId};
use servo_util::str::DOMString;

/// An HTML processing instruction node.
#[deriving(Encodable)]
pub struct ProcessingInstruction {
    pub characterdata: CharacterData,
    pub target: DOMString,
}

impl ProcessingInstructionDerived for EventTarget {
    fn is_processinginstruction(&self) -> bool {
        self.type_id == NodeTargetTypeId(ProcessingInstructionNodeTypeId)
    }
}

impl ProcessingInstruction {
    pub fn new_inherited(target: DOMString, data: DOMString, document: &JSRef<Document>) -> ProcessingInstruction {
        ProcessingInstruction {
            characterdata: CharacterData::new_inherited(ProcessingInstructionNodeTypeId, data, document),
            target: target
        }
    }

    pub fn new(target: DOMString, data: DOMString, document: &JSRef<Document>) -> Temporary<ProcessingInstruction> {
        let node = ProcessingInstruction::new_inherited(target, data, document);
        Node::reflect_node(~node, document, ProcessingInstructionBinding::Wrap)
    }
}

pub trait ProcessingInstructionMethods {
    fn Target(&self) -> DOMString;
}

impl<'a> ProcessingInstructionMethods for JSRef<'a, ProcessingInstruction> {
    fn Target(&self) -> DOMString {
        self.target.clone()
    }
}
