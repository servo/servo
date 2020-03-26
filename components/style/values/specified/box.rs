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
use num_traits::FromPrimitive;
use selectors::parser::SelectorParseErrorKind;
use std::fmt::{self, Write};
use style_traits::{CssWriter, KeywordsCollectFn, ParseError};
use style_traits::{SpecifiedValueInfo, StyleParseErrorKind, ToCss};

#[cfg(feature = "gecko")]
fn moz_display_values_enabled(context: &ParserContext) -> bool {
    context.in_ua_or_chrome_sheet() ||
        static_prefs::pref!("layout.css.xul-display-values.content.enabled")
}

#[cfg(feature = "gecko")]
fn moz_box_display_values_enabled(context: &ParserContext) -> bool {
    context.in_ua_or_chrome_sheet() ||
        static_prefs::pref!("layout.css.xul-box-display-values.content.enabled")
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
    #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
    TableCaption,
    #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
    InternalTable,
    #[cfg(feature = "gecko")]
    InternalRuby,
    #[cfg(feature = "gecko")]
    XUL,
}

#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Eq, FromPrimitive, Hash, MallocSizeOf, PartialEq, ToCss, ToShmem)]
#[repr(u8)]
pub enum DisplayInside {
    None = 0,
    #[cfg(any(feature = "servo-layout-2020", feature = "gecko"))]
    Contents,
    Flow,
    FlowRoot,
    #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
    Flex,
    #[cfg(feature = "gecko")]
    Grid,
    #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
    Table,
    #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
    TableRowGroup,
    #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
    TableColumn,
    #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
    TableColumnGroup,
    #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
    TableHeaderGroup,
    #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
    TableFooterGroup,
    #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
    TableRow,
    #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
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
    #[cfg(feature = "gecko")]
    MozBox,
    #[cfg(feature = "gecko")]
    MozGrid,
    #[cfg(feature = "gecko")]
    MozGridGroup,
    #[cfg(feature = "gecko")]
    MozGridLine,
    #[cfg(feature = "gecko")]
    MozStack,
    #[cfg(feature = "gecko")]
    MozDeck,
    #[cfg(feature = "gecko")]
    MozPopup,
}

