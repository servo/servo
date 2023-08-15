/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Query features.

use crate::parser::ParserContext;
use crate::values::computed::{self, CSSPixelLength, Resolution, Ratio};
use crate::Atom;
use cssparser::Parser;
use std::fmt;
use style_traits::ParseError;

/// A generic discriminant for an enum value.
pub type KeywordDiscriminant = u8;

type QueryFeatureGetter<T> = fn(device: &computed::Context) -> T;

/// Serializes a given discriminant.
///
/// FIXME(emilio): we could prevent this allocation if the ToCss code would
/// generate a method for keywords to get the static string or something.
pub type KeywordSerializer = fn(KeywordDiscriminant) -> String;

/// Parses a given identifier.
pub type KeywordParser = for<'a, 'i, 't> fn(
    context: &'a ParserContext,
    input: &'a mut Parser<'i, 't>,
) -> Result<KeywordDiscriminant, ParseError<'i>>;

/// An evaluator for a given feature.
///
/// This determines the kind of values that get parsed, too.
#[allow(missing_docs)]
pub enum Evaluator {
    Length(QueryFeatureGetter<CSSPixelLength>),
    Integer(QueryFeatureGetter<u32>),
    Float(QueryFeatureGetter<f32>),
    BoolInteger(QueryFeatureGetter<bool>),
    /// A non-negative number ratio, such as the one from device-pixel-ratio.
    NumberRatio(QueryFeatureGetter<Ratio>),
    /// A resolution.
    Resolution(QueryFeatureGetter<Resolution>),
    /// A keyword value.
    Enumerated {
        /// The parser to get a discriminant given a string.
        parser: KeywordParser,
        /// The serializer to get a string from a discriminant.
        ///
        /// This is guaranteed to be called with a keyword that `parser` has
        /// produced.
        serializer: KeywordSerializer,
        /// The evaluator itself. This is guaranteed to be called with a
        /// keyword that `parser` has produced.
        evaluator: fn(&computed::Context, Option<KeywordDiscriminant>) -> bool,
    },
}

/// A simple helper macro to create a keyword evaluator.
///
/// This assumes that keyword feature expressions don't accept ranges, and
/// asserts if that's not true. As of today there's nothing like that (does that
/// even make sense?).
macro_rules! keyword_evaluator {
    ($actual_evaluator:ident, $keyword_type:ty) => {{
        fn __parse<'i, 't>(
            context: &$crate::parser::ParserContext,
            input: &mut $crate::cssparser::Parser<'i, 't>,
        ) -> Result<
            $crate::queries::feature::KeywordDiscriminant,
            ::style_traits::ParseError<'i>,
        > {
            let kw = <$keyword_type as $crate::parser::Parse>::parse(context, input)?;
            Ok(kw as $crate::queries::feature::KeywordDiscriminant)
        }

        fn __serialize(kw: $crate::queries::feature::KeywordDiscriminant) -> String {
            // This unwrap is ok because the only discriminants that get
            // back to us is the ones that `parse` produces.
            let value: $keyword_type = ::num_traits::cast::FromPrimitive::from_u8(kw).unwrap();
            <$keyword_type as ::style_traits::ToCss>::to_css_string(&value)
        }

        fn __evaluate(
            context: &$crate::values::computed::Context,
            value: Option<$crate::queries::feature::KeywordDiscriminant>,
        ) -> bool {
            // This unwrap is ok because the only discriminants that get
            // back to us is the ones that `parse` produces.
            let value: Option<$keyword_type> =
                value.map(|kw| ::num_traits::cast::FromPrimitive::from_u8(kw).unwrap());
            $actual_evaluator(context, value)
        }

        $crate::queries::feature::Evaluator::Enumerated {
            parser: __parse,
            serializer: __serialize,
            evaluator: __evaluate,
        }
    }};
}

bitflags! {
    /// Different flags or toggles that change how a expression is parsed or
    /// evaluated.
    #[derive(ToShmem)]
    pub struct FeatureFlags : u8 {
        /// The feature should only be parsed in chrome and ua sheets.
        const CHROME_AND_UA_ONLY = 1 << 0;
        /// The feature requires a -webkit- prefix.
        const WEBKIT_PREFIX = 1 << 1;
        /// The feature requires the inline-axis containment.
        const CONTAINER_REQUIRES_INLINE_AXIS = 1 << 2;
        /// The feature requires the block-axis containment.
        const CONTAINER_REQUIRES_BLOCK_AXIS = 1 << 3;
        /// The feature requires containment in the physical width axis.
        const CONTAINER_REQUIRES_WIDTH_AXIS = 1 << 4;
        /// The feature requires containment in the physical height axis.
        const CONTAINER_REQUIRES_HEIGHT_AXIS = 1 << 5;
    }
}

impl FeatureFlags {
    /// Returns parsing requirement flags.
    pub fn parsing_requirements(self) -> Self {
        self.intersection(Self::CHROME_AND_UA_ONLY | Self::WEBKIT_PREFIX)
    }

    /// Returns all the container axis flags.
    pub fn all_container_axes() -> Self {
        Self::CONTAINER_REQUIRES_INLINE_AXIS |
            Self::CONTAINER_REQUIRES_BLOCK_AXIS |
            Self::CONTAINER_REQUIRES_WIDTH_AXIS |
            Self::CONTAINER_REQUIRES_HEIGHT_AXIS
    }

    /// Returns our subset of container axis flags.
    pub fn container_axes(self) -> Self {
        self.intersection(Self::all_container_axes())
    }
}

/// Whether a feature allows ranges or not.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(missing_docs)]
pub enum AllowsRanges {
    Yes,
    No,
}

/// A description of a feature.
pub struct QueryFeatureDescription {
    /// The feature name, in ascii lowercase.
    pub name: Atom,
    /// Whether min- / max- prefixes are allowed or not.
    pub allows_ranges: AllowsRanges,
    /// The evaluator, which we also use to determine which kind of value to
    /// parse.
    pub evaluator: Evaluator,
    /// Different feature-specific flags.
    pub flags: FeatureFlags,
}

impl QueryFeatureDescription {
    /// Whether this feature allows ranges.
    #[inline]
    pub fn allows_ranges(&self) -> bool {
        self.allows_ranges == AllowsRanges::Yes
    }
}

/// A simple helper to construct a `QueryFeatureDescription`.
macro_rules! feature {
    ($name:expr, $allows_ranges:expr, $evaluator:expr, $flags:expr,) => {
        $crate::queries::feature::QueryFeatureDescription {
            name: $name,
            allows_ranges: $allows_ranges,
            evaluator: $evaluator,
            flags: $flags,
        }
    };
}

impl fmt::Debug for QueryFeatureDescription {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("QueryFeatureDescription")
            .field("name", &self.name)
            .field("allows_ranges", &self.allows_ranges)
            .field("flags", &self.flags)
            .finish()
    }
}
