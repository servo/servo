/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for box properties.

use crate::custom_properties::Name as CustomPropertyName;
use crate::parser::{Parse, ParserContext};
use crate::properties::{LonghandId, PropertyDeclarationId, PropertyFlags};
use crate::properties::{PropertyId, ShorthandId};
use crate::values::generics::box_::AnimationIterationCount as GenericAnimationIterationCount;
use crate::values::generics::box_::Perspective as GenericPerspective;
use crate::values::generics::box_::{GenericVerticalAlign, VerticalAlignKeyword};
use crate::values::specified::length::{LengthPercentage, NonNegativeLength};
use crate::values::specified::{AllowQuirks, Number};
use crate::values::{CustomIdent, KeyframesName};
use crate::Atom;
use cssparser::Parser;
use selectors::parser::SelectorParseErrorKind;
use std::fmt::{self, Write};
use style_traits::{CssWriter, KeywordsCollectFn, ParseError};
use style_traits::{SpecifiedValueInfo, StyleParseErrorKind, ToCss};

#[cfg(feature = "gecko")]
fn moz_display_values_enabled(context: &ParserContext) -> bool {
    use crate::gecko_bindings::structs;
    context.in_ua_or_chrome_sheet() ||
        unsafe { structs::StaticPrefs_sVarCache_layout_css_xul_display_values_content_enabled }
}

#[cfg(feature = "gecko")]
fn moz_box_display_values_enabled(context: &ParserContext) -> bool {
    use crate::gecko_bindings::structs;
    context.in_ua_or_chrome_sheet() ||
        unsafe {
            structs::StaticPrefs_sVarCache_layout_css_xul_box_display_values_content_enabled
        }
}

/// Defines an element’s display type, which consists of
/// the two basic qualities of how an element generates boxes
/// <https://drafts.csswg.org/css-display/#propdef-display>
///
///
/// NOTE(emilio): Order is important in Gecko!
///
/// If you change it, make sure to take a look at the
/// FrameConstructionDataByDisplay stuff (both the XUL and non-XUL version), and
/// ensure it's still correct!
#[allow(missing_docs)]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    FromPrimitive,
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
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[repr(u8)]
pub enum Display {
    None = 0,
    Block,
    #[cfg(feature = "gecko")]
    FlowRoot,
    Inline,
    InlineBlock,
    ListItem,
    Table,
    InlineTable,
    TableRowGroup,
    TableColumn,
    TableColumnGroup,
    TableHeaderGroup,
    TableFooterGroup,
    TableRow,
    TableCell,
    TableCaption,
    #[parse(aliases = "-webkit-flex")]
    Flex,
    #[parse(aliases = "-webkit-inline-flex")]
    InlineFlex,
    #[cfg(feature = "gecko")]
    Grid,
    #[cfg(feature = "gecko")]
    InlineGrid,
    #[cfg(feature = "gecko")]
    Ruby,
    #[cfg(feature = "gecko")]
    RubyBase,
    #[cfg(feature = "gecko")]
    RubyBaseContainer,
    #[cfg(feature = "gecko")]
    RubyText,
    #[cfg(feature = "gecko")]
    RubyTextContainer,
    #[cfg(feature = "gecko")]
    Contents,
    #[cfg(feature = "gecko")]
    WebkitBox,
    #[cfg(feature = "gecko")]
    WebkitInlineBox,
    #[cfg(feature = "gecko")]
    #[parse(condition = "moz_box_display_values_enabled")]
    MozBox,
    #[cfg(feature = "gecko")]
    #[parse(condition = "moz_box_display_values_enabled")]
    MozInlineBox,
    #[cfg(feature = "gecko")]
    #[parse(condition = "moz_display_values_enabled")]
    MozGrid,
    #[cfg(feature = "gecko")]
    #[parse(condition = "moz_display_values_enabled")]
    MozInlineGrid,
    #[cfg(feature = "gecko")]
    #[parse(condition = "moz_display_values_enabled")]
    MozGridGroup,
    #[cfg(feature = "gecko")]
    #[parse(condition = "moz_display_values_enabled")]
    MozGridLine,
    #[cfg(feature = "gecko")]
    #[parse(condition = "moz_display_values_enabled")]
    MozStack,
    #[cfg(feature = "gecko")]
    #[parse(condition = "moz_display_values_enabled")]
    MozInlineStack,
    #[cfg(feature = "gecko")]
    #[parse(condition = "moz_display_values_enabled")]
    MozDeck,
    #[cfg(feature = "gecko")]
    #[parse(condition = "moz_display_values_enabled")]
    MozGroupbox,
    #[cfg(feature = "gecko")]
    #[parse(condition = "moz_display_values_enabled")]
    MozPopup,
}

