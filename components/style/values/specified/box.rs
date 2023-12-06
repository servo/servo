/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for box properties.

use crate::parser::{Parse, ParserContext};
#[cfg(feature = "gecko")]
use crate::properties::{LonghandId, PropertyDeclarationId, PropertyId};
use crate::values::generics::box_::{
    GenericContainIntrinsicSize, GenericLineClamp, GenericPerspective, GenericVerticalAlign,
    VerticalAlignKeyword,
};
use crate::values::specified::length::{LengthPercentage, NonNegativeLength};
use crate::values::specified::{AllowQuirks, Integer};
use crate::values::CustomIdent;
use cssparser::Parser;
use num_traits::FromPrimitive;
use std::fmt::{self, Debug, Formatter, Write};
use style_traits::{CssWriter, KeywordsCollectFn, ParseError};
use style_traits::{SpecifiedValueInfo, StyleParseErrorKind, ToCss};

#[cfg(not(feature = "servo"))]
fn flexbox_enabled() -> bool {
    true
}

#[cfg(feature = "servo")]
fn flexbox_enabled() -> bool {
    servo_config::prefs::pref_map()
        .get("layout.flexbox.enabled")
        .as_bool()
        .unwrap_or(false)
}

/// Defines an element’s display type, which consists of
/// the two basic qualities of how an element generates boxes
/// <https://drafts.csswg.org/css-display/#propdef-display>
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Eq, FromPrimitive, Hash, MallocSizeOf, PartialEq, ToCss, ToShmem)]
#[repr(u8)]
pub enum DisplayOutside {
    None = 0,
    Inline,
    Block,
    TableCaption,
    InternalTable,
    #[cfg(feature = "gecko")]
    InternalRuby,
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Eq, FromPrimitive, Hash, MallocSizeOf, PartialEq, ToCss, ToShmem)]
#[repr(u8)]
pub enum DisplayInside {
    None = 0,
    Contents,
    Flow,
    FlowRoot,
    Flex,
    #[cfg(feature = "gecko")]
    Grid,
    Table,
    TableRowGroup,
    TableColumn,
    TableColumnGroup,
    TableHeaderGroup,
    TableFooterGroup,
    TableRow,
    TableCell,
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
    WebkitBox,
}

#[allow(missing_docs)]
#[derive(
    Clone,
    Copy,
    Eq,
    FromPrimitive,
    Hash,
    MallocSizeOf,
    PartialEq,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(transparent)]
pub struct Display(u16);

/// Gecko-only impl block for Display (shared stuff later in this file):
#[allow(missing_docs)]
#[allow(non_upper_case_globals)]
impl Display {
    // Our u16 bits are used as follows:    LOOOOOOOIIIIIIII
    const LIST_ITEM_BIT: u16 = 0x8000; //^
    const DISPLAY_OUTSIDE_BITS: u16 = 7; // ^^^^^^^
    const DISPLAY_INSIDE_BITS: u16 = 8; //        ^^^^^^^^

    /// https://drafts.csswg.org/css-display/#the-display-properties
    pub const None: Self = Self::new(DisplayOutside::None, DisplayInside::None);
    pub const Contents: Self = Self::new(DisplayOutside::None, DisplayInside::Contents);
    pub const Inline: Self = Self::new(DisplayOutside::Inline, DisplayInside::Flow);
    pub const InlineBlock: Self = Self::new(DisplayOutside::Inline, DisplayInside::FlowRoot);
    pub const Block: Self = Self::new(DisplayOutside::Block, DisplayInside::Flow);
    #[cfg(feature = "gecko")]
    pub const FlowRoot: Self = Self::new(DisplayOutside::Block, DisplayInside::FlowRoot);
    pub const Flex: Self = Self::new(DisplayOutside::Block, DisplayInside::Flex);
    pub const InlineFlex: Self = Self::new(DisplayOutside::Inline, DisplayInside::Flex);
    #[cfg(feature = "gecko")]
    pub const Grid: Self = Self::new(DisplayOutside::Block, DisplayInside::Grid);
    #[cfg(feature = "gecko")]
    pub const InlineGrid: Self = Self::new(DisplayOutside::Inline, DisplayInside::Grid);
    pub const Table: Self = Self::new(DisplayOutside::Block, DisplayInside::Table);
    pub const InlineTable: Self = Self::new(DisplayOutside::Inline, DisplayInside::Table);
    pub const TableCaption: Self = Self::new(DisplayOutside::TableCaption, DisplayInside::Flow);
    #[cfg(feature = "gecko")]
    pub const Ruby: Self = Self::new(DisplayOutside::Inline, DisplayInside::Ruby);
    #[cfg(feature = "gecko")]
    pub const WebkitBox: Self = Self::new(DisplayOutside::Block, DisplayInside::WebkitBox);
    #[cfg(feature = "gecko")]
    pub const WebkitInlineBox: Self = Self::new(DisplayOutside::Inline, DisplayInside::WebkitBox);

