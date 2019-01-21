/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::document::Document;
use crate::dom::shadowroot::ShadowRoot;

macro_rules! proxy_call(
    ($fn_name:ident, $return_type:ty) => (
        pub fn $fn_name(&self) -> $return_type {
            match self {
                DocumentOrShadowRoot::Document(doc) => doc.$fn_name(),
                DocumentOrShadowRoot::ShadowRoot(root) => root.$fn_name(),
            }
        }
    );

    ($fn_name:ident, $arg1:ident, $arg1_type:ty, $return_type:ty) => (
        pub fn $fn_name(&self, $arg1: $arg1_type) -> $return_type {
            match self {
                DocumentOrShadowRoot::Document(doc) => doc.$fn_name($arg1),
                DocumentOrShadowRoot::ShadowRoot(root) => root.$fn_name($arg1),
            }
        }
    );
);

#[must_root]
#[derive(JSTraceable, MallocSizeOf)]
pub enum DocumentOrShadowRoot {
    Document(Dom<Document>),
    ShadowRoot(Dom<ShadowRoot>),
}

impl DocumentOrShadowRoot {
    proxy_call!(stylesheet_count, usize);
    proxy_call!(stylesheet_at, index, usize, Option<DomRoot<CSSStyleSheet>>);
}
