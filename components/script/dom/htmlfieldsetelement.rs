/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::HTMLFieldSetElementBinding;
use dom::bindings::codegen::Bindings::HTMLFieldSetElementBinding::HTMLFieldSetElementMethods;
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLLegendElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLFieldSetElementDerived, NodeCast};
use dom::bindings::js::{Root, RootedReference};
use dom::document::Document;
use dom::element::{AttributeMutation, Element, ElementTypeId};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlcollection::{HTMLCollection, CollectionFilter};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId, window_from_node};
use dom::validitystate::ValidityState;
use dom::virtualmethods::VirtualMethods;

use util::str::{DOMString, StaticStringVec};

#[dom_struct]
pub struct HTMLFieldSetElement {
    htmlelement: HTMLElement
}

impl HTMLFieldSetElementDerived for EventTarget {
    fn is_htmlfieldsetelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFieldSetElement)))
    }
}

impl HTMLFieldSetElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLFieldSetElement {
        HTMLFieldSetElement {
            htmlelement:
                HTMLElement::new_inherited(HTMLElementTypeId::HTMLFieldSetElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLFieldSetElement> {
        let element = HTMLFieldSetElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLFieldSetElementBinding::Wrap)
    }
}

impl HTMLFieldSetElementMethods for HTMLFieldSetElement {
    // https://www.whatwg.org/html/#dom-fieldset-elements
    fn Elements(&self) -> Root<HTMLCollection> {
        #[derive(JSTraceable, HeapSizeOf)]
        struct ElementsFilter;
        impl CollectionFilter for ElementsFilter {
            fn filter<'a>(&self, elem: &'a Element, _root: &'a Node) -> bool {
                static TAG_NAMES: StaticStringVec = &["button", "fieldset", "input",
                    "keygen", "object", "output", "select", "textarea"];
                TAG_NAMES.iter().any(|&tag_name| tag_name == &**elem.local_name())
            }
        }
        let node = NodeCast::from_ref(self);
        let filter = box ElementsFilter;
        let window = window_from_node(node);
        HTMLCollection::create(window.r(), node, filter)
    }

    // https://html.spec.whatwg.org/multipage/#dom-cva-validity
    fn Validity(&self) -> Root<ValidityState> {
        let window = window_from_node(self);
        ValidityState::new(window.r())
    }

    // https://www.whatwg.org/html/#dom-fieldset-disabled
    make_bool_getter!(Disabled);

    // https://www.whatwg.org/html/#dom-fieldset-disabled
    make_bool_setter!(SetDisabled, "disabled");
}

impl VirtualMethods for HTMLFieldSetElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &HTMLElement = HTMLElementCast::from_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &atom!(disabled) => {
                let disabled_state = match mutation {
                    AttributeMutation::Set(None) => true,
                    AttributeMutation::Set(Some(_)) => {
                        // Fieldset was already disabled before.
                        return;
                    },
                    AttributeMutation::Removed => false,
                };
                let node = NodeCast::from_ref(self);
                node.set_disabled_state(disabled_state);
                node.set_enabled_state(!disabled_state);
                let mut found_legend = false;
                let children = node.children().filter(|node| {
                    if found_legend {
                        true
                    } else if node.is_htmllegendelement() {
                        found_legend = true;
                        false
                    } else {
                        true
                    }
                });
                let fields = children.flat_map(|child| {
                    child.traverse_preorder().filter(|descendant| {
                        match descendant.r().type_id() {
                            NodeTypeId::Element(
                                    ElementTypeId::HTMLElement(
                                        HTMLElementTypeId::HTMLButtonElement)) |
                            NodeTypeId::Element(
                                    ElementTypeId::HTMLElement(
                                        HTMLElementTypeId::HTMLInputElement)) |
                            NodeTypeId::Element(
                                    ElementTypeId::HTMLElement(
                                        HTMLElementTypeId::HTMLSelectElement)) |
                            NodeTypeId::Element(
                                    ElementTypeId::HTMLElement(
                                        HTMLElementTypeId::HTMLTextAreaElement)) => {
                                true
                            },
                            _ => false,
                        }
                    })
                });
                if disabled_state {
                    for field in fields {
                        field.set_disabled_state(true);
                        field.set_enabled_state(false);
                    }
                } else {
                    for field in fields {
                        field.check_disabled_attribute();
                        field.check_ancestors_disabled_state_for_form_control();
                    }
                }
            },
            _ => {},
        }
    }
}
