/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::ProcessingInstructionBinding;
use dom::characterdata::CharacterData;
use dom::document::AbstractDocument;
use dom::node::{AbstractNode, Node, ProcessingInstructionNodeTypeId};
use servo_util::str::DOMString;

/// An HTML processing instruction node.
pub struct ProcessingInstruction {
    characterdata: CharacterData,
    target: DOMString,
}

impl ProcessingInstruction {
    pub fn new_inherited(target: DOMString, data: DOMString, document: AbstractDocument) -> ProcessingInstruction {
        ProcessingInstruction {
            characterdata: CharacterData::new_inherited(ProcessingInstructionNodeTypeId, data, document),
            target: target
        }
    }

    pub fn new(target: DOMString, data: DOMString, document: AbstractDocument) -> AbstractNode {
        let node = ProcessingInstruction::new_inherited(target, data, document);
        Node::reflect_node(@mut node, document, ProcessingInstructionBinding::Wrap)
    }
}

impl ProcessingInstruction {
    pub fn Target(&self) -> DOMString {
        self.target.clone()
    }
}
