/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::HTMLCollectionBinding::HTMLCollectionMethods;
use dom::bindings::codegen::Bindings::HTMLOptionElementBinding::HTMLOptionElementMethods;
use dom::bindings::codegen::Bindings::HTMLOptionsCollectionBinding::HTMLOptionsCollectionMethods;
use dom::bindings::codegen::Bindings::HTMLSelectElementBinding;
use dom::bindings::codegen::Bindings::HTMLSelectElementBinding::HTMLSelectElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::UnionTypes::HTMLElementOrLong;
use dom::bindings::codegen::UnionTypes::HTMLOptionElementOrHTMLOptGroupElement;
//use dom::bindings::error::ErrorResult;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::{AttributeMutation, Element};
use dom::htmlcollection::CollectionFilter;
use dom::htmlelement::HTMLElement;
use dom::htmlfieldsetelement::HTMLFieldSetElement;
use dom::htmlformelement::{FormDatumValue, FormControl, FormDatum, HTMLFormElement};
use dom::htmloptgroupelement::HTMLOptGroupElement;
use dom::htmloptionelement::HTMLOptionElement;
use dom::htmloptionscollection::HTMLOptionsCollection;
use dom::node::{Node, UnbindContext, window_from_node};
use dom::nodelist::NodeList;
use dom::validation::Validatable;
use dom::validitystate::ValidityState;
use dom::virtualmethods::VirtualMethods;
use string_cache::Atom;
use style::attr::AttrValue;
use style::element_state::*;

#[derive(JSTraceable, HeapSizeOf)]
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
            Some(optgroup) =>
                optgroup.is::<HTMLOptGroupElement>() && root.is_parent_of(&optgroup),
            None => false,
        }
    }
}

#[dom_struct]
pub struct HTMLSelectElement {
    htmlelement: HTMLElement,
    options: MutNullableHeap<JS<HTMLOptionsCollection>>,
}

static DEFAULT_SELECT_SIZE: u32 = 0;

