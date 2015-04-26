/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrValue};
use dom::attr::AttrHelpers;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::HTMLTextAreaElementBinding;
use dom::bindings::codegen::Bindings::HTMLTextAreaElementBinding::HTMLTextAreaElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, EventTargetCast, HTMLElementCast, NodeCast};
use dom::bindings::codegen::InheritTypes::{HTMLTextAreaElementDerived, HTMLFieldSetElementDerived};
use dom::bindings::codegen::InheritTypes::{KeyboardEventCast, TextDerived};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, LayoutJS, Temporary, OptionalRootable};
use dom::bindings::refcounted::Trusted;
use dom::document::{Document, DocumentHelpers};
use dom::element::{Element, AttributeHandlers};
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::eventtarget::{EventTarget, EventTargetHelpers, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::htmlformelement::FormControl;
use dom::keyboardevent::KeyboardEvent;
use dom::node::{DisabledStateHelpers, Node, NodeHelpers, NodeDamage, NodeTypeId};
use dom::node::{document_from_node, window_from_node};
use textinput::{TextInput, Lines, KeyReaction};
use dom::virtualmethods::VirtualMethods;
use dom::window::WindowHelpers;
use script_task::{ScriptMsg, Runnable};

use util::str::DOMString;
use string_cache::Atom;

use std::borrow::ToOwned;
use std::cell::Cell;

#[dom_struct]
pub struct HTMLTextAreaElement {
    htmlelement: HTMLElement,
    textinput: DOMRefCell<TextInput>,
    cols: Cell<u32>,
    rows: Cell<u32>,
    // https://html.spec.whatwg.org/multipage/#concept-textarea-dirty
    value_changed: Cell<bool>,
}

impl HTMLTextAreaElementDerived for EventTarget {
    fn is_htmltextareaelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)))
    }
}

pub trait LayoutHTMLTextAreaElementHelpers {
    #[allow(unsafe_code)]
    unsafe fn get_value_for_layout(self) -> String;
}

pub trait RawLayoutHTMLTextAreaElementHelpers {
    #[allow(unsafe_code)]
    unsafe fn get_cols_for_layout(&self) -> u32;
    #[allow(unsafe_code)]
    unsafe fn get_rows_for_layout(&self) -> u32;
}

impl LayoutHTMLTextAreaElementHelpers for LayoutJS<HTMLTextAreaElement> {
    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn get_value_for_layout(self) -> String {
        (*self.unsafe_get()).textinput.borrow_for_layout().get_content()
    }
}

impl RawLayoutHTMLTextAreaElementHelpers for HTMLTextAreaElement {
    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn get_cols_for_layout(&self) -> u32 {
        self.cols.get()
    }

    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn get_rows_for_layout(&self) -> u32 {
        self.rows.get()
    }
}

static DEFAULT_COLS: u32 = 20;
static DEFAULT_ROWS: u32 = 2;

