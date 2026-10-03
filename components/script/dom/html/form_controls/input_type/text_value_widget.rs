/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Ref;

use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::CharacterDataBinding::CharacterDataMethods;
use script_bindings::root::Dom;

use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::characterdata::CharacterData;
use crate::dom::html::form_controls::htmlinputelement::HTMLInputElement;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::shadowroot::shadowroot::ShadowRoot;
use crate::dom::shadowroot::ua_shadowroot::{
    SpecificShadowTree, UAShadowRoot, UpdateUAShadowRootForOther,
};
use crate::dom::text::Text;

#[derive(Default, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct TextValueWidget {
    shadow_tree: DomRefCell<Option<TextValueShadowTree>>,
}

impl TextValueWidget {
    pub(crate) fn update_shadow_tree(&self, cx: &mut JSContext, input: &HTMLInputElement) {
        UpdateUAShadowRootForOther::update_shadow_tree(self, cx, input)
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
}

impl SpecificShadowTree<HTMLInputElement, TextValueWidget> for TextValueShadowTree {
    fn update(&self, cx: &mut JSContext, _: &TextValueWidget, input_element: &HTMLInputElement) {
        let character_data = self.value.upcast::<CharacterData>();
        let value = input_element.value_for_shadow_dom();
        if character_data.Data() != value {
            character_data.SetData(cx, value);
        }
    }
}

impl UAShadowRoot<TextValueShadowTree> for TextValueWidget {
    fn store_for_shadow_tree(&self, cx: &mut JSContext, shadow_root: DomRoot<ShadowRoot>) {
        *self.shadow_tree.borrow_mut() = Some(TextValueShadowTree::new(cx, shadow_root.upcast()));
    }

    fn borrow_for_shadow_tree(&self) -> Ref<'_, Option<TextValueShadowTree>> {
        self.shadow_tree.borrow()
    }
}
