/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrValue};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::HTMLTextAreaElementBinding;
use dom::bindings::codegen::Bindings::HTMLTextAreaElementBinding::HTMLTextAreaElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::KeyboardEventCast;
use dom::bindings::codegen::InheritTypes::{ElementCast, EventTargetCast, HTMLElementCast, NodeCast};
use dom::bindings::codegen::InheritTypes::{HTMLTextAreaElementDerived, HTMLFieldSetElementDerived};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{LayoutJS, Root};
use dom::bindings::refcounted::Trusted;
use dom::document::Document;
use dom::element::{Element, ElementTypeId};
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::htmlformelement::FormControl;
use dom::keyboardevent::KeyboardEvent;
use dom::node::{ChildrenMutation, Node, NodeDamage};
use dom::node::{NodeTypeId, document_from_node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use msg::constellation_msg::ConstellationChan;
use script_task::ScriptTaskEventCategory::InputEvent;
use script_task::{Runnable, CommonScriptMsg};
use textinput::{TextInput, Lines, KeyReaction};

use string_cache::Atom;
use util::str::DOMString;

use std::borrow::ToOwned;
use std::cell::Cell;

#[dom_struct]
pub struct HTMLTextAreaElement {
    htmlelement: HTMLElement,
    #[ignore_heap_size_of = "#7193"]
    textinput: DOMRefCell<TextInput<ConstellationChan>>,
    cols: Cell<u32>,
    rows: Cell<u32>,
    // https://html.spec.whatwg.org/multipage/#concept-textarea-dirty
    value_changed: Cell<bool>,
}

impl HTMLTextAreaElementDerived for EventTarget {
    fn is_htmltextareaelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)))
    }
}

pub trait LayoutHTMLTextAreaElementHelpers {
    #[allow(unsafe_code)]
    unsafe fn get_value_for_layout(self) -> String;
}

pub trait RawLayoutHTMLTextAreaElementHelpers {
    #[allow(unsafe_code)]
    unsafe fn get_cols_for_layout(self) -> u32;
    #[allow(unsafe_code)]
    unsafe fn get_rows_for_layout(self) -> u32;
}

impl LayoutHTMLTextAreaElementHelpers for LayoutJS<HTMLTextAreaElement> {
    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn get_value_for_layout(self) -> String {
        (*self.unsafe_get()).textinput.borrow_for_layout().get_content()
    }
}

impl<'a> RawLayoutHTMLTextAreaElementHelpers for &'a HTMLTextAreaElement {
    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn get_cols_for_layout(self) -> u32 {
        self.cols.get()
    }

    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn get_rows_for_layout(self) -> u32 {
        self.rows.get()
    }
}

static DEFAULT_COLS: u32 = 20;
static DEFAULT_ROWS: u32 = 2;

impl HTMLTextAreaElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLTextAreaElement {
        let chan = document.window().r().constellation_chan();
        HTMLTextAreaElement {
            htmlelement:
                HTMLElement::new_inherited(HTMLElementTypeId::HTMLTextAreaElement, localName, prefix, document),
            textinput: DOMRefCell::new(TextInput::new(Lines::Multiple, "".to_owned(), chan)),
            cols: Cell::new(DEFAULT_COLS),
            rows: Cell::new(DEFAULT_ROWS),
            value_changed: Cell::new(false),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLTextAreaElement> {
        let element = HTMLTextAreaElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLTextAreaElementBinding::Wrap)
    }
}

impl HTMLTextAreaElementMethods for HTMLTextAreaElement {
    // TODO A few of these attributes have default values and additional
    // constraints

    // https://html.spec.whatwg.org/multipage/#dom-textarea-cols
    make_uint_getter!(Cols, "cols", DEFAULT_COLS);

    // https://html.spec.whatwg.org/multipage/#dom-textarea-cols
    make_limited_uint_setter!(SetCols, "cols", DEFAULT_COLS);

