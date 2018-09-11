/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Media features.

use Atom;
use cssparser::Parser;
use parser::ParserContext;
use std::fmt;
use style_traits::ParseError;
use super::Device;
use super::media_feature_expression::{AspectRatio, RangeOrOperator};
use values::computed::{CSSPixelLength, Resolution};

/// A generic discriminant for an enum value.
pub type KeywordDiscriminant = u8;

type MediaFeatureEvaluator<T> = fn(
    device: &Device,
    // null == no value was given in the query.
    value: Option<T>,
    range_or_operator: Option<RangeOrOperator>,
) -> bool;

/// Serializes a given discriminant.
///
/// FIXME(emilio): we could prevent this allocation if the ToCss code would
/// generate a method for keywords to get the static string or something.
pub type KeywordSerializer = fn(KeywordDiscriminant) -> String;

/// Parses a given identifier.
pub type KeywordParser =
    for<'a, 'i, 't> fn(context: &'a ParserContext, input: &'a mut Parser<'i, 't>)
        -> Result<KeywordDiscriminant, ParseError<'i>>;

/// An evaluator for a given media feature.
///
/// This determines the kind of values that get parsed, too.
#[allow(missing_docs)]
pub enum Evaluator {
    Length(MediaFeatureEvaluator<CSSPixelLength>),
    Integer(MediaFeatureEvaluator<u32>),
    Float(MediaFeatureEvaluator<f32>),
    BoolInteger(MediaFeatureEvaluator<bool>),
    /// An integer ratio, such as the one from device-pixel-ratio.
    IntRatio(MediaFeatureEvaluator<AspectRatio>),
    /// A resolution.
    Resolution(MediaFeatureEvaluator<Resolution>),
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
        evaluator: MediaFeatureEvaluator<KeywordDiscriminant>,
    },
    Ident(MediaFeatureEvaluator<Atom>),
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
            $crate::media_queries::media_feature::KeywordDiscriminant,
            ::style_traits::ParseError<'i>,
        > {
            let kw = <$keyword_type as $crate::parser::Parse>::parse(context, input)?;
            Ok(kw as $crate::media_queries::media_feature::KeywordDiscriminant)
        }

        fn __serialize(kw: $crate::media_queries::media_feature::KeywordDiscriminant) -> String {
            // This unwrap is ok because the only discriminants that get
            // back to us is the ones that `parse` produces.
            let value: $keyword_type = ::num_traits::cast::FromPrimitive::from_u8(kw).unwrap();
            <$keyword_type as ::style_traits::ToCss>::to_css_string(&value)
        }

        fn __evaluate(
            device: &$crate::media_queries::Device,
            value: Option<$crate::media_queries::media_feature::KeywordDiscriminant>,
            range_or_operator: Option<
                $crate::media_queries::media_feature_expression::RangeOrOperator,
            >,
        ) -> bool {
            debug_assert!(
                range_or_operator.is_none(),
                "Since when do keywords accept ranges?"
            );
            // This unwrap is ok because the only discriminants that get
            // back to us is the ones that `parse` produces.
            let value: Option<$keyword_type> =
                value.map(|kw| ::num_traits::cast::FromPrimitive::from_u8(kw).unwrap());
            $actual_evaluator(device, value)
        }

        $crate::media_queries::media_feature::Evaluator::Enumerated {
            parser: __parse,
            serializer: __serialize,
            evaluator: __evaluate,
        }
    }};
}

bitflags! {
    /// Different requirements or toggles that change how a expression is
    /// parsed.
    pub struct ParsingRequirements: u8 {
        /// The feature should only be parsed in chrome and ua sheets.
        const CHROME_AND_UA_ONLY = 1 << 0;
        /// The feature requires a -webkit- prefix.
        const WEBKIT_PREFIX = 1 << 1;
        /// The feature requires the webkit-device-pixel-ratio preference to be
        /// enabled.
        const WEBKIT_DEVICE_PIXEL_RATIO_PREF_ENABLED = 1 << 2;
    }
}

/// Whether a media feature allows ranges or not.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(missing_docs)]
pub enum AllowsRanges {
    Yes,
    No,
}

/// A description of a media feature.
pub struct MediaFeatureDescription {
    /// The media feature name, in ascii lowercase.
    pub name: Atom,
    /// Whether min- / max- prefixes are allowed or not.
    pub allows_ranges: AllowsRanges,
    /// The evaluator, which we also use to determine which kind of value to
    /// parse.
    pub evaluator: Evaluator,
    /// Different requirements that need to hold for the feature to be
    /// successfully parsed.
    pub requirements: ParsingRequirements,
}

impl MediaFeatureDescription {
    /// Whether this media feature allows ranges.
    #[inline]
    pub fn allows_ranges(&self) -> bool {
        self.allows_ranges == AllowsRanges::Yes
    }
}

/// A simple helper to construct a `MediaFeatureDescription`.
macro_rules! feature {
    ($name:expr, $allows_ranges:expr, $evaluator:expr, $reqs:expr,) => {
        $crate::media_queries::media_feature::MediaFeatureDescription {
            name: $name,
            allows_ranges: $allows_ranges,
            evaluator: $evaluator,
            requirements: $reqs,
        }
    };
}

impl fmt::Debug for MediaFeatureDescription {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MediaFeatureExpression")
            .field("name", &self.name)
            .field("allows_ranges", &self.allows_ranges)
            .field("requirements", &self.requirements)
            .finish()
    }
}
