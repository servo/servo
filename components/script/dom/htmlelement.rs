/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;
use std::default::Default;
use std::rc::Rc;

use dom_struct::dom_struct;
use html5ever::{local_name, namespace_url, ns, LocalName, Prefix};
use js::rust::HandleObject;
use script_layout_interface::QueryMsg;
use style::attr::AttrValue;
use style_dom::ElementState;

use super::customelementregistry::CustomElementState;
use crate::dom::activation::Activatable;
use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterData_Binding::CharacterDataMethods;
use crate::dom::bindings::codegen::Bindings::EventHandlerBinding::{
    EventHandlerNonNull, OnErrorEventHandlerNonNull,
};
use crate::dom::bindings::codegen::Bindings::HTMLElementBinding::HTMLElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLLabelElementBinding::HTMLLabelElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::Node_Binding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::characterdata::CharacterData;
use crate::dom::cssstyledeclaration::{CSSModificationAccess, CSSStyleDeclaration, CSSStyleOwner};
use crate::dom::customelementregistry::CallbackReaction;
use crate::dom::document::{Document, FocusType};
use crate::dom::documentfragment::DocumentFragment;
use crate::dom::domstringmap::DOMStringMap;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::elementinternals::ElementInternals;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlbodyelement::HTMLBodyElement;
use crate::dom::htmlbrelement::HTMLBRElement;
use crate::dom::htmldetailselement::HTMLDetailsElement;
use crate::dom::htmlformelement::{FormControl, HTMLFormElement};
use crate::dom::htmlframesetelement::HTMLFrameSetElement;
use crate::dom::htmlhtmlelement::HTMLHtmlElement;
use crate::dom::htmlinputelement::{HTMLInputElement, InputType};
use crate::dom::htmllabelelement::HTMLLabelElement;
use crate::dom::htmltextareaelement::HTMLTextAreaElement;
use crate::dom::node::{BindContext, Node, NodeTraits, ShadowIncluding, UnbindContext};
use crate::dom::text::Text;
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;
use crate::script_thread::ScriptThread;

#[dom_struct]
pub(crate) struct HTMLElement {
    element: Element,
    style_decl: MutNullableDom<CSSStyleDeclaration>,
    dataset: MutNullableDom<DOMStringMap>,
}

impl HTMLElement {
    pub(crate) fn new_inherited(
        tag_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLElement {
        HTMLElement::new_inherited_with_state(ElementState::empty(), tag_name, prefix, document)
    }

    pub(crate) fn new_inherited_with_state(
        state: ElementState,
        tag_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLElement {
        HTMLElement {
            element: Element::new_inherited_with_state(
                state,
                tag_name,
                ns!(html),
                prefix,
                document,
            ),
            style_decl: Default::default(),
            dataset: Default::default(),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLElement::new_inherited(local_name, prefix, document)),
            document,
            proto,
            can_gc,
        )
    }

    fn is_body_or_frameset(&self) -> bool {
        let eventtarget = self.upcast::<EventTarget>();
        eventtarget.is::<HTMLBodyElement>() || eventtarget.is::<HTMLFrameSetElement>()
    }

    /// Calls into the layout engine to generate a plain text representation
    /// of a [`HTMLElement`] as specified when getting the `.innerText` or
    /// `.outerText` in JavaScript.`
    ///
    /// <https://html.spec.whatwg.org/multipage/#get-the-text-steps>
    fn get_inner_outer_text(&self, can_gc: CanGc) -> DOMString {
        let node = self.upcast::<Node>();
        let window = node.owner_window();
        let element = self.as_element();

        // Step 1.
        let element_not_rendered = !node.is_connected() || !element.has_css_layout_box(can_gc);
        if element_not_rendered {
            return node.GetTextContent().unwrap();
        }

        window.layout_reflow(QueryMsg::ElementInnerOuterTextQuery, can_gc);
        let text = window
            .layout()
            .query_element_inner_outer_text(node.to_trusted_node_address());

        DOMString::from(text)
    }
}

impl HTMLElementMethods<crate::DomTypeHolder> for HTMLElement {
    // https://html.spec.whatwg.org/multipage/#the-style-attribute
    fn Style(&self) -> DomRoot<CSSStyleDeclaration> {
        self.style_decl.or_init(|| {
            let global = self.owner_window();
            CSSStyleDeclaration::new(
                &global,
                CSSStyleOwner::Element(Dom::from_ref(self.upcast())),
                None,
                CSSModificationAccess::ReadWrite,
            )
        })
    }

    // https://html.spec.whatwg.org/multipage/#attr-title
    make_getter!(Title, "title");
    // https://html.spec.whatwg.org/multipage/#attr-title
    make_setter!(SetTitle, "title");

