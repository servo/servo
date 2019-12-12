/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::context::LayoutContext;
use crate::dom_traversal::{Contents, NodeExt};
use crate::formatting_contexts::IndependentFormattingContext;
use crate::sizing::ContentSizesRequest;
use crate::style_ext::{ComputedValuesExt, DisplayInside};
use servo_arc::Arc;
use style::properties::ComputedValues;

#[derive(Debug)]
pub(crate) struct FloatBox {
    pub contents: IndependentFormattingContext,
}

/// Data kept during layout about the floats in a given block formatting context.
pub(crate) struct FloatContext {
    // TODO
}

impl FloatContext {
    pub fn new() -> Self {
        FloatContext {}
    }
}

impl FloatBox {
    pub fn construct<'dom>(
        context: &LayoutContext,
        node: impl NodeExt<'dom>,
        style: Arc<ComputedValues>,
        display_inside: DisplayInside,
        contents: Contents,
    ) -> Self {
        let content_sizes = ContentSizesRequest::inline_if(!style.inline_size_is_length());
        Self {
            contents: IndependentFormattingContext::construct(
                context,
                node,
                style,
                display_inside,
                contents,
                content_sizes,
            ),
        }
    }
}
