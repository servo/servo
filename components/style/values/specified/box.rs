/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for box properties.

use Atom;
use cssparser::Parser;
use parser::{Parse, ParserContext};
use std::fmt;
use style_traits::{ParseError, ToCss};
use values::CustomIdent;
use values::KeyframesName;
use values::generics::box_::AnimationIterationCount as GenericAnimationIterationCount;
use values::generics::box_::VerticalAlign as GenericVerticalAlign;
use values::specified::{AllowQuirks, Number};
use values::specified::length::LengthOrPercentage;

/// A specified value for the `vertical-align` property.
pub type VerticalAlign = GenericVerticalAlign<LengthOrPercentage>;

impl Parse for VerticalAlign {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(lop) = input.try(|i| LengthOrPercentage::parse_quirky(context, i, AllowQuirks::Yes)) {
            return Ok(GenericVerticalAlign::Length(lop));
        }

        try_match_ident_ignore_ascii_case! { input,
            "baseline" => Ok(GenericVerticalAlign::Baseline),
            "sub" => Ok(GenericVerticalAlign::Sub),
            "super" => Ok(GenericVerticalAlign::Super),
            "top" => Ok(GenericVerticalAlign::Top),
            "text-top" => Ok(GenericVerticalAlign::TextTop),
            "middle" => Ok(GenericVerticalAlign::Middle),
            "bottom" => Ok(GenericVerticalAlign::Bottom),
            "text-bottom" => Ok(GenericVerticalAlign::TextBottom),
            #[cfg(feature = "gecko")]
            "-moz-middle-with-baseline" => {
                Ok(GenericVerticalAlign::MozMiddleWithBaseline)
            },
        }
    }
}

/// https://drafts.csswg.org/css-animations/#animation-iteration-count
pub type AnimationIterationCount = GenericAnimationIterationCount<Number>;

impl Parse for AnimationIterationCount {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut ::cssparser::Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input.try(|input| input.expect_ident_matching("infinite")).is_ok() {
            return Ok(GenericAnimationIterationCount::Infinite)
        }

        let number = Number::parse_non_negative(context, input)?;
        Ok(GenericAnimationIterationCount::Number(number))
    }
}

impl AnimationIterationCount {
    /// Returns the value `1.0`.
    #[inline]
    pub fn one() -> Self {
        GenericAnimationIterationCount::Number(Number::new(1.0))
    }
}

/// A value for the `animation-name` property.
#[derive(Clone, Debug, Eq, Hash, MallocSizeOf, PartialEq, ToComputedValue)]
pub struct AnimationName(pub Option<KeyframesName>);

impl AnimationName {
    /// Get the name of the animation as an `Atom`.
    pub fn as_atom(&self) -> Option<&Atom> {
        self.0.as_ref().map(|n| n.as_atom())
    }

    /// Returns the `none` value.
    pub fn none() -> Self {
        AnimationName(None)
    }
}

impl ToCss for AnimationName {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        match self.0 {
            Some(ref name) => name.to_css(dest),
            None => dest.write_str("none"),
        }
    }
}

impl Parse for AnimationName {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(name) = input.try(|input| KeyframesName::parse(context, input)) {
            return Ok(AnimationName(Some(name)));
        }

        input.expect_ident_matching("none")?;
        Ok(AnimationName(None))
    }
}

define_css_keyword_enum! { ScrollSnapType:
    "none" => None,
    "mandatory" => Mandatory,
    "proximity" => Proximity,
}
add_impls_for_keyword_enum!(ScrollSnapType);

define_css_keyword_enum! { OverscrollBehavior:
    "auto" => Auto,
    "contain" => Contain,
    "none" => None,
}
add_impls_for_keyword_enum!(OverscrollBehavior);

define_css_keyword_enum! { OverflowClipBox:
    "padding-box" => PaddingBox,
    "content-box" => ContentBox,
}
add_impls_for_keyword_enum!(OverflowClipBox);

#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToComputedValue, ToCss)]
/// Provides a rendering hint to the user agent,
/// stating what kinds of changes the author expects
/// to perform on the element
///
/// <https://drafts.csswg.org/css-will-change/#will-change>
pub enum WillChange {
    /// Expresses no particular intent
    Auto,
    #[css(comma, iterable)]
    /// <custom-ident>
    AnimateableFeatures(Box<[CustomIdent]>),
}

impl WillChange {
    #[inline]
    /// Get default value of `will-change` as `auto`
    pub fn auto() -> WillChange {
        WillChange::Auto
    }
}

