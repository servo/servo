/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Values for CSS Box Alignment properties
//!
//! https://drafts.csswg.org/css-align/

use crate::parser::{Parse, ParserContext};
use cssparser::Parser;
use std::fmt::{self, Write};
use style_traits::{CssWriter, KeywordsCollectFn, ParseError, SpecifiedValueInfo, ToCss};

bitflags! {
    /// Constants shared by multiple CSS Box Alignment properties
    #[derive(MallocSizeOf, ToComputedValue, ToResolvedValue, ToShmem)]
    #[repr(C)]
    pub struct AlignFlags: u8 {
        // Enumeration stored in the lower 5 bits:
        /// {align,justify}-{content,items,self}: 'auto'
        const AUTO = 0;
        /// 'normal'
        const NORMAL = 1;
        /// 'start'
        const START = 2;
        /// 'end'
        const END = 3;
        /// 'flex-start'
        const FLEX_START = 4;
        /// 'flex-end'
        const FLEX_END = 5;
        /// 'center'
        const CENTER = 6;
        /// 'left'
        const LEFT = 7;
        /// 'right'
        const RIGHT = 8;
        /// 'baseline'
        const BASELINE = 9;
        /// 'last-baseline'
        const LAST_BASELINE = 10;
        /// 'stretch'
        const STRETCH = 11;
        /// 'self-start'
        const SELF_START = 12;
        /// 'self-end'
        const SELF_END = 13;
        /// 'space-between'
        const SPACE_BETWEEN = 14;
        /// 'space-around'
        const SPACE_AROUND = 15;
        /// 'space-evenly'
        const SPACE_EVENLY = 16;

        // Additional flags stored in the upper bits:
        /// 'legacy' (mutually exclusive w. SAFE & UNSAFE)
        const LEGACY = 1 << 5;
        /// 'safe'
        const SAFE = 1 << 6;
        /// 'unsafe' (mutually exclusive w. SAFE)
        const UNSAFE = 1 << 7;

        /// Mask for the additional flags above.
        const FLAG_BITS = 0b11100000;
    }
}

impl AlignFlags {
    /// Returns the enumeration value stored in the lower 5 bits.
    #[inline]
    fn value(&self) -> Self {
        *self & !AlignFlags::FLAG_BITS
    }
}

impl ToCss for AlignFlags {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        let extra_flags = *self & AlignFlags::FLAG_BITS;
        let value = self.value();

        match extra_flags {
            AlignFlags::LEGACY => {
                dest.write_str("legacy")?;
                if value.is_empty() {
                    return Ok(());
                }
                dest.write_char(' ')?;
            },
            AlignFlags::SAFE => dest.write_str("safe ")?,
            AlignFlags::UNSAFE => dest.write_str("unsafe ")?,
            _ => {
                debug_assert_eq!(extra_flags, AlignFlags::empty());
            },
        }

        dest.write_str(match value {
            AlignFlags::AUTO => "auto",
            AlignFlags::NORMAL => "normal",
            AlignFlags::START => "start",
            AlignFlags::END => "end",
            AlignFlags::FLEX_START => "flex-start",
            AlignFlags::FLEX_END => "flex-end",
            AlignFlags::CENTER => "center",
            AlignFlags::LEFT => "left",
            AlignFlags::RIGHT => "right",
            AlignFlags::BASELINE => "baseline",
            AlignFlags::LAST_BASELINE => "last baseline",
            AlignFlags::STRETCH => "stretch",
            AlignFlags::SELF_START => "self-start",
            AlignFlags::SELF_END => "self-end",
            AlignFlags::SPACE_BETWEEN => "space-between",
            AlignFlags::SPACE_AROUND => "space-around",
            AlignFlags::SPACE_EVENLY => "space-evenly",
            _ => unreachable!(),
        })
    }
}

/// An axis direction, either inline (for the `justify` properties) or block,
/// (for the `align` properties).
#[derive(Clone, Copy, PartialEq)]
pub enum AxisDirection {
    /// Block direction.
    Block,
    /// Inline direction.
    Inline,
}

/// Shared value for the `align-content` and `justify-content` properties.
///
/// <https://drafts.csswg.org/css-align/#content-distribution>
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
pub struct ContentDistribution {
    primary: AlignFlags,
    // FIXME(https://github.com/w3c/csswg-drafts/issues/1002): This will need to
    // accept fallback alignment, eventually.
}

