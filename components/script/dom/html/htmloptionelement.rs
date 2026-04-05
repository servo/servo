/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::convert::TryInto;

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, QualName, local_name, ns};
use js::context::JSContext;
use js::rust::HandleObject;
use style::str::{split_html_space_chars, str_join};
use stylo_dom::ElementState;

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use crate::dom::bindings::codegen::Bindings::HTMLOptionElementBinding::HTMLOptionElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLSelectElementBinding::HTMLSelectElement_Binding::HTMLSelectElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::characterdata::CharacterData;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, CustomElementCreationMode, Element, ElementCreator};
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmlformelement::HTMLFormElement;
use crate::dom::html::htmloptgroupelement::HTMLOptGroupElement;
use crate::dom::html::htmlscriptelement::HTMLScriptElement;
use crate::dom::html::htmlselectelement::HTMLSelectElement;
use crate::dom::node::{
    BindContext, ChildrenMutation, CloneChildrenFlag, MoveContext, Node, NodeTraits,
    ShadowIncluding, UnbindContext,
};
use crate::dom::text::Text;
use crate::dom::types::DocumentFragment;
use crate::dom::validation::Validatable;
use crate::dom::validitystate::ValidationFlags;
use crate::dom::virtualmethods::VirtualMethods;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLOptionElement {
    htmlelement: HTMLElement,

    /// <https://html.spec.whatwg.org/multipage/#attr-option-selected>
    selectedness: Cell<bool>,

    /// <https://html.spec.whatwg.org/multipage/#concept-option-dirtiness>
    dirtiness: Cell<bool>,
}

