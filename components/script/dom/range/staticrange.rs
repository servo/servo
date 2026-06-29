/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::reflector::reflect_dom_object_with_proto_and_cx;

use crate::dom::abstractrange::AbstractRange;
use crate::dom::bindings::codegen::Bindings::StaticRangeBinding::{
    StaticRangeInit, StaticRangeMethods,
};
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::NodeTypeId;
use crate::dom::bindings::root::DomRoot;
use crate::dom::document::Document;
use crate::dom::node::Node;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct StaticRange {
    abstract_range: AbstractRange,
}

impl StaticRange {
    fn new_inherited(
        start_container: &Node,
        start_offset: u32,
        end_container: &Node,
        end_offset: u32,
    ) -> StaticRange {
        StaticRange {
            abstract_range: AbstractRange::new_inherited(
                start_container,
                start_offset,
                end_container,
                end_offset,
            ),
        }
    }
    pub(crate) fn new_with_doc(
        cx: &mut JSContext,
        document: &Document,
        proto: Option<HandleObject>,
        init: &StaticRangeInit,
    ) -> DomRoot<StaticRange> {
        StaticRange::new_with_proto(cx, document, proto, init)
    }

    pub(crate) fn new_with_proto(
        cx: &mut JSContext,
        document: &Document,
        proto: Option<HandleObject>,
        init: &StaticRangeInit,
    ) -> DomRoot<StaticRange> {
        reflect_dom_object_with_proto_and_cx(
            Box::new(StaticRange::new_inherited(
                &init.startContainer,
                init.startOffset,
                &init.endContainer,
                init.endOffset,
            )),
            document.window(),
            proto,
            cx,
        )
    }
}

impl StaticRangeMethods<crate::DomTypeHolder> for StaticRange {
    /// <https://dom.spec.whatwg.org/#dom-staticrange-staticrange>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        init: &StaticRangeInit,
    ) -> Fallible<DomRoot<StaticRange>> {
        match init.startContainer.type_id() {
            NodeTypeId::DocumentType | NodeTypeId::Attr => {
                return Err(Error::InvalidNodeType(Some(
                    "Invalid node type: startContainer cannot be DocumentType or Attr node".into(),
                )));
            },
            _ => (),
        }
        match init.endContainer.type_id() {
            NodeTypeId::DocumentType | NodeTypeId::Attr => {
                return Err(Error::InvalidNodeType(Some(
                    "Invalid node type: endContainer cannot be DocumentType or Attr node".into(),
                )));
            },
            _ => (),
        }
        let document = window.Document();
        Ok(StaticRange::new_with_doc(cx, &document, proto, init))
    }
}
