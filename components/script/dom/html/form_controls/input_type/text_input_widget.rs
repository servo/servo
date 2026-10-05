/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::Ref;

use html5ever::{local_name, ns};
use js::context::JSContext;
use markup5ever::QualName;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::CharacterDataBinding::CharacterDataMethods;
use script_bindings::codegen::GenericBindings::DocumentBinding::DocumentMethods;
use script_bindings::codegen::GenericBindings::NodeBinding::NodeMethods;
use script_bindings::inheritance::Castable;
use script_bindings::root::{Dom, DomRoot};
use script_bindings::str::DOMString;
use servo_base::text::{RangeAny, Utf32CodeUnits};
use style::selector_parser::PseudoElement;

use crate::dom::bindings::conversions::DerivedFrom;
use crate::dom::bindings::root::MutNullableDom;
use crate::dom::characterdata::CharacterData;
use crate::dom::document::Document;
use crate::dom::element::{CustomElementCreationMode, Element, ElementCreator};
use crate::dom::html::form_controls::text_control::TextControlElement;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::shadowroot::shadowroot::ShadowRoot;
use crate::dom::shadowroot::ua_shadowroot::{
    SpecificShadowTree, UAShadowRoot, UpdateUAShadowRootForOther,
};

const PASSWORD_REPLACEMENT_CHAR: char = '●';

#[derive(Default, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct TextInputWidget {
    shadow_tree: DomRefCell<Option<TextInputWidgetShadowTree>>,
}

impl TextInputWidget {
    pub(crate) fn update_shadow_tree(
        &self,
        cx: &mut JSContext,
        element: &(impl TextControlElement + DerivedFrom<Element>),
    ) {
        UpdateUAShadowRootForOther::update_shadow_tree(self, cx, element)
    }

    pub(crate) fn update_placeholder_contents(
        &self,
        cx: &mut JSContext,
        element: &(impl TextControlElement + DerivedFrom<Element>),
    ) {
        self.ensure_shadow_tree(cx, element.upcast())
            .update_placeholder(cx, element);
    }

    /// Returns whether `new_range` was successfully set on an existing text run
    pub(crate) fn set_text_run_selection(
        &self,
        new_range: Option<RangeAny<Utf32CodeUnits>>,
    ) -> bool {
        if let Some(shadow_tree) = &*self.shadow_tree.borrow() &&
            let Some(character_data) = shadow_tree.value_character_data()
        {
            character_data.set_text_run_selection(new_range)
        } else {
            false
        }
    }
}

#[derive(JSTraceable, MallocSizeOf, PartialEq)]
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
    placeholder_container: MutNullableDom<Element>,
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
            placeholder_container: MutNullableDom::new(None),
        }
    }

    /// Initialize the placeholder container only when it is necessary. This would help the performance of input
    /// element with shadow dom that is quite bulky.
    fn init_placeholder_container_if_necessary(
        &self,
        cx: &mut JSContext,
        element: &impl TextControlElement,
    ) -> Option<DomRoot<Element>> {
        if let Some(placeholder_container) = self.placeholder_container.get() {
            return Some(placeholder_container);
        }
        // If there is no placeholder text and we haven't already created one then it is
        // not necessary to initialize a new placeholder container.
        let placeholder = element.placeholder_text();
        if placeholder.is_empty() {
            return None;
        }

        let element = element.as_element();
        let placeholder_container = create_ua_widget_div_with_text_node(
            cx,
            &element.owner_document(),
            self.inner_container.upcast::<Node>(),
            PseudoElement::Placeholder,
            true,
        );
        self.placeholder_container
            .set(Some(&*placeholder_container));
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
                character_data.SetData(cx, placeholder_value.clone());
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
    let text_node = document.CreateTextNode(cx, DOMString::new());

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

impl<Element: TextControlElement> SpecificShadowTree<Element, TextInputWidget>
    for TextInputWidgetShadowTree
{
    // TODO(stevennovaryo): The rest of textual input shadow dom structure should act
    // like an exstension to this one.
    fn update(&self, cx: &mut JSContext, _: &TextInputWidget, element: &Element) {
        let value = element.value_text();
        let value_text = if element.is_password_field() {
            value
                .str()
                .chars()
                .map(|_| PASSWORD_REPLACEMENT_CHAR)
                .collect::<String>()
                .into()
        } else {
            value
        };

        if let Some(character_data) = self.value_character_data() &&
            character_data.Data() != value_text
        {
            character_data.SetData(cx, value_text);
        }
    }
}

impl UAShadowRoot<TextInputWidgetShadowTree> for TextInputWidget {
    fn store_for_shadow_tree(&self, cx: &mut JSContext, shadow_root: DomRoot<ShadowRoot>) {
        *self.shadow_tree.borrow_mut() =
            Some(TextInputWidgetShadowTree::new(cx, shadow_root.upcast()));
    }

    fn borrow_for_shadow_tree(&self) -> Ref<'_, Option<TextInputWidgetShadowTree>> {
        self.shadow_tree.borrow()
    }
}