impl Parse for WillChange {
    /// auto | <animateable-feature>#
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<WillChange, ParseError<'i>> {
        if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
            return Ok(WillChange::Auto);
        }

        let custom_idents = input.parse_comma_separated(|i| {
            let location = i.current_source_location();
            CustomIdent::from_ident(location, i.expect_ident()?, &[
                "will-change",
                "none",
                "all",
                "auto",
            ])
        })?;

        Ok(WillChange::AnimateableFeatures(custom_idents.into_boxed_slice()))
    }
}

bitflags! {
    #[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
    #[derive(ToComputedValue)]
    /// These constants match Gecko's `NS_STYLE_TOUCH_ACTION_*` constants.
    pub struct TouchAction: u8 {
        /// `none` variant
        const TOUCH_ACTION_NONE = 1 << 0;
        /// `auto` variant
        const TOUCH_ACTION_AUTO = 1 << 1;
        /// `pan-x` variant
        const TOUCH_ACTION_PAN_X = 1 << 2;
        /// `pan-y` variant
        const TOUCH_ACTION_PAN_Y = 1 << 3;
        /// `manipulation` variant
        const TOUCH_ACTION_MANIPULATION = 1 << 4;
    }
}

impl TouchAction {
    #[inline]
    /// Get default `touch-action` as `auto`
    pub fn auto() -> TouchAction {
        TouchAction::TOUCH_ACTION_AUTO
    }
}

impl ToCss for TouchAction {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            TouchAction::TOUCH_ACTION_NONE => dest.write_str("none"),
            TouchAction::TOUCH_ACTION_AUTO => dest.write_str("auto"),
            TouchAction::TOUCH_ACTION_MANIPULATION => dest.write_str("manipulation"),
            _ if self.contains(TouchAction::TOUCH_ACTION_PAN_X | TouchAction::TOUCH_ACTION_PAN_Y) => {
                dest.write_str("pan-x pan-y")
            },
            _ if self.contains(TouchAction::TOUCH_ACTION_PAN_X) => {
                dest.write_str("pan-x")
            },
            _ if self.contains(TouchAction::TOUCH_ACTION_PAN_Y) => {
                dest.write_str("pan-y")
            },
            _ => panic!("invalid touch-action value"),
        }
    }
}

impl Parse for TouchAction {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<TouchAction, ParseError<'i>> {
        try_match_ident_ignore_ascii_case! { input,
            "auto" => Ok(TouchAction::TOUCH_ACTION_AUTO),
            "none" => Ok(TouchAction::TOUCH_ACTION_NONE),
            "manipulation" => Ok(TouchAction::TOUCH_ACTION_MANIPULATION),
            "pan-x" => {
                if input.try(|i| i.expect_ident_matching("pan-y")).is_ok() {
                    Ok(TouchAction::TOUCH_ACTION_PAN_X | TouchAction::TOUCH_ACTION_PAN_Y)
                } else {
                    Ok(TouchAction::TOUCH_ACTION_PAN_X)
                }
            },
            "pan-y" => {
                if input.try(|i| i.expect_ident_matching("pan-x")).is_ok() {
                    Ok(TouchAction::TOUCH_ACTION_PAN_X | TouchAction::TOUCH_ACTION_PAN_Y)
                } else {
                    Ok(TouchAction::TOUCH_ACTION_PAN_Y)
                }
            },
        }
    }
}

#[cfg(feature = "gecko")]
impl_bitflags_conversions!(TouchAction);

/// Asserts that all touch-action matches its NS_STYLE_TOUCH_ACTION_* value.
#[cfg(feature = "gecko")]
#[inline]
pub fn assert_touch_action_matches() {
    use gecko_bindings::structs;

    macro_rules! check_touch_action {
        ( $( $a:ident => $b:path),*, ) => {
            $(
                debug_assert_eq!(structs::$a as u8, $b.bits());
            )*
        }
    }

    check_touch_action! {
        NS_STYLE_TOUCH_ACTION_NONE => TouchAction::TOUCH_ACTION_NONE,
        NS_STYLE_TOUCH_ACTION_AUTO => TouchAction::TOUCH_ACTION_AUTO,
        NS_STYLE_TOUCH_ACTION_PAN_X => TouchAction::TOUCH_ACTION_PAN_X,
        NS_STYLE_TOUCH_ACTION_PAN_Y => TouchAction::TOUCH_ACTION_PAN_Y,
        NS_STYLE_TOUCH_ACTION_MANIPULATION => TouchAction::TOUCH_ACTION_MANIPULATION,
    }
}
