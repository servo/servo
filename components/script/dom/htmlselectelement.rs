/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::default::Default;
use std::iter;

use webrender_api::units::DeviceIntRect;
use ipc_channel::ipc;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name};
use js::rust::HandleObject;
use style::attr::AttrValue;
use stylo_dom::ElementState;
use embedder_traits::{SelectElementOptionOrOptgroup, SelectElementOption};
use euclid::{Size2D, Point2D, Rect};
use embedder_traits::EmbedderMsg;

use crate::dom::bindings::codegen::GenericBindings::HTMLOptGroupElementBinding::HTMLOptGroupElement_Binding::HTMLOptGroupElementMethods;
use crate::dom::activation::Activatable;
use crate::dom::attr::Attr;
use crate::dom::bindings::cell::{DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLCollectionBinding::HTMLCollectionMethods;
use crate::dom::bindings::codegen::Bindings::HTMLOptionElementBinding::HTMLOptionElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLOptionsCollectionBinding::HTMLOptionsCollectionMethods;
use crate::dom::bindings::codegen::Bindings::HTMLSelectElementBinding::HTMLSelectElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::{
    ShadowRootMode, SlotAssignmentMode,
};
use crate::dom::bindings::codegen::GenericBindings::CharacterDataBinding::CharacterData_Binding::CharacterDataMethods;
use crate::dom::bindings::codegen::UnionTypes::{
    HTMLElementOrLong, HTMLOptionElementOrHTMLOptGroupElement,
};
use crate::dom::bindings::error::ErrorResult;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::characterdata::CharacterData;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlcollection::CollectionFilter;
use crate::dom::htmldivelement::HTMLDivElement;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlfieldsetelement::HTMLFieldSetElement;
use crate::dom::htmlformelement::{FormControl, FormDatum, FormDatumValue, HTMLFormElement};
use crate::dom::htmloptgroupelement::HTMLOptGroupElement;
use crate::dom::htmloptionelement::HTMLOptionElement;
use crate::dom::htmloptionscollection::HTMLOptionsCollection;
use crate::dom::node::{BindContext, ChildrenMutation, Node, NodeTraits, UnbindContext};
use crate::dom::nodelist::NodeList;
use crate::dom::shadowroot::IsUserAgentWidget;
use crate::dom::text::Text;
use crate::dom::validation::{Validatable, is_barred_by_datalist_ancestor};
use crate::dom::validitystate::{ValidationFlags, ValidityState};
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

const DEFAULT_SELECT_SIZE: u32 = 0;

const SELECT_BOX_STYLE: &str = "
    display: flex;
    align-items: center;
    height: 100%;
";

const TEXT_CONTAINER_STYLE: &str = "flex: 1;";

const CHEVRON_CONTAINER_STYLE: &str = "
    font-size: 16px;
    margin: 4px;
";

#[derive(JSTraceable, MallocSizeOf)]
struct OptionsFilter;
impl CollectionFilter for OptionsFilter {
    fn filter<'a>(&self, elem: &'a Element, root: &'a Node) -> bool {
        if !elem.is::<HTMLOptionElement>() {
            return false;
        }

        let node = elem.upcast::<Node>();
        if root.is_parent_of(node) {
            return true;
        }

        match node.GetParentNode() {
            Some(optgroup) => optgroup.is::<HTMLOptGroupElement>() && root.is_parent_of(&optgroup),
            None => false,
        }
    }
}

#[dom_struct]
pub(crate) struct HTMLSelectElement {
    htmlelement: HTMLElement,
    options: MutNullableDom<HTMLOptionsCollection>,
    form_owner: MutNullableDom<HTMLFormElement>,
    labels_node_list: MutNullableDom<NodeList>,
    validity_state: MutNullableDom<ValidityState>,
    shadow_tree: DomRefCell<Option<ShadowTree>>,
}

/// Holds handles to all elements in the UA shadow tree
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct ShadowTree {
    selected_option: Dom<Text>,
}