    // Internal table boxes.

    pub const TableRowGroup: Self =
        Self::new(DisplayOutside::InternalTable, DisplayInside::TableRowGroup);

    pub const TableHeaderGroup: Self = Self::new(
        DisplayOutside::InternalTable,
        DisplayInside::TableHeaderGroup,
    );

    pub const TableFooterGroup: Self = Self::new(
        DisplayOutside::InternalTable,
        DisplayInside::TableFooterGroup,
    );

    pub const TableColumn: Self =
        Self::new(DisplayOutside::InternalTable, DisplayInside::TableColumn);

    pub const TableColumnGroup: Self = Self::new(
        DisplayOutside::InternalTable,
        DisplayInside::TableColumnGroup,
    );

    pub const TableRow: Self = Self::new(DisplayOutside::InternalTable, DisplayInside::TableRow);

    pub const TableCell: Self = Self::new(DisplayOutside::InternalTable, DisplayInside::TableCell);

    /// Internal ruby boxes.
    #[cfg(feature = "gecko")]
    pub const RubyBase: Self = Self::new(DisplayOutside::InternalRuby, DisplayInside::RubyBase);
    #[cfg(feature = "gecko")]
    pub const RubyBaseContainer: Self = Self::new(
        DisplayOutside::InternalRuby,
        DisplayInside::RubyBaseContainer,
    );
    #[cfg(feature = "gecko")]
    pub const RubyText: Self = Self::new(DisplayOutside::InternalRuby, DisplayInside::RubyText);
    #[cfg(feature = "gecko")]
    pub const RubyTextContainer: Self = Self::new(
        DisplayOutside::InternalRuby,
        DisplayInside::RubyTextContainer,
    );

    /// Make a raw display value from <display-outside> and <display-inside> values.
    #[inline]
    const fn new(outside: DisplayOutside, inside: DisplayInside) -> Self {
        let o: u16 = ((outside as u8) as u16) << Self::DISPLAY_INSIDE_BITS;
        let i: u16 = (inside as u8) as u16;
        Self(o | i)
    }

    /// Make a display enum value from <display-outside> and <display-inside> values.
    #[inline]
    fn from3(outside: DisplayOutside, inside: DisplayInside, list_item: bool) -> Self {
        let v = Self::new(outside, inside);
        if !list_item {
            return v;
        }
        Self(v.0 | Self::LIST_ITEM_BIT)
    }

    /// Accessor for the <display-inside> value.
    #[inline]
    pub fn inside(&self) -> DisplayInside {
        DisplayInside::from_u16(self.0 & ((1 << Self::DISPLAY_INSIDE_BITS) - 1)).unwrap()
    }

    /// Accessor for the <display-outside> value.
    #[inline]
    pub fn outside(&self) -> DisplayOutside {
        DisplayOutside::from_u16(
            (self.0 >> Self::DISPLAY_INSIDE_BITS) & ((1 << Self::DISPLAY_OUTSIDE_BITS) - 1),
        )
        .unwrap()
    }

    /// Returns the raw underlying u16 value.
    #[inline]
    pub const fn to_u16(&self) -> u16 {
        self.0
    }

    /// Whether this is `display: inline` (or `inline list-item`).
    #[inline]
    pub fn is_inline_flow(&self) -> bool {
        self.outside() == DisplayOutside::Inline && self.inside() == DisplayInside::Flow
    }

    /// Returns whether this `display` value is some kind of list-item.
    #[inline]
    pub const fn is_list_item(&self) -> bool {
        (self.0 & Self::LIST_ITEM_BIT) != 0
    }

    /// Returns whether this `display` value is a ruby level container.
    pub fn is_ruby_level_container(&self) -> bool {
        match *self {
            #[cfg(feature = "gecko")]
            Display::RubyBaseContainer | Display::RubyTextContainer => true,
            _ => false,
        }
    }

    /// Returns whether this `display` value is one of the types for ruby.
    pub fn is_ruby_type(&self) -> bool {
        match self.inside() {
            #[cfg(feature = "gecko")]
            DisplayInside::Ruby |
            DisplayInside::RubyBase |
            DisplayInside::RubyText |
            DisplayInside::RubyBaseContainer |
            DisplayInside::RubyTextContainer => true,
            _ => false,
        }
    }
}