impl Display {
    /// The initial display value.
    #[inline]
    pub fn inline() -> Self {
        Display::Inline
    }

    /// Returns whether this "display" value is the display of a flex or
    /// grid container.
    ///
    /// This is used to implement various style fixups.
    pub fn is_item_container(&self) -> bool {
        match *self {
            Display::Flex | Display::InlineFlex => true,
            #[cfg(feature = "gecko")]
            Display::Grid | Display::InlineGrid => true,
            _ => false,
        }
    }

    /// Returns whether an element with this display type is a line
    /// participant, which means it may lay its children on the same
    /// line as itself.
    pub fn is_line_participant(&self) -> bool {
        match *self {
            Display::Inline => true,
            #[cfg(feature = "gecko")]
            Display::Contents | Display::Ruby | Display::RubyBaseContainer => true,
            _ => false,
        }
    }

    /// Returns whether this "display" value is one of the types for
    /// ruby.
    #[cfg(feature = "gecko")]
    pub fn is_ruby_type(&self) -> bool {
        matches!(
            *self,
            Display::Ruby |
                Display::RubyBase |
                Display::RubyText |
                Display::RubyBaseContainer |
                Display::RubyTextContainer
        )
    }

    /// Returns whether this "display" value is a ruby level container.
    #[cfg(feature = "gecko")]
    pub fn is_ruby_level_container(&self) -> bool {
        matches!(
            *self,
            Display::RubyBaseContainer | Display::RubyTextContainer
        )
    }

    /// Convert this display into an equivalent block display.
    ///
    /// Also used for style adjustments.
    pub fn equivalent_block_display(&self, _is_root_element: bool) -> Self {
        match *self {
            // Values that have a corresponding block-outside version.
            Display::InlineTable => Display::Table,
            Display::InlineFlex => Display::Flex,

            #[cfg(feature = "gecko")]
            Display::InlineGrid => Display::Grid,
            #[cfg(feature = "gecko")]
            Display::WebkitInlineBox => Display::WebkitBox,

            // Special handling for contents and list-item on the root
            // element for Gecko.
            #[cfg(feature = "gecko")]
            Display::Contents | Display::ListItem if _is_root_element => Display::Block,

            // These are not changed by blockification.
            Display::None | Display::Block | Display::Flex | Display::ListItem | Display::Table => {
                *self
            },

            #[cfg(feature = "gecko")]
            Display::Contents | Display::FlowRoot | Display::Grid | Display::WebkitBox => *self,

            // Everything else becomes block.
            _ => Display::Block,
        }
    }

    /// Convert this display into an inline-outside display.
    ///
    /// Ideally it should implement spec: https://drafts.csswg.org/css-display/#inlinify
    /// but the spec isn't stable enough, so we copy what Gecko does for now.
    #[cfg(feature = "gecko")]
    pub fn inlinify(&self) -> Self {
        match *self {
            Display::Block | Display::FlowRoot => Display::InlineBlock,
            Display::Table => Display::InlineTable,
            Display::Flex => Display::InlineFlex,
            Display::Grid => Display::InlineGrid,
            // XXX bug 1105868 this should probably be InlineListItem:
            Display::ListItem => Display::Inline,
            Display::MozBox => Display::MozInlineBox,
            Display::MozStack => Display::MozInlineStack,
            Display::WebkitBox => Display::WebkitInlineBox,
            other => other,
        }
    }

    /// Returns true if the value is `Contents`
    #[inline]
    pub fn is_contents(&self) -> bool {
        match *self {
            #[cfg(feature = "gecko")]
            Display::Contents => true,
            _ => false,
        }
    }

    /// Returns true if the value is `None`
    #[inline]
    pub fn is_none(&self) -> bool {
        *self == Display::None
    }
}