impl HTMLSelectElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLSelectElement {
        HTMLSelectElement {
            htmlelement: HTMLElement::new_inherited_with_state(
                ElementState::ENABLED | ElementState::VALID,
                local_name,
                prefix,
                document,
            ),
            options: Default::default(),
            form_owner: Default::default(),
            labels_node_list: Default::default(),
            validity_state: Default::default(),
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
    ) -> DomRoot<HTMLSelectElement> {
        let n = Node::reflect_node_with_proto(
            Box::new(HTMLSelectElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
        );

        n.upcast::<Node>().set_weird_parser_insertion_mode();
        n
    }

    // https://html.spec.whatwg.org/multipage/#concept-select-option-list
    pub(crate) fn list_of_options(
        &self,
    ) -> impl Iterator<Item = DomRoot<HTMLOptionElement>> + use<'_> {
        self.upcast::<Node>().children().flat_map(|node| {
            if node.is::<HTMLOptionElement>() {
                let node = DomRoot::downcast::<HTMLOptionElement>(node).unwrap();
                Choice3::First(iter::once(node))
            } else if node.is::<HTMLOptGroupElement>() {
                Choice3::Second(node.children().filter_map(DomRoot::downcast))
            } else {
                Choice3::Third(iter::empty())
            }
        })
    }

    // https://html.spec.whatwg.org/multipage/#placeholder-label-option
    fn get_placeholder_label_option(&self) -> Option<DomRoot<HTMLOptionElement>> {
        if self.Required() && !self.Multiple() && self.display_size() == 1 {
            self.list_of_options().next().filter(|node| {
                let parent = node.upcast::<Node>().GetParentNode();
                node.Value().is_empty() && parent.as_deref() == Some(self.upcast())
            })
        } else {
            None
        }
    }

    // https://html.spec.whatwg.org/multipage/#the-select-element:concept-form-reset-control
    pub(crate) fn reset(&self) {
        for opt in self.list_of_options() {
            opt.set_selectedness(opt.DefaultSelected());
            opt.set_dirtiness(false);
        }
        self.ask_for_reset();
    }

    // https://html.spec.whatwg.org/multipage/#ask-for-a-reset
    pub(crate) fn ask_for_reset(&self) {
        if self.Multiple() {
            return;
        }

        let mut first_enabled: Option<DomRoot<HTMLOptionElement>> = None;
        let mut last_selected: Option<DomRoot<HTMLOptionElement>> = None;

        for opt in self.list_of_options() {
            if opt.Selected() {
                opt.set_selectedness(false);
                last_selected = Some(DomRoot::from_ref(&opt));
            }
            let element = opt.upcast::<Element>();
            if first_enabled.is_none() && !element.disabled_state() {
                first_enabled = Some(DomRoot::from_ref(&opt));
            }
        }

        if let Some(last_selected) = last_selected {
            last_selected.set_selectedness(true);
        } else if self.display_size() == 1 {
            if let Some(first_enabled) = first_enabled {
                first_enabled.set_selectedness(true);
            }
        }
    }