/// Shared Display impl for both Gecko and Servo.
#[allow(non_upper_case_globals)]
impl Display {
    /// The initial display value.
    #[inline]
    pub fn inline() -> Self {
        Display::Inline
    }

    /// <https://drafts.csswg.org/css2/visuren.html#x13>
    #[cfg(feature = "servo")]
    #[inline]
    pub fn is_atomic_inline_level(&self) -> bool {
        match *self {
            Display::InlineBlock | Display::InlineFlex => true,
            Display::InlineTable => true,
            _ => false,
        }
    }

    /// Returns whether this `display` value is the display of a flex or
    /// grid container.
    ///
    /// This is used to implement various style fixups.
    pub fn is_item_container(&self) -> bool {
        match self.inside() {
            DisplayInside::Flex => true,
            #[cfg(feature = "gecko")]
            DisplayInside::Grid => true,
            _ => false,
        }
    }

    /// Returns whether an element with this display type is a line
    /// participant, which means it may lay its children on the same
    /// line as itself.
    pub fn is_line_participant(&self) -> bool {
        if self.is_inline_flow() {
            return true;
        }
        match *self {
            #[cfg(feature = "gecko")]
            Display::Contents | Display::Ruby | Display::RubyBaseContainer => true,
            _ => false,
        }
    }

    /// Convert this display into an equivalent block display.
    ///
    /// Also used for :root style adjustments.
    pub fn equivalent_block_display(&self, _is_root_element: bool) -> Self {
        {
            // Special handling for `contents` and `list-item`s on the root element.
            if _is_root_element && (self.is_contents() || self.is_list_item()) {
                return Display::Block;
            }
        }

        match self.outside() {
            DisplayOutside::Inline => {
                let inside = match self.inside() {
                    // `inline-block` blockifies to `block` rather than
                    // `flow-root`, for legacy reasons.
                    DisplayInside::FlowRoot => DisplayInside::Flow,
                    inside => inside,
                };
                Display::from3(DisplayOutside::Block, inside, self.is_list_item())
            },
            DisplayOutside::Block | DisplayOutside::None => *self,
            _ => Display::Block,
        }
    }

    /// Convert this display into an equivalent inline-outside display.
    /// https://drafts.csswg.org/css-display/#inlinify
    #[cfg(feature = "gecko")]
    pub fn inlinify(&self) -> Self {
        match self.outside() {
            DisplayOutside::Block => {
                let inside = match self.inside() {
                    // `display: block` inlinifies to `display: inline-block`,
                    // rather than `inline`, for legacy reasons.
                    DisplayInside::Flow => DisplayInside::FlowRoot,
                    inside => inside,
                };
                Display::from3(DisplayOutside::Inline, inside, self.is_list_item())
            },
            _ => *self,
        }
    }

    /// Returns true if the value is `Contents`
    #[inline]
    pub fn is_contents(&self) -> bool {
        match *self {
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

impl ToCss for Display {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        let outside = self.outside();
        let inside = self.inside();
        match *self {
            Display::Block | Display::Inline => outside.to_css(dest),
            Display::InlineBlock => dest.write_str("inline-block"),
            #[cfg(feature = "gecko")]
            Display::WebkitInlineBox => dest.write_str("-webkit-inline-box"),
            Display::TableCaption => dest.write_str("table-caption"),
            _ => match (outside, inside) {
                #[cfg(feature = "gecko")]
                (DisplayOutside::Inline, DisplayInside::Grid) => dest.write_str("inline-grid"),
                (DisplayOutside::Inline, DisplayInside::Flex) => dest.write_str("inline-flex"),
                (DisplayOutside::Inline, DisplayInside::Table) => dest.write_str("inline-table"),
                #[cfg(feature = "gecko")]
                (DisplayOutside::Block, DisplayInside::Ruby) => dest.write_str("block ruby"),
                (_, inside) => {
                    if self.is_list_item() {
                        if outside != DisplayOutside::Block {
                            outside.to_css(dest)?;
                            dest.write_char(' ')?;
                        }
                        if inside != DisplayInside::Flow {
                            inside.to_css(dest)?;
                            dest.write_char(' ')?;
                        }
                        dest.write_str("list-item")
                    } else {
                        inside.to_css(dest)
                    }
                },
            },
        }
    }
}

/// <display-inside> = flow | flow-root | table | flex | grid | ruby
/// https://drafts.csswg.org/css-display/#typedef-display-inside
fn parse_display_inside<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<DisplayInside, ParseError<'i>> {
    Ok(try_match_ident_ignore_ascii_case! { input,
        "flow" => DisplayInside::Flow,
        "flex" if flexbox_enabled() => DisplayInside::Flex,
        "flow-root" => DisplayInside::FlowRoot,
        "table" => DisplayInside::Table,
        #[cfg(feature = "gecko")]
        "grid" => DisplayInside::Grid,
        #[cfg(feature = "gecko")]
        "ruby" => DisplayInside::Ruby,
    })
}

/// <display-outside> = block | inline | run-in
/// https://drafts.csswg.org/css-display/#typedef-display-outside
fn parse_display_outside<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<DisplayOutside, ParseError<'i>> {
    Ok(try_match_ident_ignore_ascii_case! { input,
        "block" => DisplayOutside::Block,
        "inline" => DisplayOutside::Inline,
        // FIXME(bug 2056): not supported in layout yet:
        //"run-in" => DisplayOutside::RunIn,
    })
}

