/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for properties related to animations and transitions.

use crate::custom_properties::Name as CustomPropertyName;
use crate::parser::{Parse, ParserContext};
use crate::properties::{LonghandId, PropertyDeclarationId, PropertyId, ShorthandId};
use crate::values::generics::animation as generics;
use crate::values::specified::{LengthPercentage, NonNegativeNumber};
use crate::values::{CustomIdent, KeyframesName, TimelineName};
use crate::Atom;
use cssparser::Parser;
use std::fmt::{self, Write};
use style_traits::{
    CssWriter, KeywordsCollectFn, ParseError, SpecifiedValueInfo, StyleParseErrorKind, ToCss,
};

/// A given transition property, that is either `All`, a longhand or shorthand
/// property, or an unsupported or custom property.
#[derive(
    Clone, Debug, Eq, Hash, MallocSizeOf, PartialEq, ToComputedValue, ToResolvedValue, ToShmem,
)]
pub enum TransitionProperty {
    /// A shorthand.
    Shorthand(ShorthandId),
    /// A longhand transitionable property.
    Longhand(LonghandId),
    /// A custom property.
    Custom(CustomPropertyName),
    /// Unrecognized property which could be any non-transitionable, custom property, or
    /// unknown property.
    Unsupported(CustomIdent),
}

impl ToCss for TransitionProperty {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        use crate::values::serialize_atom_name;
        match *self {
            TransitionProperty::Shorthand(ref s) => s.to_css(dest),
            TransitionProperty::Longhand(ref l) => l.to_css(dest),
            TransitionProperty::Custom(ref name) => {
                dest.write_str("--")?;
                serialize_atom_name(name, dest)
            },
            TransitionProperty::Unsupported(ref i) => i.to_css(dest),
        }
    }
}

impl Parse for TransitionProperty {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        let ident = input.expect_ident()?;

        let id = match PropertyId::parse_ignoring_rule_type(&ident, context) {
            Ok(id) => id,
            Err(..) => {
                return Ok(TransitionProperty::Unsupported(CustomIdent::from_ident(
                    location,
                    ident,
                    &["none"],
                )?));
            },
        };

        Ok(match id.as_shorthand() {
            Ok(s) => TransitionProperty::Shorthand(s),
            Err(longhand_or_custom) => match longhand_or_custom {
                PropertyDeclarationId::Longhand(id) => TransitionProperty::Longhand(id),
                PropertyDeclarationId::Custom(custom) => TransitionProperty::Custom(custom.clone()),
            },
        })
    }
}

impl SpecifiedValueInfo for TransitionProperty {
    fn collect_completion_keywords(f: KeywordsCollectFn) {
        // `transition-property` can actually accept all properties and
        // arbitrary identifiers, but `all` is a special one we'd like
        // to list.
        f(&["all"]);
    }
}

impl TransitionProperty {
    /// Returns `all`.
    #[inline]
    pub fn all() -> Self {
        TransitionProperty::Shorthand(ShorthandId::All)
    }

    /// Convert TransitionProperty to nsCSSPropertyID.
    #[cfg(feature = "gecko")]
    pub fn to_nscsspropertyid(
        &self,
    ) -> Result<crate::gecko_bindings::structs::nsCSSPropertyID, ()> {
        Ok(match *self {
            TransitionProperty::Shorthand(ShorthandId::All) => {
                crate::gecko_bindings::structs::nsCSSPropertyID::eCSSPropertyExtra_all_properties
            },
            TransitionProperty::Shorthand(ref id) => id.to_nscsspropertyid(),
            TransitionProperty::Longhand(ref id) => id.to_nscsspropertyid(),
            TransitionProperty::Custom(..) | TransitionProperty::Unsupported(..) => return Err(()),
        })
    }
}

/// https://drafts.csswg.org/css-animations/#animation-iteration-count
#[derive(Clone, Debug, MallocSizeOf, PartialEq, Parse, SpecifiedValueInfo, ToCss, ToShmem)]
pub enum AnimationIterationCount {
    /// A `<number>` value.
    Number(NonNegativeNumber),
    /// The `infinite` keyword.
    Infinite,
}

impl AnimationIterationCount {
    /// Returns the value `1.0`.
    #[inline]
    pub fn one() -> Self {
        Self::Number(NonNegativeNumber::new(1.0))
    }
}

/// A value for the `animation-name` property.
#[derive(
    Clone,
    Debug,
    Eq,
    Hash,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[value_info(other_values = "none")]
#[repr(C)]
pub struct AnimationName(pub KeyframesName);

impl AnimationName {
    /// Get the name of the animation as an `Atom`.
    pub fn as_atom(&self) -> Option<&Atom> {
        if self.is_none() {
            return None;
        }
        Some(self.0.as_atom())
    }

    /// Returns the `none` value.
    pub fn none() -> Self {
        AnimationName(KeyframesName::none())
    }

    /// Returns whether this is the none value.
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl Parse for AnimationName {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(name) = input.try_parse(|input| KeyframesName::parse(context, input)) {
            return Ok(AnimationName(name));
        }

        input.expect_ident_matching("none")?;
        Ok(AnimationName(KeyframesName::none()))
    }
}

/// A value for the <Scroller> used in scroll().
///
/// https://drafts.csswg.org/scroll-animations-1/rewrite#typedef-scroller
#[derive(
    Clone,
    Debug,
    Eq,
    Hash,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum Scroller {
    /// The nearest ancestor scroll container. (Default.)
    Nearest,
    /// The document viewport as the scroll container.
    Root,
    /// Specifies to use the element’s own principal box as the scroll container.
    #[css(keyword = "self")]
    SelfElement,
}

