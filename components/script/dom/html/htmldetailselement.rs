/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, Ref};
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, QualName, local_name, ns};
use js::rust::HandleObject;
use script_bindings::domstring::DOMString;
use style::selector_parser::PseudoElement;

use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLDetailsElementBinding::HTMLDetailsElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLSlotElementBinding::HTMLSlotElement_Binding::HTMLSlotElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::GetRootNodeOptions;
use crate::dom::bindings::codegen::Bindings::NodeBinding::Node_Binding::NodeMethods;
use crate::dom::bindings::codegen::UnionTypes::ElementOrText;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, CustomElementCreationMode, Element, ElementCreator};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmlslotelement::HTMLSlotElement;
use crate::dom::node::{
    BindContext, ChildrenMutation, IsShadowTree, Node, NodeDamage, NodeTraits, ShadowIncluding,
    UnbindContext,
};
use crate::dom::text::Text;
use crate::dom::toggleevent::ToggleEvent;
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
    summary: Dom<HTMLSlotElement>,
    details_content: Dom<HTMLSlotElement>,
    /// The summary that is displayed if no other summary exists
    implicit_summary: Dom<HTMLElement>,
}

#[dom_struct]
pub(crate) struct HTMLDetailsElement {
    htmlelement: HTMLElement,
    toggle_counter: Cell<u32>,

    /// Represents the UA widget for the details element
    shadow_tree: DomRefCell<Option<ShadowTree>>,
}

/// Tracks all [details name groups](https://html.spec.whatwg.org/multipage/#details-name-group)
/// within a tree.
#[derive(Clone, Default, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct DetailsNameGroups {
    /// Map from `name` attribute to a list of details elements.
    pub(crate) groups: HashMap<DOMString, Vec<Dom<HTMLDetailsElement>>>,
}

/// Describes how to proceed in case two details elements in the same
/// [details name groups](https://html.spec.whatwg.org/multipage/#details-name-group) are
/// open at the same time
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExclusivityConflictResolution {
    CloseThisElement,
    CloseExistingOpenElement,
}

impl DetailsNameGroups {
    fn register_details_element(&mut self, details_element: &HTMLDetailsElement) {
        let name = details_element.Name();
        if name.is_empty() {
            return;
        }

        debug!("Registering details element with name={name:?}");
        let details_elements_with_the_same_name = self.groups.entry(name).or_default();

        // The spec tells us to keep the list in tree order, but that's not actually necessary.
        details_elements_with_the_same_name.push(Dom::from_ref(details_element));
    }

    fn unregister_details_element(
        &mut self,
        name: DOMString,
        details_element: &HTMLDetailsElement,
    ) {
        if name.is_empty() {
            return;
        }

        debug!("Unregistering details element with name={name:?}");
        let Entry::Occupied(mut entry) = self.groups.entry(name) else {
            panic!("details element is not registered");
        };
        entry
            .get_mut()
            .retain(|group_member| details_element != &**group_member);
    }

    /// Returns an iterator over all members with the given name, except for `details`.
    fn group_members_for(
        &self,
        name: &DOMString,
        details: &HTMLDetailsElement,
    ) -> impl Iterator<Item = DomRoot<HTMLDetailsElement>> {
        self.groups
            .get(name)
            .map(|members| members.iter())
            .expect("No details element with the given name was registered for the tree")
            .filter(move |member| **member != details)
            .map(|member| member.as_rooted())
    }
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