    // https://www.whatwg.org/html/#dom-fe-disabled
    make_bool_getter!(Disabled);

    // https://www.whatwg.org/html/#dom-fe-disabled
    make_bool_setter!(SetDisabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#attr-fe-name
    make_getter!(Name);

    // https://html.spec.whatwg.org/multipage/#attr-fe-name
    make_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-textarea-placeholder
    make_getter!(Placeholder);

    // https://html.spec.whatwg.org/multipage/#dom-textarea-placeholder
    make_setter!(SetPlaceholder, "placeholder");

    // https://html.spec.whatwg.org/multipage/#attr-textarea-readonly
    make_bool_getter!(ReadOnly);

    // https://html.spec.whatwg.org/multipage/#attr-textarea-readonly
    make_bool_setter!(SetReadOnly, "readonly");

    // https://html.spec.whatwg.org/multipage/#dom-textarea-required
    make_bool_getter!(Required);

    // https://html.spec.whatwg.org/multipage/#dom-textarea-required
    make_bool_setter!(SetRequired, "required");

    // https://html.spec.whatwg.org/multipage/#dom-textarea-rows
    make_uint_getter!(Rows, "rows", DEFAULT_ROWS);

    // https://html.spec.whatwg.org/multipage/#dom-textarea-rows
    make_limited_uint_setter!(SetRows, "rows", DEFAULT_ROWS);

    // https://html.spec.whatwg.org/multipage/#dom-textarea-wrap
    make_getter!(Wrap);

    // https://html.spec.whatwg.org/multipage/#dom-textarea-wrap
    make_setter!(SetWrap, "wrap");

    // https://html.spec.whatwg.org/multipage/#dom-textarea-type
    fn Type(&self) -> DOMString {
        "textarea".to_owned()
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea-defaultvalue
    fn DefaultValue(&self) -> DOMString {
        let node = NodeCast::from_ref(self);
        node.GetTextContent().unwrap()
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea-defaultvalue
    fn SetDefaultValue(&self, value: DOMString) {
        let node = NodeCast::from_ref(self);
        node.SetTextContent(Some(value));

        // if the element's dirty value flag is false, then the element's
        // raw value must be set to the value of the element's textContent IDL attribute
        if !self.value_changed.get() {
            self.reset();
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea-value
    fn Value(&self) -> DOMString {
        self.textinput.borrow().get_content()
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea-value
    fn SetValue(&self, value: DOMString) {
        // TODO move the cursor to the end of the field
        self.textinput.borrow_mut().set_content(value);
        self.value_changed.set(true);

        self.force_relayout();
    }
}


impl HTMLTextAreaElement {
    // https://html.spec.whatwg.org/multipage/#concept-fe-mutable
    pub fn mutable(&self) -> bool {
        // https://html.spec.whatwg.org/multipage/#the-textarea-element:concept-fe-mutable
        !(self.Disabled() || self.ReadOnly())
    }
    pub fn reset(&self) {
        // https://html.spec.whatwg.org/multipage/#the-textarea-element:concept-form-reset-control
        self.SetValue(self.DefaultValue());
        self.value_changed.set(false);
    }
}


impl HTMLTextAreaElement {
    fn force_relayout(&self) {
        let doc = document_from_node(self);
        let node = NodeCast::from_ref(self);
        doc.r().content_changed(node, NodeDamage::OtherNodeDamage)
    }

    fn dispatch_change_event(&self) {
        let window = window_from_node(self);
        let window = window.r();
        let event = Event::new(GlobalRef::Window(window),
                               "input".to_owned(),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable);

        let target = EventTargetCast::from_ref(self);
        target.dispatch_event(event.r());
    }
}

impl VirtualMethods for HTMLTextAreaElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &HTMLElement = HTMLElementCast::from_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: &Attr) {
        if let Some(ref s) = self.super_type() {
            s.after_set_attr(attr);
        }

        match attr.local_name() {
            &atom!("disabled") => {
                let node = NodeCast::from_ref(self);
                node.set_disabled_state(true);
                node.set_enabled_state(false);
            },
            &atom!("cols") => {
                match *attr.value() {
                    AttrValue::UInt(_, value) => self.cols.set(value),
                    _ => panic!("Expected an AttrValue::UInt"),
                }
            },
            &atom!("rows") => {
                match *attr.value() {
                    AttrValue::UInt(_, value) => self.rows.set(value),
                    _ => panic!("Expected an AttrValue::UInt"),
                }
            },
            _ => ()
        }
    }

    fn before_remove_attr(&self, attr: &Attr) {
        if let Some(ref s) = self.super_type() {
            s.before_remove_attr(attr);
        }

        match attr.local_name() {
            &atom!("disabled") => {
                let node = NodeCast::from_ref(self);
                node.set_disabled_state(false);
                node.set_enabled_state(true);
                node.check_ancestors_disabled_state_for_form_control();
            },
            &atom!("cols") => {
                self.cols.set(DEFAULT_COLS);
            },
            &atom!("rows") => {
                self.rows.set(DEFAULT_ROWS);
            },
            _ => ()
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        let node = NodeCast::from_ref(self);
        node.check_ancestors_disabled_state_for_form_control();
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("cols") => AttrValue::from_limited_u32(value, DEFAULT_COLS),
            &atom!("rows") => AttrValue::from_limited_u32(value, DEFAULT_ROWS),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }

    fn unbind_from_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.unbind_from_tree(tree_in_doc);
        }

        let node = NodeCast::from_ref(self);
        if node.ancestors().any(|ancestor| ancestor.r().is_htmlfieldsetelement()) {
            node.check_ancestors_disabled_state_for_form_control();
        } else {
            node.check_disabled_attribute();
        }
    }

    fn children_changed(&self, mutation: &ChildrenMutation) {
        if let Some(ref s) = self.super_type() {
            s.children_changed(mutation);
        }
        if !self.value_changed.get() {
            self.reset();
        }
    }

    // copied and modified from htmlinputelement.rs
    fn handle_event(&self, event: &Event) {
        if let Some(s) = self.super_type() {
            s.handle_event(event);
        }

        if &*event.Type() == "click" && !event.DefaultPrevented() {
            //TODO: set the editing position for text inputs

            let doc = document_from_node(self);
            doc.r().request_focus(ElementCast::from_ref(self));
        } else if &*event.Type() == "keydown" && !event.DefaultPrevented() {
            let keyevent: Option<&KeyboardEvent> = KeyboardEventCast::to_ref(event);
            keyevent.map(|kevent| {
                match self.textinput.borrow_mut().handle_keydown(kevent) {
                    KeyReaction::TriggerDefaultAction => (),
                    KeyReaction::DispatchInput => {
                        self.value_changed.set(true);

                        if event.IsTrusted() {
                            let window = window_from_node(self);
                            let window = window.r();
                            let chan = window.script_chan();
                            let handler = Trusted::new(window.get_cx(), self, chan.clone());
                            let dispatcher = ChangeEventRunnable {
                                element: handler,
                            };
                            let _ = chan.send(CommonScriptMsg::RunnableMsg(InputEvent, box dispatcher));
                        }

                        self.force_relayout();
                    }
                    KeyReaction::Nothing => (),
                }
            });
        }
    }
}

impl<'a> FormControl<'a> for &'a HTMLTextAreaElement {
    fn to_element(self) -> &'a Element {
        ElementCast::from_ref(self)
    }
}

pub struct ChangeEventRunnable {
    element: Trusted<HTMLTextAreaElement>,
}

impl Runnable for ChangeEventRunnable {
    fn handler(self: Box<ChangeEventRunnable>) {
        let target = self.element.root();
        target.r().dispatch_change_event();
    }
}