#[allow(missing_docs)]
#[derive(
    Clone,
    Copy,
    Debug,
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
    #[cfg(any(feature = "servo-layout-2020", feature = "gecko"))]
    pub const Contents: Self = Self::new(DisplayOutside::None, DisplayInside::Contents);
    pub const Inline: Self = Self::new(DisplayOutside::Inline, DisplayInside::Flow);
    pub const InlineBlock: Self = Self::new(DisplayOutside::Inline, DisplayInside::FlowRoot);
    pub const Block: Self = Self::new(DisplayOutside::Block, DisplayInside::Flow);
    #[cfg(feature = "gecko")]
    pub const FlowRoot: Self = Self::new(DisplayOutside::Block, DisplayInside::FlowRoot);
    #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
    pub const Flex: Self = Self::new(DisplayOutside::Block, DisplayInside::Flex);
    #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
    pub const InlineFlex: Self = Self::new(DisplayOutside::Inline, DisplayInside::Flex);
    #[cfg(feature = "gecko")]
    pub const Grid: Self = Self::new(DisplayOutside::Block, DisplayInside::Grid);
    #[cfg(feature = "gecko")]
    pub const InlineGrid: Self = Self::new(DisplayOutside::Inline, DisplayInside::Grid);
    #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
    pub const Table: Self = Self::new(DisplayOutside::Block, DisplayInside::Table);
    #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
    pub const InlineTable: Self = Self::new(DisplayOutside::Inline, DisplayInside::Table);
    #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
    pub const TableCaption: Self = Self::new(DisplayOutside::TableCaption, DisplayInside::Flow);
    #[cfg(feature = "gecko")]
    pub const Ruby: Self = Self::new(DisplayOutside::Inline, DisplayInside::Ruby);
    #[cfg(feature = "gecko")]
    pub const WebkitBox: Self = Self::new(DisplayOutside::Block, DisplayInside::WebkitBox);
    #[cfg(feature = "gecko")]
    pub const WebkitInlineBox: Self = Self::new(DisplayOutside::Inline, DisplayInside::WebkitBox);

    // Internal table boxes.

    #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
    pub const TableRowGroup: Self =
        Self::new(DisplayOutside::InternalTable, DisplayInside::TableRowGroup);

    #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
    pub const TableHeaderGroup: Self = Self::new(
        DisplayOutside::InternalTable,
        DisplayInside::TableHeaderGroup,
    );

    #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
    pub const TableFooterGroup: Self = Self::new(
        DisplayOutside::InternalTable,
        DisplayInside::TableFooterGroup,
    );

    #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
    pub const TableColumn: Self =
        Self::new(DisplayOutside::InternalTable, DisplayInside::TableColumn);

    #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
    pub const TableColumnGroup: Self = Self::new(
        DisplayOutside::InternalTable,
        DisplayInside::TableColumnGroup,
    );

    #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
    pub const TableRow: Self = Self::new(DisplayOutside::InternalTable, DisplayInside::TableRow);

    #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
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

    /// XUL boxes.
    #[cfg(feature = "gecko")]
    pub const MozBox: Self = Self::new(DisplayOutside::Block, DisplayInside::MozBox);
    #[cfg(feature = "gecko")]
    pub const MozInlineBox: Self = Self::new(DisplayOutside::Inline, DisplayInside::MozBox);
    #[cfg(feature = "gecko")]
    pub const MozGrid: Self = Self::new(DisplayOutside::XUL, DisplayInside::MozGrid);
    #[cfg(feature = "gecko")]
    pub const MozGridGroup: Self = Self::new(DisplayOutside::XUL, DisplayInside::MozGridGroup);
    #[cfg(feature = "gecko")]
    pub const MozGridLine: Self = Self::new(DisplayOutside::XUL, DisplayInside::MozGridLine);
    #[cfg(feature = "gecko")]
    pub const MozStack: Self = Self::new(DisplayOutside::XUL, DisplayInside::MozStack);
    #[cfg(feature = "gecko")]
    pub const MozDeck: Self = Self::new(DisplayOutside::XUL, DisplayInside::MozDeck);
    #[cfg(feature = "gecko")]
    pub const MozPopup: Self = Self::new(DisplayOutside::XUL, DisplayInside::MozPopup);

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
            Display::InlineBlock => true,
            #[cfg(any(feature = "servo-layout-2013"))]
            Display::InlineFlex | Display::InlineTable => true,
            _ => false,
        }
    }

    /// Returns whether this `display` value is the display of a flex or
    /// grid container.
    ///
    /// This is used to implement various style fixups.
    pub fn is_item_container(&self) -> bool {
        match self.inside() {
            #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
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
        match *self {
            Display::Inline => true,
            #[cfg(feature = "gecko")]
            Display::Contents | Display::Ruby | Display::RubyBaseContainer => true,
            _ => false,
        }
    }

    /// Convert this display into an equivalent block display.
    ///
    /// Also used for :root style adjustments.
    pub fn equivalent_block_display(&self, _is_root_element: bool) -> Self {
        #[cfg(feature = "gecko")]
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
            #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
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
            #[cfg(feature = "gecko")]
            Display::MozInlineBox => dest.write_str("-moz-inline-box"),
            #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
            Display::TableCaption => dest.write_str("table-caption"),
            _ => match (outside, inside) {
                #[cfg(feature = "gecko")]
                (DisplayOutside::Inline, DisplayInside::Grid) => dest.write_str("inline-grid"),
                #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
                (DisplayOutside::Inline, DisplayInside::Flex) |
                (DisplayOutside::Inline, DisplayInside::Table) => {
                    dest.write_str("inline-")?;
                    inside.to_css(dest)
                },
                #[cfg(feature = "gecko")]
                (DisplayOutside::Block, DisplayInside::Ruby) => dest.write_str("block ruby"),
                (_, inside) => {
                    if self.is_list_item() {
                        if outside != DisplayOutside::Block {
                            outside.to_css(dest)?;
                            dest.write_str(" ")?;
                        }
                        if inside != DisplayInside::Flow {
                            inside.to_css(dest)?;
                            dest.write_str(" ")?;
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
        #[cfg(any(feature = "servo-layout-2020", feature = "gecko"))]
        "flow-root" => DisplayInside::FlowRoot,
        #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
        "table" => DisplayInside::Table,
        #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
        "flex" => DisplayInside::Flex,
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
    Ok(try_match_ident_ignore_ascii_case! { input,
        "list-item" => (),
    })
}

impl Parse for Display {
    #[allow(unused)] // `context` isn't used for servo-2020 for now
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Display, ParseError<'i>> {
        // Parse all combinations of <display-inside/outside>? and `list-item`? first.
        let mut got_list_item = input.try(parse_list_item).is_ok();
        let mut inside = if got_list_item {
            input.try(parse_display_inside_for_list_item)
        } else {
            input.try(parse_display_inside)
        };
        // <display-listitem> = <display-outside>? && [ flow | flow-root ]? && list-item
        // https://drafts.csswg.org/css-display/#typedef-display-listitem
        if !got_list_item && is_valid_inside_for_list_item(&inside) {
            got_list_item = input.try(parse_list_item).is_ok();
        }
        let outside = input.try(parse_display_outside);
        if outside.is_ok() {
            if !got_list_item && (inside.is_err() || is_valid_inside_for_list_item(&inside)) {
                got_list_item = input.try(parse_list_item).is_ok();
            }
            if inside.is_err() {
                inside = if got_list_item {
                    input.try(parse_display_inside_for_list_item)
                } else {
                    input.try(parse_display_inside)
                };
                if !got_list_item && is_valid_inside_for_list_item(&inside) {
                    got_list_item = input.try(parse_list_item).is_ok();
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
            #[cfg(any(feature = "servo-layout-2020", feature = "gecko"))]
            "contents" => Display::Contents,
            "inline-block" => Display::InlineBlock,
            #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
            "inline-table" => Display::InlineTable,
            #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
            "-webkit-flex" => Display::Flex,
            #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
            "inline-flex" | "-webkit-inline-flex" => Display::InlineFlex,
            #[cfg(feature = "gecko")]
            "inline-grid" => Display::InlineGrid,
            #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
            "table-caption" => Display::TableCaption,
            #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
            "table-row-group" => Display::TableRowGroup,
            #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
            "table-header-group" => Display::TableHeaderGroup,
            #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
            "table-footer-group" => Display::TableFooterGroup,
            #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
            "table-column" => Display::TableColumn,
            #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
            "table-column-group" => Display::TableColumnGroup,
            #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
            "table-row" => Display::TableRow,
            #[cfg(any(feature = "servo-layout-2013", feature = "gecko"))]
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
            #[cfg(feature = "gecko")]
            "-moz-box" if moz_box_display_values_enabled(context) => Display::MozBox,
            #[cfg(feature = "gecko")]
            "-moz-inline-box" if moz_box_display_values_enabled(context) => Display::MozInlineBox,
            #[cfg(feature = "gecko")]
            "-moz-grid" if moz_display_values_enabled(context) => Display::MozGrid,
            #[cfg(feature = "gecko")]
            "-moz-grid-group" if moz_display_values_enabled(context) => Display::MozGridGroup,
            #[cfg(feature = "gecko")]
            "-moz-grid-line" if moz_display_values_enabled(context) => Display::MozGridLine,
            #[cfg(feature = "gecko")]
            "-moz-stack" if moz_display_values_enabled(context) => Display::MozStack,
            #[cfg(feature = "gecko")]
            "-moz-deck" if moz_display_values_enabled(context) => Display::MozDeck,
            #[cfg(feature = "gecko")]
            "-moz-popup" if moz_display_values_enabled(context) => Display::MozPopup,
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
        #[cfg(feature = "gecko")]
        LonghandId::Translate | LonghandId::Rotate | LonghandId::Scale | LonghandId::OffsetPath => {
            WillChangeBits::TRANSFORM
        },
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
    #[css(skip)]
    ButtonBevel,
    /// The focus outline box inside of a button.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    ButtonFocus,
    /// The caret of a text area
    #[css(skip)]
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
    #[css(skip)]
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
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MenulistText,
    /// An editable textfield with a dropdown list (a combobox).
    #[css(skip)]
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
    MozMenulistArrowButton,
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
    RangeThumb, // FIXME: This should not be exposed to content.
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

impl Appearance {
    /// Returns whether we're the `none` value.
    #[inline]
    pub fn is_none(self) -> bool {
        self == Appearance::None
    }
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