/// A specified value for the `vertical-align` property.
pub type VerticalAlign = GenericVerticalAlign<LengthPercentage>;

impl Parse for VerticalAlign {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(lp) = input.try(|i| LengthPercentage::parse_quirky(context, i, AllowQuirks::Yes))
        {
            return Ok(GenericVerticalAlign::Length(lp));
        }

        Ok(GenericVerticalAlign::Keyword(VerticalAlignKeyword::parse(
            input,
        )?))
    }
}

/// https://drafts.csswg.org/css-animations/#animation-iteration-count
pub type AnimationIterationCount = GenericAnimationIterationCount<Number>;

impl Parse for AnimationIterationCount {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut ::cssparser::Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input
            .try(|input| input.expect_ident_matching("infinite"))
            .is_ok()
        {
            return Ok(GenericAnimationIterationCount::Infinite);
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
#[derive(
    Clone,
    Debug,
    Eq,
    Hash,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[value_info(other_values = "none")]
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
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
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
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(name) = input.try(|input| KeyframesName::parse(context, input)) {
            return Ok(AnimationName(Some(name)));
        }

        input.expect_ident_matching("none")?;
        Ok(AnimationName(None))
    }
}

/// https://drafts.csswg.org/css-scroll-snap-1/#snap-axis
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
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
pub enum ScrollSnapAxis {
    X,
    Y,
    Block,
    Inline,
    Both,
}

/// https://drafts.csswg.org/css-scroll-snap-1/#snap-strictness
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
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
pub enum ScrollSnapStrictness {
    #[css(skip)]
    None, // Used to represent scroll-snap-type: none.  It's not parsed.
    Mandatory,
    Proximity,
}

/// https://drafts.csswg.org/css-scroll-snap-1/#scroll-snap-type
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct ScrollSnapType {
    axis: ScrollSnapAxis,
    strictness: ScrollSnapStrictness,
}

impl ScrollSnapType {
    /// Returns `none`.
    #[inline]
    pub fn none() -> Self {
        Self {
            axis: ScrollSnapAxis::Both,
            strictness: ScrollSnapStrictness::None,
        }
    }
}

impl Parse for ScrollSnapType {
    /// none | [ x | y | block | inline | both ] [ mandatory | proximity ]?
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input
            .try(|input| input.expect_ident_matching("none"))
            .is_ok()
        {
            return Ok(ScrollSnapType::none());
        }

        let axis = ScrollSnapAxis::parse(input)?;
        let strictness = input
            .try(ScrollSnapStrictness::parse)
            .unwrap_or(ScrollSnapStrictness::Proximity);
        Ok(Self { axis, strictness })
    }
}

impl ToCss for ScrollSnapType {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.strictness == ScrollSnapStrictness::None {
            return dest.write_str("none");
        }
        self.axis.to_css(dest)?;
        if self.strictness != ScrollSnapStrictness::Proximity {
            dest.write_str(" ")?;
            self.strictness.to_css(dest)?;
        }
        Ok(())
    }
}

/// Specified value of scroll-snap-align keyword value.
#[allow(missing_docs)]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    FromPrimitive,
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
pub enum ScrollSnapAlignKeyword {
    None,
    Start,
    End,
    Center,
}

/// https://drafts.csswg.org/css-scroll-snap-1/#scroll-snap-align
#[allow(missing_docs)]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct ScrollSnapAlign {
    block: ScrollSnapAlignKeyword,
    inline: ScrollSnapAlignKeyword,
}

impl ScrollSnapAlign {
    /// Returns `none`.
    #[inline]
    pub fn none() -> Self {
        ScrollSnapAlign {
            block: ScrollSnapAlignKeyword::None,
            inline: ScrollSnapAlignKeyword::None,
        }
    }
}

impl Parse for ScrollSnapAlign {
    /// [ none | start | end | center ]{1,2}
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<ScrollSnapAlign, ParseError<'i>> {
        let block = ScrollSnapAlignKeyword::parse(input)?;
        let inline = input.try(ScrollSnapAlignKeyword::parse).unwrap_or(block);
        Ok(ScrollSnapAlign { block, inline })
    }
}