/// (flow | flow-root)?
fn parse_display_inside_for_list_item<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<DisplayInside, ParseError<'i>> {
    Ok(try_match_ident_ignore_ascii_case! { input,
        "flow" => DisplayInside::Flow,
        #[cfg(feature = "gecko")]
        "flow-root" => DisplayInside::FlowRoot,
    })
}
/// Test a <display-inside> Result for same values as above.
fn is_valid_inside_for_list_item<'i>(inside: &Result<DisplayInside, ParseError<'i>>) -> bool {
    match inside {
        Ok(DisplayInside::Flow) => true,
        #[cfg(feature = "gecko")]
        Ok(DisplayInside::FlowRoot) => true,
        _ => false,
    }
}

/// Parse `list-item`.
fn parse_list_item<'i, 't>(input: &mut Parser<'i, 't>) -> Result<(), ParseError<'i>> {
    Ok(input.expect_ident_matching("list-item")?)
}

impl Parse for Display {
    #[allow(unused)] // `context` isn't used for servo-2020 for now
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Display, ParseError<'i>> {
        // Parse all combinations of <display-inside/outside>? and `list-item`? first.
        let mut got_list_item = input.try_parse(parse_list_item).is_ok();
        let mut inside = if got_list_item {
            input.try_parse(parse_display_inside_for_list_item)
        } else {
            input.try_parse(parse_display_inside)
        };
        // <display-listitem> = <display-outside>? && [ flow | flow-root ]? && list-item
        // https://drafts.csswg.org/css-display/#typedef-display-listitem
        if !got_list_item && is_valid_inside_for_list_item(&inside) {
            got_list_item = input.try_parse(parse_list_item).is_ok();
        }
        let outside = input.try_parse(parse_display_outside);
        if outside.is_ok() {
            if !got_list_item && (inside.is_err() || is_valid_inside_for_list_item(&inside)) {
                got_list_item = input.try_parse(parse_list_item).is_ok();
            }
            if inside.is_err() {
                inside = if got_list_item {
                    input.try_parse(parse_display_inside_for_list_item)
                } else {
                    input.try_parse(parse_display_inside)
                };
                if !got_list_item && is_valid_inside_for_list_item(&inside) {
                    got_list_item = input.try_parse(parse_list_item).is_ok();
                }
            }
        }
        if got_list_item || inside.is_ok() || outside.is_ok() {
            let inside = inside.unwrap_or(DisplayInside::Flow);
            let outside = outside.unwrap_or(match inside {
                // "If <display-outside> is omitted, the element’s outside display type
                // defaults to block — except for ruby, which defaults to inline."
                // https://drafts.csswg.org/css-display/#inside-model
                #[cfg(feature = "gecko")]
                DisplayInside::Ruby => DisplayOutside::Inline,
                _ => DisplayOutside::Block,
            });
            return Ok(Display::from3(outside, inside, got_list_item));
        }

        // Now parse the single-keyword `display` values.
        Ok(try_match_ident_ignore_ascii_case! { input,
            "none" => Display::None,
            "contents" => Display::Contents,
            "inline-block" => Display::InlineBlock,
            "inline-table" => Display::InlineTable,
            "-webkit-flex" if flexbox_enabled() => Display::Flex,
            "inline-flex" | "-webkit-inline-flex" if flexbox_enabled() => Display::InlineFlex,
            #[cfg(feature = "gecko")]
            "inline-grid" => Display::InlineGrid,
            "table-caption" => Display::TableCaption,
            "table-row-group" => Display::TableRowGroup,
            "table-header-group" => Display::TableHeaderGroup,
            "table-footer-group" => Display::TableFooterGroup,
            "table-column" => Display::TableColumn,
            "table-column-group" => Display::TableColumnGroup,
            "table-row" => Display::TableRow,
            "table-cell" => Display::TableCell,
            #[cfg(feature = "gecko")]
            "ruby-base" => Display::RubyBase,
            #[cfg(feature = "gecko")]
            "ruby-base-container" => Display::RubyBaseContainer,
            #[cfg(feature = "gecko")]
            "ruby-text" => Display::RubyText,
            #[cfg(feature = "gecko")]
            "ruby-text-container" => Display::RubyTextContainer,
            #[cfg(feature = "gecko")]
            "-webkit-box" => Display::WebkitBox,
            #[cfg(feature = "gecko")]
            "-webkit-inline-box" => Display::WebkitInlineBox,
        })
    }
}