    pub(crate) fn push_form_data(&self, data_set: &mut Vec<FormDatum>) {
        if self.Name().is_empty() {
            return;
        }
        for opt in self.list_of_options() {
            let element = opt.upcast::<Element>();
            if opt.Selected() && element.enabled_state() {
                data_set.push(FormDatum {
                    ty: self.Type(),
                    name: self.Name(),
                    value: FormDatumValue::String(opt.Value()),
                });
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#concept-select-pick
    pub(crate) fn pick_option(&self, picked: &HTMLOptionElement) {
        if !self.Multiple() {
            let picked = picked.upcast();
            for opt in self.list_of_options() {
                if opt.upcast::<HTMLElement>() != picked {
                    opt.set_selectedness(false);
                }
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#concept-select-size
    fn display_size(&self) -> u32 {
        if self.Size() == 0 {
            if self.Multiple() { 4 } else { 1 }
        } else {
            self.Size()
        }
    }

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

        let select_box = HTMLDivElement::new(local_name!("div"), None, &document, None, can_gc);
        select_box.upcast::<Element>().set_string_attribute(
            &local_name!("style"),
            SELECT_BOX_STYLE.into(),
            can_gc,
        );

        let text_container = HTMLDivElement::new(local_name!("div"), None, &document, None, can_gc);
        text_container.upcast::<Element>().set_string_attribute(
            &local_name!("style"),
            TEXT_CONTAINER_STYLE.into(),
            can_gc,
        );
        select_box
            .upcast::<Node>()
            .AppendChild(text_container.upcast::<Node>(), can_gc)
            .unwrap();

        let text = Text::new(DOMString::new(), &document, can_gc);
        let _ = self.shadow_tree.borrow_mut().insert(ShadowTree {
            selected_option: text.as_traced(),
        });
        text_container
            .upcast::<Node>()
            .AppendChild(text.upcast::<Node>(), can_gc)
            .unwrap();

        let chevron_container =
            HTMLDivElement::new(local_name!("div"), None, &document, None, can_gc);
        chevron_container.upcast::<Element>().set_string_attribute(
            &local_name!("style"),
            CHEVRON_CONTAINER_STYLE.into(),
            can_gc,
        );
        chevron_container
            .upcast::<Node>()
            .SetTextContent(Some("▾".into()), can_gc);
        select_box
            .upcast::<Node>()
            .AppendChild(chevron_container.upcast::<Node>(), can_gc)
            .unwrap();

        root.upcast::<Node>()
            .AppendChild(select_box.upcast::<Node>(), can_gc)
            .unwrap();
    }

    fn shadow_tree(&self, can_gc: CanGc) -> Ref<'_, ShadowTree> {
        if !self.upcast::<Element>().is_shadow_host() {
            self.create_shadow_tree(can_gc);
        }

        Ref::filter_map(self.shadow_tree.borrow(), Option::as_ref)
            .ok()
            .expect("UA shadow tree was not created")
    }

    pub(crate) fn update_shadow_tree(&self, can_gc: CanGc) {
        let shadow_tree = self.shadow_tree(can_gc);

        let selected_option_text = self
            .selected_option()
            .or_else(|| self.list_of_options().next())
            .map(|option| option.displayed_label())
            .unwrap_or_default();

        // Replace newlines with whitespace, then collapse and trim whitespace
        let displayed_text = itertools::join(selected_option_text.split_whitespace(), " ");

        shadow_tree
            .selected_option
            .upcast::<CharacterData>()
            .SetData(displayed_text.trim().into());
    }

    pub(crate) fn selection_changed(&self, can_gc: CanGc) {
        self.update_shadow_tree(can_gc);

        self.upcast::<EventTarget>()
            .fire_bubbling_event(atom!("change"), can_gc);
    }

    fn selected_option(&self) -> Option<DomRoot<HTMLOptionElement>> {
        self.list_of_options().find(|opt_elem| opt_elem.Selected())
    }

    pub(crate) fn show_menu(&self, can_gc: CanGc) -> Option<usize> {
        let (ipc_sender, ipc_receiver) = ipc::channel().expect("Failed to create IPC channel!");

        // Collect list of optgroups and options
        let mut index = 0;
        let mut embedder_option_from_option = |option: &HTMLOptionElement| {
            let embedder_option = SelectElementOption {
                id: index,
                label: option.displayed_label().into(),
                is_disabled: option.Disabled(),
            };
            index += 1;
            embedder_option
        };
        let options = self
            .upcast::<Node>()
            .children()
            .flat_map(|child| {
                if let Some(option) = child.downcast::<HTMLOptionElement>() {
                    return Some(embedder_option_from_option(option).into());
                }

                if let Some(optgroup) = child.downcast::<HTMLOptGroupElement>() {
                    let options = optgroup
                        .upcast::<Node>()
                        .children()
                        .flat_map(DomRoot::downcast::<HTMLOptionElement>)
                        .map(|option| embedder_option_from_option(&option))
                        .collect();
                    let label = optgroup.Label().into();

                    return Some(SelectElementOptionOrOptgroup::Optgroup { label, options });
                }

                None
            })
            .collect();

        let rect = self.upcast::<Node>().bounding_content_box_or_zero(can_gc);
        let rect = Rect::new(
            Point2D::new(rect.origin.x.to_px(), rect.origin.y.to_px()),
            Size2D::new(rect.size.width.to_px(), rect.size.height.to_px()),
        );

        let selected_index = self.list_of_options().position(|option| option.Selected());

        let document = self.owner_document();
        document.send_to_embedder(EmbedderMsg::ShowSelectElementMenu(
            document.webview_id(),
            options,
            selected_index,
            DeviceIntRect::from_untyped(&rect.to_box2d()),
            ipc_sender,
        ));

        let Ok(response) = ipc_receiver.recv() else {
            log::error!("Failed to receive response");
            return None;
        };

        if response.is_some() && response != selected_index {
            self.selection_changed(can_gc);
        }

        response
    }
}

impl HTMLSelectElementMethods<crate::DomTypeHolder> for HTMLSelectElement {
    /// <https://html.spec.whatwg.org/multipage/#dom-select-add>
    fn Add(
        &self,
        element: HTMLOptionElementOrHTMLOptGroupElement,
        before: Option<HTMLElementOrLong>,
    ) -> ErrorResult {
        self.Options().Add(element, before)
    }

    // https://html.spec.whatwg.org/multipage/#dom-fe-disabled
    make_bool_getter!(Disabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fe-disabled
    make_bool_setter!(SetDisabled, "disabled");

    /// <https://html.spec.whatwg.org/multipage/#dom-fae-form>
    fn GetForm(&self) -> Option<DomRoot<HTMLFormElement>> {
        self.form_owner()
    }

    // https://html.spec.whatwg.org/multipage/#dom-select-multiple
    make_bool_getter!(Multiple, "multiple");

    // https://html.spec.whatwg.org/multipage/#dom-select-multiple
    make_bool_setter!(SetMultiple, "multiple");

    // https://html.spec.whatwg.org/multipage/#dom-fe-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#dom-fe-name
    make_atomic_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-select-required
    make_bool_getter!(Required, "required");

    // https://html.spec.whatwg.org/multipage/#dom-select-required
    make_bool_setter!(SetRequired, "required");

    // https://html.spec.whatwg.org/multipage/#dom-select-size
    make_uint_getter!(Size, "size", DEFAULT_SELECT_SIZE);

    // https://html.spec.whatwg.org/multipage/#dom-select-size
    make_uint_setter!(SetSize, "size", DEFAULT_SELECT_SIZE);

    /// <https://html.spec.whatwg.org/multipage/#dom-select-type>
    fn Type(&self) -> DOMString {
        DOMString::from(if self.Multiple() {
            "select-multiple"
        } else {
            "select-one"
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    make_labels_getter!(Labels, labels_node_list);

    /// <https://html.spec.whatwg.org/multipage/#dom-select-options>
    fn Options(&self) -> DomRoot<HTMLOptionsCollection> {
        self.options.or_init(|| {
            let window = self.owner_window();
            HTMLOptionsCollection::new(&window, self, Box::new(OptionsFilter), CanGc::note())
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-select-length>
    fn Length(&self) -> u32 {
        self.Options().Length()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-select-length>
    fn SetLength(&self, length: u32, can_gc: CanGc) {
        self.Options().SetLength(length, can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-select-item>
    fn Item(&self, index: u32) -> Option<DomRoot<Element>> {
        self.Options().upcast().Item(index)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-select-item>
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<Element>> {
        self.Options().IndexedGetter(index)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-select-setter>
    fn IndexedSetter(
        &self,
        index: u32,
        value: Option<&HTMLOptionElement>,
        can_gc: CanGc,
    ) -> ErrorResult {
        self.Options().IndexedSetter(index, value, can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-select-nameditem>
    fn NamedItem(&self, name: DOMString) -> Option<DomRoot<HTMLOptionElement>> {
        self.Options()
            .NamedGetter(name)
            .and_then(DomRoot::downcast::<HTMLOptionElement>)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-select-remove>
    fn Remove_(&self, index: i32) {
        self.Options().Remove(index)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-select-remove>
    fn Remove(&self) {
        self.upcast::<Element>().Remove()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-select-value>
    fn Value(&self) -> DOMString {
        self.selected_option()
            .map(|opt_elem| opt_elem.Value())
            .unwrap_or_default()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-select-value>
    fn SetValue(&self, value: DOMString) {
        let mut opt_iter = self.list_of_options();
        // Reset until we find an <option> with a matching value
        for opt in opt_iter.by_ref() {
            if opt.Value() == value {
                opt.set_selectedness(true);
                opt.set_dirtiness(true);
                break;
            }
            opt.set_selectedness(false);
        }
        // Reset remaining <option> elements
        for opt in opt_iter {
            opt.set_selectedness(false);
        }

        self.validity_state()
            .perform_validation_and_update(ValidationFlags::VALUE_MISSING, CanGc::note());
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-select-selectedindex>
    fn SelectedIndex(&self) -> i32 {
        self.list_of_options()
            .enumerate()
            .filter(|(_, opt_elem)| opt_elem.Selected())
            .map(|(i, _)| i as i32)
            .next()
            .unwrap_or(-1)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-select-selectedindex>
    fn SetSelectedIndex(&self, index: i32, can_gc: CanGc) {
        let mut opt_iter = self.list_of_options();
        for opt in opt_iter.by_ref().take(index as usize) {
            opt.set_selectedness(false);
        }
        if let Some(opt) = opt_iter.next() {
            opt.set_selectedness(true);
            opt.set_dirtiness(true);
            // Reset remaining <option> elements
            for opt in opt_iter {
                opt.set_selectedness(false);
            }
        }

        // TODO: Track whether the selected element actually changed
        self.update_shadow_tree(can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-cva-willvalidate>
    fn WillValidate(&self) -> bool {
        self.is_instance_validatable()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-cva-validity>
    fn Validity(&self) -> DomRoot<ValidityState> {
        self.validity_state()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-cva-checkvalidity>
    fn CheckValidity(&self, can_gc: CanGc) -> bool {
        self.check_validity(can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-cva-reportvalidity>
    fn ReportValidity(&self, can_gc: CanGc) -> bool {
        self.report_validity(can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-cva-validationmessage>
    fn ValidationMessage(&self) -> DOMString {
        self.validation_message()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-cva-setcustomvalidity>
    fn SetCustomValidity(&self, error: DOMString) {
        self.validity_state().set_custom_error_message(error);
    }
}

impl VirtualMethods for HTMLSelectElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation, can_gc: CanGc) {
        self.super_type()
            .unwrap()
            .attribute_mutated(attr, mutation, can_gc);
        match *attr.local_name() {
            local_name!("required") => {
                self.validity_state()
                    .perform_validation_and_update(ValidationFlags::VALUE_MISSING, can_gc);
            },
            local_name!("disabled") => {
                let el = self.upcast::<Element>();
                match mutation {
                    AttributeMutation::Set(_) => {
                        el.set_disabled_state(true);
                        el.set_enabled_state(false);
                    },
                    AttributeMutation::Removed => {
                        el.set_disabled_state(false);
                        el.set_enabled_state(true);
                        el.check_ancestors_disabled_state_for_form_control();
                    },
                }

                self.validity_state()
                    .perform_validation_and_update(ValidationFlags::VALUE_MISSING, can_gc);
            },
            local_name!("form") => {
                self.form_attribute_mutated(mutation, can_gc);
            },
            _ => {},
        }
    }

    fn bind_to_tree(&self, context: &BindContext, can_gc: CanGc) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context, can_gc);
        }

        self.upcast::<Element>()
            .check_ancestors_disabled_state_for_form_control();
    }

    fn unbind_from_tree(&self, context: &UnbindContext, can_gc: CanGc) {
        self.super_type().unwrap().unbind_from_tree(context, can_gc);

        let node = self.upcast::<Node>();
        let el = self.upcast::<Element>();
        if node
            .ancestors()
            .any(|ancestor| ancestor.is::<HTMLFieldSetElement>())
        {
            el.check_ancestors_disabled_state_for_form_control();
        } else {
            el.check_disabled_attribute();
        }
    }

    fn children_changed(&self, mutation: &ChildrenMutation) {
        if let Some(s) = self.super_type() {
            s.children_changed(mutation);
        }

        self.update_shadow_tree(CanGc::note());
    }

    fn parse_plain_attribute(&self, local_name: &LocalName, value: DOMString) -> AttrValue {
        match *local_name {
            local_name!("size") => AttrValue::from_u32(value.into(), DEFAULT_SELECT_SIZE),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(local_name, value),
        }
    }
}

impl FormControl for HTMLSelectElement {
    fn form_owner(&self) -> Option<DomRoot<HTMLFormElement>> {
        self.form_owner.get()
    }

    fn set_form_owner(&self, form: Option<&HTMLFormElement>) {
        self.form_owner.set(form);
    }

    fn to_element(&self) -> &Element {
        self.upcast::<Element>()
    }
}

impl Validatable for HTMLSelectElement {
    fn as_element(&self) -> &Element {
        self.upcast()
    }

    fn validity_state(&self) -> DomRoot<ValidityState> {
        self.validity_state
            .or_init(|| ValidityState::new(&self.owner_window(), self.upcast(), CanGc::note()))
    }

    fn is_instance_validatable(&self) -> bool {
        // https://html.spec.whatwg.org/multipage/#enabling-and-disabling-form-controls%3A-the-disabled-attribute%3Abarred-from-constraint-validation
        // https://html.spec.whatwg.org/multipage/#the-datalist-element%3Abarred-from-constraint-validation
        !self.upcast::<Element>().disabled_state() && !is_barred_by_datalist_ancestor(self.upcast())
    }

    fn perform_validation(
        &self,
        validate_flags: ValidationFlags,
        _can_gc: CanGc,
    ) -> ValidationFlags {
        let mut failed_flags = ValidationFlags::empty();

        // https://html.spec.whatwg.org/multipage/#suffering-from-being-missing
        // https://html.spec.whatwg.org/multipage/#the-select-element%3Asuffering-from-being-missing
        if validate_flags.contains(ValidationFlags::VALUE_MISSING) && self.Required() {
            let placeholder = self.get_placeholder_label_option();
            let is_value_missing = !self
                .list_of_options()
                .any(|e| e.Selected() && placeholder != Some(e));
            failed_flags.set(ValidationFlags::VALUE_MISSING, is_value_missing);
        }

        failed_flags
    }
}

impl Activatable for HTMLSelectElement {
    fn as_element(&self) -> &Element {
        self.upcast()
    }

    fn is_instance_activatable(&self) -> bool {
        true
    }

    /// <https://html.spec.whatwg.org/multipage/#input-activation-behavior>
    fn activation_behavior(&self, _event: &Event, _target: &EventTarget, can_gc: CanGc) {
        let Some(selected_value) = self.show_menu(can_gc) else {
            // The user did not select a value
            return;
        };

        self.SetSelectedIndex(selected_value as i32, can_gc);
    }
}

enum Choice3<I, J, K> {
    First(I),
    Second(J),
    Third(K),
}

impl<I, J, K, T> Iterator for Choice3<I, J, K>
where
    I: Iterator<Item = T>,
    J: Iterator<Item = T>,
    K: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        match *self {
            Choice3::First(ref mut i) => i.next(),
            Choice3::Second(ref mut j) => j.next(),
            Choice3::Third(ref mut k) => k.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match *self {
            Choice3::First(ref i) => i.size_hint(),
            Choice3::Second(ref j) => j.size_hint(),
            Choice3::Third(ref k) => k.size_hint(),
        }
    }
}