impl HTMLTextAreaElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLTextAreaElement {
        let chan = document.window().root().r().constellation_chan();
        HTMLTextAreaElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLTextAreaElement, localName, prefix, document),
            textinput: DOMRefCell::new(TextInput::new(Lines::Multiple, "".to_owned(), Some(chan))),
            cols: Cell::new(DEFAULT_COLS),
            rows: Cell::new(DEFAULT_ROWS),
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

    // https://html.spec.whatwg.org/multipage/#dom-textarea-cols
    make_uint_getter!(Cols);

    // https://html.spec.whatwg.org/multipage/#dom-textarea-cols
    make_uint_setter!(SetCols, "cols");

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
    make_uint_getter!(Rows);

    // https://html.spec.whatwg.org/multipage/#dom-textarea-rows
    make_uint_setter!(SetRows, "rows");

    // https://html.spec.whatwg.org/multipage/#dom-textarea-wrap
    make_getter!(Wrap);

    // https://html.spec.whatwg.org/multipage/#dom-textarea-wrap
    make_setter!(SetWrap, "wrap");

    // https://html.spec.whatwg.org/multipage/#dom-textarea-type
    fn Type(self) -> DOMString {
        "textarea".to_owned()
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea-defaultvalue
    fn DefaultValue(self) -> DOMString {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.GetTextContent().unwrap()
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea-defaultvalue
    fn SetDefaultValue(self, value: DOMString) {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.SetTextContent(Some(value));

        // if the element's dirty value flag is false, then the element's
        // raw value must be set to the value of the element's textContent IDL attribute
        if !self.value_changed.get() {
            self.reset();
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea-value
    fn Value(self) -> DOMString {
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let textinput = self.textinput.borrow();
        textinput.get_content()
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea-value
    fn SetValue(self, value: DOMString) {
        // TODO move the cursor to the end of the field
        self.textinput.borrow_mut().set_content(value);
        self.value_changed.set(true);

        self.force_relayout();
    }
}

pub trait HTMLTextAreaElementHelpers {
    fn mutable(self) -> bool;
    fn reset(self);
}

impl<'a> HTMLTextAreaElementHelpers for JSRef<'a, HTMLTextAreaElement> {
    // https://html.spec.whatwg.org/multipage/#concept-fe-mutable
    fn mutable(self) -> bool {
        // https://html.spec.whatwg.org/multipage/#the-textarea-element:concept-fe-mutable
        !(self.Disabled() || self.ReadOnly())
    }
    fn reset(self) {
        // https://html.spec.whatwg.org/multipage/#the-textarea-element:concept-form-reset-control
        self.SetValue(self.DefaultValue());
        self.value_changed.set(false);
    }
}

trait PrivateHTMLTextAreaElementHelpers {
    fn force_relayout(self);
    fn dispatch_change_event(self);
}

impl<'a> PrivateHTMLTextAreaElementHelpers for JSRef<'a, HTMLTextAreaElement> {
    fn force_relayout(self) {
        let doc = document_from_node(self).root();
        let node: JSRef<Node> = NodeCast::from_ref(self);
        doc.r().content_changed(node, NodeDamage::OtherNodeDamage)
    }

    fn dispatch_change_event(self) {
        let window = window_from_node(self).root();
        let window = window.r();
        let event = Event::new(GlobalRef::Window(window),
                               "input".to_owned(),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable).root();

        let target: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        target.dispatch_event(event.r());
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLTextAreaElement> {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: JSRef<Attr>) {
        if let Some(ref s) = self.super_type() {
            s.after_set_attr(attr);
        }

        match attr.local_name() {
            &atom!("disabled") => {
                let node: JSRef<Node> = NodeCast::from_ref(*self);
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

    fn before_remove_attr(&self, attr: JSRef<Attr>) {
        if let Some(ref s) = self.super_type() {
            s.before_remove_attr(attr);
        }

        match attr.local_name() {
            &atom!("disabled") => {
                let node: JSRef<Node> = NodeCast::from_ref(*self);
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
        if let Some(ref s) = self.super_type() {
            s.unbind_from_tree(tree_in_doc);
        }

        let node: JSRef<Node> = NodeCast::from_ref(*self);
        if node.ancestors().any(|ancestor| ancestor.root().r().is_htmlfieldsetelement()) {
            node.check_ancestors_disabled_state_for_form_control();
        } else {
            node.check_disabled_attribute();
        }
    }

    fn child_inserted(&self, child: JSRef<Node>) {
        if let Some(s) = self.super_type() {
            s.child_inserted(child);
        }

        if child.is_text() && !self.value_changed.get() {
            self.reset();
        }
    }

    // copied and modified from htmlinputelement.rs
    fn handle_event(&self, event: JSRef<Event>) {
        if let Some(s) = self.super_type() {
            s.handle_event(event);
        }

        if &*event.Type() == "click" && !event.DefaultPrevented() {
            //TODO: set the editing position for text inputs

            let doc = document_from_node(*self).root();
            doc.r().request_focus(ElementCast::from_ref(*self));
        } else if &*event.Type() == "keydown" && !event.DefaultPrevented() {
            let keyevent: Option<JSRef<KeyboardEvent>> = KeyboardEventCast::to_ref(event);
            keyevent.map(|kevent| {
                match self.textinput.borrow_mut().handle_keydown(kevent) {
                    KeyReaction::TriggerDefaultAction => (),
                    KeyReaction::DispatchInput => {
                        self.value_changed.set(true);

                        if event.IsTrusted() {
                            let window = window_from_node(*self).root();
                            let window = window.r();
                            let chan = window.script_chan();
                            let handler = Trusted::new(window.get_cx(), *self , chan.clone());
                            let dispatcher = ChangeEventRunnable {
                                element: handler,
                            };
                            let _ = chan.send(ScriptMsg::RunnableMsg(box dispatcher));
                        }

                        self.force_relayout();
                    }
                    KeyReaction::Nothing => (),
                }
            });
        }
    }
}

impl<'a> FormControl<'a> for JSRef<'a, HTMLTextAreaElement> {
    fn to_element(self) -> JSRef<'a, Element> {
        ElementCast::from_ref(self)
    }
}

pub struct ChangeEventRunnable {
    element: Trusted<HTMLTextAreaElement>,
}

impl Runnable for ChangeEventRunnable {
    fn handler(self: Box<ChangeEventRunnable>) {
        let target = self.element.to_temporary().root();
        target.r().dispatch_change_event();
    }
}
