/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::Ref;

use html5ever::{local_name, ns};
use js::context::JSContext;
use markup5ever::QualName;
use script_bindings::codegen::GenericBindings::CharacterDataBinding::CharacterDataMethods;
use script_bindings::codegen::GenericBindings::DocumentBinding::DocumentMethods;
use script_bindings::codegen::GenericBindings::NodeBinding::NodeMethods;
use script_bindings::inheritance::Castable;
use script_bindings::root::{Dom, DomRoot};
use style::selector_parser::PseudoElement;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::characterdata::CharacterData;
use crate::dom::document::Document;
use crate::dom::element::{CustomElementCreationMode, Element, ElementCreator};
use crate::dom::node::{Node, NodeTraits};
use crate::dom::textcontrol::TextControlElement;

const PASSWORD_REPLACEMENT_CHAR: char = '●';

#[derive(Default, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct TextInputWidget {
    shadow_tree: DomRefCell<Option<TextInputWidgetShadowTree>>,
}

impl TextInputWidget {
    /// Get the shadow tree for this [`HTMLInputElement`], if it is created and valid, otherwise
    /// recreate the shadow tree and return it.
    fn get_or_create_shadow_tree(
        &self,
        cx: &mut JSContext,
        text_control_element: &impl TextControlElement,
    ) -> Ref<'_, TextInputWidgetShadowTree> {
        {
            if let Ok(shadow_tree) = Ref::filter_map(self.shadow_tree.borrow(), |shadow_tree| {
                shadow_tree.as_ref()
            }) {
                return shadow_tree;
            }
        }

        let element = text_control_element.upcast::<Element>();
        let shadow_root = element
            .shadow_root()
            .unwrap_or_else(|| element.attach_ua_shadow_root(cx, true));
        let shadow_root = shadow_root.upcast();
        *self.shadow_tree.borrow_mut() = Some(TextInputWidgetShadowTree::new(cx, shadow_root));
        self.get_or_create_shadow_tree(cx, text_control_element)
    }

    pub(crate) fn update_shadow_tree(&self, cx: &mut JSContext, element: &impl TextControlElement) {
        self.get_or_create_shadow_tree(cx, element).update(element)
    }

    pub(crate) fn update_placeholder_contents(
        &self,
        cx: &mut JSContext,
        element: &impl TextControlElement,
    ) {
        self.get_or_create_shadow_tree(cx, element)
            .update_placeholder(cx, element);
    }
}

#[derive(Clone, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
/// Contains reference to text control inner editor and placeholder container element in the UA
/// shadow tree for `text`, `password`, `url`, `tel`, and `email` input. The following is the
/// structure of the shadow tree.
///
/// ```
/// <input type="text">
///     #shadow-root
///         <div id="inner-container">
///             <div id="input-editor"></div>
///             <div id="input-placeholder"></div>
///         </div>
/// </input>
/// ```
///
// TODO(stevennovaryo): We are trying to use CSS to mimic Chrome and Firefox's layout for the <input> element.
//                      But, this could be slower in performance and does have some discrepancies. For example,
//                      they would try to vertically align <input> text baseline with the baseline of other
//                      TextNode within an inline flow. Another example is the horizontal scroll.
// FIXME(#38263): Refactor these logics into a TextControl wrapper that would decouple all textual input.
pub(crate) struct TextInputWidgetShadowTree {
    inner_container: Dom<Element>,
    text_container: Dom<Element>,
    placeholder_container: DomRefCell<Option<Dom<Element>>>,
}

impl TextInputWidgetShadowTree {
    pub(crate) fn new(cx: &mut JSContext, shadow_root: &Node) -> Self {
        let document = shadow_root.owner_document();
        let inner_container = Element::create(
            cx,
            QualName::new(None, ns!(html), local_name!("div")),
            None,
            &document,
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Asynchronous,
            None,
        );

        Node::replace_all(cx, Some(inner_container.upcast()), shadow_root.upcast());
        inner_container
            .upcast::<Node>()
            .set_implemented_pseudo_element(PseudoElement::ServoTextControlInnerContainer);

        let text_container = create_ua_widget_div_with_text_node(
            cx,
            &document,
            inner_container.upcast(),
            PseudoElement::ServoTextControlInnerEditor,
            false,
        );

        Self {
            inner_container: inner_container.as_traced(),
            text_container: text_container.as_traced(),
            placeholder_container: DomRefCell::new(None),
        }
    }