impl ToCss for ScrollSnapAlign {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        self.block.to_css(dest)?;
        if self.block != self.inline {
            dest.write_str(" ")?;
            self.inline.to_css(dest)?;
        }
        Ok(())
    }
}

#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
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
pub enum OverscrollBehavior {
    Auto,
    Contain,
    None,
}

#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
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
pub enum OverflowAnchor {
    Auto,
    None,
}

#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
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
pub enum OverflowClipBox {
    PaddingBox,
    ContentBox,
}

#[derive(
    Clone,
    Debug,
    Default,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[css(comma)]
#[repr(C)]
/// Provides a rendering hint to the user agent, stating what kinds of changes
/// the author expects to perform on the element.
///
/// `auto` is represented by an empty `features` list.
///
/// <https://drafts.csswg.org/css-will-change/#will-change>
pub struct WillChange {
    /// The features that are supposed to change.
    ///
    /// TODO(emilio): Consider using ArcSlice since we just clone them from the
    /// specified value? That'd save an allocation, which could be worth it.
    #[css(iterable, if_empty = "auto")]
    features: crate::OwnedSlice<CustomIdent>,
    /// A bitfield with the kind of change that the value will create, based
    /// on the above field.
    #[css(skip)]
    bits: WillChangeBits,
}

impl WillChange {
    #[inline]
    /// Get default value of `will-change` as `auto`
    pub fn auto() -> Self {
        Self::default()
    }
}

bitflags! {
    /// The change bits that we care about.
    #[derive(Default, MallocSizeOf, SpecifiedValueInfo, ToComputedValue, ToResolvedValue, ToShmem)]
    #[repr(C)]
    pub struct WillChangeBits: u8 {
        /// Whether the stacking context will change.
        const STACKING_CONTEXT = 1 << 0;
        /// Whether `transform` will change.
        const TRANSFORM = 1 << 1;
        /// Whether `scroll-position` will change.
        const SCROLL = 1 << 2;
        /// Whether `opacity` will change.
        const OPACITY = 1 << 3;
        /// Fixed pos containing block.
        const FIXPOS_CB = 1 << 4;
        /// Abs pos containing block.
        const ABSPOS_CB = 1 << 5;
    }
}

fn change_bits_for_longhand(longhand: LonghandId) -> WillChangeBits {
    let mut flags = match longhand {
        LonghandId::Opacity => WillChangeBits::OPACITY,
        LonghandId::Transform => WillChangeBits::TRANSFORM,
        _ => WillChangeBits::empty(),
    };

    let property_flags = longhand.flags();
    if property_flags.contains(PropertyFlags::CREATES_STACKING_CONTEXT) {
        flags |= WillChangeBits::STACKING_CONTEXT;
    }
    if property_flags.contains(PropertyFlags::FIXPOS_CB) {
        flags |= WillChangeBits::FIXPOS_CB;
    }
    if property_flags.contains(PropertyFlags::ABSPOS_CB) {
        flags |= WillChangeBits::ABSPOS_CB;
    }
    flags
}

fn change_bits_for_maybe_property(ident: &str, context: &ParserContext) -> WillChangeBits {
    let id = match PropertyId::parse_ignoring_rule_type(ident, context) {
        Ok(id) => id,
        Err(..) => return WillChangeBits::empty(),
    };

    match id.as_shorthand() {
        Ok(shorthand) => shorthand
            .longhands()
            .fold(WillChangeBits::empty(), |flags, p| {
                flags | change_bits_for_longhand(p)
            }),
        Err(PropertyDeclarationId::Longhand(longhand)) => change_bits_for_longhand(longhand),
        Err(PropertyDeclarationId::Custom(..)) => WillChangeBits::empty(),
    }
}

impl Parse for WillChange {
    /// auto | <animateable-feature>#
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input
            .try(|input| input.expect_ident_matching("auto"))
            .is_ok()
        {
            return Ok(Self::default());
        }

        let mut bits = WillChangeBits::empty();
        let custom_idents = input.parse_comma_separated(|i| {
            let location = i.current_source_location();
            let parser_ident = i.expect_ident()?;
            let ident = CustomIdent::from_ident(
                location,
                parser_ident,
                &["will-change", "none", "all", "auto"],
            )?;

            if ident.0 == atom!("scroll-position") {
                bits |= WillChangeBits::SCROLL;
            } else {
                bits |= change_bits_for_maybe_property(&parser_ident, context);
            }
            Ok(ident)
        })?;