impl ContentDistribution {
    /// The initial value 'normal'
    #[inline]
    pub fn normal() -> Self {
        Self::new(AlignFlags::NORMAL)
    }

    /// `start`
    #[inline]
    pub fn start() -> Self {
        Self::new(AlignFlags::START)
    }

    /// The initial value 'normal'
    #[inline]
    pub fn new(primary: AlignFlags) -> Self {
        Self { primary }
    }

    /// Returns whether this value is a <baseline-position>.
    pub fn is_baseline_position(&self) -> bool {
        matches!(
            self.primary.value(),
            AlignFlags::BASELINE | AlignFlags::LAST_BASELINE
        )
    }

    /// The primary alignment
    #[inline]
    pub fn primary(self) -> AlignFlags {
        self.primary
    }

    /// Parse a value for align-content / justify-content.
    pub fn parse<'i, 't>(
        input: &mut Parser<'i, 't>,
        axis: AxisDirection,
    ) -> Result<Self, ParseError<'i>> {
        // NOTE Please also update the `list_keywords` function below
        //      when this function is updated.

        // Try to parse normal first
        if input.try(|i| i.expect_ident_matching("normal")).is_ok() {
            return Ok(ContentDistribution::normal());
        }

        // Parse <baseline-position>, but only on the block axis.
        if axis == AxisDirection::Block {
            if let Ok(value) = input.try(parse_baseline) {
                return Ok(ContentDistribution::new(value));
            }
        }

        // <content-distribution>
        if let Ok(value) = input.try(parse_content_distribution) {
            return Ok(ContentDistribution::new(value));
        }

        // <overflow-position>? <content-position>
        let overflow_position = input
            .try(parse_overflow_position)
            .unwrap_or(AlignFlags::empty());

        let content_position = try_match_ident_ignore_ascii_case! { input,
            "start" => AlignFlags::START,
            "end" => AlignFlags::END,
            "flex-start" => AlignFlags::FLEX_START,
            "flex-end" => AlignFlags::FLEX_END,
            "center" => AlignFlags::CENTER,
            "left" if axis == AxisDirection::Inline => AlignFlags::LEFT,
            "right" if axis == AxisDirection::Inline => AlignFlags::RIGHT,
        };

        Ok(ContentDistribution::new(
            content_position | overflow_position,
        ))
    }

    fn list_keywords(f: KeywordsCollectFn, axis: AxisDirection) {
        f(&["normal"]);
        if axis == AxisDirection::Block {
            list_baseline_keywords(f);
        }
        list_content_distribution_keywords(f);
        list_overflow_position_keywords(f);
        f(&["start", "end", "flex-start", "flex-end", "center"]);
        if axis == AxisDirection::Inline {
            f(&["left", "right"]);
        }
    }
}

/// Value for the `align-content` property.
///
/// <https://drafts.csswg.org/css-align/#propdef-align-content>
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(transparent)]
pub struct AlignContent(pub ContentDistribution);

impl Parse for AlignContent {
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        // NOTE Please also update `impl SpecifiedValueInfo` below when
        //      this function is updated.
        Ok(AlignContent(ContentDistribution::parse(
            input,
            AxisDirection::Block,
        )?))
    }
}

impl SpecifiedValueInfo for AlignContent {
    fn collect_completion_keywords(f: KeywordsCollectFn) {
        ContentDistribution::list_keywords(f, AxisDirection::Block);
    }
}

/// Value for the `align-tracks` property.
///
/// <https://github.com/w3c/csswg-drafts/issues/4650>
#[derive(
    Clone,
    Debug,
    Default,
    Eq,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(transparent)]
#[css(comma)]
pub struct AlignTracks(
    #[css(iterable, if_empty = "normal")]
    pub crate::OwnedSlice<AlignContent>
);

impl Parse for AlignTracks {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let values = input.parse_comma_separated(|input| {
            AlignContent::parse(context, input)
        })?;
        Ok(AlignTracks(values.into()))
    }
}

/// Value for the `justify-content` property.
///
/// <https://drafts.csswg.org/css-align/#propdef-justify-content>
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(transparent)]
pub struct JustifyContent(pub ContentDistribution);

