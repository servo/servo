/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::abstractrange::{AbstractRange};
use crate::dom::bindings::codegen::Bindings::StaticRangeBinding::{self, StaticRangeInit};
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::inheritance::NodeTypeId;
use crate::dom::document::Document;
use crate::dom::node::Node;
use crate::dom::window::Window;
use dom_struct::dom_struct;

#[dom_struct]
pub struct StaticRange {
    abstractrange: AbstractRange,
}

impl StaticRange {
    fn new_inherited(
        start_container: &Node,
        start_offset: u32,
        end_container: &Node,
        end_offset: u32,
    ) -> StaticRange {
        StaticRange {
            abstractrange: AbstractRange::new_inherited(
                start_container,
                start_offset,
                end_container,
                end_offset,
            ),
        }
    }

    pub fn new(document: &Document, init: &StaticRangeInit) -> DomRoot<StaticRange> {
        let staticrange = reflect_dom_object(
            Box::new(StaticRange::new_inherited(
                &init.startContainer,
                init.startOffset,
                &init.endContainer,
                init.endOffset,
            )),
            document.window(),
            StaticRangeBinding::Wrap,
        );
        staticrange
    }

    // https://dom.spec.whatwg.org/#dom-staticrange-staticrange
    #[allow(non_snake_case)]
    pub fn Constructor(window: &Window, init: &StaticRangeInit) -> Fallible<DomRoot<StaticRange>> {
        match init.startContainer.type_id() {
            NodeTypeId::DocumentType | NodeTypeId::Attr => {
                return Err(Error::InvalidNodeType);
            },
            _ => (),
        }
        match init.endContainer.type_id() {
            NodeTypeId::DocumentType | NodeTypeId::Attr => {
                return Err(Error::InvalidNodeType);
            },
            _ => (),
        }
        let document = window.Document();
        Ok(StaticRange::new(&document, init))
    }
}
