/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrValue};
use dom::attr::AttrHelpers;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::HTMLTextAreaElementBinding;
use dom::bindings::codegen::Bindings::HTMLTextAreaElementBinding::HTMLTextAreaElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast, NodeCast};
use dom::bindings::codegen::InheritTypes::{HTMLTextAreaElementDerived, HTMLFieldSetElementDerived};
use dom::bindings::codegen::InheritTypes::{KeyboardEventCast, TextDerived, HTMLFormElementCast};
use dom::bindings::js::{JS, JSRef, Temporary, OptionalRootable};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::{Document, DocumentHelpers};
use dom::element::{AttributeHandlers, HTMLTextAreaElementTypeId, Element};
use dom::event::Event;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::htmlformelement::{FormControl, HTMLFormElement};
use dom::keyboardevent::KeyboardEvent;
use dom::node::{DisabledStateHelpers, Node, NodeHelpers, OtherNodeDamage, ElementNodeTypeId};
use dom::node::{document_from_node};
use textinput::{Multiple, TextInput, TriggerDefaultAction, DispatchInput, Nothing};
use dom::virtualmethods::VirtualMethods;

use servo_util::str::DOMString;
use string_cache::Atom;

use std::cell::Cell;

#[dom_struct]
pub struct HTMLTextAreaElement {
    htmlelement: HTMLElement,
    textinput: DOMRefCell<TextInput>,

    // https://html.spec.whatwg.org/multipage/forms.html#concept-textarea-dirty
    value_changed: Cell<bool>,
}

impl HTMLTextAreaElementDerived for EventTarget {
    fn is_htmltextareaelement(&self) -> bool {
        *self.type_id() == NodeTargetTypeId(ElementNodeTypeId(HTMLTextAreaElementTypeId))
    }
}

pub trait LayoutHTMLTextAreaElementHelpers {
    unsafe fn get_value_for_layout(self) -> String;
}

impl LayoutHTMLTextAreaElementHelpers for JS<HTMLTextAreaElement> {
    #[allow(unrooted_must_root)]
    unsafe fn get_value_for_layout(self) -> String {
        (*self.unsafe_get()).textinput.borrow_for_layout().get_content()
    }
}

static DEFAULT_COLS: u32 = 20;
static DEFAULT_ROWS: u32 = 2;

impl HTMLTextAreaElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLTextAreaElement {
        HTMLTextAreaElement {
            htmlelement: HTMLElement::new_inherited(HTMLTextAreaElementTypeId, localName, prefix, document),
            textinput: DOMRefCell::new(TextInput::new(Multiple, "".to_string())),
            value_changed: Cell::new(false),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLTextAreaElement> {
        let element = HTMLTextAreaElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLTextAreaElementBinding::Wrap)
    }
}

impl<'a> HTMLTextAreaElementMethods for JSRef<'a, HTMLTextAreaElement> {
    // TODO A few of these attributes have default values and additional
    // constraints

    // https://html.spec.whatwg.org/multipage/forms.html#dom-textarea-cols
    make_uint_getter!(Cols)

    // https://html.spec.whatwg.org/multipage/forms.html#dom-textarea-cols
    make_uint_setter!(SetCols, "cols")

    // http://www.whatwg.org/html/#dom-fe-disabled
    make_bool_getter!(Disabled)

    // http://www.whatwg.org/html/#dom-fe-disabled
    make_bool_setter!(SetDisabled, "disabled")

    // https://html.spec.whatwg.org/multipage/forms.html#attr-fe-name
    make_getter!(Name)

    // https://html.spec.whatwg.org/multipage/forms.html#attr-fe-name
    make_setter!(SetName, "name")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-textarea-placeholder
    make_getter!(Placeholder)

    // https://html.spec.whatwg.org/multipage/forms.html#dom-textarea-placeholder
    make_setter!(SetPlaceholder, "placeholder")

    // https://html.spec.whatwg.org/multipage/forms.html#attr-textarea-readonly
    make_bool_getter!(ReadOnly)

    // https://html.spec.whatwg.org/multipage/forms.html#attr-textarea-readonly
    make_bool_setter!(SetReadOnly, "readOnly")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-textarea-required
    make_bool_getter!(Required)

    // https://html.spec.whatwg.org/multipage/forms.html#dom-textarea-required
    make_bool_setter!(SetRequired, "required")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-textarea-rows
    make_uint_getter!(Rows)

    // https://html.spec.whatwg.org/multipage/forms.html#dom-textarea-rows
    make_uint_setter!(SetRows, "rows")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-textarea-wrap
    make_getter!(Wrap)

    // https://html.spec.whatwg.org/multipage/forms.html#dom-textarea-wrap
    make_setter!(SetWrap, "wrap")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-textarea-type
    fn Type(self) -> DOMString {
        "textarea".to_string()
    }

    // https://html.spec.whatwg.org/multipage/forms.html#dom-textarea-defaultvalue
    fn DefaultValue(self) -> DOMString {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.GetTextContent().unwrap()
    }

    // https://html.spec.whatwg.org/multipage/forms.html#dom-textarea-defaultvalue
    fn SetDefaultValue(self, value: DOMString) {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.SetTextContent(Some(value));

        // if the element's dirty value flag is false, then the element's
        // raw value must be set to the value of the element's textContent IDL attribute
        if !self.value_changed.get() {
            self.SetValue(node.GetTextContent().unwrap());
        }
    }

