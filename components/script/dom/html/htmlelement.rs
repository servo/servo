/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;
use std::default::Default;
use std::rc::Rc;

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, QualName, local_name, ns};
use js::context::JSContext;
use js::rust::HandleObject;
use layout_api::{QueryMsg, ScrollContainerQueryFlags, ScrollContainerResponse};
use script_bindings::codegen::GenericBindings::DocumentBinding::DocumentMethods;
use style::attr::AttrValue;
use stylo_dom::ElementState;

use crate::dom::activation::Activatable;
use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterData_Binding::CharacterDataMethods;
use crate::dom::bindings::codegen::Bindings::EventHandlerBinding::{
    EventHandlerNonNull, OnErrorEventHandlerNonNull,
};
use crate::dom::bindings::codegen::Bindings::HTMLElementBinding::HTMLElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLLabelElementBinding::HTMLLabelElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLOrSVGElementBinding::FocusOptions;
use crate::dom::bindings::codegen::Bindings::NodeBinding::Node_Binding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::ShadowRoot_Binding::ShadowRootMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::characterdata::CharacterData;
use crate::dom::css::cssstyledeclaration::{
    CSSModificationAccess, CSSStyleDeclaration, CSSStyleOwner,
};
use crate::dom::customelementregistry::{CallbackReaction, CustomElementState};
use crate::dom::document::{Document, FocusInitiator};
use crate::dom::document_event_handler::character_to_code;
use crate::dom::documentfragment::DocumentFragment;
use crate::dom::domstringmap::DOMStringMap;
use crate::dom::element::{
    AttributeMutation, CustomElementCreationMode, Element, ElementCreator,
    is_element_affected_by_legacy_background_presentational_hint,
};
use crate::dom::elementinternals::ElementInternals;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::html::htmlbodyelement::HTMLBodyElement;
use crate::dom::html::htmldetailselement::HTMLDetailsElement;
use crate::dom::html::htmlformelement::{FormControl, HTMLFormElement};
use crate::dom::html::htmlframesetelement::HTMLFrameSetElement;
use crate::dom::html::htmlhtmlelement::HTMLHtmlElement;
use crate::dom::html::htmlinputelement::{HTMLInputElement, InputType};
use crate::dom::html::htmllabelelement::HTMLLabelElement;
use crate::dom::html::htmltextareaelement::HTMLTextAreaElement;
use crate::dom::htmlformelement::FormControlElementHelpers;
use crate::dom::medialist::MediaList;
use crate::dom::node::{
    BindContext, MoveContext, Node, NodeTraits, ShadowIncluding, UnbindContext,
    from_untrusted_node_address,
};
use crate::dom::shadowroot::ShadowRoot;
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
    pub(crate) fn get_inner_outer_text(&self) -> DOMString {
        let node = self.upcast::<Node>();
        let window = node.owner_window();
        let element = self.as_element();

        // Step 1.
        let element_not_rendered = !node.is_connected() || !element.has_css_layout_box();
        if element_not_rendered {
            return node.GetTextContent().unwrap();
        }

        window.layout_reflow(QueryMsg::ElementInnerOuterTextQuery);
        let text = window
            .layout()
            .query_element_inner_outer_text(node.to_trusted_node_address());

        DOMString::from(text)
    }

    /// <https://html.spec.whatwg.org/multipage/#set-the-inner-text-steps>
    pub(crate) fn set_inner_text(&self, cx: &mut JSContext, input: DOMString) {
        // Step 1: Let fragment be the rendered text fragment for value given element's node
        // document.
        let fragment = self.rendered_text_fragment(cx, input);

        // Step 2: Replace all with fragment within element.
        Node::replace_all(cx, Some(fragment.upcast()), self.upcast::<Node>());
    }

    /// <https://html.spec.whatwg.org/multipage/#matches-the-environment>
    pub(crate) fn media_attribute_matches_media_environment(&self) -> bool {
        // A string matches the environment of the user if it is the empty string,
        // a string consisting of only ASCII whitespace, or is a media query list that
        // matches the user's environment according to the definitions given in Media Queries. [MQ]
        self.upcast::<Element>()
            .get_attribute(&local_name!("media"))
            .is_none_or(|media| {
                MediaList::matches_environment(&self.owner_document(), &media.value())
            })
    }

    /// <https://html.spec.whatwg.org/multipage/#editing-host>
    pub(crate) fn is_editing_host(&self) -> bool {
        // > An editing host is either an HTML element with its contenteditable attribute in the true state or plaintext-only state,
        matches!(&*self.ContentEditable().str(), "true" | "plaintext-only")
        // > or a child HTML element of a Document whose design mode enabled is true.
        // TODO
    }
}