    // https://html.spec.whatwg.org/multipage/#attr-lang
    make_getter!(Lang, "lang");
    // https://html.spec.whatwg.org/multipage/#attr-lang
    make_setter!(SetLang, "lang");

    // https://html.spec.whatwg.org/multipage/#the-dir-attribute
    make_enumerated_getter!(
        Dir,
        "dir",
        "ltr" | "rtl" | "auto",
        missing => "",
        invalid => ""
    );

    // https://html.spec.whatwg.org/multipage/#the-dir-attribute
    make_setter!(SetDir, "dir");

    // https://html.spec.whatwg.org/multipage/#dom-hidden
    make_bool_getter!(Hidden, "hidden");
    // https://html.spec.whatwg.org/multipage/#dom-hidden
    make_bool_setter!(SetHidden, "hidden");

    // https://html.spec.whatwg.org/multipage/#globaleventhandlers
    global_event_handlers!(NoOnload);

    // https://html.spec.whatwg.org/multipage/#documentandelementeventhandlers
    document_and_element_event_handlers!();

    // https://html.spec.whatwg.org/multipage/#dom-dataset
    fn Dataset(&self) -> DomRoot<DOMStringMap> {
        self.dataset.or_init(|| DOMStringMap::new(self))
    }

    // https://html.spec.whatwg.org/multipage/#handler-onerror
    fn GetOnerror(&self, can_gc: CanGc) -> Option<Rc<OnErrorEventHandlerNonNull>> {
        if self.is_body_or_frameset() {
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().GetOnerror()
            } else {
                None
            }
        } else {
            self.upcast::<EventTarget>()
                .get_event_handler_common("error", can_gc)
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-onerror
    fn SetOnerror(&self, listener: Option<Rc<OnErrorEventHandlerNonNull>>) {
        if self.is_body_or_frameset() {
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().SetOnerror(listener)
            }
        } else {
            // special setter for error
            self.upcast::<EventTarget>()
                .set_error_event_handler("error", listener)
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-onload
    fn GetOnload(&self, can_gc: CanGc) -> Option<Rc<EventHandlerNonNull>> {
        if self.is_body_or_frameset() {
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().GetOnload()
            } else {
                None
            }
        } else {
            self.upcast::<EventTarget>()
                .get_event_handler_common("load", can_gc)
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-onload
    fn SetOnload(&self, listener: Option<Rc<EventHandlerNonNull>>) {
        if self.is_body_or_frameset() {
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().SetOnload(listener)
            }
        } else {
            self.upcast::<EventTarget>()
                .set_event_handler_common("load", listener)
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-onblur
    fn GetOnblur(&self, can_gc: CanGc) -> Option<Rc<EventHandlerNonNull>> {
        if self.is_body_or_frameset() {
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().GetOnblur()
            } else {
                None
            }
        } else {
            self.upcast::<EventTarget>()
                .get_event_handler_common("blur", can_gc)
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-onblur
    fn SetOnblur(&self, listener: Option<Rc<EventHandlerNonNull>>) {
        if self.is_body_or_frameset() {
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().SetOnblur(listener)
            }
        } else {
            self.upcast::<EventTarget>()
                .set_event_handler_common("blur", listener)
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-onfocus
    fn GetOnfocus(&self, can_gc: CanGc) -> Option<Rc<EventHandlerNonNull>> {
        if self.is_body_or_frameset() {
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().GetOnfocus()
            } else {
                None
            }
        } else {
            self.upcast::<EventTarget>()
                .get_event_handler_common("focus", can_gc)
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-onfocus
    fn SetOnfocus(&self, listener: Option<Rc<EventHandlerNonNull>>) {
        if self.is_body_or_frameset() {
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().SetOnfocus(listener)
            }
        } else {
            self.upcast::<EventTarget>()
                .set_event_handler_common("focus", listener)
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-onresize
    fn GetOnresize(&self, can_gc: CanGc) -> Option<Rc<EventHandlerNonNull>> {
        if self.is_body_or_frameset() {
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().GetOnresize()
            } else {
                None
            }
        } else {
            self.upcast::<EventTarget>()
                .get_event_handler_common("resize", can_gc)
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-onresize
    fn SetOnresize(&self, listener: Option<Rc<EventHandlerNonNull>>) {
        if self.is_body_or_frameset() {
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().SetOnresize(listener)
            }
        } else {
            self.upcast::<EventTarget>()
                .set_event_handler_common("resize", listener)
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-onscroll
    fn GetOnscroll(&self, can_gc: CanGc) -> Option<Rc<EventHandlerNonNull>> {
        if self.is_body_or_frameset() {
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().GetOnscroll()
            } else {
                None
            }
        } else {
            self.upcast::<EventTarget>()
                .get_event_handler_common("scroll", can_gc)
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-onscroll
    fn SetOnscroll(&self, listener: Option<Rc<EventHandlerNonNull>>) {
        if self.is_body_or_frameset() {
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().SetOnscroll(listener)
            }
        } else {
            self.upcast::<EventTarget>()
                .set_event_handler_common("scroll", listener)
        }
    }

    // https://html.spec.whatwg.org/multipage/#attr-itemtype
    fn Itemtypes(&self) -> Option<Vec<DOMString>> {
        let atoms = self
            .element
            .get_tokenlist_attribute(&local_name!("itemtype"));

        if atoms.is_empty() {
            return None;
        }

        let mut item_attr_values = HashSet::new();
        for attr_value in &atoms {
            item_attr_values.insert(DOMString::from(String::from(attr_value.trim())));
        }

        Some(item_attr_values.into_iter().collect())
    }

    // https://html.spec.whatwg.org/multipage/#names:-the-itemprop-attribute
    fn PropertyNames(&self) -> Option<Vec<DOMString>> {
        let atoms = self
            .element
            .get_tokenlist_attribute(&local_name!("itemprop"));

        if atoms.is_empty() {
            return None;
        }

        let mut item_attr_values = HashSet::new();
        for attr_value in &atoms {
            item_attr_values.insert(DOMString::from(String::from(attr_value.trim())));
        }

        Some(item_attr_values.into_iter().collect())
    }

    // https://html.spec.whatwg.org/multipage/#dom-click
    fn Click(&self, can_gc: CanGc) {
        let element = self.as_element();
        if element.disabled_state() {
            return;
        }
        if element.click_in_progress() {
            return;
        }
        element.set_click_in_progress(true);

        self.upcast::<Node>()
            .fire_synthetic_mouse_event_not_trusted(DOMString::from("click"), can_gc);
        element.set_click_in_progress(false);
    }

    // https://html.spec.whatwg.org/multipage/#dom-focus
    fn Focus(&self, can_gc: CanGc) {
        // TODO: Mark the element as locked for focus and run the focusing steps.
        // https://html.spec.whatwg.org/multipage/#focusing-steps
        let document = self.owner_document();
        document.request_focus(Some(self.upcast()), FocusType::Element, can_gc);
    }

    // https://html.spec.whatwg.org/multipage/#dom-blur
    fn Blur(&self, can_gc: CanGc) {
        // TODO: Run the unfocusing steps.
        if !self.as_element().focus_state() {
            return;
        }
        // https://html.spec.whatwg.org/multipage/#unfocusing-steps
        let document = self.owner_document();
        document.request_focus(None, FocusType::Element, can_gc);
    }

    // https://drafts.csswg.org/cssom-view/#dom-htmlelement-offsetparent
    fn GetOffsetParent(&self, can_gc: CanGc) -> Option<DomRoot<Element>> {
        if self.is::<HTMLBodyElement>() || self.is::<HTMLHtmlElement>() {
            return None;
        }

        let node = self.upcast::<Node>();
        let window = self.owner_window();
        let (element, _) = window.offset_parent_query(node, can_gc);

        element
    }

    // https://drafts.csswg.org/cssom-view/#dom-htmlelement-offsettop
    fn OffsetTop(&self, can_gc: CanGc) -> i32 {
        if self.is::<HTMLBodyElement>() {
            return 0;
        }

        let node = self.upcast::<Node>();
        let window = self.owner_window();
        let (_, rect) = window.offset_parent_query(node, can_gc);

        rect.origin.y.to_nearest_px()
    }

    // https://drafts.csswg.org/cssom-view/#dom-htmlelement-offsetleft
    fn OffsetLeft(&self, can_gc: CanGc) -> i32 {
        if self.is::<HTMLBodyElement>() {
            return 0;
        }

        let node = self.upcast::<Node>();
        let window = self.owner_window();
        let (_, rect) = window.offset_parent_query(node, can_gc);

        rect.origin.x.to_nearest_px()
    }

    // https://drafts.csswg.org/cssom-view/#dom-htmlelement-offsetwidth
    fn OffsetWidth(&self, can_gc: CanGc) -> i32 {
        let node = self.upcast::<Node>();
        let window = self.owner_window();
        let (_, rect) = window.offset_parent_query(node, can_gc);

        rect.size.width.to_nearest_px()
    }

    // https://drafts.csswg.org/cssom-view/#dom-htmlelement-offsetheight
    fn OffsetHeight(&self, can_gc: CanGc) -> i32 {
        let node = self.upcast::<Node>();
        let window = self.owner_window();
        let (_, rect) = window.offset_parent_query(node, can_gc);

        rect.size.height.to_nearest_px()
    }

    /// <https://html.spec.whatwg.org/multipage/#the-innertext-idl-attribute>
    fn InnerText(&self, can_gc: CanGc) -> DOMString {
        self.get_inner_outer_text(can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#set-the-inner-text-steps>
    fn SetInnerText(&self, input: DOMString, can_gc: CanGc) {
        // Step 1: Let fragment be the rendered text fragment for value given element's node
        // document.
        let fragment = self.rendered_text_fragment(input, can_gc);

        // Step 2: Replace all with fragment within element.
        Node::replace_all(Some(fragment.upcast()), self.upcast::<Node>());
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-outertext>
    fn GetOuterText(&self, can_gc: CanGc) -> Fallible<DOMString> {
        Ok(self.get_inner_outer_text(can_gc))
    }

    /// <https://html.spec.whatwg.org/multipage/#the-innertext-idl-attribute:dom-outertext-2>
    fn SetOuterText(&self, input: DOMString, can_gc: CanGc) -> Fallible<()> {
        // Step 1: If this's parent is null, then throw a "NoModificationAllowedError" DOMException.
        let Some(parent) = self.upcast::<Node>().GetParentNode() else {
            return Err(Error::NoModificationAllowed);
        };

        let node = self.upcast::<Node>();
        let document = self.owner_document();

        // Step 2: Let next be this's next sibling.
        let next = node.GetNextSibling();

        // Step 3: Let previous be this's previous sibling.
        let previous = node.GetPreviousSibling();

        // Step 4: Let fragment be the rendered text fragment for the given value given this's node
        // document.
        let fragment = self.rendered_text_fragment(input, can_gc);

        // Step 5: If fragment has no children, then append a new Text node whose data is the empty
        // string and node document is this's node document to fragment.
        if fragment.upcast::<Node>().children_count() == 0 {
            let text_node = Text::new(DOMString::from("".to_owned()), &document, can_gc);

            fragment.upcast::<Node>().AppendChild(text_node.upcast())?;
        }

        // Step 6: Replace this with fragment within this's parent.
        parent.ReplaceChild(fragment.upcast(), node)?;

        // Step 7: If next is non-null and next's previous sibling is a Text node, then merge with
        // the next text node given next's previous sibling.
        if let Some(next_sibling) = next {
            if let Some(node) = next_sibling.GetPreviousSibling() {
                Self::merge_with_the_next_text_node(node);
            }
        }

        // Step 8: If previous is a Text node, then merge with the next text node given previous.
        if let Some(previous) = previous {
            Self::merge_with_the_next_text_node(previous)
        }

        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-translate
    fn Translate(&self) -> bool {
        self.as_element().is_translate_enabled()
    }

    // https://html.spec.whatwg.org/multipage/#dom-translate
    fn SetTranslate(&self, yesno: bool, can_gc: CanGc) {
        self.as_element().set_string_attribute(
            &html5ever::local_name!("translate"),
            match yesno {
                true => DOMString::from("yes"),
                false => DOMString::from("no"),
            },
            can_gc,
        );
    }

    // https://html.spec.whatwg.org/multipage/#dom-contenteditable
    fn ContentEditable(&self) -> DOMString {
        // TODO: https://github.com/servo/servo/issues/12776
        self.as_element()
            .get_attribute(&ns!(), &local_name!("contenteditable"))
            .map(|attr| DOMString::from(&**attr.value()))
            .unwrap_or_else(|| DOMString::from("inherit"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-contenteditable
    fn SetContentEditable(&self, _: DOMString) {
        // TODO: https://github.com/servo/servo/issues/12776
        warn!("The contentEditable attribute is not implemented yet");
    }

    // https://html.spec.whatwg.org/multipage/#dom-contenteditable
    fn IsContentEditable(&self) -> bool {
        // TODO: https://github.com/servo/servo/issues/12776
        false
    }
    /// <https://html.spec.whatwg.org/multipage#dom-attachinternals>
    fn AttachInternals(&self) -> Fallible<DomRoot<ElementInternals>> {
        let element = self.as_element();
        // Step 1: If this's is value is not null, then throw a "NotSupportedError" DOMException
        if element.get_is().is_some() {
            return Err(Error::NotSupported);
        }

        // Step 2: Let definition be the result of looking up a custom element definition
        // Note: the element can pass this check without yet being a custom
        // element, as long as there is a registered definition
        // that could upgrade it to one later.
        let registry = self.owner_document().window().CustomElements();
        let definition = registry.lookup_definition(self.as_element().local_name(), None);

        // Step 3: If definition is null, then throw an "NotSupportedError" DOMException
        let definition = match definition {
            Some(definition) => definition,
            None => return Err(Error::NotSupported),
        };

        // Step 4: If definition's disable internals is true, then throw a "NotSupportedError" DOMException
        if definition.disable_internals {
            return Err(Error::NotSupported);
        }

        // Step 5: If this's attached internals is non-null, then throw an "NotSupportedError" DOMException
        let internals = element.ensure_element_internals();
        if internals.attached() {
            return Err(Error::NotSupported);
        }

        // Step 6: If this's custom element state is not "precustomized" or "custom",
        // then throw a "NotSupportedError" DOMException.
        if !matches!(
            element.get_custom_element_state(),
            CustomElementState::Precustomized | CustomElementState::Custom
        ) {
            return Err(Error::NotSupported);
        }

        if self.is_form_associated_custom_element() {
            element.init_state_for_internals();
        }

        // Step 6-7: Set this's attached internals to a new ElementInternals instance
        internals.set_attached();
        Ok(internals)
    }

    // https://html.spec.whatwg.org/multipage/#dom-fe-autofocus
    fn Autofocus(&self) -> bool {
        self.element.has_attribute(&local_name!("autofocus"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-fe-autofocus
    fn SetAutofocus(&self, autofocus: bool, can_gc: CanGc) {
        self.element
            .set_bool_attribute(&local_name!("autofocus"), autofocus, can_gc);
    }
}

fn append_text_node_to_fragment(
    document: &Document,
    fragment: &DocumentFragment,
    text: String,
    can_gc: CanGc,
) {
    let text = Text::new(DOMString::from(text), document, can_gc);
    fragment
        .upcast::<Node>()
        .AppendChild(text.upcast())
        .unwrap();
}

// https://html.spec.whatwg.org/multipage/#attr-data-*

static DATA_PREFIX: &str = "data-";
static DATA_HYPHEN_SEPARATOR: char = '\x2d';

fn to_snake_case(name: DOMString) -> DOMString {
    let mut attr_name = String::with_capacity(name.len() + DATA_PREFIX.len());
    attr_name.push_str(DATA_PREFIX);
    for ch in name.chars() {
        if ch.is_ascii_uppercase() {
            attr_name.push(DATA_HYPHEN_SEPARATOR);
            attr_name.push(ch.to_ascii_lowercase());
        } else {
            attr_name.push(ch);
        }
    }
    DOMString::from(attr_name)
}

// https://html.spec.whatwg.org/multipage/#attr-data-*
// if this attribute is in snake case with a data- prefix,
// this function returns a name converted to camel case
// without the data prefix.

fn to_camel_case(name: &str) -> Option<DOMString> {
    if !name.starts_with(DATA_PREFIX) {
        return None;
    }
    let name = &name[5..];
    let has_uppercase = name.chars().any(|curr_char| curr_char.is_ascii_uppercase());
    if has_uppercase {
        return None;
    }
    let mut result = String::with_capacity(name.len().saturating_sub(DATA_PREFIX.len()));
    let mut name_chars = name.chars();
    while let Some(curr_char) = name_chars.next() {
        //check for hyphen followed by character
        if curr_char == DATA_HYPHEN_SEPARATOR {
            if let Some(next_char) = name_chars.next() {
                if next_char.is_ascii_lowercase() {
                    result.push(next_char.to_ascii_uppercase());
                } else {
                    result.push(curr_char);
                    result.push(next_char);
                }
            } else {
                result.push(curr_char);
            }
        } else {
            result.push(curr_char);
        }
    }
    Some(DOMString::from(result))
}

impl HTMLElement {
    pub(crate) fn set_custom_attr(
        &self,
        name: DOMString,
        value: DOMString,
        can_gc: CanGc,
    ) -> ErrorResult {
        if name
            .chars()
            .skip_while(|&ch| ch != '\u{2d}')
            .nth(1)
            .is_some_and(|ch| ch.is_ascii_lowercase())
        {
            return Err(Error::Syntax);
        }
        self.as_element()
            .set_custom_attribute(to_snake_case(name), value, can_gc)
    }

    pub(crate) fn get_custom_attr(&self, local_name: DOMString) -> Option<DOMString> {
        // FIXME(ajeffrey): Convert directly from DOMString to LocalName
        let local_name = LocalName::from(to_snake_case(local_name));
        self.as_element()
            .get_attribute(&ns!(), &local_name)
            .map(|attr| {
                DOMString::from(&**attr.value()) // FIXME(ajeffrey): Convert directly from AttrValue to DOMString
            })
    }

    pub(crate) fn delete_custom_attr(&self, local_name: DOMString) {
        // FIXME(ajeffrey): Convert directly from DOMString to LocalName
        let local_name = LocalName::from(to_snake_case(local_name));
        self.as_element().remove_attribute(&ns!(), &local_name);
    }

    /// <https://html.spec.whatwg.org/multipage/#category-label>
    pub(crate) fn is_labelable_element(&self) -> bool {
        match self.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(type_id)) => match type_id {
                HTMLElementTypeId::HTMLInputElement => {
                    self.downcast::<HTMLInputElement>().unwrap().input_type() != InputType::Hidden
                },
                HTMLElementTypeId::HTMLButtonElement |
                HTMLElementTypeId::HTMLMeterElement |
                HTMLElementTypeId::HTMLOutputElement |
                HTMLElementTypeId::HTMLProgressElement |
                HTMLElementTypeId::HTMLSelectElement |
                HTMLElementTypeId::HTMLTextAreaElement => true,
                _ => self.is_form_associated_custom_element(),
            },
            _ => false,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#form-associated-custom-element>
    pub(crate) fn is_form_associated_custom_element(&self) -> bool {
        if let Some(definition) = self.as_element().get_custom_element_definition() {
            definition.is_autonomous() && definition.form_associated
        } else {
            false
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#category-listed>
    pub(crate) fn is_listed_element(&self) -> bool {
        match self.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(type_id)) => match type_id {
                HTMLElementTypeId::HTMLButtonElement |
                HTMLElementTypeId::HTMLFieldSetElement |
                HTMLElementTypeId::HTMLInputElement |
                HTMLElementTypeId::HTMLObjectElement |
                HTMLElementTypeId::HTMLOutputElement |
                HTMLElementTypeId::HTMLSelectElement |
                HTMLElementTypeId::HTMLTextAreaElement => true,
                _ => self.is_form_associated_custom_element(),
            },
            _ => false,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#category-submit>
    pub(crate) fn is_submittable_element(&self) -> bool {
        match self.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(type_id)) => match type_id {
                HTMLElementTypeId::HTMLButtonElement |
                HTMLElementTypeId::HTMLInputElement |
                HTMLElementTypeId::HTMLSelectElement |
                HTMLElementTypeId::HTMLTextAreaElement => true,
                _ => self.is_form_associated_custom_element(),
            },
            _ => false,
        }
    }

    pub(crate) fn supported_prop_names_custom_attr(&self) -> Vec<DOMString> {
        let element = self.as_element();
        element
            .attrs()
            .iter()
            .filter_map(|attr| {
                let raw_name = attr.local_name();
                to_camel_case(raw_name)
            })
            .collect()
    }

    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    // This gets the nth label in tree order.
    pub(crate) fn label_at(&self, index: u32) -> Option<DomRoot<Node>> {
        let element = self.as_element();

        // Traverse entire tree for <label> elements that have
        // this as their control.
        // There is room for performance optimization, as we don't need
        // the actual result of GetControl, only whether the result
        // would match self.
        // (Even more room for performance optimization: do what
        // nodelist ChildrenList does and keep a mutation-aware cursor
        // around; this may be hard since labels need to keep working
        // even as they get detached into a subtree and reattached to
        // a document.)
        let root_element = element.root_element();
        let root_node = root_element.upcast::<Node>();
        root_node
            .traverse_preorder(ShadowIncluding::No)
            .filter_map(DomRoot::downcast::<HTMLLabelElement>)
            .filter(|elem| match elem.GetControl() {
                Some(control) => &*control == self,
                _ => false,
            })
            .nth(index as usize)
            .map(|n| DomRoot::from_ref(n.upcast::<Node>()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    // This counts the labels of the element, to support NodeList::Length
    pub(crate) fn labels_count(&self) -> u32 {
        // see label_at comments about performance
        let element = self.as_element();
        let root_element = element.root_element();
        let root_node = root_element.upcast::<Node>();
        root_node
            .traverse_preorder(ShadowIncluding::No)
            .filter_map(DomRoot::downcast::<HTMLLabelElement>)
            .filter(|elem| match elem.GetControl() {
                Some(control) => &*control == self,
                _ => false,
            })
            .count() as u32
    }

    // https://html.spec.whatwg.org/multipage/#the-directionality.
    // returns Some if can infer direction by itself or from child nodes
    // returns None if requires to go up to parent
    pub(crate) fn directionality(&self) -> Option<String> {
        let element_direction: &str = &self.Dir();

        if element_direction == "ltr" {
            return Some("ltr".to_owned());
        }

        if element_direction == "rtl" {
            return Some("rtl".to_owned());
        }

        if let Some(input) = self.downcast::<HTMLInputElement>() {
            if input.input_type() == InputType::Tel {
                return Some("ltr".to_owned());
            }
        }

        if element_direction == "auto" {
            if let Some(directionality) = self
                .downcast::<HTMLInputElement>()
                .and_then(|input| input.auto_directionality())
            {
                return Some(directionality);
            }

            if let Some(area) = self.downcast::<HTMLTextAreaElement>() {
                return Some(area.auto_directionality());
            }
        }

        // TODO(NeverHappened): Implement condition
        // If the element's dir attribute is in the auto state OR
        // If the element is a bdi element and the dir attribute is not in a defined state
        // (i.e. it is not present or has an invalid value)
        // Requires bdi element implementation (https://html.spec.whatwg.org/multipage/#the-bdi-element)

        None
    }

    // https://html.spec.whatwg.org/multipage/#the-summary-element:activation-behaviour
    pub(crate) fn summary_activation_behavior(&self) {
        // Step 1
        if !self.is_summary_for_its_parent_details() {
            return;
        }

        // Step 2
        let parent_details = self.upcast::<Node>().GetParentNode().unwrap();

        // Step 3
        parent_details
            .downcast::<HTMLDetailsElement>()
            .unwrap()
            .toggle();
    }

    // https://html.spec.whatwg.org/multipage/#summary-for-its-parent-details
    fn is_summary_for_its_parent_details(&self) -> bool {
        // Step 1
        let summary_node = self.upcast::<Node>();
        if !summary_node.has_parent() {
            return false;
        }

        // Step 2
        let parent = &summary_node.GetParentNode().unwrap();

        // Step 3
        if !parent.is::<HTMLDetailsElement>() {
            return false;
        }

        // Step 4 & 5
        let first_summary_element = parent
            .child_elements()
            .find(|el| el.local_name() == &local_name!("summary"));
        match first_summary_element {
            Some(first_summary) => &*first_summary == self.as_element(),
            None => false,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#rendered-text-fragment>
    fn rendered_text_fragment(&self, input: DOMString, can_gc: CanGc) -> DomRoot<DocumentFragment> {
        // Step 1: Let fragment be a new DocumentFragment whose node document is document.
        let document = self.owner_document();
        let fragment = DocumentFragment::new(&document, can_gc);

        // Step 2: Let position be a position variable for input, initially pointing at the start
        // of input.
        let mut position = input.chars().peekable();

        // Step 3: Let text be the empty string.
        let mut text = String::new();

        // Step 4
        while let Some(ch) = position.next() {
            match ch {
                // While position is not past the end of input, and the code point at position is
                // either U+000A LF or U+000D CR:
                '\u{000A}' | '\u{000D}' => {
                    if ch == '\u{000D}' && position.peek() == Some(&'\u{000A}') {
                        // a \r\n pair should only generate one <br>,
                        // so just skip the \r.
                        position.next();
                    }

                    if !text.is_empty() {
                        append_text_node_to_fragment(&document, &fragment, text, can_gc);
                        text = String::new();
                    }

                    let br = HTMLBRElement::new(local_name!("br"), None, &document, None, can_gc);
                    fragment.upcast::<Node>().AppendChild(br.upcast()).unwrap();
                },
                _ => {
                    // Collect a sequence of code points that are not U+000A LF or U+000D CR from
                    // input given position, and set text to the result.
                    text.push(ch);
                },
            }
        }

        // If text is not the empty string, then append a new Text node whose data is text and node
        // document is document to fragment.
        if !text.is_empty() {
            append_text_node_to_fragment(&document, &fragment, text, can_gc);
        }

        fragment
    }

    /// Checks whether a given [`DomRoot<Node>`] and its next sibling are
    /// of type [`Text`], and if so merges them into a single [`Text`]
    /// node.
    ///
    /// <https://html.spec.whatwg.org/multipage/#merge-with-the-next-text-node>
    fn merge_with_the_next_text_node(node: DomRoot<Node>) {
        // Make sure node is a Text node
        if !node.is::<Text>() {
            return;
        }

        // Step 1: Let next be node's next sibling.
        let next = match node.GetNextSibling() {
            Some(next) => next,
            None => return,
        };

        // Step 2: If next is not a Text node, then return.
        if !next.is::<Text>() {
            return;
        }
        // Step 3: Replace data with node, node's data's length, 0, and next's data.
        let node_chars = node.downcast::<CharacterData>().expect("Node is Text");
        let next_chars = next.downcast::<CharacterData>().expect("Next node is Text");
        node_chars
            .ReplaceData(node_chars.Length(), 0, next_chars.Data())
            .expect("Got chars from Text");

        // Step 4:Remove next.
        next.remove_self();
    }
}

impl VirtualMethods for HTMLElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.as_element() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        let element = self.as_element();
        match (attr.local_name(), mutation) {
            (name, AttributeMutation::Set(_)) if name.starts_with("on") => {
                let evtarget = self.upcast::<EventTarget>();
                let source_line = 1; //TODO(#9604) get current JS execution line
                evtarget.set_event_handler_uncompiled(
                    self.owner_window().get_url(),
                    source_line,
                    &name[2..],
                    // FIXME(ajeffrey): Convert directly from AttrValue to DOMString
                    DOMString::from(&**attr.value()),
                );
            },
            (&local_name!("form"), mutation) if self.is_form_associated_custom_element() => {
                self.form_attribute_mutated(mutation);
            },
            // Adding a "disabled" attribute disables an enabled form element.
            (&local_name!("disabled"), AttributeMutation::Set(_))
                if self.is_form_associated_custom_element() && element.enabled_state() =>
            {
                element.set_disabled_state(true);
                element.set_enabled_state(false);
                ScriptThread::enqueue_callback_reaction(
                    element,
                    CallbackReaction::FormDisabled(true),
                    None,
                );
            },
            // Removing the "disabled" attribute may enable a disabled
            // form element, but a fieldset ancestor may keep it disabled.
            (&local_name!("disabled"), AttributeMutation::Removed)
                if self.is_form_associated_custom_element() && element.disabled_state() =>
            {
                element.set_disabled_state(false);
                element.set_enabled_state(true);
                element.check_ancestors_disabled_state_for_form_control();
                if element.enabled_state() {
                    ScriptThread::enqueue_callback_reaction(
                        element,
                        CallbackReaction::FormDisabled(false),
                        None,
                    );
                }
            },
            (&local_name!("readonly"), mutation) if self.is_form_associated_custom_element() => {
                match mutation {
                    AttributeMutation::Set(_) => {
                        element.set_read_write_state(true);
                    },
                    AttributeMutation::Removed => {
                        element.set_read_write_state(false);
                    },
                }
            },
            _ => {},
        }
    }

    fn bind_to_tree(&self, context: &BindContext) {
        if let Some(super_type) = self.super_type() {
            super_type.bind_to_tree(context);
        }
        let element = self.as_element();
        element.update_sequentially_focusable_status(CanGc::note());

        // Binding to a tree can disable a form control if one of the new
        // ancestors is a fieldset.
        if self.is_form_associated_custom_element() && element.enabled_state() {
            element.check_ancestors_disabled_state_for_form_control();
            if element.disabled_state() {
                ScriptThread::enqueue_callback_reaction(
                    element,
                    CallbackReaction::FormDisabled(true),
                    None,
                );
            }
        }
    }

    fn unbind_from_tree(&self, context: &UnbindContext) {
        if let Some(super_type) = self.super_type() {
            super_type.unbind_from_tree(context);
        }

        // Unbinding from a tree might enable a form control, if a
        // fieldset ancestor is the only reason it was disabled.
        // (The fact that it's enabled doesn't do much while it's
        // disconnected, but it is an observable fact to keep track of.)
        let element = self.as_element();
        if self.is_form_associated_custom_element() && element.disabled_state() {
            element.check_disabled_attribute();
            element.check_ancestors_disabled_state_for_form_control();
            if element.enabled_state() {
                ScriptThread::enqueue_callback_reaction(
                    element,
                    CallbackReaction::FormDisabled(false),
                    None,
                );
            }
        }
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match *name {
            local_name!("itemprop") => AttrValue::from_serialized_tokenlist(value.into()),
            local_name!("itemtype") => AttrValue::from_serialized_tokenlist(value.into()),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }
}

impl Activatable for HTMLElement {
    fn as_element(&self) -> &Element {
        self.upcast::<Element>()
    }

    fn is_instance_activatable(&self) -> bool {
        self.as_element().local_name() == &local_name!("summary")
    }

    // Basically used to make the HTMLSummaryElement activatable (which has no IDL definition)
    fn activation_behavior(&self, _event: &Event, _target: &EventTarget, _can_gc: CanGc) {
        self.summary_activation_behavior();
    }
}
// Form-associated custom elements are the same interface type as
// normal HTMLElements, so HTMLElement needs to have the FormControl trait
// even though it's usually more specific trait implementations, like the
// HTMLInputElement one, that we really want. (Alternately we could put
// the FormControl trait on ElementInternals, but that raises lifetime issues.)
impl FormControl for HTMLElement {
    fn form_owner(&self) -> Option<DomRoot<HTMLFormElement>> {
        debug_assert!(self.is_form_associated_custom_element());
        self.as_element()
            .get_element_internals()
            .and_then(|e| e.form_owner())
    }

    fn set_form_owner(&self, form: Option<&HTMLFormElement>) {
        debug_assert!(self.is_form_associated_custom_element());
        self.as_element()
            .ensure_element_internals()
            .set_form_owner(form);
    }

    fn to_element(&self) -> &Element {
        debug_assert!(self.is_form_associated_custom_element());
        self.as_element()
    }

    fn is_listed(&self) -> bool {
        debug_assert!(self.is_form_associated_custom_element());
        true
    }

    // TODO candidate_for_validation, satisfies_constraints traits
}