        Ok(Self {
            features: custom_idents.into(),
            bits,
        })
    }
}

bitflags! {
    /// Values for the `touch-action` property.
    #[derive(MallocSizeOf, SpecifiedValueInfo, ToComputedValue, ToResolvedValue, ToShmem)]
    /// These constants match Gecko's `NS_STYLE_TOUCH_ACTION_*` constants.
    #[value_info(other_values = "auto,none,manipulation,pan-x,pan-y")]
    #[repr(C)]
    pub struct TouchAction: u8 {
        /// `none` variant
        const NONE = 1 << 0;
        /// `auto` variant
        const AUTO = 1 << 1;
        /// `pan-x` variant
        const PAN_X = 1 << 2;
        /// `pan-y` variant
        const PAN_Y = 1 << 3;
        /// `manipulation` variant
        const MANIPULATION = 1 << 4;
    }
}

impl TouchAction {
    #[inline]
    /// Get default `touch-action` as `auto`
    pub fn auto() -> TouchAction {
        TouchAction::AUTO
    }
}

impl ToCss for TouchAction {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match *self {
            TouchAction::NONE => dest.write_str("none"),
            TouchAction::AUTO => dest.write_str("auto"),
            TouchAction::MANIPULATION => dest.write_str("manipulation"),
            _ if self.contains(TouchAction::PAN_X | TouchAction::PAN_Y) => {
                dest.write_str("pan-x pan-y")
            },
            _ if self.contains(TouchAction::PAN_X) => dest.write_str("pan-x"),
            _ if self.contains(TouchAction::PAN_Y) => dest.write_str("pan-y"),
            _ => panic!("invalid touch-action value"),
        }
    }
}

impl Parse for TouchAction {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<TouchAction, ParseError<'i>> {
        try_match_ident_ignore_ascii_case! { input,
            "auto" => Ok(TouchAction::AUTO),
            "none" => Ok(TouchAction::NONE),
            "manipulation" => Ok(TouchAction::MANIPULATION),
            "pan-x" => {
                if input.try(|i| i.expect_ident_matching("pan-y")).is_ok() {
                    Ok(TouchAction::PAN_X | TouchAction::PAN_Y)
                } else {
                    Ok(TouchAction::PAN_X)
                }
            },
            "pan-y" => {
                if input.try(|i| i.expect_ident_matching("pan-x")).is_ok() {
                    Ok(TouchAction::PAN_X | TouchAction::PAN_Y)
                } else {
                    Ok(TouchAction::PAN_Y)
                }
            },
        }
    }
}

bitflags! {
    #[derive(MallocSizeOf, SpecifiedValueInfo, ToComputedValue, ToResolvedValue, ToShmem)]
    #[value_info(other_values = "none,strict,content,size,layout,paint")]
    #[repr(C)]
    /// Constants for contain: https://drafts.csswg.org/css-contain/#contain-property
    pub struct Contain: u8 {
        /// `none` variant, just for convenience.
        const NONE = 0;
        /// 'size' variant, turns on size containment
        const SIZE = 1 << 0;
        /// `layout` variant, turns on layout containment
        const LAYOUT = 1 << 1;
        /// `paint` variant, turns on paint containment
        const PAINT = 1 << 2;
        /// `strict` variant, turns on all types of containment
        const STRICT = 1 << 3;
        /// 'content' variant, turns on layout and paint containment
        const CONTENT = 1 << 4;
        /// variant with all the bits that contain: strict turns on
        const STRICT_BITS = Contain::LAYOUT.bits | Contain::PAINT.bits | Contain::SIZE.bits;
        /// variant with all the bits that contain: content turns on
        const CONTENT_BITS = Contain::LAYOUT.bits | Contain::PAINT.bits;
    }
}

impl ToCss for Contain {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.is_empty() {
            return dest.write_str("none");
        }
        if self.contains(Contain::STRICT) {
            return dest.write_str("strict");
        }
        if self.contains(Contain::CONTENT) {
            return dest.write_str("content");
        }