impl Scroller {
    /// Returns true if it is default.
    #[inline]
    fn is_default(&self) -> bool {
        matches!(*self, Self::Nearest)
    }
}

impl Default for Scroller {
    fn default() -> Self {
        Self::Nearest
    }
}

/// A value for the <Axis> used in scroll(), or a value for {scroll|view}-timeline-axis.
///
/// https://drafts.csswg.org/scroll-animations-1/#typedef-axis
/// https://drafts.csswg.org/scroll-animations-1/#scroll-timeline-axis
/// https://drafts.csswg.org/scroll-animations-1/#view-timeline-axis
#[derive(
    Clone,
    Debug,
    Eq,
    Hash,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum ScrollAxis {
    /// The block axis of the scroll container. (Default.)
    Block = 0,
    /// The inline axis of the scroll container.
    Inline = 1,
    /// The vertical block axis of the scroll container.
    Vertical = 2,
    /// The horizontal axis of the scroll container.
    Horizontal = 3,
}

impl ScrollAxis {
    /// Returns true if it is default.
    #[inline]
    pub fn is_default(&self) -> bool {
        matches!(*self, Self::Block)
    }
}

impl Default for ScrollAxis {
    fn default() -> Self {
        Self::Block
    }
}

/// The scroll() notation.
/// https://drafts.csswg.org/scroll-animations-1/#scroll-notation
#[derive(
    Clone,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[css(function = "scroll")]
#[repr(C)]
pub struct ScrollFunction {
    /// The scroll container element whose scroll position drives the progress of the timeline.
    #[css(skip_if = "Scroller::is_default")]
    pub scroller: Scroller,
    /// The axis of scrolling that drives the progress of the timeline.
    #[css(skip_if = "ScrollAxis::is_default")]
    pub axis: ScrollAxis,
}

impl ScrollFunction {
    /// Parse the inner function arguments of `scroll()`.
    fn parse_arguments<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        // <scroll()> = scroll( [ <scroller> || <axis> ]? )
        // https://drafts.csswg.org/scroll-animations-1/#funcdef-scroll
        let mut scroller = None;
        let mut axis = None;
        loop {
            if scroller.is_none() {
                scroller = input.try_parse(Scroller::parse).ok();
            }

            if axis.is_none() {
                axis = input.try_parse(ScrollAxis::parse).ok();
                if axis.is_some() {
                    continue;
                }
            }
            break;
        }

        Ok(Self {
            scroller: scroller.unwrap_or_default(),
            axis: axis.unwrap_or_default(),
        })
    }
}

impl generics::ViewFunction<LengthPercentage> {
    /// Parse the inner function arguments of `view()`.
    fn parse_arguments<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        // <view()> = view( [ <axis> || <'view-timeline-inset'> ]? )
        // https://drafts.csswg.org/scroll-animations-1/#funcdef-view
        let mut axis = None;
        let mut inset = None;
        loop {
            if axis.is_none() {
                axis = input.try_parse(ScrollAxis::parse).ok();
            }

            if inset.is_none() {
                inset = input
                    .try_parse(|i| ViewTimelineInset::parse(context, i))
                    .ok();
                if inset.is_some() {
                    continue;
                }
            }
            break;
        }

        Ok(Self {
            inset: inset.unwrap_or_default(),
            axis: axis.unwrap_or_default(),
        })
    }
}

/// A specified value for the `animation-timeline` property.
pub type AnimationTimeline = generics::GenericAnimationTimeline<LengthPercentage>;

impl Parse for AnimationTimeline {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        use crate::values::generics::animation::ViewFunction;

        // <single-animation-timeline> = auto | none | <custom-ident> | <scroll()> | <view()>
        // https://drafts.csswg.org/css-animations-2/#typedef-single-animation-timeline

        if input.try_parse(|i| i.expect_ident_matching("auto")).is_ok() {
            return Ok(Self::Auto);
        }

        if input.try_parse(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(AnimationTimeline::Timeline(TimelineName::none()));
        }

        if let Ok(name) = input.try_parse(|i| TimelineName::parse(context, i)) {
            return Ok(AnimationTimeline::Timeline(name));
        }

        // Parse possible functions
        let location = input.current_source_location();
        let function = input.expect_function()?.clone();
        input.parse_nested_block(move |i| {
            match_ignore_ascii_case! { &function,
                "scroll" => ScrollFunction::parse_arguments(i).map(Self::Scroll),
                "view" => ViewFunction::parse_arguments(context, i).map(Self::View),
                _ => {
                    Err(location.new_custom_error(
                        StyleParseErrorKind::UnexpectedFunction(function.clone())
                    ))
                },
            }
        })
    }
}

/// A value for the scroll-timeline-name or view-timeline-name.
pub type ScrollTimelineName = AnimationName;

/// A specified value for the `view-timeline-inset` property.
pub type ViewTimelineInset = generics::GenericViewTimelineInset<LengthPercentage>;

impl Parse for ViewTimelineInset {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        use crate::values::specified::LengthPercentageOrAuto;

        let start = LengthPercentageOrAuto::parse(context, input)?;
        let end = match input.try_parse(|input| LengthPercentageOrAuto::parse(context, input)) {
            Ok(end) => end,
            Err(_) => start.clone(),
        };

        Ok(Self { start, end })
    }
}