impl Parse for JustifyContent {
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        // NOTE Please also update `impl SpecifiedValueInfo` below when
        //      this function is updated.
        Ok(JustifyContent(ContentDistribution::parse(
            input,
            AxisDirection::Inline,
        )?))
    }
}

impl SpecifiedValueInfo for JustifyContent {
    fn collect_completion_keywords(f: KeywordsCollectFn) {
        ContentDistribution::list_keywords(f, AxisDirection::Inline);
    }
}
/// Value for the `justify-tracks` property.
///
/// <https://github.com/w3c/csswg-drafts/issues/4650>
#[derive(
    Clone,
    Debug,
    Default,
    Eq,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(transparent)]
#[css(comma)]
pub struct JustifyTracks(
    #[css(iterable, if_empty = "normal")]
    pub crate::OwnedSlice<JustifyContent>
);

impl Parse for JustifyTracks {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let values = input.parse_comma_separated(|input| {
            JustifyContent::parse(context, input)
        })?;
        Ok(JustifyTracks(values.into()))
    }
}

/// <https://drafts.csswg.org/css-align/#self-alignment>
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(transparent)]
pub struct SelfAlignment(pub AlignFlags);

impl SelfAlignment {
    /// The initial value 'auto'
    #[inline]
    pub fn auto() -> Self {
        SelfAlignment(AlignFlags::AUTO)
    }

    /// Returns whether this value is valid for both axis directions.
    pub fn is_valid_on_both_axes(&self) -> bool {
        match self.0.value() {
            // left | right are only allowed on the inline axis.
            AlignFlags::LEFT | AlignFlags::RIGHT => false,

            _ => true,
        }
    }

    /// Parse a self-alignment value on one of the axis.
    pub fn parse<'i, 't>(
        input: &mut Parser<'i, 't>,
        axis: AxisDirection,
    ) -> Result<Self, ParseError<'i>> {
        // NOTE Please also update the `list_keywords` function below
        //      when this function is updated.

        // <baseline-position>
        //
        // It's weird that this accepts <baseline-position>, but not
        // justify-content...
        if let Ok(value) = input.try(parse_baseline) {
            return Ok(SelfAlignment(value));
        }

        // auto | normal | stretch
        if let Ok(value) = input.try(parse_auto_normal_stretch) {
            return Ok(SelfAlignment(value));
        }

        // <overflow-position>? <self-position>
        let overflow_position = input
            .try(parse_overflow_position)
            .unwrap_or(AlignFlags::empty());
        let self_position = parse_self_position(input, axis)?;
        Ok(SelfAlignment(overflow_position | self_position))
    }

    fn list_keywords(f: KeywordsCollectFn, axis: AxisDirection) {
        list_baseline_keywords(f);
        list_auto_normal_stretch(f);
        list_overflow_position_keywords(f);
        list_self_position_keywords(f, axis);
    }
}

/// The specified value of the align-self property.
///
/// <https://drafts.csswg.org/css-align/#propdef-align-self>
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct AlignSelf(pub SelfAlignment);

impl Parse for AlignSelf {
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        // NOTE Please also update `impl SpecifiedValueInfo` below when
        //      this function is updated.
        Ok(AlignSelf(SelfAlignment::parse(
            input,
            AxisDirection::Block,
        )?))
    }
}

impl SpecifiedValueInfo for AlignSelf {
    fn collect_completion_keywords(f: KeywordsCollectFn) {
        SelfAlignment::list_keywords(f, AxisDirection::Block);
    }
}

/// The specified value of the justify-self property.
///
/// <https://drafts.csswg.org/css-align/#propdef-justify-self>
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct JustifySelf(pub SelfAlignment);

impl Parse for JustifySelf {
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        // NOTE Please also update `impl SpecifiedValueInfo` below when
        //      this function is updated.
        Ok(JustifySelf(SelfAlignment::parse(
            input,
            AxisDirection::Inline,
        )?))
    }
}

impl SpecifiedValueInfo for JustifySelf {
    fn collect_completion_keywords(f: KeywordsCollectFn) {
        SelfAlignment::list_keywords(f, AxisDirection::Inline);
    }
}

/// Value of the `align-items` property
///
/// <https://drafts.csswg.org/css-align/#propdef-align-items>
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct AlignItems(pub AlignFlags);

impl AlignItems {
    /// The initial value 'normal'
    #[inline]
    pub fn normal() -> Self {
        AlignItems(AlignFlags::NORMAL)
    }
}

