/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Copy of Stylo Gecko's [`style::values::specified::gecko::IntersectionObserverRootMargin`] implementation.
//! TODO(#35907): make a thin wrapper and remove copied codes

use std::fmt;

use app_units::Au;
use cssparser::Parser;
use euclid::default::{Rect, SideOffsets2D};
use style::parser::{Parse, ParserContext};
use style::values::specified::intersection_observer::IntersectionObserverMargin;
use style_traits::{CssWriter, ParseError, ToCss};

/// The value of an IntersectionObserver's rootMargin property.
///
/// Only bare px or percentage values are allowed. Other length units and
/// calc() values are not allowed.
///
/// <https://w3c.github.io/IntersectionObserver/#parse-a-root-margin>

#[repr(transparent)]
pub struct IntersectionObserverRootMargin(pub IntersectionObserverMargin);

impl Parse for IntersectionObserverRootMargin {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        IntersectionObserverMargin::parse(context, input).map(IntersectionObserverRootMargin)
    }
}

impl ToCss for IntersectionObserverRootMargin {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        self.0.to_css(dest)
    }
}

// TODO(stevennovaryo): move this to the wrapper later
impl IntersectionObserverRootMargin {
    // Resolve to used values.
    pub(crate) fn resolve_percentages_with_basis(
        &self,
        containing_block: Rect<Au>,
    ) -> SideOffsets2D<Au> {
        let inner = &self.0.0;
        SideOffsets2D::new(
            inner.0.to_used_value(containing_block.height()),
            inner.1.to_used_value(containing_block.width()),
            inner.2.to_used_value(containing_block.height()),
            inner.3.to_used_value(containing_block.width()),
        )
    }
}