    // https://html.spec.whatwg.org/multipage/forms.html#dom-textarea-value
    fn Value(self) -> DOMString {
        self.textinput.borrow().get_content()
    }

    // https://html.spec.whatwg.org/multipage/forms.html#dom-textarea-value
    fn SetValue(self, value: DOMString) {
        self.textinput.borrow_mut().set_content(value);
        self.force_relayout();
    }
}

trait PrivateHTMLTextAreaElementHelpers {
    fn force_relayout(self);
}

impl<'a> PrivateHTMLTextAreaElementHelpers for JSRef<'a, HTMLTextAreaElement> {
    fn force_relayout(self) {
        let doc = document_from_node(self).root();
        let node: JSRef<Node> = NodeCast::from_ref(self);
        doc.content_changed(node, OtherNodeDamage)
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLTextAreaElement> {
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(attr),
            _ => ()
        }

        match attr.local_name() {
            &atom!("disabled") => {
                let node: JSRef<Node> = NodeCast::from_ref(*self);
                node.set_disabled_state(true);
                node.set_enabled_state(false);
            },
            _ => ()
        }
    }

    fn before_remove_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.before_remove_attr(attr),
            _ => ()
        }

        match attr.local_name() {
            &atom!("disabled") => {
                let node: JSRef<Node> = NodeCast::from_ref(*self);
                node.set_disabled_state(false);
                node.set_enabled_state(true);
                node.check_ancestors_disabled_state_for_form_control();
            },
            _ => ()
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        match self.super_type() {
            Some(ref s) => s.bind_to_tree(tree_in_doc),
            _ => (),
        }

        let node: JSRef<Node> = NodeCast::from_ref(*self);
        node.check_ancestors_disabled_state_for_form_control();
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("cols") => AttrValue::from_u32(value, DEFAULT_COLS),
            &atom!("rows") => AttrValue::from_u32(value, DEFAULT_ROWS),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }

    fn unbind_from_tree(&self, tree_in_doc: bool) {
        match self.super_type() {
            Some(ref s) => s.unbind_from_tree(tree_in_doc),
            _ => (),
        }

        let node: JSRef<Node> = NodeCast::from_ref(*self);
        if node.ancestors().any(|ancestor| ancestor.is_htmlfieldsetelement()) {
            node.check_ancestors_disabled_state_for_form_control();
        } else {
            node.check_disabled_attribute();
        }
    }

    fn child_inserted(&self, child: JSRef<Node>) {
        match self.super_type() {
            Some(s) => {
                s.child_inserted(child);
            }
            _ => (),
        }

        if child.is_text() {
            self.reset();
        }
    }

    // copied and modified from htmlinputelement.rs
    fn handle_event(&self, event: JSRef<Event>) {
        match self.super_type() {
            Some(s) => {
                s.handle_event(event);
            }
            _ => (),
        }

        if "click" == event.Type().as_slice() && !event.DefaultPrevented() {
            //TODO: set the editing position for text inputs

            let doc = document_from_node(*self).root();
            doc.request_focus(ElementCast::from_ref(*self));
        } else if "keydown" == event.Type().as_slice() && !event.DefaultPrevented() {
            let keyevent: Option<JSRef<KeyboardEvent>> = KeyboardEventCast::to_ref(event);
            keyevent.map(|event| {
                match self.textinput.borrow_mut().handle_keydown(event) {
                    TriggerDefaultAction => (),
                    DispatchInput => {
                        self.force_relayout();
                        self.value_changed.set(true);
                    }
                    Nothing => (),
                }
            });
        }
    }
}

impl Reflectable for HTMLTextAreaElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}

impl<'a> FormControl<'a> for JSRef<'a, HTMLTextAreaElement> {
    // FIXME: This is wrong (https://github.com/servo/servo/issues/3553)
    //        but we need html5ever to do it correctly
    fn form_owner(self) -> Option<Temporary<HTMLFormElement>> {
        // https://html.spec.whatwg.org/multipage/forms.html#reset-the-form-owner
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        let owner = elem.get_string_attribute(&atom!("form"));
        if !owner.is_empty() {
            let doc = document_from_node(self).root();
            let owner = doc.GetElementById(owner).root();
            match owner {
                Some(o) => {
                    let maybe_form: Option<JSRef<HTMLFormElement>> = HTMLFormElementCast::to_ref(*o);
                    if maybe_form.is_some() {
                        return maybe_form.map(Temporary::from_rooted);
                    }
                },
                _ => ()
            }
        }
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.ancestors().filter_map(|a| HTMLFormElementCast::to_ref(a)).next()
            .map(Temporary::from_rooted)
    }

    fn to_element(self) -> JSRef<'a, Element> {
        ElementCast::from_ref(self)
    }

    // https://html.spec.whatwg.org/multipage/forms.html#concept-fe-mutable
    fn mutable(self) -> bool {
        // https://html.spec.whatwg.org/multipage/forms.html#the-textarea-element:concept-fe-mutable
        !(self.Disabled() || self.ReadOnly())
    }

    fn reset(self) {
        self.SetValue(self.DefaultValue());
        self.value_changed.set(false);
    }
}
