/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::htmlinputelement::HTMLInputElement;
use crate::dom::htmlinputelement::text_value_widget::TextValueWidget;
use crate::dom::input_element::input_type::SpecificInputType;

#[derive(Default, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct ButtonInputType {
    text_value_widget: DomRefCell<TextValueWidget>,
}

impl SpecificInputType for ButtonInputType {
    fn update_shadow_tree(&self, cx: &mut JSContext, input: &HTMLInputElement) {
        self.text_value_widget
            .borrow()
            .update_shadow_tree(cx, input)
    }
}
