/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Ref;

use js::context::JSContext;
use script_bindings::codegen::GenericBindings::CharacterDataBinding::CharacterDataMethods;
use script_bindings::root::Dom;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::characterdata::CharacterData;
use crate::dom::element::Element;
use crate::dom::htmlinputelement::HTMLInputElement;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::text::Text;

#[derive(Default, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct TextValueWidget {
    shadow_tree: DomRefCell<Option<TextValueShadowTree>>,
}

impl TextValueWidget {
    /// Get the shadow tree for this [`HTMLInputElement`], if it is created and valid, otherwise
    /// recreate the shadow tree and return it.
    fn get_or_create_shadow_tree(
        &self,
        cx: &mut JSContext,
        input: &HTMLInputElement,
    ) -> Ref<'_, TextValueShadowTree> {
        {
            if let Ok(shadow_tree) = Ref::filter_map(self.shadow_tree.borrow(), |shadow_tree| {
                shadow_tree.as_ref()
            }) {
                return shadow_tree;
            }
        }

        let element = input.upcast::<Element>();
        let shadow_root = element
            .shadow_root()
            .unwrap_or_else(|| element.attach_ua_shadow_root(cx, true));
        let shadow_root = shadow_root.upcast();
        *self.shadow_tree.borrow_mut() = Some(TextValueShadowTree::new(cx, shadow_root));
        self.get_or_create_shadow_tree(cx, input)
    }

    pub(crate) fn update_shadow_tree(&self, cx: &mut JSContext, input: &HTMLInputElement) {
        self.get_or_create_shadow_tree(cx, input).update(input)
    }
}

#[derive(Clone, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct TextValueShadowTree {
    value: Dom<Text>,
}

impl TextValueShadowTree {
    fn new(cx: &mut JSContext, shadow_root: &Node) -> Self {
        let value = Text::new(cx, Default::default(), &shadow_root.owner_document());
        Node::replace_all(cx, Some(value.upcast()), shadow_root);
        Self {
            value: value.as_traced(),
        }
    }

    fn update(&self, input_element: &HTMLInputElement) {
        let character_data = self.value.upcast::<CharacterData>();
        let value = input_element.value_for_shadow_dom();
        if character_data.Data() != value {
            character_data.SetData(value);
        }
    }
}