    /// Initialize the placeholder container only when it is necessary. This would help the performance of input
    /// element with shadow dom that is quite bulky.
    fn init_placeholder_container_if_necessary(
        &self,
        cx: &mut JSContext,
        element: &impl TextControlElement,
    ) -> Option<DomRoot<Element>> {
        if let Some(placeholder_container) = &*self.placeholder_container.borrow() {
            return Some(placeholder_container.root_element());
        }
        // If there is no placeholder text and we haven't already created one then it is
        // not necessary to initialize a new placeholder container.
        let placeholder = element.placeholder_text();
        if placeholder.is_empty() {
            return None;
        }

        let placeholder_container = create_ua_widget_div_with_text_node(
            cx,
            &element.owner_document(),
            self.inner_container.upcast::<Node>(),
            PseudoElement::Placeholder,
            true,
        );
        *self.placeholder_container.borrow_mut() = Some(placeholder_container.as_traced());
        Some(placeholder_container)
    }

    fn placeholder_character_data(
        &self,
        cx: &mut JSContext,
        element: &impl TextControlElement,
    ) -> Option<DomRoot<CharacterData>> {
        self.init_placeholder_container_if_necessary(cx, element)
            .and_then(|placeholder_container| {
                let first_child = placeholder_container.upcast::<Node>().GetFirstChild()?;
                Some(DomRoot::from_ref(first_child.downcast::<CharacterData>()?))
            })
    }

    pub(crate) fn update_placeholder(&self, cx: &mut JSContext, element: &impl TextControlElement) {
        if let Some(character_data) = self.placeholder_character_data(cx, element) {
            let placeholder_value = element.placeholder_text();
            if character_data.Data() != *placeholder_value {
                character_data.SetData(placeholder_value.clone());
            }
        }
    }

    fn value_character_data(&self) -> Option<DomRoot<CharacterData>> {
        Some(DomRoot::from_ref(
            self.text_container
                .upcast::<Node>()
                .GetFirstChild()?
                .downcast::<CharacterData>()?,
        ))
    }

    // TODO(stevennovaryo): The rest of textual input shadow dom structure should act
    // like an exstension to this one.
    pub(crate) fn update(&self, element: &impl TextControlElement) {
        // The addition of zero-width space here forces the text input to have an inline formatting
        // context that might otherwise be trimmed if there's no text. This is important to ensure
        // that the input element is at least as tall as the line gap of the caret:
        // <https://drafts.csswg.org/css-ui/#element-with-default-preferred-size>.
        //
        // This is also used to ensure that the caret will still be rendered when the input is empty.
        // TODO: Could append `<br>` element to prevent collapses and avoid this hack, but we would
        //       need to fix the rendering of caret beforehand.
        let value = element.value_text();
        let value_text = match (value.is_empty(), element.is_password_field()) {
            // For a password input, we replace all of the character with its replacement char.
            (false, true) => value
                .str()
                .chars()
                .map(|_| PASSWORD_REPLACEMENT_CHAR)
                .collect::<String>()
                .into(),
            (false, _) => value,
            (true, _) => "\u{200B}".into(),
        };

        if let Some(character_data) = self.value_character_data() {
            if character_data.Data() != value_text {
                character_data.SetData(value_text);
            }
        }
    }
}

/// Create a div element with a text node within an UA Widget and either append or prepend it to
/// the designated parent. This is used to create the text container for input elements.
fn create_ua_widget_div_with_text_node(
    cx: &mut JSContext,
    document: &Document,
    parent: &Node,
    implemented_pseudo: PseudoElement,
    as_first_child: bool,
) -> DomRoot<Element> {
    let el = Element::create(
        cx,
        QualName::new(None, ns!(html), local_name!("div")),
        None,
        document,
        ElementCreator::ScriptCreated,
        CustomElementCreationMode::Asynchronous,
        None,
    );

    parent
        .upcast::<Node>()
        .AppendChild(cx, el.upcast::<Node>())
        .unwrap();
    el.upcast::<Node>()
        .set_implemented_pseudo_element(implemented_pseudo);
    let text_node = document.CreateTextNode(cx, "".into());

    if !as_first_child {
        el.upcast::<Node>()
            .AppendChild(cx, text_node.upcast::<Node>())
            .unwrap();
    } else {
        el.upcast::<Node>()
            .InsertBefore(
                cx,
                text_node.upcast::<Node>(),
                el.upcast::<Node>().GetFirstChild().as_deref(),
            )
            .unwrap();
    }
    el
}
