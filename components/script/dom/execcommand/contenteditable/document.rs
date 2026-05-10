/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;

use crate::dom::bindings::codegen::Bindings::DocumentBinding::{
    DocumentMethods, ElementCreationOptions,
};
use crate::dom::bindings::codegen::UnionTypes::StringOrElementCreationOptions;
use crate::dom::bindings::root::DomRoot;
use crate::dom::document::Document;
use crate::dom::element::Element;

impl Document {
    pub(crate) fn create_element(&self, cx: &mut JSContext, name: &str) -> DomRoot<Element> {
        let element_options =
            StringOrElementCreationOptions::ElementCreationOptions(ElementCreationOptions {
                is: None,
            });
        self.CreateElement(cx, name.into(), element_options)
            .expect("Must always be able to create element")
    }
}
