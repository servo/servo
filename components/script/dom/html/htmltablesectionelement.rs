/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, QualName, local_name, ns};
use js::context::JSContext;
use js::rust::HandleObject;
use style::attr::{AttrValue, LengthOrPercentageOrAuto};
use style::color::AbsoluteColor;

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::HTMLTableSectionElementBinding::HTMLTableSectionElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::error::{ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, LayoutDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{CustomElementCreationMode, Element, ElementCreator};
use crate::dom::html::htmlcollection::HTMLCollection;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmltablerowelement::HTMLTableRowElement;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLTableSectionElement {
    htmlelement: HTMLElement,
}

impl HTMLTableSectionElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLTableSectionElement {
        HTMLTableSectionElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLTableSectionElement> {
        let n = Node::reflect_node_with_proto(
            cx,
            Box::new(HTMLTableSectionElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        );

        n.upcast::<Node>().set_weird_parser_insertion_mode();
        n
    }
}

impl HTMLTableSectionElementMethods<crate::DomTypeHolder> for HTMLTableSectionElement {
    /// <https://html.spec.whatwg.org/multipage/#dom-tbody-rows>
    fn Rows(&self) -> DomRoot<HTMLCollection> {
        HTMLCollection::new_with_filter_fn(
            &self.owner_window(),
            self.upcast(),
            |element, root| {
                element.is::<HTMLTableRowElement>() &&
                    element.upcast::<Node>().GetParentNode().as_deref() == Some(root)
            },
            CanGc::deprecated_note(),
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-tbody-insertrow>
    fn InsertRow(&self, cx: &mut JSContext, index: i32) -> Fallible<DomRoot<HTMLElement>> {
        let node = self.upcast::<Node>();
        node.insert_cell_or_row(
            cx,
            index,
            || self.Rows(),
            |cx| {
                let row = Element::create(
                    cx,
                    QualName::new(None, ns!(html), local_name!("tr")),
                    None,
                    &node.owner_doc(),
                    ElementCreator::ScriptCreated,
                    CustomElementCreationMode::Asynchronous,
                    None,
                );
                DomRoot::downcast::<HTMLTableRowElement>(row).unwrap()
            },
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-tbody-deleterow>
    fn DeleteRow(&self, cx: &mut JSContext, index: i32) -> ErrorResult {
        let node = self.upcast::<Node>();
        node.delete_cell_or_row(cx, index, || self.Rows(), |n| n.is::<HTMLTableRowElement>())
    }
}

impl LayoutDom<'_, HTMLTableSectionElement> {
    pub(crate) fn get_background_color(self) -> Option<AbsoluteColor> {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("bgcolor"))
            .and_then(AttrValue::as_color)
            .cloned()
    }

    pub(crate) fn get_height(self) -> LengthOrPercentageOrAuto {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("height"))
            .map(AttrValue::as_dimension)
            .cloned()
            .unwrap_or(LengthOrPercentageOrAuto::Auto)
    }
}

impl VirtualMethods for HTMLTableSectionElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_affects_presentational_hints(&self, attr: &Attr) -> bool {
        match attr.local_name() {
            &local_name!("height") => true,
            _ => self
                .super_type()
                .unwrap()
                .attribute_affects_presentational_hints(attr),
        }
    }

    fn parse_plain_attribute(&self, local_name: &LocalName, value: DOMString) -> AttrValue {
        match *local_name {
            local_name!("bgcolor") => AttrValue::from_legacy_color(value.into()),
            local_name!("height") => AttrValue::from_dimension(value.into()),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(local_name, value),
        }
    }
}