impl Parse for AlignItems {
    // normal | stretch | <baseline-position> |
    // <overflow-position>? <self-position>
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        // NOTE Please also update `impl SpecifiedValueInfo` below when
        //      this function is updated.

        // <baseline-position>
        if let Ok(baseline) = input.try(parse_baseline) {
            return Ok(AlignItems(baseline));
        }

        // normal | stretch
        if let Ok(value) = input.try(parse_normal_stretch) {
            return Ok(AlignItems(value));
        }
        // <overflow-position>? <self-position>
        let overflow = input
            .try(parse_overflow_position)
            .unwrap_or(AlignFlags::empty());
        let self_position = parse_self_position(input, AxisDirection::Block)?;
        Ok(AlignItems(self_position | overflow))
    }
}

impl SpecifiedValueInfo for AlignItems {
    fn collect_completion_keywords(f: KeywordsCollectFn) {
        list_baseline_keywords(f);
        list_normal_stretch(f);
        list_overflow_position_keywords(f);
        list_self_position_keywords(f, AxisDirection::Block);
    }
}

/// Value of the `justify-items` property
///
/// <https://drafts.csswg.org/css-align/#justify-items-property>
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToCss, ToResolvedValue, ToShmem)]
#[repr(C)]
pub struct JustifyItems(pub AlignFlags);

impl JustifyItems {
    /// The initial value 'legacy'
    #[inline]
    pub fn legacy() -> Self {
        JustifyItems(AlignFlags::LEGACY)
    }

    /// The value 'normal'
    #[inline]
    pub fn normal() -> Self {
        JustifyItems(AlignFlags::NORMAL)
    }
}

impl Parse for JustifyItems {
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        // NOTE Please also update `impl SpecifiedValueInfo` below when
        //      this function is updated.

        // <baseline-position>
        //
        // It's weird that this accepts <baseline-position>, but not
        // justify-content...
        if let Ok(baseline) = input.try(parse_baseline) {
            return Ok(JustifyItems(baseline));
        }

        // normal | stretch
        if let Ok(value) = input.try(parse_normal_stretch) {
            return Ok(JustifyItems(value));
        }

        // legacy | [ legacy && [ left | right | center ] ]
        if let Ok(value) = input.try(parse_legacy) {
            return Ok(JustifyItems(value));
        }

        // <overflow-position>? <self-position>
        let overflow = input
            .try(parse_overflow_position)
            .unwrap_or(AlignFlags::empty());
        let self_position = parse_self_position(input, AxisDirection::Inline)?;
        Ok(JustifyItems(overflow | self_position))
    }
}

impl SpecifiedValueInfo for JustifyItems {
    fn collect_completion_keywords(f: KeywordsCollectFn) {
        list_baseline_keywords(f);
        list_normal_stretch(f);
        list_legacy_keywords(f);
        list_overflow_position_keywords(f);
        list_self_position_keywords(f, AxisDirection::Inline);
    }
}

// auto | normal | stretch
fn parse_auto_normal_stretch<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<AlignFlags, ParseError<'i>> {
    // NOTE Please also update the `list_auto_normal_stretch` function
    //      below when this function is updated.
    try_match_ident_ignore_ascii_case! { input,
        "auto" => Ok(AlignFlags::AUTO),
        "normal" => Ok(AlignFlags::NORMAL),
        "stretch" => Ok(AlignFlags::STRETCH),
    }
}

fn list_auto_normal_stretch(f: KeywordsCollectFn) {
    f(&["auto", "normal", "stretch"]);
}

// normal | stretch
fn parse_normal_stretch<'i, 't>(input: &mut Parser<'i, 't>) -> Result<AlignFlags, ParseError<'i>> {
    // NOTE Please also update the `list_normal_stretch` function below
    //      when this function is updated.
    try_match_ident_ignore_ascii_case! { input,
        "normal" => Ok(AlignFlags::NORMAL),
        "stretch" => Ok(AlignFlags::STRETCH),
    }
}

fn list_normal_stretch(f: KeywordsCollectFn) {
    f(&["normal", "stretch"]);
}

