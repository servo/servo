/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::ProcessingInstructionBinding::ProcessingInstructionMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::characterdata::CharacterData;
use crate::dom::document::Document;
use crate::dom::node::Node;

/// An HTML processing instruction node.
#[dom_struct]
pub struct ProcessingInstruction {
    characterdata: CharacterData,
    target: DOMString,
}

impl ProcessingInstruction {
    fn new_inherited(
        target: DOMString,
        data: DOMString,
        document: &Document,
    ) -> ProcessingInstruction {
        ProcessingInstruction {
            characterdata: CharacterData::new_inherited(data, document),
            target,
        }
    }

    pub fn new(
        target: DOMString,
        data: DOMString,
        document: &Document,
    ) -> DomRoot<ProcessingInstruction> {
        Node::reflect_node(
            Box::new(ProcessingInstruction::new_inherited(target, data, document)),
            document,
        )
    }
}

impl ProcessingInstruction {
    pub fn target(&self) -> &DOMString {
        &self.target
    }
}

impl ProcessingInstructionMethods for ProcessingInstruction {
    // https://dom.spec.whatwg.org/#dom-processinginstruction-target
    fn Target(&self) -> DOMString {
        self.target.clone()
    }
}