impl SpecifiedValueInfo for Display {
    fn collect_completion_keywords(f: KeywordsCollectFn) {
        f(&[
            "block",
            "contents",
            "flex",
            "flow-root",
            "flow-root list-item",
            "grid",
            "inline",
            "inline-block",
            "inline-flex",
            "inline-grid",
            "inline-table",
            "inline list-item",
            "inline flow-root list-item",
            "list-item",
            "none",
            "block ruby",
            "ruby",
            "ruby-base",
            "ruby-base-container",
            "ruby-text",
            "ruby-text-container",
            "table",
            "table-caption",
            "table-cell",
            "table-column",
            "table-column-group",
            "table-footer-group",
            "table-header-group",
            "table-row",
            "table-row-group",
            "-webkit-box",
            "-webkit-inline-box",
        ]);
    }
}

impl Debug for Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Display")
            .field("List Item", &self.is_list_item())
            .field("Inside", &self.inside())
            .field("Outside", &self.outside())
            .finish()
    }
}

/// A specified value for the `contain-intrinsic-size` property.
pub type ContainIntrinsicSize = GenericContainIntrinsicSize<NonNegativeLength>;

/// A specified value for the `line-clamp` property.
pub type LineClamp = GenericLineClamp<Integer>;

/// A specified value for the `vertical-align` property.
pub type VerticalAlign = GenericVerticalAlign<LengthPercentage>;

impl Parse for VerticalAlign {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(lp) =
            input.try_parse(|i| LengthPercentage::parse_quirky(context, i, AllowQuirks::Yes))
        {
            return Ok(GenericVerticalAlign::Length(lp));
        }

        Ok(GenericVerticalAlign::Keyword(VerticalAlignKeyword::parse(
            input,
        )?))
    }
}

/// A specified value for the `baseline-source` property.
/// https://drafts.csswg.org/css-inline-3/#baseline-source
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
    ToShmem,
    ToComputedValue,
    ToResolvedValue,
)]
#[repr(u8)]
pub enum BaselineSource {
    /// `Last` for `inline-block`, `First` otherwise.
    Auto,
    /// Use first baseline for alignment.
    First,
    /// Use last baseline for alignment.
    Last,
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
            .try_parse(|input| input.expect_ident_matching("none"))
            .is_ok()
        {
            return Ok(ScrollSnapType::none());
        }

        let axis = ScrollSnapAxis::parse(input)?;
        let strictness = input
            .try_parse(ScrollSnapStrictness::parse)
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
            dest.write_char(' ')?;
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
        let inline = input
            .try_parse(ScrollSnapAlignKeyword::parse)
            .unwrap_or(block);
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
            dest.write_char(' ')?;
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
pub enum ScrollSnapStop {
    Normal,
    Always,
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
    pub struct WillChangeBits: u16 {
        /// Whether a property which can create a stacking context **on any
        /// box** will change.
        const STACKING_CONTEXT_UNCONDITIONAL = 1 << 0;
        /// Whether `transform` or related properties will change.
        const TRANSFORM = 1 << 1;
        /// Whether `scroll-position` will change.
        const SCROLL = 1 << 2;
        /// Whether `contain` will change.
        const CONTAIN = 1 << 3;
        /// Whether `opacity` will change.
        const OPACITY = 1 << 4;
        /// Whether `perspective` will change.
        const PERSPECTIVE = 1 << 5;
        /// Whether `z-index` will change.
        const Z_INDEX = 1 << 6;
        /// Whether any property which creates a containing block for non-svg
        /// text frames will change.
        const FIXPOS_CB_NON_SVG = 1 << 7;
        /// Whether the position property will change.
        const POSITION = 1 << 8;
    }
}

#[cfg(feature = "gecko")]
fn change_bits_for_longhand(longhand: LonghandId) -> WillChangeBits {
    match longhand {
        LonghandId::Opacity => WillChangeBits::OPACITY,
        LonghandId::Contain => WillChangeBits::CONTAIN,
        LonghandId::Perspective => WillChangeBits::PERSPECTIVE,
        LonghandId::Position => {
            WillChangeBits::STACKING_CONTEXT_UNCONDITIONAL | WillChangeBits::POSITION
        },
        LonghandId::ZIndex => WillChangeBits::Z_INDEX,
        LonghandId::Transform |
        LonghandId::TransformStyle |
        LonghandId::Translate |
        LonghandId::Rotate |
        LonghandId::Scale |
        LonghandId::OffsetPath => WillChangeBits::TRANSFORM,
        LonghandId::BackdropFilter | LonghandId::Filter => {
            WillChangeBits::STACKING_CONTEXT_UNCONDITIONAL | WillChangeBits::FIXPOS_CB_NON_SVG
        },
        LonghandId::MixBlendMode |
        LonghandId::Isolation |
        LonghandId::MaskImage |
        LonghandId::ClipPath => WillChangeBits::STACKING_CONTEXT_UNCONDITIONAL,
        _ => WillChangeBits::empty(),
    }
}

#[cfg(feature = "gecko")]
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

#[cfg(feature = "gecko")]
impl Parse for WillChange {
    /// auto | <animateable-feature>#
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input
            .try_parse(|input| input.expect_ident_matching("auto"))
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

