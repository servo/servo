/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, Ref};

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name};
use js::rust::HandleObject;
use script_layout_interface::parse_details_stylesheet;
use style::attr::AttrValue;

use super::element::ElementCreator;
use super::types::HTMLLinkElement;
use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLDetailsElementBinding::HTMLDetailsElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLSlotElementBinding::HTMLSlotElement_Binding::HTMLSlotElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::Node_Binding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::{
    ShadowRootMode, SlotAssignmentMode,
};
use crate::dom::bindings::codegen::UnionTypes::ElementOrText;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlslotelement::HTMLSlotElement;
use crate::dom::node::{BindContext, ChildrenMutation, Node, NodeDamage, NodeTraits};
use crate::dom::shadowroot::IsUserAgentWidget;
use crate::dom::text::Text;
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

/// The summary that should be presented if no `<summary>` element is present
const DEFAULT_SUMMARY: &str = "Details";

/// Holds handles to all slots in the UA shadow tree
///
/// The composition of the tree is described in
/// <https://html.spec.whatwg.org/multipage/#the-details-and-summary-elements>
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct ShadowTree {
    summary_slot: Dom<HTMLSlotElement>,
    descendants_slot: Dom<HTMLSlotElement>,
    fallback_summary: Dom<HTMLElement>,
}

#[dom_struct]
pub(crate) struct HTMLDetailsElement {
    htmlelement: HTMLElement,
    toggle_counter: Cell<u32>,

    /// Represents the UA widget for the details element
    shadow_tree: DomRefCell<Option<ShadowTree>>,
}