impl HTMLElementMethods<crate::DomTypeHolder> for HTMLElement {
    /// <https://html.spec.whatwg.org/multipage/#the-style-attribute>
    fn Style(&self, can_gc: CanGc) -> DomRoot<CSSStyleDeclaration> {
        self.style_decl.or_init(|| {
            let global = self.owner_window();
            CSSStyleDeclaration::new(
                &global,
                CSSStyleOwner::Element(Dom::from_ref(self.upcast())),
                None,
                CSSModificationAccess::ReadWrite,
                can_gc,
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

    /// <https://html.spec.whatwg.org/multipage/#dom-dataset>
    fn Dataset(&self, can_gc: CanGc) -> DomRoot<DOMStringMap> {
        self.dataset.or_init(|| DOMStringMap::new(self, can_gc))
    }

    /// <https://html.spec.whatwg.org/multipage/#handler-onerror>
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

    /// <https://html.spec.whatwg.org/multipage/#handler-onerror>
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

    /// <https://html.spec.whatwg.org/multipage/#handler-onload>
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

    /// <https://html.spec.whatwg.org/multipage/#handler-onload>
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

    /// <https://html.spec.whatwg.org/multipage/#handler-onblur>
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

    /// <https://html.spec.whatwg.org/multipage/#handler-onblur>
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

    /// <https://html.spec.whatwg.org/multipage/#handler-onfocus>
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

    /// <https://html.spec.whatwg.org/multipage/#handler-onfocus>
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

    /// <https://html.spec.whatwg.org/multipage/#handler-onresize>
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

    /// <https://html.spec.whatwg.org/multipage/#handler-onresize>
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

    /// <https://html.spec.whatwg.org/multipage/#handler-onscroll>
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

    /// <https://html.spec.whatwg.org/multipage/#handler-onscroll>
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

    /// <https://html.spec.whatwg.org/multipage/#attr-itemtype>
    fn Itemtypes(&self) -> Option<Vec<DOMString>> {
        let atoms = self
            .element
            .get_tokenlist_attribute(&local_name!("itemtype"));

        if atoms.is_empty() {
            return None;
        }

        #[expect(clippy::mutable_key_type)]
        // See `impl Hash for DOMString`.
        let mut item_attr_values = HashSet::new();
        for attr_value in &atoms {
            item_attr_values.insert(DOMString::from(String::from(attr_value.trim())));
        }

        Some(item_attr_values.into_iter().collect())
    }

    /// <https://html.spec.whatwg.org/multipage/#names:-the-itemprop-attribute>
    fn PropertyNames(&self) -> Option<Vec<DOMString>> {
        let atoms = self
            .element
            .get_tokenlist_attribute(&local_name!("itemprop"));

        if atoms.is_empty() {
            return None;
        }

        #[expect(clippy::mutable_key_type)]
        // See `impl Hash for DOMString`.
        let mut item_attr_values = HashSet::new();
        for attr_value in &atoms {
            item_attr_values.insert(DOMString::from(String::from(attr_value.trim())));
        }

        Some(item_attr_values.into_iter().collect())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-click>
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
            .fire_synthetic_pointer_event_not_trusted(atom!("click"), can_gc);
        element.set_click_in_progress(false);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-focus>
    fn Focus(&self, options: &FocusOptions, can_gc: CanGc) {
        // TODO: Mark the element as locked for focus and run the focusing steps.
        // <https://html.spec.whatwg.org/multipage/#focusing-steps>
        let document = self.owner_document();
        document.request_focus_with_options(
            Some(self.upcast()),
            FocusInitiator::Script,
            FocusOptions {
                preventScroll: options.preventScroll,
            },
            can_gc,
        );
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-blur>
    fn Blur(&self, can_gc: CanGc) {
        // TODO: Run the unfocusing steps. Focus the top-level document, not
        //       the current document.
        if !self.as_element().focus_state() {
            return;
        }
        // https://html.spec.whatwg.org/multipage/#unfocusing-steps
        let document = self.owner_document();
        document.request_focus(None, FocusInitiator::Script, can_gc);
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-htmlelement-scrollparent>
    #[expect(unsafe_code)]
    fn ScrollParent(&self) -> Option<DomRoot<Element>> {
        self.owner_window()
            .scroll_container_query(
                Some(self.upcast()),
                ScrollContainerQueryFlags::ForScrollParent,
            )
            .and_then(|response| match response {
                ScrollContainerResponse::Viewport(_) => self.owner_document().GetScrollingElement(),
                ScrollContainerResponse::Element(parent_node_address, _) => {
                    let node = unsafe { from_untrusted_node_address(parent_node_address) };
                    DomRoot::downcast(node)
                },
            })
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-htmlelement-offsetparent>
    fn GetOffsetParent(&self) -> Option<DomRoot<Element>> {
        if self.is::<HTMLBodyElement>() || self.upcast::<Element>().is_root() {
            return None;
        }

        let node = self.upcast::<Node>();
        let window = self.owner_window();
        let (element, _) = window.offset_parent_query(node);

        element
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-htmlelement-offsettop>
    fn OffsetTop(&self) -> i32 {
        if self.is_body_element() {
            return 0;
        }

        let node = self.upcast::<Node>();
        let window = self.owner_window();
        let (_, rect) = window.offset_parent_query(node);

        rect.origin.y.to_nearest_px()
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-htmlelement-offsetleft>
    fn OffsetLeft(&self) -> i32 {
        if self.is_body_element() {
            return 0;
        }

        let node = self.upcast::<Node>();
        let window = self.owner_window();
        let (_, rect) = window.offset_parent_query(node);

        rect.origin.x.to_nearest_px()
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-htmlelement-offsetwidth>
    fn OffsetWidth(&self) -> i32 {
        let node = self.upcast::<Node>();
        let window = self.owner_window();
        let (_, rect) = window.offset_parent_query(node);

        rect.size.width.to_nearest_px()
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-htmlelement-offsetheight>
    fn OffsetHeight(&self) -> i32 {
        let node = self.upcast::<Node>();
        let window = self.owner_window();
        let (_, rect) = window.offset_parent_query(node);

        rect.size.height.to_nearest_px()
    }

    /// <https://html.spec.whatwg.org/multipage/#the-innertext-idl-attribute>
    fn InnerText(&self) -> DOMString {
        self.get_inner_outer_text()
    }

    /// <https://html.spec.whatwg.org/multipage/#set-the-inner-text-steps>
    fn SetInnerText(&self, cx: &mut JSContext, input: DOMString) {
        self.set_inner_text(cx, input)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-outertext>
    fn GetOuterText(&self) -> Fallible<DOMString> {
        Ok(self.get_inner_outer_text())
    }

    /// <https://html.spec.whatwg.org/multipage/#the-innertext-idl-attribute:dom-outertext-2>
    fn SetOuterText(&self, cx: &mut JSContext, input: DOMString) -> Fallible<()> {
        // Step 1: If this's parent is null, then throw a "NoModificationAllowedError" DOMException.
        let Some(parent) = self.upcast::<Node>().GetParentNode() else {
            return Err(Error::NoModificationAllowed(None));
        };

        let node = self.upcast::<Node>();
        let document = self.owner_document();

        // Step 2: Let next be this's next sibling.
        let next = node.GetNextSibling();

        // Step 3: Let previous be this's previous sibling.
        let previous = node.GetPreviousSibling();

        // Step 4: Let fragment be the rendered text fragment for the given value given this's node
        // document.
        let fragment = self.rendered_text_fragment(cx, input);

        // Step 5: If fragment has no children, then append a new Text node whose data is the empty
        // string and node document is this's node document to fragment.
        if fragment.upcast::<Node>().children_count() == 0 {
            let text_node = Text::new(
                DOMString::from("".to_owned()),
                &document,
                CanGc::from_cx(cx),
            );

            fragment
                .upcast::<Node>()
                .AppendChild(cx, text_node.upcast())?;
        }

        // Step 6: Replace this with fragment within this's parent.
        parent.ReplaceChild(cx, fragment.upcast(), node)?;

        // Step 7: If next is non-null and next's previous sibling is a Text node, then merge with
        // the next text node given next's previous sibling.
        if let Some(next_sibling) = next {
            if let Some(node) = next_sibling.GetPreviousSibling() {
                Self::merge_with_the_next_text_node(cx, node);
            }
        }

        // Step 8: If previous is a Text node, then merge with the next text node given previous.
        if let Some(previous) = previous {
            Self::merge_with_the_next_text_node(cx, previous)
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-translate>
    fn Translate(&self) -> bool {
        self.as_element().is_translate_enabled()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-translate>
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
    make_enumerated_getter!(
        ContentEditable,
        "contenteditable",
        "true" | "false" | "plaintext-only",
        missing => "inherit",
        invalid => "inherit",
        empty => "true"
    );

    /// <https://html.spec.whatwg.org/multipage/#dom-contenteditable>
    fn SetContentEditable(&self, value: DOMString, can_gc: CanGc) -> ErrorResult {
        let lower_value = value.to_ascii_lowercase();
        let element = self.upcast::<Element>();
        let attr_name = &local_name!("contenteditable");
        match lower_value.as_ref() {
            // > On setting, if the new value is an ASCII case-insensitive match for the string "inherit", then the content attribute must be removed,
            "inherit" => {
                element.remove_attribute_by_name(attr_name, can_gc);
            },
            // > if the new value is an ASCII case-insensitive match for the string "true", then the content attribute must be set to the string "true",
            // > if the new value is an ASCII case-insensitive match for the string "plaintext-only", then the content attribute must be set to the string "plaintext-only",
            // > if the new value is an ASCII case-insensitive match for the string "false", then the content attribute must be set to the string "false",
            "true" | "false" | "plaintext-only" => {
                element.set_attribute(attr_name, AttrValue::String(lower_value), can_gc);
            },
            // > and otherwise the attribute setter must throw a "SyntaxError" DOMException.
            _ => return Err(Error::Syntax(None)),
        };
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-iscontenteditable>
    fn IsContentEditable(&self) -> bool {
        // > The isContentEditable IDL attribute, on getting, must return true if the element is either an editing host or editable, and false otherwise.
        self.upcast::<Node>().is_editable_or_editing_host()
    }

    /// <https://html.spec.whatwg.org/multipage#dom-attachinternals>
    fn AttachInternals(&self, can_gc: CanGc) -> Fallible<DomRoot<ElementInternals>> {
        let element = self.as_element();
        // Step 1: If this's is value is not null, then throw a "NotSupportedError" DOMException
        if element.get_is().is_some() {
            return Err(Error::NotSupported(None));
        }

        // Step 2: Let definition be the result of looking up a custom element definition
        // Note: the element can pass this check without yet being a custom
        // element, as long as there is a registered definition
        // that could upgrade it to one later.
        let registry = self.owner_window().CustomElements();
        let definition = registry.lookup_definition(self.as_element().local_name(), None);

        // Step 3: If definition is null, then throw an "NotSupportedError" DOMException
        let definition = match definition {
            Some(definition) => definition,
            None => return Err(Error::NotSupported(None)),
        };

        // Step 4: If definition's disable internals is true, then throw a "NotSupportedError" DOMException
        if definition.disable_internals {
            return Err(Error::NotSupported(None));
        }

        // Step 5: If this's attached internals is non-null, then throw an "NotSupportedError" DOMException
        let internals = element.ensure_element_internals(can_gc);
        if internals.attached() {
            return Err(Error::NotSupported(None));
        }

        // Step 6: If this's custom element state is not "precustomized" or "custom",
        // then throw a "NotSupportedError" DOMException.
        if !matches!(
            element.get_custom_element_state(),
            CustomElementState::Precustomized | CustomElementState::Custom
        ) {
            return Err(Error::NotSupported(None));
        }

        if self.is_form_associated_custom_element() {
            element.init_state_for_internals();
        }

        // Step 6-7: Set this's attached internals to a new ElementInternals instance
        internals.set_attached();
        Ok(internals)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-noncedelement-nonce>
    fn Nonce(&self) -> DOMString {
        self.as_element().nonce_value().into()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-noncedelement-nonce>
    fn SetNonce(&self, value: DOMString) {
        self.as_element()
            .update_nonce_internal_slot(value.to_string())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-fe-autofocus>
    fn Autofocus(&self) -> bool {
        self.element.has_attribute(&local_name!("autofocus"))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-fe-autofocus>
    fn SetAutofocus(&self, autofocus: bool, can_gc: CanGc) {
        self.element
            .set_bool_attribute(&local_name!("autofocus"), autofocus, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-tabindex>
    fn TabIndex(&self) -> i32 {
        self.element.tab_index()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-tabindex>
    fn SetTabIndex(&self, tab_index: i32, can_gc: CanGc) {
        self.element
            .set_int_attribute(&local_name!("tabindex"), tab_index, can_gc);
    }

    // https://html.spec.whatwg.org/multipage/#dom-accesskey
    make_getter!(AccessKey, "accesskey");

    // https://html.spec.whatwg.org/multipage/#dom-accesskey
    make_setter!(SetAccessKey, "accesskey");

    /// <https://html.spec.whatwg.org/multipage/#dom-accesskeylabel>
    fn AccessKeyLabel(&self) -> DOMString {
        // The accessKeyLabel IDL attribute must return a string that represents the element's
        // assigned access key, if any. If the element does not have one, then the IDL attribute
        // must return the empty string.
        if !self.element.has_attribute(&local_name!("accesskey")) {
            return Default::default();
        }

        let access_key_string = self
            .element
            .get_string_attribute(&local_name!("accesskey"))
            .to_string();

        #[cfg(target_os = "macos")]
        let access_key_label = format!("⌃⌥{access_key_string}");
        #[cfg(not(target_os = "macos"))]
        let access_key_label = format!("Alt+Shift+{access_key_string}");

        access_key_label.into()
    }
}

fn append_text_node_to_fragment(
    cx: &mut JSContext,
    document: &Document,
    fragment: &DocumentFragment,
    text: String,
) {
    let text = Text::new(DOMString::from(text), document, CanGc::from_cx(cx));
    fragment
        .upcast::<Node>()
        .AppendChild(cx, text.upcast())
        .unwrap();
}

impl HTMLElement {
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

    /// <https://html.spec.whatwg.org/multipage/#the-body-element-2>
    pub(crate) fn is_body_element(&self) -> bool {
        let self_node = self.upcast::<Node>();
        self_node.GetParentNode().is_some_and(|parent| {
            let parent_node = parent.upcast::<Node>();
            (self_node.is::<HTMLBodyElement>() || self_node.is::<HTMLFrameSetElement>()) &&
                parent_node.is::<HTMLHtmlElement>() &&
                self_node
                    .preceding_siblings()
                    .all(|n| !n.is::<HTMLBodyElement>() && !n.is::<HTMLFrameSetElement>())
        })
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
        let element_direction = &self.Dir();

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
        debug_assert!(self.as_element().local_name() == &local_name!("summary"));

        // Step 1. If this summary element is not the summary for its parent details, then return.
        if !self.is_a_summary_for_its_parent_details() {
            return;
        }

        // Step 2. Let parent be this summary element's parent.
        let parent = if self.is_implicit_summary_element() {
            DomRoot::downcast::<HTMLDetailsElement>(self.containing_shadow_root().unwrap().Host())
                .unwrap()
        } else {
            self.upcast::<Node>()
                .GetParentNode()
                .and_then(DomRoot::downcast::<HTMLDetailsElement>)
                .unwrap()
        };

        // Step 3. If the open attribute is present on parent, then remove it.
        // Otherwise, set parent's open attribute to the empty string.
        parent.toggle();
    }

    /// <https://html.spec.whatwg.org/multipage/#summary-for-its-parent-details>
    pub(crate) fn is_a_summary_for_its_parent_details(&self) -> bool {
        if self.is_implicit_summary_element() {
            return true;
        }

        // Step 1. If this summary element has no parent, then return false.
        // Step 2. Let parent be this summary element's parent.
        let Some(parent) = self.upcast::<Node>().GetParentNode() else {
            return false;
        };

        // Step 3. If parent is not a details element, then return false.
        let Some(details) = parent.downcast::<HTMLDetailsElement>() else {
            return false;
        };

        // Step 4. If parent's first summary element child is not this summary
        // element, then return false.
        // Step 5. Return true.
        details
            .find_corresponding_summary_element()
            .is_some_and(|summary| &*summary == self.upcast())
    }

    /// Whether or not this is an implicitly generated `<summary>`
    /// element for a UA `<details>` shadow tree
    fn is_implicit_summary_element(&self) -> bool {
        // Note that non-implicit summary elements are not actually inside
        // the UA shadow tree, they're only assigned to a slot inside it.
        // Therefore they don't cause false positives here
        self.containing_shadow_root()
            .as_deref()
            .map(ShadowRoot::Host)
            .is_some_and(|host| host.is::<HTMLDetailsElement>())
    }

    /// <https://html.spec.whatwg.org/multipage/#rendered-text-fragment>
    fn rendered_text_fragment(
        &self,
        cx: &mut JSContext,
        input: DOMString,
    ) -> DomRoot<DocumentFragment> {
        // Step 1: Let fragment be a new DocumentFragment whose node document is document.
        let document = self.owner_document();
        let fragment = DocumentFragment::new(&document, CanGc::from_cx(cx));

        // Step 2: Let position be a position variable for input, initially pointing at the start
        // of input.
        let input = input.str();
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
                        append_text_node_to_fragment(cx, &document, &fragment, text);
                        text = String::new();
                    }

                    let br = Element::create(
                        cx,
                        QualName::new(None, ns!(html), local_name!("br")),
                        None,
                        &document,
                        ElementCreator::ScriptCreated,
                        CustomElementCreationMode::Asynchronous,
                        None,
                    );
                    fragment
                        .upcast::<Node>()
                        .AppendChild(cx, br.upcast())
                        .unwrap();
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
            append_text_node_to_fragment(cx, &document, &fragment, text);
        }

        fragment
    }

    /// Checks whether a given [`DomRoot<Node>`] and its next sibling are
    /// of type [`Text`], and if so merges them into a single [`Text`]
    /// node.
    ///
    /// <https://html.spec.whatwg.org/multipage/#merge-with-the-next-text-node>
    fn merge_with_the_next_text_node(cx: &mut JSContext, node: DomRoot<Node>) {
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
        next.remove_self(cx);
    }

    /// <https://html.spec.whatwg.org/multipage/#keyboard-shortcuts-processing-model>
    /// > Whenever an element's accesskey attribute is set, changed, or removed, the user agent must
    /// > update the element's assigned access key by running the following steps:
    fn update_assigned_access_key(&self) {
        // 1. If the element has no accesskey attribute, then skip to the fallback step below.
        if !self.element.has_attribute(&local_name!("accesskey")) {
            // This is the same as steps 4 and 5 below.
            self.owner_document()
                .event_handler()
                .unassign_access_key(self);
        }

        // 2. Otherwise, split the attribute's value on ASCII whitespace, and let keys be the resulting tokens.
        let attribute_value = self.element.get_string_attribute(&local_name!("accesskey"));
        let string_view = attribute_value.str();
        let values = string_view.split_html_space_characters();

        // 3. For each value in keys in turn, in the order the tokens appeared in the attribute's
        //    value, run the following substeps:
        for value in values {
            // 1. If the value is not a string exactly one code point in length, then skip the
            //    remainder of these steps for this value.
            let mut characters = value.chars();
            let Some(character) = characters.next() else {
                continue;
            };
            if characters.count() > 0 {
                continue;
            }

            // 2. If the value does not correspond to a key on the system's keyboard, then skip the
            //    remainder of these steps for this value.
            let Some(code) = character_to_code(character) else {
                continue;
            };

            // 3. If the user agent can find a mix of zero or more modifier keys that, combined with
            //    the key that corresponds to the value given in the attribute, can be used as the
            //    access key, then the user agent may assign that combination of keys as the element's
            //    assigned access key and return.
            self.owner_document()
                .event_handler()
                .assign_access_key(self, code);
            return;
        }

        // 4. Fallback: Optionally, the user agent may assign a key combination of its choosing as
        //    the element's assigned access key and then return.
        // We do not do this.

        // 5. If this step is reached, the element has no assigned access key.
        self.owner_document()
            .event_handler()
            .unassign_access_key(self);
    }
}

impl VirtualMethods for HTMLElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.as_element() as &dyn VirtualMethods)
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
        let element = self.as_element();
        match (attr.local_name(), mutation) {
            // https://html.spec.whatwg.org/multipage/#event-handler-attributes:event-handler-content-attributes-3
            (name, mutation)
                if name.starts_with("on") && EventTarget::is_content_event_handler(name) =>
            {
                let evtarget = self.upcast::<EventTarget>();
                let event_name = &name[2..];
                match mutation {
                    // https://html.spec.whatwg.org/multipage/#activate-an-event-handler
                    AttributeMutation::Set(..) => {
                        let source = &**attr.value();
                        let source_line = 1; // TODO(#9604) get current JS execution line
                        evtarget.set_event_handler_uncompiled(
                            self.owner_window().get_url(),
                            source_line,
                            event_name,
                            source,
                        );
                    },
                    // https://html.spec.whatwg.org/multipage/#deactivate-an-event-handler
                    AttributeMutation::Removed => {
                        evtarget.set_event_handler_common::<EventHandlerNonNull>(event_name, None);
                    },
                }
            },

            (&local_name!("accesskey"), ..) => {
                self.update_assigned_access_key();
            },
            (&local_name!("form"), mutation) if self.is_form_associated_custom_element() => {
                self.form_attribute_mutated(mutation, CanGc::from_cx(cx));
            },
            // Adding a "disabled" attribute disables an enabled form element.
            (&local_name!("disabled"), AttributeMutation::Set(..))
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
                    AttributeMutation::Set(..) => {
                        element.set_read_write_state(true);
                    },
                    AttributeMutation::Removed => {
                        element.set_read_write_state(false);
                    },
                }
            },
            (&local_name!("nonce"), mutation) => match mutation {
                AttributeMutation::Set(..) => {
                    let nonce = &**attr.value();
                    element.update_nonce_internal_slot(nonce.to_owned());
                },
                AttributeMutation::Removed => {
                    element.update_nonce_internal_slot("".to_owned());
                },
            },
            _ => {},
        }
    }

    fn bind_to_tree(&self, cx: &mut JSContext, context: &BindContext) {
        if let Some(super_type) = self.super_type() {
            super_type.bind_to_tree(cx, context);
        }

        // Binding to a tree can disable a form control if one of the new
        // ancestors is a fieldset.
        let element = self.as_element();
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

        if element.has_attribute(&local_name!("accesskey")) {
            self.update_assigned_access_key();
        }
    }

    /// <https://html.spec.whatwg.org/multipage#dom-trees:concept-node-remove-ext>
    fn unbind_from_tree(&self, context: &UnbindContext, can_gc: CanGc) {
        // 1. Let document be removedNode's node document.
        let document = self.owner_document();

        // 2. If document's focused area is removedNode, then set document's focused area to
        // document's viewport, and set document's relevant global object's navigation API's focus
        // changed during ongoing navigation to false.
        //
        // TODO: Should this also happen for non-HTML elements such as SVG elements?
        let element = self.as_element();
        if document
            .get_focused_element()
            .is_some_and(|focused_element| &*focused_element == element)
        {
            document.request_focus(None, FocusInitiator::Script, can_gc);
        }

        // 3. If removedNode is an element whose namespace is the HTML namespace, and this standard
        // defines HTML element removing steps for removedNode's local name, then run the
        // corresponding HTML element removing steps given removedNode, isSubtreeRoot, and
        // oldAncestor.
        if let Some(super_type) = self.super_type() {
            super_type.unbind_from_tree(context, can_gc);
        }

        // 4. If removedNode is a form-associated element with a non-null form owner and removedNode
        // and its form owner are no longer in the same tree, then reset the form owner of
        // removedNode.
        //
        // Unbinding from a tree might enable a form control, if a
        // fieldset ancestor is the only reason it was disabled.
        // (The fact that it's enabled doesn't do much while it's
        // disconnected, but it is an observable fact to keep track of.)
        //
        // TODO: This should likely just call reset on form owner.
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

        if element.has_attribute(&local_name!("accesskey")) {
            self.owner_document()
                .event_handler()
                .unassign_access_key(self);
        }
    }

    fn attribute_affects_presentational_hints(&self, attr: &Attr) -> bool {
        let element = self.upcast::<Element>();
        if is_element_affected_by_legacy_background_presentational_hint(
            element.namespace(),
            element.local_name(),
        ) && attr.local_name() == &local_name!("background")
        {
            return true;
        }

        self.super_type()
            .unwrap()
            .attribute_affects_presentational_hints(attr)
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        let element = self.upcast::<Element>();
        match *name {
            local_name!("itemprop") => AttrValue::from_serialized_tokenlist(value.into()),
            local_name!("itemtype") => AttrValue::from_serialized_tokenlist(value.into()),
            local_name!("background")
                if is_element_affected_by_legacy_background_presentational_hint(
                    element.namespace(),
                    element.local_name(),
                ) =>
            {
                AttrValue::from_resolved_url(
                    &self.owner_document().base_url().get_arc(),
                    value.into(),
                )
            },
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-trees:html-element-moving-steps>
    fn moving_steps(&self, context: &MoveContext, can_gc: CanGc) {
        // Step 1. If movedNode is an element whose namespace is the HTML namespace, and this
        // standard defines HTML element moving steps for movedNode's local name, then run the
        // corresponding HTML element moving steps given movedNode.
        if let Some(super_type) = self.super_type() {
            super_type.moving_steps(context, can_gc);
        }

        // Step 2. If movedNode is a form-associated element with a non-null form owner and
        // movedNode and its form owner are no longer in the same tree, then reset the form owner of
        // movedNode.
        if let Some(form_control) = self.upcast::<Element>().as_maybe_form_control() {
            form_control.moving_steps(can_gc)
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
            .ensure_element_internals(CanGc::note())
            .set_form_owner(form);
    }

    fn to_element(&self) -> &Element {
        self.as_element()
    }

    fn is_listed(&self) -> bool {
        debug_assert!(self.is_form_associated_custom_element());
        true
    }

    // TODO satisfies_constraints traits
}