            if context.in_ua_sheet() && ident.0 == atom!("-moz-fixed-pos-containing-block") {
                bits |= WillChangeBits::FIXPOS_CB_NON_SVG;
            } else if ident.0 == atom!("scroll-position") {
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
    #[derive(MallocSizeOf, SpecifiedValueInfo, ToComputedValue, ToCss, ToResolvedValue, ToShmem, Parse)]
    #[css(bitflags(single = "none,auto,manipulation", mixed = "pan-x,pan-y,pinch-zoom"))]
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
        /// `pinch-zoom` variant
        const PINCH_ZOOM = 1 << 5;
    }
}

impl TouchAction {
    #[inline]
    /// Get default `touch-action` as `auto`
    pub fn auto() -> TouchAction {
        TouchAction::AUTO
    }
}

bitflags! {
    #[derive(MallocSizeOf, Parse, SpecifiedValueInfo, ToComputedValue, ToCss, ToResolvedValue, ToShmem)]
    #[css(bitflags(single = "none,strict,content", mixed="size,layout,style,paint,inline-size", overlapping_bits))]
    #[repr(C)]
    /// Constants for contain: https://drafts.csswg.org/css-contain/#contain-property
    pub struct Contain: u8 {
        /// `none` variant, just for convenience.
        const NONE = 0;
        /// `inline-size` variant, turns on single-axis inline size containment
        const INLINE_SIZE = 1 << 0;
        /// `block-size` variant, turns on single-axis block size containment, internal only
        const BLOCK_SIZE = 1 << 1;
        /// `layout` variant, turns on layout containment
        const LAYOUT = 1 << 2;
        /// `style` variant, turns on style containment
        const STYLE = 1 << 3;
        /// `paint` variant, turns on paint containment
        const PAINT = 1 << 4;
        /// 'size' variant, turns on size containment
        const SIZE = 1 << 5 | Contain::INLINE_SIZE.bits | Contain::BLOCK_SIZE.bits;
        /// `content` variant, turns on layout and paint containment
        const CONTENT = 1 << 6 | Contain::LAYOUT.bits | Contain::STYLE.bits | Contain::PAINT.bits;
        /// `strict` variant, turns on all types of containment
        const STRICT = 1 << 7 | Contain::LAYOUT.bits | Contain::STYLE.bits | Contain::PAINT.bits | Contain::SIZE.bits;
    }
}

impl Parse for ContainIntrinsicSize {
    /// none | <length> | auto <length>
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(l) = input.try_parse(|i| NonNegativeLength::parse(context, i)) {
            return Ok(Self::Length(l));
        }

        if input.try_parse(|i| i.expect_ident_matching("auto")).is_ok() {
            let l = NonNegativeLength::parse(context, input)?;
            return Ok(Self::AutoLength(l));
        }

        input.expect_ident_matching("none")?;
        Ok(Self::None)
    }
}

impl Parse for LineClamp {
    /// none | <positive-integer>
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(i) =
            input.try_parse(|i| crate::values::specified::PositiveInteger::parse(context, i))
        {
            return Ok(Self(i.0));
        }
        input.expect_ident_matching("none")?;
        Ok(Self::none())
    }
}