impl HTMLDetailsElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLDetailsElement {
        HTMLDetailsElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            toggle_counter: Cell::new(0),
            shadow_tree: Default::default(),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLDetailsElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLDetailsElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
        )
    }

    pub(crate) fn toggle(&self) {
        self.SetOpen(!self.Open());
    }

    fn shadow_tree(&self, can_gc: CanGc) -> Ref<'_, ShadowTree> {
        if !self.upcast::<Element>().is_shadow_host() {
            self.create_shadow_tree(can_gc);
        }

        Ref::filter_map(self.shadow_tree.borrow(), Option::as_ref)
            .ok()
            .expect("UA shadow tree was not created")
    }

    /// <https://html.spec.whatwg.org/multipage/#the-details-and-summary-elements>
    fn create_shadow_tree(&self, can_gc: CanGc) {
        let document = self.owner_document();
        let root = self
            .upcast::<Element>()
            .attach_shadow(
                IsUserAgentWidget::Yes,
                ShadowRootMode::Closed,
                false,
                false,
                false,
                SlotAssignmentMode::Manual,
                can_gc,
            )
            .expect("Attaching UA shadow root failed");

        // > The details element is expected to have an internal shadow tree with three child elements:
        // > 1. The first child element is a slot that is expected to take the details element's first summary
        // >    element child, if any.
        let summary_slot = HTMLSlotElement::new(local_name!("slot"), None, &document, None, can_gc);
        summary_slot.upcast::<Element>().set_attribute(
            &local_name!("id"),
            AttrValue::from_atomic("internal-summary-slot".to_owned()),
            can_gc,
        );
        root.upcast::<Node>()
            .AppendChild(summary_slot.upcast::<Node>(), can_gc)
            .unwrap();

        // >    This element has a single child summary element called the default summary which has
        // >    text content that is implementation-defined (and probably locale-specific).
        let fallback_summary =
            HTMLElement::new(local_name!("summary"), None, &document, None, can_gc);
        fallback_summary.upcast::<Element>().set_attribute(
            &local_name!("id"),
            AttrValue::from_atomic("internal-fallback-summary".to_owned()),
            can_gc,
        );
        fallback_summary
            .upcast::<Node>()
            .SetTextContent(Some(DEFAULT_SUMMARY.into()), can_gc);
        summary_slot
            .upcast::<Node>()
            .AppendChild(fallback_summary.upcast::<Node>(), can_gc)
            .unwrap();

        // > 2. The second child element is a slot that is expected to take the details element's
        // >    remaining descendants, if any. This element has no contents.
        let descendants_slot =
            HTMLSlotElement::new(local_name!("slot"), None, &document, None, can_gc);
        descendants_slot.upcast::<Element>().set_attribute(
            &local_name!("id"),
            AttrValue::from_atomic("internal-contents-slot".to_owned()),
            can_gc,
        );
        root.upcast::<Node>()
            .AppendChild(descendants_slot.upcast::<Node>(), can_gc)
            .unwrap();

        // > 3. The third child element is either a link or style element with the following
        // >    styles for the default summary:
        let link_element = HTMLLinkElement::new(
            local_name!("link"),
            None,
            &document,
            None,
            ElementCreator::ScriptCreated,
            can_gc,
        );
        link_element.upcast::<Element>().set_attribute(
            &local_name!("rel"),
            AttrValue::String("stylesheet".to_owned()),
            can_gc,
        );
        root.upcast::<Node>()
            .AppendChild(link_element.upcast::<Node>(), can_gc)
            .unwrap();

        // TODO(stevennovaryo): We need a mechanism for parsing and storing a stylesheet as
        //                      we shouldn't reparse this each time we access the element.
        let details_stylesheet = parse_details_stylesheet(
            link_element
                .upcast::<Node>()
                .owner_doc()
                .style_shared_lock(),
        );
        link_element.set_stylesheet(details_stylesheet.unwrap());

        let _ = self.shadow_tree.borrow_mut().insert(ShadowTree {
            summary_slot: summary_slot.as_traced(),
            descendants_slot: descendants_slot.as_traced(),
            fallback_summary: fallback_summary.as_traced(),
        });
        self.upcast::<Node>()
            .dirty(crate::dom::node::NodeDamage::Other);
    }

    pub(crate) fn find_corresponding_summary_element(&self) -> Option<DomRoot<HTMLElement>> {
        self.upcast::<Node>()
            .children()
            .filter_map(DomRoot::downcast::<HTMLElement>)
            .find(|html_element| {
                html_element.upcast::<Element>().local_name() == &local_name!("summary")
            })
    }

    fn update_shadow_tree_contents(&self, can_gc: CanGc) {
        let shadow_tree = self.shadow_tree(can_gc);

        let mut discovered_first_summary_element = false;
        let mut slottable_children = vec![];

        // Assign all children to its corresponding slots.
        for child in self.upcast::<Node>().children() {
            if let Some(element) = child.downcast::<Element>() {
                // Assign the first summary element to the summary slot.
                if element.local_name() == &local_name!("summary") &&
                    !discovered_first_summary_element
                {
                    shadow_tree
                        .summary_slot
                        .Assign(vec![ElementOrText::Element(DomRoot::from_ref(element))]);
                    discovered_first_summary_element = true;
                    continue;
                }

                slottable_children.push(ElementOrText::Element(DomRoot::from_ref(element)));
            }

            if let Some(text) = child.downcast::<Text>() {
                slottable_children.push(ElementOrText::Text(DomRoot::from_ref(text)));
            }
        }
        shadow_tree.descendants_slot.Assign(slottable_children);

        // If there are no summary element, assign fallback summary instead.
        if !discovered_first_summary_element {
            shadow_tree
                .summary_slot
                .Assign(vec![ElementOrText::Element(DomRoot::upcast(
                    shadow_tree.fallback_summary.as_rooted(),
                ))]);
        }
    }
}

impl HTMLDetailsElementMethods<crate::DomTypeHolder> for HTMLDetailsElement {
    // https://html.spec.whatwg.org/multipage/#dom-details-open
    make_bool_getter!(Open, "open");

    // https://html.spec.whatwg.org/multipage/#dom-details-open
    make_bool_setter!(SetOpen, "open");
}

impl VirtualMethods for HTMLDetailsElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation, can_gc: CanGc) {
        self.super_type()
            .unwrap()
            .attribute_mutated(attr, mutation, can_gc);

        if attr.local_name() == &local_name!("open") {
            let counter = self.toggle_counter.get() + 1;
            self.toggle_counter.set(counter);

            let this = Trusted::new(self);
            self.owner_global()
                .task_manager()
                .dom_manipulation_task_source()
                .queue(task!(details_notification_task_steps: move || {
                    let this = this.root();
                    if counter == this.toggle_counter.get() {
                        this.upcast::<EventTarget>().fire_event(atom!("toggle"), CanGc::note());
                    }
                }));
            self.upcast::<Node>().dirty(NodeDamage::Other);
        }
    }

    fn children_changed(&self, mutation: &ChildrenMutation) {
        self.super_type().unwrap().children_changed(mutation);

        self.update_shadow_tree_contents(CanGc::note());
    }

    fn bind_to_tree(&self, context: &BindContext, can_gc: CanGc) {
        self.super_type().unwrap().bind_to_tree(context, can_gc);

        self.update_shadow_tree_contents(CanGc::note());
    }
}