        let mut has_any = false;
        macro_rules! maybe_write_value {
            ($ident:path => $str:expr) => {
                if self.contains($ident) {
                    if has_any {
                        dest.write_str(" ")?;
                    }
                    has_any = true;
                    dest.write_str($str)?;
                }
            };
        }
        maybe_write_value!(Contain::SIZE => "size");
        maybe_write_value!(Contain::LAYOUT => "layout");
        maybe_write_value!(Contain::PAINT => "paint");

        debug_assert!(has_any);
        Ok(())
    }
}

impl Parse for Contain {
    /// none | strict | content | [ size || layout || paint ]
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Contain, ParseError<'i>> {
        let mut result = Contain::empty();
        while let Ok(name) = input.try(|i| i.expect_ident_cloned()) {
            let flag = match_ignore_ascii_case! { &name,
                "size" => Some(Contain::SIZE),
                "layout" => Some(Contain::LAYOUT),
                "paint" => Some(Contain::PAINT),
                "strict" if result.is_empty() => return Ok(Contain::STRICT | Contain::STRICT_BITS),
                "content" if result.is_empty() => return Ok(Contain::CONTENT | Contain::CONTENT_BITS),
                "none" if result.is_empty() => return Ok(result),
                _ => None
            };

            let flag = match flag {
                Some(flag) if !result.contains(flag) => flag,
                _ => {
                    return Err(
                        input.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(name))
                    );
                },
            };
            result.insert(flag);
        }

        if !result.is_empty() {
            Ok(result)
        } else {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }
}

/// A specified value for the `perspective` property.
pub type Perspective = GenericPerspective<NonNegativeLength>;

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

#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(
    Clone, Copy, Debug, Eq, Hash, MallocSizeOf, Parse, PartialEq, SpecifiedValueInfo, ToCss, ToShmem,
)]
/// https://drafts.csswg.org/css-box/#propdef-float
pub enum Float {
    Left,
    Right,
    None,
    // https://drafts.csswg.org/css-logical-props/#float-clear
    InlineStart,
    InlineEnd,
}

#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(
    Clone, Copy, Debug, Eq, Hash, MallocSizeOf, Parse, PartialEq, SpecifiedValueInfo, ToCss, ToShmem,
)]
/// https://drafts.csswg.org/css-box/#propdef-clear
pub enum Clear {
    None,
    Left,
    Right,
    Both,
    // https://drafts.csswg.org/css-logical-props/#float-clear
    InlineStart,
    InlineEnd,
}

/// https://drafts.csswg.org/css-ui/#propdef-resize
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(
    Clone, Copy, Debug, Eq, Hash, MallocSizeOf, Parse, PartialEq, SpecifiedValueInfo, ToCss, ToShmem,
)]
pub enum Resize {
    None,
    Both,
    Horizontal,
    Vertical,
    // https://drafts.csswg.org/css-logical-1/#resize
    Inline,
    Block,
}