/// https://drafts.csswg.org/css-contain-2/#content-visibility
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
pub enum ContentVisibility {
    /// `auto` variant, the element turns on layout containment, style containment, and paint
    /// containment. In addition, if the element is not relevant to the user (such as by being
    /// offscreen) it also skips its content
    Auto,
    /// `hidden` variant, the element skips its content
    Hidden,
    /// 'visible' variant, no effect
    Visible,
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    MallocSizeOf,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    Parse,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
#[allow(missing_docs)]
/// https://drafts.csswg.org/css-contain-3/#container-type
pub enum ContainerType {
    /// The `normal` variant.
    Normal,
    /// The `inline-size` variant.
    InlineSize,
    /// The `size` variant.
    Size,
}

impl ContainerType {
    /// Is this container-type: normal?
    pub fn is_normal(self) -> bool {
        self == Self::Normal
    }

    /// Is this type containing size in any way?
    pub fn is_size_container_type(self) -> bool {
        !self.is_normal()
    }
}

/// https://drafts.csswg.org/css-contain-3/#container-name
#[repr(transparent)]
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
pub struct ContainerName(#[css(iterable, if_empty = "none")] pub crate::OwnedSlice<CustomIdent>);

impl ContainerName {
    /// Return the `none` value.
    pub fn none() -> Self {
        Self(Default::default())
    }

    /// Returns whether this is the `none` value.
    pub fn is_none(&self) -> bool {
        self.0.is_empty()
    }

    fn parse_internal<'i>(
        input: &mut Parser<'i, '_>,
        for_query: bool,
    ) -> Result<Self, ParseError<'i>> {
        let mut idents = vec![];
        let location = input.current_source_location();
        let first = input.expect_ident()?;
        if !for_query && first.eq_ignore_ascii_case("none") {
            return Ok(Self::none());
        }
        const DISALLOWED_CONTAINER_NAMES: &'static [&'static str] = &["none", "not", "or", "and"];
        idents.push(CustomIdent::from_ident(
            location,
            first,
            DISALLOWED_CONTAINER_NAMES,
        )?);
        if !for_query {
            while let Ok(name) = input.try_parse(|input| {
                let ident = input.expect_ident()?;
                CustomIdent::from_ident(location, &ident, DISALLOWED_CONTAINER_NAMES)
            }) {
                idents.push(name);
            }
        }
        Ok(ContainerName(idents.into()))
    }

    /// https://github.com/w3c/csswg-drafts/issues/7203
    /// Only a single name allowed in @container rule.
    /// Disallow none for container-name in @container rule.
    pub fn parse_for_query<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_internal(input, /* for_query = */ true)
    }
}

impl Parse for ContainerName {
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_internal(input, /* for_query = */ false)
    }
}

/// A specified value for the `perspective` property.
pub type Perspective = GenericPerspective<NonNegativeLength>;

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
/// https://drafts.csswg.org/css2/#propdef-clear
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
    /// Default appearance for the element.
    ///
    /// This value doesn't make sense for -moz-default-appearance, but we don't bother to guard
    /// against parsing it.
    Auto,
    /// A searchfield.
    Searchfield,
    /// A multi-line text field, e.g. HTML <textarea>.
    Textarea,
    /// A checkbox element.
    Checkbox,
    /// A radio element within a radio group.
    Radio,
    /// A dropdown list.
    Menulist,
    /// List boxes.
    Listbox,
    /// A horizontal meter bar.
    Meter,
    /// A horizontal progress bar.
    ProgressBar,
    /// A typical dialog button.
    Button,
    /// A single-line text field, e.g. HTML <input type=text>.
    Textfield,
    /// The dropdown button(s) that open up a dropdown list.
    MenulistButton,
    /// Various arrows that go in buttons
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    ButtonArrowDown,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    ButtonArrowNext,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    ButtonArrowPrevious,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    ButtonArrowUp,
    /// A dual toolbar button (e.g., a Back button with a dropdown)
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Dualbutton,
    /// A groupbox.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Groupbox,
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
    /// The text part of a dropdown list, to left of button.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MenulistText,
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
    /// The meter bar's meter indicator.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Meterchunk,
    /// The "arrowed" part of the dropdown button that open up a dropdown list.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozMenulistArrowButton,
    /// For HTML's <input type=number>
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    NumberInput,
    /// The progress bar's progress indicator
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Progresschunk,
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
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Range,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    RangeThumb,
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
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    ScrollbarthumbHorizontal,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    ScrollbarthumbVertical,
    /// The scrollbar track.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    ScrollbartrackHorizontal,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    ScrollbartrackVertical,
    /// The scroll corner
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    Scrollcorner,
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
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozWinBorderlessGlass,
    /// -moz-apperance style used in setting proper glass margins.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozWinExcludeGlass,

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
    MozWindowTitlebar,
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozWindowTitlebarMaximized,

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
    /// Parse a legacy break-between value for `page-break-{before,after}`.
    ///
    /// See https://drafts.csswg.org/css-break/#page-break-properties.
    #[cfg(feature = "gecko")]
    #[inline]
    pub(crate) fn parse_legacy<'i>(
        _: &ParserContext,
        input: &mut Parser<'i, '_>,
    ) -> Result<Self, ParseError<'i>> {
        let break_value = BreakBetween::parse(input)?;
        match break_value {
            BreakBetween::Always => Ok(BreakBetween::Page),
            BreakBetween::Auto | BreakBetween::Avoid | BreakBetween::Left | BreakBetween::Right => {
                Ok(break_value)
            },
            BreakBetween::Page => {
                Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
            },
        }
    }

    /// Serialize a legacy break-between value for `page-break-*`.
    ///
    /// See https://drafts.csswg.org/css-break/#page-break-properties.
    #[cfg(feature = "gecko")]
    pub(crate) fn to_css_legacy<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
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
    AvoidPage,
    AvoidColumn,
}