impl HTMLSelectElement {
    fn new_inherited(local_name: Atom,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLSelectElement {
        HTMLSelectElement {
            htmlelement:
                HTMLElement::new_inherited_with_state(IN_ENABLED_STATE,
                                                      local_name, prefix, document),
                options: Default::default()
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: Atom,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLSelectElement> {
        Node::reflect_node(box HTMLSelectElement::new_inherited(local_name, prefix, document),
                           document,
                           HTMLSelectElementBinding::Wrap)
    }

    // https://html.spec.whatwg.org/multipage/#the-select-element:concept-form-reset-control
    pub fn reset(&self) {
        let node = self.upcast::<Node>();
        for opt in node.traverse_preorder().filter_map(Root::downcast::<HTMLOptionElement>) {
            opt.set_selectedness(opt.DefaultSelected());
            opt.set_dirtiness(false);
        }
        self.ask_for_reset();
    }

    // https://html.spec.whatwg.org/multipage/#ask-for-a-reset
    pub fn ask_for_reset(&self) {
        if self.Multiple() {
            return;
        }

        let mut first_enabled: Option<Root<HTMLOptionElement>> = None;
        let mut last_selected: Option<Root<HTMLOptionElement>> = None;

        let node = self.upcast::<Node>();
        for opt in node.traverse_preorder().filter_map(Root::downcast::<HTMLOptionElement>) {
            if opt.Selected() {
                opt.set_selectedness(false);
                last_selected = Some(Root::from_ref(&opt));
            }
            let element = opt.upcast::<Element>();
            if first_enabled.is_none() && !element.disabled_state() {
                first_enabled = Some(Root::from_ref(&opt));
            }
        }

        if let Some(last_selected) = last_selected {
            last_selected.set_selectedness(true);
        } else {
            if self.display_size() == 1 {
                if let Some(first_enabled) = first_enabled {
                    first_enabled.set_selectedness(true);
                }
            }
        }
    }

    pub fn push_form_data(&self, data_set: &mut Vec<FormDatum>) {
        let node = self.upcast::<Node>();
        if self.Name().is_empty() {
            return;
        }
        for opt in node.traverse_preorder().filter_map(Root::downcast::<HTMLOptionElement>) {
            let element = opt.upcast::<Element>();
            if opt.Selected() && element.enabled_state() {
                data_set.push(FormDatum {
                    ty: self.Type(),
                    name: self.Name(),
                    value:  FormDatumValue::String(opt.Value())
                });
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#concept-select-pick
    pub fn pick_option(&self, picked: &HTMLOptionElement) {
        if !self.Multiple() {
            let node = self.upcast::<Node>();
            let picked = picked.upcast();
            for opt in node.traverse_preorder().filter_map(Root::downcast::<HTMLOptionElement>) {
                if opt.upcast::<HTMLElement>() != picked {
                    opt.set_selectedness(false);
                }
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#concept-select-size
    fn display_size(&self) -> u32 {
         if self.Size() == 0 {
             if self.Multiple() {
                 4
             } else {
                 1
             }
         } else {
             self.Size()
         }
     }
}

impl HTMLSelectElementMethods for HTMLSelectElement {
    // https://html.spec.whatwg.org/multipage/#dom-cva-validity
    fn Validity(&self) -> Root<ValidityState> {
        let window = window_from_node(self);
        ValidityState::new(&window, self.upcast())
    }

    // Note: this function currently only exists for union.html.
    // https://html.spec.whatwg.org/multipage/#dom-select-add
    fn Add(&self, _element: HTMLOptionElementOrHTMLOptGroupElement, _before: Option<HTMLElementOrLong>) {
    }

    // https://html.spec.whatwg.org/multipage/#dom-fe-disabled
    make_bool_getter!(Disabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fe-disabled
    make_bool_setter!(SetDisabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fae-form
    fn GetForm(&self) -> Option<Root<HTMLFormElement>> {
        self.form_owner()
    }

    // https://html.spec.whatwg.org/multipage/#dom-select-multiple
    make_bool_getter!(Multiple, "multiple");

    // https://html.spec.whatwg.org/multipage/#dom-select-multiple
    make_bool_setter!(SetMultiple, "multiple");

    // https://html.spec.whatwg.org/multipage/#dom-fe-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#dom-fe-name
    make_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-select-size
    make_uint_getter!(Size, "size", DEFAULT_SELECT_SIZE);

    // https://html.spec.whatwg.org/multipage/#dom-select-size
    make_uint_setter!(SetSize, "size", DEFAULT_SELECT_SIZE);

    // https://html.spec.whatwg.org/multipage/#dom-select-type
    fn Type(&self) -> DOMString {
        DOMString::from(if self.Multiple() {
            "select-multiple"
        } else {
            "select-one"
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    fn Labels(&self) -> Root<NodeList> {
        self.upcast::<HTMLElement>().labels()
    }

    // https://html.spec.whatwg.org/multipage/#dom-select-options
    fn Options(&self) -> Root<HTMLOptionsCollection> {
        self.options.or_init(|| {
            let window = window_from_node(self);
            HTMLOptionsCollection::new(
                &window, self.upcast(), box OptionsFilter)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-select-length
    fn Length(&self) -> u32 {
        self.Options().Length()
    }

    // https://html.spec.whatwg.org/multipage/#dom-select-length
    fn SetLength(&self, length: u32) {
        self.Options().SetLength(length)
    }

    // https://html.spec.whatwg.org/multipage/#dom-select-item
    fn Item(&self, index: u32) -> Option<Root<Element>> {
        self.Options().upcast().Item(index)
    }

    // https://html.spec.whatwg.org/multipage/#dom-select-item
    fn IndexedGetter(&self, index: u32) -> Option<Root<Element>> {
        self.Options().IndexedGetter(index)
    }

    // https://html.spec.whatwg.org/multipage/#dom-select-nameditem
    fn NamedItem(&self, name: DOMString) -> Option<Root<HTMLOptionElement>> {
        self.Options().NamedGetter(name).map_or(None, |e| Root::downcast::<HTMLOptionElement>(e))
    }

    // https://html.spec.whatwg.org/multipage/#dom-select-remove
    fn Remove_(&self, index: i32) {
        self.Options().Remove(index)
    }

    // https://html.spec.whatwg.org/multipage/#dom-select-remove
    fn Remove(&self) {
        self.upcast::<Element>().Remove()
    }
}

impl VirtualMethods for HTMLSelectElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        if attr.local_name() == &atom!("disabled") {
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
                }
            }
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        self.upcast::<Element>().check_ancestors_disabled_state_for_form_control();
    }

    fn unbind_from_tree(&self, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(context);

        let node = self.upcast::<Node>();
        let el = self.upcast::<Element>();
        if node.ancestors().any(|ancestor| ancestor.is::<HTMLFieldSetElement>()) {
            el.check_ancestors_disabled_state_for_form_control();
        } else {
            el.check_disabled_attribute();
        }
    }

    fn parse_plain_attribute(&self, local_name: &Atom, value: DOMString) -> AttrValue {
        match *local_name {
            atom!("size") => AttrValue::from_u32(value.into(), DEFAULT_SELECT_SIZE),
            _ => self.super_type().unwrap().parse_plain_attribute(local_name, value),
        }
    }
}

impl FormControl for HTMLSelectElement {}

impl Validatable for HTMLSelectElement {}