/// The value for the `appearance` property.
///
/// https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-appearance
#[allow(missing_docs)]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToCss,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum Appearance {
    /// No appearance at all.
    None,
    /// A typical dialog button.
    Button,
    /// Various arrows that go in buttons
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    ButtonArrowDown,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    ButtonArrowNext,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    ButtonArrowPrevious,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    ButtonArrowUp,
    /// A rectangular button that contains complex content
    /// like images (e.g. HTML <button> elements)
    ButtonBevel,
    /// The focus outline box inside of a button.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    ButtonFocus,
    /// The caret of a text area
    Caret,
    /// A dual toolbar button (e.g., a Back button with a dropdown)
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Dualbutton,
    /// A groupbox.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Groupbox,
    /// A inner-spin button.
    InnerSpinButton,
    /// List boxes.
    Listbox,
    /// A listbox item.
    Listitem,
    /// Menu Bar background
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Menubar,
    /// <menu> and <menuitem> appearances
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Menuitem,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Checkmenuitem,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Radiomenuitem,
    /// For text on non-iconic menuitems only
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Menuitemtext,
    /// A dropdown list.
    Menulist,
    /// The dropdown button(s) that open up a dropdown list.
    MenulistButton,
    /// The text part of a dropdown list, to left of button.
    MenulistText,
    /// An editable textfield with a dropdown list (a combobox).
    MenulistTextfield,
    /// Menu Popup background.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Menupopup,
    /// menu checkbox/radio appearances
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Menucheckbox,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Menuradio,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Menuseparator,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Menuarrow,
    /// An image in the menu gutter, like in bookmarks or history.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Menuimage,
    /// A horizontal meter bar.
    #[parse(aliases = "meterbar")]
    Meter,
    /// The meter bar's meter indicator.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Meterchunk,
    /// The "arrowed" part of the dropdown button that open up a dropdown list.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozMenulistButton,
    /// For HTML's <input type=number>
    NumberInput,
    /// A horizontal progress bar.
    #[parse(aliases = "progressbar")]
    ProgressBar,
    /// The progress bar's progress indicator
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Progresschunk,
    /// A vertical progress bar.
    ProgressbarVertical,
    /// A checkbox element.
    Checkbox,
    /// A radio element within a radio group.
    Radio,
    /// A generic container that always repaints on state changes. This is a
    /// hack to make XUL checkboxes and radio buttons work.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    CheckboxContainer,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    RadioContainer,
    /// The label part of a checkbox or radio button, used for painting a focus
    /// outline.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    CheckboxLabel,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    RadioLabel,
    /// nsRangeFrame and its subparts
    Range,
    RangeThumb,
    /// The resizer background area in a status bar for the resizer widget in
    /// the corner of a window.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Resizerpanel,
    /// The resizer itself.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Resizer,
    /// A slider.
    ScaleHorizontal,
    ScaleVertical,
    /// A slider's thumb.
    ScalethumbHorizontal,
    ScalethumbVertical,
    /// If the platform supports it, the left/right chunks of the slider thumb.
    Scalethumbstart,
    Scalethumbend,
    /// The ticks for a slider.
    Scalethumbtick,
    /// A scrollbar.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Scrollbar,
    /// A small scrollbar.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    ScrollbarSmall,
    /// The scrollbar slider
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    ScrollbarHorizontal,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    ScrollbarVertical,
    /// A scrollbar button (up/down/left/right).
    /// Keep these in order (some code casts these values to `int` in order to
    /// compare them against each other).
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    ScrollbarbuttonUp,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    ScrollbarbuttonDown,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    ScrollbarbuttonLeft,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    ScrollbarbuttonRight,
    /// The scrollbar thumb.
    ScrollbarthumbHorizontal,
    ScrollbarthumbVertical,
    /// The scrollbar track.
    ScrollbartrackHorizontal,
    ScrollbartrackVertical,
    /// The scroll corner
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Scrollcorner,
    /// A searchfield.
    Searchfield,
    /// A separator.  Can be horizontal or vertical.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Separator,
    /// A spin control (up/down control for time/date pickers).
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Spinner,
    /// The up button of a spin control.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    SpinnerUpbutton,
    /// The down button of a spin control.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    SpinnerDownbutton,
    /// The textfield of a spin control
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    SpinnerTextfield,
    /// A splitter.  Can be horizontal or vertical.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Splitter,
    /// A status bar in a main application window.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Statusbar,
    /// A single pane of a status bar.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Statusbarpanel,
    /// A single tab in a tab widget.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Tab,
    /// A single pane (inside the tabpanels container).
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Tabpanel,
    /// The tab panels container.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Tabpanels,
    /// The tabs scroll arrows (left/right).
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    TabScrollArrowBack,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    TabScrollArrowForward,
    /// A multi-line text field, e.g. HTML <textarea>.
    #[parse(aliases = "textfield-multiline")]
    Textarea,
    /// A single-line text field, e.g. HTML <input type=text>.
    Textfield,
    /// A toolbar in an application window.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Toolbar,
    /// A single toolbar button (with no associated dropdown).
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Toolbarbutton,
    /// The dropdown portion of a toolbar button
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    ToolbarbuttonDropdown,
    /// The gripper for a toolbar.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Toolbargripper,
    /// The toolbox that contains the toolbars.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Toolbox,
    /// A tooltip.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Tooltip,
    /// A listbox or tree widget header
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Treeheader,
    /// An individual header cell
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Treeheadercell,
    /// The sort arrow for a header.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Treeheadersortarrow,
    /// A tree item.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Treeitem,
    /// A tree widget branch line
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Treeline,
    /// A tree widget twisty.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Treetwisty,
    /// Open tree widget twisty.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Treetwistyopen,
    /// A tree widget.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Treeview,
    /// Window and dialog backgrounds.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Window,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Dialog,

    /// Vista Rebars.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozWinCommunicationsToolbox,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozWinMediaToolbox,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozWinBrowsertabbarToolbox,
    /// Vista glass.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozWinGlass,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozWinBorderlessGlass,
    /// -moz-apperance style used in setting proper glass margins.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozWinExcludeGlass,

    /// Titlebar elements on the Mac.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozMacFullscreenButton,
    /// Mac help button.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozMacHelpButton,

    /// Windows themed window frame elements.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozWindowButtonBox,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozWindowButtonBoxMaximized,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozWindowButtonClose,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozWindowButtonMaximize,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozWindowButtonMinimize,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozWindowButtonRestore,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozWindowFrameBottom,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozWindowFrameLeft,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozWindowFrameRight,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozWindowTitlebar,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozWindowTitlebarMaximized,

    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozGtkInfoBar,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozMacActiveSourceListSelection,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozMacDisclosureButtonClosed,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozMacDisclosureButtonOpen,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozMacSourceList,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozMacSourceListSelection,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozMacVibrancyDark,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozMacVibrancyLight,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozMacVibrantTitlebarDark,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozMacVibrantTitlebarLight,

    /// A non-disappearing scrollbar.
    #[css(skip)]
    ScrollbarNonDisappearing,

    /// A themed focus outline (for outline:auto).
    ///
    /// This isn't exposed to CSS at all, just here for convenience.
    #[css(skip)]
    FocusOutline,

    /// A dummy variant that should be last to let the GTK widget do hackery.
    #[css(skip)]
    Count,
}