// <baseline-position>
fn parse_baseline<'i, 't>(input: &mut Parser<'i, 't>) -> Result<AlignFlags, ParseError<'i>> {
    // NOTE Please also update the `list_baseline_keywords` function
    //      below when this function is updated.
    try_match_ident_ignore_ascii_case! { input,
        "baseline" => Ok(AlignFlags::BASELINE),
        "first" => {
            input.expect_ident_matching("baseline")?;
            Ok(AlignFlags::BASELINE)
        },
        "last" => {
            input.expect_ident_matching("baseline")?;
            Ok(AlignFlags::LAST_BASELINE)
        },
    }
}

fn list_baseline_keywords(f: KeywordsCollectFn) {
    f(&["baseline", "first baseline", "last baseline"]);
}

// <content-distribution>
fn parse_content_distribution<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<AlignFlags, ParseError<'i>> {
    // NOTE Please also update the `list_content_distribution_keywords`
    //      function below when this function is updated.
    try_match_ident_ignore_ascii_case! { input,
        "stretch" => Ok(AlignFlags::STRETCH),
        "space-between" => Ok(AlignFlags::SPACE_BETWEEN),
        "space-around" => Ok(AlignFlags::SPACE_AROUND),
        "space-evenly" => Ok(AlignFlags::SPACE_EVENLY),
    }
}

fn list_content_distribution_keywords(f: KeywordsCollectFn) {
    f(&["stretch", "space-between", "space-around", "space-evenly"]);
}

// <overflow-position>
fn parse_overflow_position<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<AlignFlags, ParseError<'i>> {
    // NOTE Please also update the `list_overflow_position_keywords`
    //      function below when this function is updated.
    try_match_ident_ignore_ascii_case! { input,
        "safe" => Ok(AlignFlags::SAFE),
        "unsafe" => Ok(AlignFlags::UNSAFE),
    }
}

fn list_overflow_position_keywords(f: KeywordsCollectFn) {
    f(&["safe", "unsafe"]);
}

// <self-position> | left | right in the inline axis.
fn parse_self_position<'i, 't>(
    input: &mut Parser<'i, 't>,
    axis: AxisDirection,
) -> Result<AlignFlags, ParseError<'i>> {
    // NOTE Please also update the `list_self_position_keywords`
    //      function below when this function is updated.
    Ok(try_match_ident_ignore_ascii_case! { input,
        "start" => AlignFlags::START,
        "end" => AlignFlags::END,
        "flex-start" => AlignFlags::FLEX_START,
        "flex-end" => AlignFlags::FLEX_END,
        "center" => AlignFlags::CENTER,
        "self-start" => AlignFlags::SELF_START,
        "self-end" => AlignFlags::SELF_END,
        "left" if axis == AxisDirection::Inline => AlignFlags::LEFT,
        "right" if axis == AxisDirection::Inline => AlignFlags::RIGHT,
    })
}

fn list_self_position_keywords(f: KeywordsCollectFn, axis: AxisDirection) {
    f(&[
        "start",
        "end",
        "flex-start",
        "flex-end",
        "center",
        "self-start",
        "self-end",
    ]);
    if axis == AxisDirection::Inline {
        f(&["left", "right"]);
    }
}

fn parse_left_right_center<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<AlignFlags, ParseError<'i>> {
    // NOTE Please also update the `list_legacy_keywords` function below
    //      when this function is updated.
    Ok(try_match_ident_ignore_ascii_case! { input,
        "left" => AlignFlags::LEFT,
        "right" => AlignFlags::RIGHT,
        "center" => AlignFlags::CENTER,
    })
}

// legacy | [ legacy && [ left | right | center ] ]
fn parse_legacy<'i, 't>(input: &mut Parser<'i, 't>) -> Result<AlignFlags, ParseError<'i>> {
    // NOTE Please also update the `list_legacy_keywords` function below
    //      when this function is updated.
    let flags = try_match_ident_ignore_ascii_case! { input,
        "legacy" => {
            let flags = input.try(parse_left_right_center)
                .unwrap_or(AlignFlags::empty());

            return Ok(AlignFlags::LEGACY | flags)
        },
        "left" => AlignFlags::LEFT,
        "right" => AlignFlags::RIGHT,
        "center" => AlignFlags::CENTER,
    };

    input.expect_ident_matching("legacy")?;
    Ok(AlignFlags::LEGACY | flags)
}

fn list_legacy_keywords(f: KeywordsCollectFn) {
    f(&["legacy", "left", "right", "center"]);
}