impl BreakWithin {
    /// Parse a legacy break-between value for `page-break-inside`.
    ///
    /// See https://drafts.csswg.org/css-break/#page-break-properties.
    #[cfg(feature = "gecko")]
    #[inline]
    pub(crate) fn parse_legacy<'i>(
        _: &ParserContext,
        input: &mut Parser<'i, '_>,
    ) -> Result<Self, ParseError<'i>> {
        let break_value = BreakWithin::parse(input)?;
        match break_value {
            BreakWithin::Auto | BreakWithin::Avoid => Ok(break_value),
            BreakWithin::AvoidPage | BreakWithin::AvoidColumn => {
                Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
            },
        }
    }

    /// Serialize a legacy break-between value for `page-break-inside`.
    ///
    /// See https://drafts.csswg.org/css-break/#page-break-properties.
    #[cfg(feature = "gecko")]
    pub(crate) fn to_css_legacy<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match *self {
            BreakWithin::Auto | BreakWithin::Avoid => self.to_css(dest),
            BreakWithin::AvoidPage | BreakWithin::AvoidColumn => Ok(()),
        }
    }
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
    Clip,
}

// This can be derived once we remove or keep `-moz-hidden-unscrollable`
// indefinitely.
impl Parse for Overflow {
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Ok(try_match_ident_ignore_ascii_case! { input,
            "visible" => Self::Visible,
            "hidden" => Self::Hidden,
            "scroll" => Self::Scroll,
            "auto" => Self::Auto,
            #[cfg(feature = "gecko")]
            "clip" => Self::Clip,
            #[cfg(feature = "gecko")]
            "-moz-hidden-unscrollable" if static_prefs::pref!("layout.css.overflow-moz-hidden-unscrollable.enabled") => {
                Overflow::Clip
            },
            #[cfg(feature = "gecko")]
            "overlay" if static_prefs::pref!("layout.css.overflow-overlay.enabled") => {
                Overflow::Auto
            },
        })
    }
}

impl Overflow {
    /// Return true if the value will create a scrollable box.
    #[inline]
    pub fn is_scrollable(&self) -> bool {
        matches!(*self, Self::Hidden | Self::Scroll | Self::Auto)
    }
    /// Convert the value to a scrollable value if it's not already scrollable.
    /// This maps `visible` to `auto` and `clip` to `hidden`.
    #[inline]
    pub fn to_scrollable(&self) -> Self {
        match *self {
            Self::Hidden | Self::Scroll | Self::Auto => *self,
            Self::Visible => Self::Auto,
            #[cfg(feature = "gecko")]
            Self::Clip => Self::Hidden,
        }
    }
}

bitflags! {
    #[derive(MallocSizeOf, SpecifiedValueInfo, ToCss, ToComputedValue, ToResolvedValue, ToShmem, Parse)]
    #[repr(C)]
    #[css(bitflags(single = "auto", mixed = "stable,both-edges", validate_mixed="Self::has_stable"))]
    /// Values for scrollbar-gutter:
    /// <https://drafts.csswg.org/css-overflow-3/#scrollbar-gutter-property>
    pub struct ScrollbarGutter: u8 {
        /// `auto` variant. Just for convenience if there is no flag set.
        const AUTO = 0;
        /// `stable` variant.
        const STABLE = 1 << 0;
        /// `both-edges` variant.
        const BOTH_EDGES = 1 << 1;
    }
}

impl ScrollbarGutter {
    #[inline]
    fn has_stable(&self) -> bool {
        self.intersects(Self::STABLE)
    }
}