impl HTMLOptionElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLOptionElement {
        HTMLOptionElement {
            htmlelement: HTMLElement::new_inherited_with_state(
                ElementState::ENABLED,
                local_name,
                prefix,
                document,
            ),
            selectedness: Cell::new(false),
            dirtiness: Cell::new(false),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLOptionElement> {
        Node::reflect_node_with_proto(
            cx,
            Box::new(HTMLOptionElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        )
    }

    pub(crate) fn set_selectedness(&self, selected: bool) {
        self.selectedness.set(selected);
        // Bump the tree version so that any live HTMLCollection (e.g. selectedOptions)
        // rooted at an ancestor invalidates its cached length and cursor.
        self.upcast::<Node>().rev_version();
    }

    pub(crate) fn set_dirtiness(&self, dirtiness: bool) {
        self.dirtiness.set(dirtiness);
    }

    fn pick_if_selected_and_reset(&self) {
        if let Some(select) = self.owner_select_element() {
            if self.Selected() {
                select.pick_option(self);
            }
            select.ask_for_reset();
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-option-index>
    fn index(&self) -> i32 {
        let Some(owner_select) = self.owner_select_element() else {
            return 0;
        };

        let Some(position) = owner_select.list_of_options().position(|n| &*n == self) else {
            // An option should always be in it's owner's list of options, but it's not worth a browser panic
            warn!("HTMLOptionElement called index_in_select at a select that did not contain it");
            return 0;
        };

        position.try_into().unwrap_or(0)
    }

    fn owner_select_element(&self) -> Option<DomRoot<HTMLSelectElement>> {
        let parent = self.upcast::<Node>().GetParentNode()?;

        if parent.is::<HTMLOptGroupElement>() {
            DomRoot::downcast::<HTMLSelectElement>(parent.GetParentNode()?)
        } else {
            DomRoot::downcast::<HTMLSelectElement>(parent)
        }
    }

    fn update_select_validity(&self, can_gc: CanGc) {
        if let Some(select) = self.owner_select_element() {
            select
                .validity_state(can_gc)
                .perform_validation_and_update(ValidationFlags::all(), can_gc);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-option-label>
    ///
    /// Note that this is not equivalent to <https://html.spec.whatwg.org/multipage/#dom-option-label>.
    pub(crate) fn displayed_label(&self) -> DOMString {
        // > The label of an option element is the value of the label content attribute, if there is one
        // > and its value is not the empty string, or, otherwise, the value of the element's text IDL attribute.
        let label = self
            .upcast::<Element>()
            .get_string_attribute(&local_name!("label"));

        if label.is_empty() {
            return self.Text();
        }

        label
    }

    /// <https://html.spec.whatwg.org/multipage/#option-element-nearest-ancestor-select>
    pub(crate) fn nearest_ancestor_select(&self) -> Option<DomRoot<HTMLSelectElement>> {
        // Step 1. Let ancestorOptgroup be null.
        // NOTE: We only care whether the value is non-null, so a boolean is enough
        let mut did_see_ancestor_optgroup = false;

        // Step 2. For each ancestor of option's ancestors, in reverse tree order:
        for ancestor in self
            .upcast::<Node>()
            .ancestors()
            .filter_map(DomRoot::downcast::<Element>)
        {
            // Step 2.1 If ancestor is a datalist, hr, or option element, then return null.
            if matches!(
                ancestor.local_name(),
                &local_name!("datalist") | &local_name!("hr") | &local_name!("option")
            ) {
                return None;
            }

            // Step 2.2 If ancestor is an optgroup element
            if ancestor.local_name() == &local_name!("optgroup") {
                // Step 2.1 If ancestorOptgroup is not null, then return null.
                if did_see_ancestor_optgroup {
                    return None;
                }

                // Step 2.2 Set ancestorOptgroup to ancestor.
                did_see_ancestor_optgroup = true;
            }

            // Step 2.3 If ancestor is a select, then return ancestor.
            if let Some(select) = DomRoot::downcast::<HTMLSelectElement>(ancestor) {
                return Some(select);
            }
        }

        // Step 3. Return null.
        None
    }

    /// <https://html.spec.whatwg.org/multipage/#maybe-clone-an-option-into-selectedcontent>
    pub(crate) fn maybe_clone_an_option_into_selectedcontent(&self, cx: &mut JSContext) {
        // Step 1. Let select be option's option element nearest ancestor select.
        let select = self.nearest_ancestor_select();

        // Step 2. If all of the following conditions are true:
        // * select is not null;
        // * option's selectedness is true; and
        // * select's enabled selectedcontent is not null,
        // * then run clone an option into a selectedcontent given option and select's enabled selectedcontent.
        if self.selectedness.get() {
            if let Some(selectedcontent) =
                select.and_then(|select| select.get_enabled_selectedcontent())
            {
                self.clone_an_option_into_selectedcontent(cx, &selectedcontent);
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#clone-an-option-into-a-selectedcontent>
    fn clone_an_option_into_selectedcontent(&self, cx: &mut JSContext, selectedcontent: &Element) {
        // Step 1. Let documentFragment be a new DocumentFragment whose node document is option's node document.
        let document_fragment = DocumentFragment::new(cx, &self.owner_document());

        // Step 2. For each child of option's children:
        for child in self.upcast::<Node>().children() {
            // Step 2.1 Let childClone be the result of running clone given child with subtree set to true.
            let child_clone = Node::clone(cx, &child, None, CloneChildrenFlag::CloneChildren, None);

            // Step 2.2 Append childClone to documentFragment.
            let _ = document_fragment
                .upcast::<Node>()
                .AppendChild(cx, &child_clone);
        }

        // Step 3. Replace all with documentFragment within selectedcontent.
        Node::replace_all(
            cx,
            Some(document_fragment.upcast()),
            selectedcontent.upcast(),
        );
    }
}

impl HTMLOptionElementMethods<crate::DomTypeHolder> for HTMLOptionElement {
    /// <https://html.spec.whatwg.org/multipage/#dom-option>
    fn Option(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        text: DOMString,
        value: Option<DOMString>,
        default_selected: bool,
        selected: bool,
    ) -> Fallible<DomRoot<HTMLOptionElement>> {
        let element = Element::create(
            cx,
            QualName::new(None, ns!(html), local_name!("option")),
            None,
            &window.Document(),
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Synchronous,
            proto,
        );

        let option = DomRoot::downcast::<HTMLOptionElement>(element).unwrap();

        if !text.is_empty() {
            option
                .upcast::<Node>()
                .set_text_content_for_element(cx, Some(text))
        }

        if let Some(val) = value {
            option.SetValue(val)
        }

        option.SetDefaultSelected(default_selected);
        option.set_selectedness(selected);
        option.update_select_validity(CanGc::from_cx(cx));
        Ok(option)
    }

    // https://html.spec.whatwg.org/multipage/#dom-option-disabled
    make_bool_getter!(Disabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-option-disabled
    make_bool_setter!(SetDisabled, "disabled");

    /// <https://html.spec.whatwg.org/multipage/#dom-option-text>
    fn Text(&self) -> DOMString {
        let mut content = DOMString::new();

        let mut iterator = self.upcast::<Node>().traverse_preorder(ShadowIncluding::No);
        while let Some(node) = iterator.peek() {
            if let Some(element) = node.downcast::<Element>() {
                let html_script = element.is::<HTMLScriptElement>();
                let svg_script = *element.namespace() == ns!(svg) &&
                    element.local_name() == &local_name!("script");
                if html_script || svg_script {
                    iterator.next_skipping_children();
                    continue;
                }
            }

            if node.is::<Text>() {
                let characterdata = node.downcast::<CharacterData>().unwrap();
                content.push_str(&characterdata.Data().str());
            }

            iterator.next();
        }

        DOMString::from(str_join(split_html_space_chars(&content.str()), " "))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-option-text>
    fn SetText(&self, cx: &mut JSContext, value: DOMString) {
        self.upcast::<Node>()
            .set_text_content_for_element(cx, Some(value))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-option-form>
    fn GetForm(&self) -> Option<DomRoot<HTMLFormElement>> {
        let parent = self.upcast::<Node>().GetParentNode().and_then(|p| {
            if p.is::<HTMLOptGroupElement>() {
                p.upcast::<Node>().GetParentNode()
            } else {
                Some(p)
            }
        });

        parent.and_then(|p| p.downcast::<HTMLSelectElement>().and_then(|s| s.GetForm()))
    }

    /// <https://html.spec.whatwg.org/multipage/#attr-option-value>
    fn Value(&self) -> DOMString {
        let element = self.upcast::<Element>();
        let attr = &local_name!("value");
        if element.has_attribute(attr) {
            element.get_string_attribute(attr)
        } else {
            self.Text()
        }
    }

    // https://html.spec.whatwg.org/multipage/#attr-option-value
    make_setter!(SetValue, "value");

    /// <https://html.spec.whatwg.org/multipage/#attr-option-label>
    fn Label(&self) -> DOMString {
        let element = self.upcast::<Element>();
        let attr = &local_name!("label");
        if element.has_attribute(attr) {
            element.get_string_attribute(attr)
        } else {
            self.Text()
        }
    }

    // https://html.spec.whatwg.org/multipage/#attr-option-label
    make_setter!(SetLabel, "label");

    // https://html.spec.whatwg.org/multipage/#dom-option-defaultselected
    make_bool_getter!(DefaultSelected, "selected");

    // https://html.spec.whatwg.org/multipage/#dom-option-defaultselected
    make_bool_setter!(SetDefaultSelected, "selected");

    /// <https://html.spec.whatwg.org/multipage/#dom-option-selected>
    fn Selected(&self) -> bool {
        self.selectedness.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-option-selected>
    fn SetSelected(&self, selected: bool, can_gc: CanGc) {
        self.dirtiness.set(true);
        self.set_selectedness(selected);
        self.pick_if_selected_and_reset();
        self.update_select_validity(can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-option-index>
    fn Index(&self) -> i32 {
        self.index()
    }
}

impl VirtualMethods for HTMLOptionElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(
        &self,
        cx: &mut js::context::JSContext,
        attr: &Attr,
        mutation: AttributeMutation,
    ) {
        self.super_type()
            .unwrap()
            .attribute_mutated(cx, attr, mutation);
        match *attr.local_name() {
            local_name!("disabled") => {
                let el = self.upcast::<Element>();
                match mutation {
                    AttributeMutation::Set(..) => {
                        el.set_disabled_state(true);
                        el.set_enabled_state(false);
                    },
                    AttributeMutation::Removed => {
                        el.set_disabled_state(false);
                        el.set_enabled_state(true);
                        el.check_parent_disabled_state_for_option();
                    },
                }
                self.update_select_validity(CanGc::from_cx(cx));
            },
            local_name!("selected") => {
                let mut selectedness_changed = false;
                match mutation {
                    AttributeMutation::Set(..) => {
                        // https://html.spec.whatwg.org/multipage/#concept-option-selectedness
                        if !self.dirtiness.get() && !self.selectedness.get() {
                            self.set_selectedness(true);
                            selectedness_changed = true;
                        }
                    },
                    AttributeMutation::Removed => {
                        // https://html.spec.whatwg.org/multipage/#concept-option-selectedness
                        if !self.dirtiness.get() && self.selectedness.get() {
                            self.set_selectedness(false);
                            selectedness_changed = true;
                        }
                    },
                }

                if selectedness_changed {
                    self.pick_if_selected_and_reset();

                    if let Some(select_element) = self.owner_select_element() {
                        select_element.update_shadow_tree(cx);
                    }
                }

                self.update_select_validity(CanGc::from_cx(cx));
            },
            local_name!("label") => {
                // The label of the selected option is displayed inside the select element, so we need to repaint
                // when it changes
                if let Some(select_element) = self.owner_select_element() {
                    select_element.update_shadow_tree(cx);
                }
            },
            _ => {},
        }
    }

    fn bind_to_tree(&self, cx: &mut JSContext, context: &BindContext) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(cx, context);
        }

        self.upcast::<Element>()
            .check_parent_disabled_state_for_option();

        self.pick_if_selected_and_reset();
        self.update_select_validity(CanGc::from_cx(cx));
    }

    fn unbind_from_tree(&self, context: &UnbindContext, can_gc: CanGc) {
        self.super_type().unwrap().unbind_from_tree(context, can_gc);

        if let Some(select) = context
            .parent
            .inclusive_ancestors(ShadowIncluding::No)
            .find_map(DomRoot::downcast::<HTMLSelectElement>)
        {
            select
                .validity_state(can_gc)
                .perform_validation_and_update(ValidationFlags::all(), can_gc);
            select.ask_for_reset();
        }

        let node = self.upcast::<Node>();
        let el = self.upcast::<Element>();
        if node.GetParentNode().is_some() {
            el.check_parent_disabled_state_for_option();
        } else {
            el.check_disabled_attribute();
        }
    }

    fn children_changed(&self, cx: &mut JSContext, mutation: &ChildrenMutation) {
        if let Some(super_type) = self.super_type() {
            super_type.children_changed(cx, mutation);
        }

        // Changing the descendants of a selected option can change it's displayed label
        // if it does not have a label attribute
        if !self
            .upcast::<Element>()
            .has_attribute(&local_name!("label"))
        {
            if let Some(owner_select) = self.owner_select_element() {
                if owner_select
                    .selected_option()
                    .is_some_and(|selected_option| self == &*selected_option)
                {
                    owner_select.update_shadow_tree(cx);
                }
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#the-option-element:html-element-moving-steps>
    fn moving_steps(&self, context: &MoveContext, can_gc: CanGc) {
        if let Some(super_type) = self.super_type() {
            super_type.moving_steps(context, can_gc);
        }

        // The option HTML element moving steps, given movedNode and oldParent, are to run update an
        // option's nearest ancestor select given movedNode.
        let element = self.upcast::<Element>();
        if let Some(old_parent) = context.old_parent {
            if let Some(select) = old_parent
                .inclusive_ancestors(ShadowIncluding::No)
                .find_map(DomRoot::downcast::<HTMLSelectElement>)
            {
                select
                    .validity_state(can_gc)
                    .perform_validation_and_update(ValidationFlags::all(), can_gc);
                select.ask_for_reset();
            }

            if self.upcast::<Node>().GetParentNode().is_some() {
                element.check_parent_disabled_state_for_option();
            } else {
                element.check_disabled_attribute();
            }
        }

        element.check_parent_disabled_state_for_option();

        self.pick_if_selected_and_reset();
        self.update_select_validity(can_gc);
    }
}
