/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use js::context::JSContext;
use url::Url;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::str::DOMString;
use crate::dom::htmlinputelement::text_input_widget::TextInputWidget;
use crate::dom::input_element::HTMLInputElement;
use crate::dom::input_element::input_type::SpecificInputType;

#[derive(Default, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct UrlInputType {
    text_input_widget: DomRefCell<TextInputWidget>,
}

impl SpecificInputType for UrlInputType {
    fn sanitize_value(&self, _input: &HTMLInputElement, value: &mut DOMString) {
        value.strip_newlines();
        value.strip_leading_and_trailing_ascii_whitespace();
    }

    /// <https://html.spec.whatwg.org/multipage/#url-state-(type=url):suffering-from-a-type-mismatch>
    fn suffers_from_type_mismatch(&self, _input: &HTMLInputElement, value: &DOMString) -> bool {
        Url::parse(&value.str()).is_err()
    }

    fn update_shadow_tree(&self, cx: &mut JSContext, input: &HTMLInputElement) {
        self.text_input_widget
            .borrow()
            .update_shadow_tree(cx, input)
    }

    fn update_placeholder_contents(&self, cx: &mut JSContext, input: &HTMLInputElement) {
        self.text_input_widget
            .borrow()
            .update_placeholder_contents(cx, input)
    }
}