/// A kind of break between two boxes.
///
/// https://drafts.csswg.org/css-break/#break-between
#[allow(missing_docs)]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToCss,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum BreakBetween {
    Always,
    Auto,
    Page,
    Avoid,
    Left,
    Right,
}

impl BreakBetween {
    /// Parse a legacy break-between value for `page-break-*`.
    ///
    /// See https://drafts.csswg.org/css-break/#page-break-properties.
    #[inline]
    pub fn parse_legacy<'i>(input: &mut Parser<'i, '_>) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        let ident = input.expect_ident()?;
        let break_value = match BreakBetween::from_ident(ident) {
            Ok(v) => v,
            Err(()) => {
                return Err(location
                    .new_custom_error(SelectorParseErrorKind::UnexpectedIdent(ident.clone())));
            },
        };
        match break_value {
            BreakBetween::Always => Ok(BreakBetween::Page),
            BreakBetween::Auto | BreakBetween::Avoid | BreakBetween::Left | BreakBetween::Right => {
                Ok(break_value)
            },
            BreakBetween::Page => {
                Err(location
                    .new_custom_error(SelectorParseErrorKind::UnexpectedIdent(ident.clone())))
            },
        }
    }

    /// Serialize a legacy break-between value for `page-break-*`.
    ///
    /// See https://drafts.csswg.org/css-break/#page-break-properties.
    pub fn to_css_legacy<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match *self {
            BreakBetween::Auto | BreakBetween::Avoid | BreakBetween::Left | BreakBetween::Right => {
                self.to_css(dest)
            },
            BreakBetween::Page => dest.write_str("always"),
            BreakBetween::Always => Ok(()),
        }
    }
}

/// A kind of break within a box.
///
/// https://drafts.csswg.org/css-break/#break-within
#[allow(missing_docs)]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToCss,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum BreakWithin {
    Auto,
    Avoid,
}

/// The value for the `overflow-x` / `overflow-y` properties.
#[allow(missing_docs)]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToCss,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum Overflow {
    Visible,
    Hidden,
    Scroll,
    Auto,
    #[cfg(feature = "gecko")]
    MozHiddenUnscrollable,
}