    fn create_shadow_tree(&self, can_gc: CanGc) {
        let document = self.owner_document();
        // TODO(stevennovaryo): Reimplement details styling so that it would not
        //                      mess the cascading and require some reparsing.
        let root = self.upcast::<Element>().attach_ua_shadow_root(true, can_gc);

        let summary = Element::create(
            QualName::new(None, ns!(html), local_name!("slot")),
            None,
            &document,
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Asynchronous,
            None,
            can_gc,
        );
        let summary = DomRoot::downcast::<HTMLSlotElement>(summary).unwrap();
        root.upcast::<Node>()
            .AppendChild(summary.upcast::<Node>(), can_gc)
            .unwrap();

        let fallback_summary = Element::create(
            QualName::new(None, ns!(html), local_name!("summary")),
            None,
            &document,
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Asynchronous,
            None,
            can_gc,
        );
        let fallback_summary = DomRoot::downcast::<HTMLElement>(fallback_summary).unwrap();
        fallback_summary
            .upcast::<Node>()
            .set_text_content_for_element(Some(DEFAULT_SUMMARY.into()), can_gc);
        summary
            .upcast::<Node>()
            .AppendChild(fallback_summary.upcast::<Node>(), can_gc)
            .unwrap();

        let details_content = Element::create(
            QualName::new(None, ns!(html), local_name!("slot")),
            None,
            &document,
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Asynchronous,
            None,
            can_gc,
        );
        let details_content = DomRoot::downcast::<HTMLSlotElement>(details_content).unwrap();

        root.upcast::<Node>()
            .AppendChild(details_content.upcast::<Node>(), can_gc)
            .unwrap();
        details_content
            .upcast::<Node>()
            .set_implemented_pseudo_element(PseudoElement::DetailsContent);

        let _ = self.shadow_tree.borrow_mut().insert(ShadowTree {
            summary: summary.as_traced(),
            details_content: details_content.as_traced(),
            implicit_summary: fallback_summary.as_traced(),
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

        if let Some(summary) = self.find_corresponding_summary_element() {
            shadow_tree
                .summary
                .Assign(vec![ElementOrText::Element(DomRoot::upcast(summary))]);
        }

        let mut slottable_children = vec![];
        for child in self.upcast::<Node>().children() {
            if let Some(element) = child.downcast::<Element>() {
                if element.local_name() == &local_name!("summary") {
                    continue;
                }

                slottable_children.push(ElementOrText::Element(DomRoot::from_ref(element)));
            }

            if let Some(text) = child.downcast::<Text>() {
                slottable_children.push(ElementOrText::Text(DomRoot::from_ref(text)));
            }
        }
        shadow_tree.details_content.Assign(slottable_children);
    }

    fn update_shadow_tree_styles(&self, can_gc: CanGc) {
        let shadow_tree = self.shadow_tree(can_gc);

        // Manually update the list item style of the implicit summary element.
        // Unlike the other summaries, this summary is in the shadow tree and
        // can't be styled with UA sheets
        let implicit_summary_list_item_style = if self.Open() {
            "disclosure-open"
        } else {
            "disclosure-closed"
        };
        let implicit_summary_style = format!(
            "display: list-item;
            counter-increment: list-item 0;
            list-style: {implicit_summary_list_item_style} inside;"
        );
        shadow_tree
            .implicit_summary
            .upcast::<Element>()
            .set_string_attribute(&local_name!("style"), implicit_summary_style.into(), can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#ensure-details-exclusivity-by-closing-the-given-element-if-needed>
    /// <https://html.spec.whatwg.org/multipage/#ensure-details-exclusivity-by-closing-other-elements-if-needed>
    fn ensure_details_exclusivity(
        &self,
        conflict_resolution_behaviour: ExclusivityConflictResolution,
    ) {
        // NOTE: This method implements two spec algorithms that are very similar to each other, distinguished by the
        // `conflict_resolution_behaviour` argument. Steps that are different between the two are annotated with two
        // spec comments.

        // Step 1. Assert: element has an open attribute.
        // Step 1. If element does not have an open attribute, then return.
        if !self.Open() {
            if conflict_resolution_behaviour ==
                ExclusivityConflictResolution::CloseExistingOpenElement
            {
                unreachable!()
            } else {
                return;
            }
        }

        // Step 2. If element does not have a name attribute, or its name attribute
        // is the empty string, then return.
        let name = self.Name();
        if name.is_empty() {
            return;
        }

        // Step 3. Let groupMembers be a list of elements, containing all elements in element's
        // details name group except for element, in tree order.
        // Step 4. For each element otherElement of groupMembers:
        //     Step 4.1 If the open attribute is set on otherElement, then:

        // NOTE: We implement an optimization that allows us to easily find details group members when the
        // root of the tree is a document or shadow root, which is why this looks a bit more complicated.
        let other_open_member = if let Some(shadow_root) = self.containing_shadow_root() {
            shadow_root
                .details_name_groups()
                .group_members_for(&name, self)
                .find(|group_member| group_member.Open())
        } else if self.upcast::<Node>().is_in_a_document_tree() {
            self.owner_document()
                .details_name_groups()
                .group_members_for(&name, self)
                .find(|group_member| group_member.Open())
        } else {
            // This is the slow case, which is hopefully not too common.
            self.upcast::<Node>()
                .GetRootNode(&GetRootNodeOptions::empty())
                .traverse_preorder(ShadowIncluding::No)
                .flat_map(DomRoot::downcast::<HTMLDetailsElement>)
                .filter(|details_element| {
                    details_element
                        .upcast::<Element>()
                        .get_string_attribute(&local_name!("name")) ==
                        name
                })
                .filter(|group_member| &**group_member != self)
                .find(|group_member| group_member.Open())
        };

        if let Some(other_open_member) = other_open_member {
            // Step 4.1.1 Assert: otherElement is the only element in groupMembers that has the open attribute set.
            // Step 4.1.2 Remove the open attribute on otherElement.
            // Step 4.1.3 Break.
            //
            // Step 4.1.1 Remove the open attribute on element.
            // Step 4.1.2 Break.
            // NOTE: We don't bother to assert here and don't need to "break" since we're not in a loop.
            match conflict_resolution_behaviour {
                ExclusivityConflictResolution::CloseThisElement => self.SetOpen(false),
                ExclusivityConflictResolution::CloseExistingOpenElement => {
                    other_open_member.SetOpen(false)
                },
            }
        }
    }
}

impl HTMLDetailsElementMethods<crate::DomTypeHolder> for HTMLDetailsElement {
    // https://html.spec.whatwg.org/multipage/#dom-details-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#dom-details-name
    make_atomic_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-details-open
    make_bool_getter!(Open, "open");

    // https://html.spec.whatwg.org/multipage/#dom-details-open
    make_bool_setter!(SetOpen, "open");
}

impl VirtualMethods for HTMLDetailsElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    /// <https://html.spec.whatwg.org/multipage/#the-details-element:concept-element-attributes-change-ext>
    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation, can_gc: CanGc) {
        self.super_type()
            .unwrap()
            .attribute_mutated(attr, mutation, can_gc);

        // Step 1. If namespace is not null, then return.
        if *attr.namespace() != ns!() {
            return;
        }

        // Step 2. If localName is name, then ensure details exclusivity by closing the given element if needed
        // given element.
        if attr.local_name() == &local_name!("name") {
            let old_name: Option<DOMString> = match mutation {
                AttributeMutation::Set(old, _) => old.map(|value| value.to_string().into()),
                AttributeMutation::Removed => Some(attr.value().to_string().into()),
            };

            if let Some(shadow_root) = self.containing_shadow_root() {
                if let Some(old_name) = old_name {
                    shadow_root
                        .details_name_groups()
                        .unregister_details_element(old_name, self);
                }
                if matches!(mutation, AttributeMutation::Set(..)) {
                    shadow_root
                        .details_name_groups()
                        .register_details_element(self);
                }
            } else if self.upcast::<Node>().is_in_a_document_tree() {
                let document = self.owner_document();
                if let Some(old_name) = old_name {
                    document
                        .details_name_groups()
                        .unregister_details_element(old_name, self);
                }
                if matches!(mutation, AttributeMutation::Set(..)) {
                    document
                        .details_name_groups()
                        .register_details_element(self);
                }
            }

            self.ensure_details_exclusivity(ExclusivityConflictResolution::CloseThisElement);
        }
        // Step 3. If localName is open, then:
        else if attr.local_name() == &local_name!("open") {
            self.update_shadow_tree_styles(can_gc);

            let counter = self.toggle_counter.get().wrapping_add(1);
            self.toggle_counter.set(counter);
            let (old_state, new_state) = if self.Open() {
                ("closed", "open")
            } else {
                ("open", "closed")
            };

            let this = Trusted::new(self);
            self.owner_global()
                .task_manager()
                .dom_manipulation_task_source()
                .queue(task!(details_notification_task_steps: move || {
                    let this = this.root();
                    if counter == this.toggle_counter.get() {
                        let event = ToggleEvent::new(
                            this.global().as_window(),
                            atom!("toggle"),
                            EventBubbles::DoesNotBubble,
                            EventCancelable::NotCancelable,
                            DOMString::from(old_state),
                            DOMString::from(new_state),
                            None,
                            CanGc::note(),
                        );
                        let event = event.upcast::<Event>();
                        event.fire(this.upcast::<EventTarget>(), CanGc::note());
                    }
                }));
            self.upcast::<Node>().dirty(NodeDamage::Other);

            // Step 3.2. If oldValue is null and value is not null, then ensure details exclusivity
            // by closing other elements if needed given element.
            let was_previously_closed = match mutation {
                AttributeMutation::Set(old, _) => old.is_none(),
                AttributeMutation::Removed => false,
            };
            if was_previously_closed && self.Open() {
                self.ensure_details_exclusivity(
                    ExclusivityConflictResolution::CloseExistingOpenElement,
                );
            }

            self.upcast::<Element>().set_open_state(self.Open());
        }
    }

    fn children_changed(&self, mutation: &ChildrenMutation, can_gc: CanGc) {
        self.super_type()
            .unwrap()
            .children_changed(mutation, can_gc);

        self.update_shadow_tree_contents(can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#the-details-element:html-element-insertion-steps>
    fn bind_to_tree(&self, context: &BindContext, can_gc: CanGc) {
        self.super_type().unwrap().bind_to_tree(context, can_gc);

        self.update_shadow_tree_contents(can_gc);
        self.update_shadow_tree_styles(can_gc);

        if context.tree_is_in_a_document_tree {
            // If this is true then we can't have been in a document tree previously, so
            // we register ourselves.
            self.owner_document()
                .details_name_groups()
                .register_details_element(self);
        }

        let was_already_in_shadow_tree = context.is_shadow_tree == IsShadowTree::Yes;
        if !was_already_in_shadow_tree {
            if let Some(shadow_root) = self.containing_shadow_root() {
                shadow_root
                    .details_name_groups()
                    .register_details_element(self);
            }
        }

        // Step 1. Ensure details exclusivity by closing the given element if needed given insertedNode.
        self.ensure_details_exclusivity(ExclusivityConflictResolution::CloseThisElement);
    }

    fn unbind_from_tree(&self, context: &UnbindContext, can_gc: CanGc) {
        self.super_type().unwrap().unbind_from_tree(context, can_gc);

        if context.tree_is_in_a_document_tree && !self.upcast::<Node>().is_in_a_document_tree() {
            self.owner_document()
                .details_name_groups()
                .unregister_details_element(self.Name(), self);
        }

        if !self.upcast::<Node>().is_in_a_shadow_tree() {
            if let Some(old_shadow_root) = self.containing_shadow_root() {
                // If we used to be in a shadow root, but aren't anymore, then unregister this details
                // element.
                old_shadow_root
                    .details_name_groups()
                    .unregister_details_element(self.Name(), self);
            }
        }
    }
}
