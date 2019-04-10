/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for CSS values that are related to motion path.

use crate::parser::{Parse, ParserContext};
use crate::values::specified::SVGPathData;
use cssparser::Parser;
use style_traits::{ParseError, StyleParseErrorKind};

/// The offset-path value.
///
/// https://drafts.fxtf.org/motion-1/#offset-path-property
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
pub enum OffsetPath {
    // We could merge SVGPathData into ShapeSource, so we could reuse them. However,
    // we don't want to support other value for offset-path, so use SVGPathData only for now.
    /// Path value for path(<string>).
    #[css(function)]
    Path(SVGPathData),
    /// None value.
    #[animation(error)]
    None,
    // Bug 1186329: Implement ray(), <basic-shape>, <geometry-box>, and <url>.
}

impl OffsetPath {
    /// Return None.
    #[inline]
    pub fn none() -> Self {
        OffsetPath::None
    }
}

impl Parse for OffsetPath {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        // Parse none.
        if input.try(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(OffsetPath::none());
        }

        // Parse possible functions.
        let location = input.current_source_location();
        let function = input.expect_function()?.clone();
        input.parse_nested_block(move |i| {
            match_ignore_ascii_case! { &function,
                // Bug 1186329: Implement the parser for ray(), <basic-shape>, <geometry-box>,
                // and <url>.
                "path" => SVGPathData::parse(context, i).map(OffsetPath::Path),
                _ => {
                    Err(location.new_custom_error(
                        StyleParseErrorKind::UnexpectedFunction(function.clone())
                    ))
                },
            }
        })
    }
}
