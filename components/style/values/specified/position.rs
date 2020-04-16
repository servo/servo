/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! CSS handling for the specified value of
//! [`position`][position]s
//!
//! [position]: https://drafts.csswg.org/css-backgrounds-3/#position

use crate::parser::{Parse, ParserContext};
use crate::selector_map::PrecomputedHashMap;
use crate::str::HTML_SPACE_CHARACTERS;
use crate::values::computed::LengthPercentage as ComputedLengthPercentage;
use crate::values::computed::{Context, Percentage, ToComputedValue};
use crate::values::generics::position::Position as GenericPosition;
use crate::values::generics::position::PositionComponent as GenericPositionComponent;
use crate::values::generics::position::PositionOrAuto as GenericPositionOrAuto;
use crate::values::generics::position::ZIndex as GenericZIndex;
use crate::values::specified::{AllowQuirks, Integer, LengthPercentage};
use crate::Atom;
use crate::Zero;
use cssparser::Parser;
use selectors::parser::SelectorParseErrorKind;
use servo_arc::Arc;
use std::fmt::{self, Write};
use style_traits::values::specified::AllowedNumericType;
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};

/// The specified value of a CSS `<position>`
pub type Position = GenericPosition<HorizontalPosition, VerticalPosition>;

/// The specified value of an `auto | <position>`.
pub type PositionOrAuto = GenericPositionOrAuto<Position>;

/// The specified value of a horizontal position.
pub type HorizontalPosition = PositionComponent<HorizontalPositionKeyword>;

/// The specified value of a vertical position.
pub type VerticalPosition = PositionComponent<VerticalPositionKeyword>;

/// The specified value of a component of a CSS `<position>`.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
pub enum PositionComponent<S> {
    /// `center`
    Center,
    /// `<length-percentage>`
    Length(LengthPercentage),
    /// `<side> <length-percentage>?`
    Side(S, Option<LengthPercentage>),
}

/// A keyword for the X direction.
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
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[allow(missing_docs)]
#[repr(u8)]
pub enum HorizontalPositionKeyword {
    Left,
    Right,
}

/// A keyword for the Y direction.
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
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[allow(missing_docs)]
#[repr(u8)]
pub enum VerticalPositionKeyword {
    Top,
    Bottom,
}

impl Parse for Position {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let position = Self::parse_three_value_quirky(context, input, AllowQuirks::No)?;
        if position.is_three_value_syntax() {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }
        Ok(position)
    }
}

impl Position {
    /// Parses a `<bg-position>`, with quirks.
    pub fn parse_three_value_quirky<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_quirks: AllowQuirks,
    ) -> Result<Self, ParseError<'i>> {
        match input.try(|i| PositionComponent::parse_quirky(context, i, allow_quirks)) {
            Ok(x_pos @ PositionComponent::Center) => {
                if let Ok(y_pos) =
                    input.try(|i| PositionComponent::parse_quirky(context, i, allow_quirks))
                {
                    return Ok(Self::new(x_pos, y_pos));
                }
                let x_pos = input
                    .try(|i| PositionComponent::parse_quirky(context, i, allow_quirks))
                    .unwrap_or(x_pos);
                let y_pos = PositionComponent::Center;
                return Ok(Self::new(x_pos, y_pos));
            },
            Ok(PositionComponent::Side(x_keyword, lp)) => {
                if input.try(|i| i.expect_ident_matching("center")).is_ok() {
                    let x_pos = PositionComponent::Side(x_keyword, lp);
                    let y_pos = PositionComponent::Center;
                    return Ok(Self::new(x_pos, y_pos));
                }
                if let Ok(y_keyword) = input.try(VerticalPositionKeyword::parse) {
                    let y_lp = input
                        .try(|i| LengthPercentage::parse_quirky(context, i, allow_quirks))
                        .ok();
                    let x_pos = PositionComponent::Side(x_keyword, lp);
                    let y_pos = PositionComponent::Side(y_keyword, y_lp);
                    return Ok(Self::new(x_pos, y_pos));
                }
                let x_pos = PositionComponent::Side(x_keyword, None);
                let y_pos = lp.map_or(PositionComponent::Center, PositionComponent::Length);
                return Ok(Self::new(x_pos, y_pos));
            },
            Ok(x_pos @ PositionComponent::Length(_)) => {
                if let Ok(y_keyword) = input.try(VerticalPositionKeyword::parse) {
                    let y_pos = PositionComponent::Side(y_keyword, None);
                    return Ok(Self::new(x_pos, y_pos));
                }
                if let Ok(y_lp) =
                    input.try(|i| LengthPercentage::parse_quirky(context, i, allow_quirks))
                {
                    let y_pos = PositionComponent::Length(y_lp);
                    return Ok(Self::new(x_pos, y_pos));
                }
                let y_pos = PositionComponent::Center;
                let _ = input.try(|i| i.expect_ident_matching("center"));
                return Ok(Self::new(x_pos, y_pos));
            },
            Err(_) => {},
        }
        let y_keyword = VerticalPositionKeyword::parse(input)?;
        let lp_and_x_pos: Result<_, ParseError> = input.try(|i| {
            let y_lp = i
                .try(|i| LengthPercentage::parse_quirky(context, i, allow_quirks))
                .ok();
            if let Ok(x_keyword) = i.try(HorizontalPositionKeyword::parse) {
                let x_lp = i
                    .try(|i| LengthPercentage::parse_quirky(context, i, allow_quirks))
                    .ok();
                let x_pos = PositionComponent::Side(x_keyword, x_lp);
                return Ok((y_lp, x_pos));
            };
            i.expect_ident_matching("center")?;
            let x_pos = PositionComponent::Center;
            Ok((y_lp, x_pos))
        });
        if let Ok((y_lp, x_pos)) = lp_and_x_pos {
            let y_pos = PositionComponent::Side(y_keyword, y_lp);
            return Ok(Self::new(x_pos, y_pos));
        }
        let x_pos = PositionComponent::Center;
        let y_pos = PositionComponent::Side(y_keyword, None);
        Ok(Self::new(x_pos, y_pos))
    }

    /// `center center`
    #[inline]
    pub fn center() -> Self {
        Self::new(PositionComponent::Center, PositionComponent::Center)
    }

    /// Returns true if this uses a 3 value syntax.
    #[inline]
    fn is_three_value_syntax(&self) -> bool {
        self.horizontal.component_count() != self.vertical.component_count()
    }
}

impl ToCss for Position {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match (&self.horizontal, &self.vertical) {
            (
                x_pos @ &PositionComponent::Side(_, Some(_)),
                &PositionComponent::Length(ref y_lp),
            ) => {
                x_pos.to_css(dest)?;
                dest.write_str(" top ")?;
                y_lp.to_css(dest)
            },
            (
                &PositionComponent::Length(ref x_lp),
                y_pos @ &PositionComponent::Side(_, Some(_)),
            ) => {
                dest.write_str("left ")?;
                x_lp.to_css(dest)?;
                dest.write_str(" ")?;
                y_pos.to_css(dest)
            },
            (x_pos, y_pos) => {
                x_pos.to_css(dest)?;
                dest.write_str(" ")?;
                y_pos.to_css(dest)
            },
        }
    }
}

impl<S: Parse> Parse for PositionComponent<S> {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_quirky(context, input, AllowQuirks::No)
    }
}

impl<S: Parse> PositionComponent<S> {
    /// Parses a component of a CSS position, with quirks.
    pub fn parse_quirky<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_quirks: AllowQuirks,
    ) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("center")).is_ok() {
            return Ok(PositionComponent::Center);
        }
        if let Ok(lp) = input.try(|i| LengthPercentage::parse_quirky(context, i, allow_quirks)) {
            return Ok(PositionComponent::Length(lp));
        }
        let keyword = S::parse(context, input)?;
        let lp = input
            .try(|i| LengthPercentage::parse_quirky(context, i, allow_quirks))
            .ok();
        Ok(PositionComponent::Side(keyword, lp))
    }
}

impl<S> GenericPositionComponent for PositionComponent<S> {
    fn is_center(&self) -> bool {
        match *self {
            PositionComponent::Center => true,
            PositionComponent::Length(LengthPercentage::Percentage(ref per)) => per.0 == 0.5,
            // 50% from any side is still the center.
            PositionComponent::Side(_, Some(LengthPercentage::Percentage(ref per))) => per.0 == 0.5,
            _ => false,
        }
    }
}

impl<S> PositionComponent<S> {
    /// `0%`
    pub fn zero() -> Self {
        PositionComponent::Length(LengthPercentage::Percentage(Percentage::zero()))
    }

    /// Returns the count of this component.
    fn component_count(&self) -> usize {
        match *self {
            PositionComponent::Length(..) | PositionComponent::Center => 1,
            PositionComponent::Side(_, ref lp) => {
                if lp.is_some() {
                    2
                } else {
                    1
                }
            },
        }
    }
}

impl<S: Side> ToComputedValue for PositionComponent<S> {
    type ComputedValue = ComputedLengthPercentage;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            PositionComponent::Center => ComputedLengthPercentage::new_percent(Percentage(0.5)),
            PositionComponent::Side(ref keyword, None) => {
                let p = Percentage(if keyword.is_start() { 0. } else { 1. });
                ComputedLengthPercentage::new_percent(p)
            },
            PositionComponent::Side(ref keyword, Some(ref length)) if !keyword.is_start() => {
                let length = length.to_computed_value(context);
                // We represent `<end-side> <length>` as `calc(100% - <length>)`.
                ComputedLengthPercentage::hundred_percent_minus(length, AllowedNumericType::All)
            },
            PositionComponent::Side(_, Some(ref length)) |
            PositionComponent::Length(ref length) => length.to_computed_value(context),
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        PositionComponent::Length(ToComputedValue::from_computed_value(computed))
    }
}

impl<S: Side> PositionComponent<S> {
    /// The initial specified value of a position component, i.e. the start side.
    pub fn initial_specified_value() -> Self {
        PositionComponent::Side(S::start(), None)
    }
}

/// Represents a side, either horizontal or vertical, of a CSS position.
pub trait Side {
    /// Returns the start side.
    fn start() -> Self;

    /// Returns whether this side is the start side.
    fn is_start(&self) -> bool;
}

impl Side for HorizontalPositionKeyword {
    #[inline]
    fn start() -> Self {
        HorizontalPositionKeyword::Left
    }

    #[inline]
    fn is_start(&self) -> bool {
        *self == Self::start()
    }
}

impl Side for VerticalPositionKeyword {
    #[inline]
    fn start() -> Self {
        VerticalPositionKeyword::Top
    }

    #[inline]
    fn is_start(&self) -> bool {
        *self == Self::start()
    }
}

bitflags! {
    /// Controls how the auto-placement algorithm works
    /// specifying exactly how auto-placed items get flowed into the grid
    #[derive(
        MallocSizeOf,
        SpecifiedValueInfo,
        ToComputedValue,
        ToResolvedValue,
        ToShmem
    )]
    #[value_info(other_values = "row,column,dense")]
    #[repr(C)]
    pub struct GridAutoFlow: u8 {
        /// 'row' - mutually exclusive with 'column'
        const ROW = 1 << 0;
        /// 'column' - mutually exclusive with 'row'
        const COLUMN = 1 << 1;
        /// 'dense'
        const DENSE = 1 << 2;
    }
}

impl Parse for GridAutoFlow {
    /// [ row | column ] || dense
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<GridAutoFlow, ParseError<'i>> {
        let mut track = None;
        let mut dense = GridAutoFlow::empty();

        while !input.is_exhausted() {
            let location = input.current_source_location();
            let ident = input.expect_ident()?;
            let success = match_ignore_ascii_case! { &ident,
                "row" if track.is_none() => {
                    track = Some(GridAutoFlow::ROW);
                    true
                },
                "column" if track.is_none() => {
                    track = Some(GridAutoFlow::COLUMN);
                    true
                },
                "dense" if dense.is_empty() => {
                    dense = GridAutoFlow::DENSE;
                    true
                },
                _ => false,
            };
            if !success {
                return Err(location
                    .new_custom_error(SelectorParseErrorKind::UnexpectedIdent(ident.clone())));
            }
        }

        if track.is_some() || !dense.is_empty() {
            Ok(track.unwrap_or(GridAutoFlow::ROW) | dense)
        } else {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }
}

impl ToCss for GridAutoFlow {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if *self == GridAutoFlow::ROW {
            return dest.write_str("row");
        }

        if *self == GridAutoFlow::COLUMN {
            return dest.write_str("column");
        }

        if *self == GridAutoFlow::ROW | GridAutoFlow::DENSE {
            return dest.write_str("dense");
        }

        if *self == GridAutoFlow::COLUMN | GridAutoFlow::DENSE {
            return dest.write_str("column dense");
        }

        debug_assert!(false, "Unknown or invalid grid-autoflow value");
        Ok(())
    }
}

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
#[repr(C)]
/// https://drafts.csswg.org/css-grid/#named-grid-area
pub struct TemplateAreas {
    /// `named area` containing for each template area
    #[css(skip)]
    pub areas: crate::OwnedSlice<NamedArea>,
    /// The original CSS string value of each template area
    #[css(iterable)]
    pub strings: crate::OwnedSlice<crate::OwnedStr>,
    /// The number of columns of the grid.
    #[css(skip)]
    pub width: u32,
}

impl TemplateAreas {
    /// Transform `vector` of str into `template area`
    pub fn from_vec(strings: Vec<crate::OwnedStr>) -> Result<Self, ()> {
        if strings.is_empty() {
            return Err(());
        }
        let mut areas: Vec<NamedArea> = vec![];
        let mut width = 0;
        {
            let mut row = 0u32;
            let mut area_indices = PrecomputedHashMap::<Atom, usize>::default();
            for string in &strings {
                let mut current_area_index: Option<usize> = None;
                row += 1;
                let mut column = 0u32;
                for token in TemplateAreasTokenizer(string) {
                    column += 1;
                    let name = if let Some(token) = token? {
                        Atom::from(token)
                    } else {
                        if let Some(index) = current_area_index.take() {
                            if areas[index].columns.end != column {
                                return Err(());
                            }
                        }
                        continue;
                    };
                    if let Some(index) = current_area_index {
                        if areas[index].name == name {
                            if areas[index].rows.start == row {
                                areas[index].columns.end += 1;
                            }
                            continue;
                        }
                        if areas[index].columns.end != column {
                            return Err(());
                        }
                    }
                    if let Some(index) = area_indices.get(&name).cloned() {
                        if areas[index].columns.start != column || areas[index].rows.end != row {
                            return Err(());
                        }
                        areas[index].rows.end += 1;
                        current_area_index = Some(index);
                        continue;
                    }
                    let index = areas.len();
                    assert!(area_indices.insert(name.clone(), index).is_none());
                    areas.push(NamedArea {
                        name,
                        columns: UnsignedRange {
                            start: column,
                            end: column + 1,
                        },
                        rows: UnsignedRange {
                            start: row,
                            end: row + 1,
                        },
                    });
                    current_area_index = Some(index);
                }
                if let Some(index) = current_area_index {
                    if areas[index].columns.end != column + 1 {
                        assert_ne!(areas[index].rows.start, row);
                        return Err(());
                    }
                }
                if row == 1 {
                    width = column;
                } else if width != column {
                    return Err(());
                }
            }
        }
        Ok(TemplateAreas {
            areas: areas.into(),
            strings: strings.into(),
            width,
        })
    }
}

impl Parse for TemplateAreas {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let mut strings = vec![];
        while let Ok(string) =
            input.try(|i| i.expect_string().map(|s| s.as_ref().to_owned().into()))
        {
            strings.push(string);
        }

        TemplateAreas::from_vec(strings)
            .map_err(|()| input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
}

/// Arc type for `Arc<TemplateAreas>`
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
#[repr(transparent)]
pub struct TemplateAreasArc(#[ignore_malloc_size_of = "Arc"] pub Arc<TemplateAreas>);

impl Parse for TemplateAreasArc {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let parsed = TemplateAreas::parse(context, input)?;
        Ok(TemplateAreasArc(Arc::new(parsed)))
    }
}

/// A range of rows or columns. Using this instead of std::ops::Range for FFI
/// purposes.
#[repr(C)]
#[derive(
    Clone,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
pub struct UnsignedRange {
    /// The start of the range.
    pub start: u32,
    /// The end of the range.
    pub end: u32,
}

#[derive(
    Clone,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
/// Not associated with any particular grid item, but can be referenced from the
/// grid-placement properties.
pub struct NamedArea {
    /// Name of the `named area`
    pub name: Atom,
    /// Rows of the `named area`
    pub rows: UnsignedRange,
    /// Columns of the `named area`
    pub columns: UnsignedRange,
}

/// Tokenize the string into a list of the tokens,
/// using longest-match semantics
struct TemplateAreasTokenizer<'a>(&'a str);

impl<'a> Iterator for TemplateAreasTokenizer<'a> {
    type Item = Result<Option<&'a str>, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        let rest = self.0.trim_start_matches(HTML_SPACE_CHARACTERS);
        if rest.is_empty() {
            return None;
        }
        if rest.starts_with('.') {
            self.0 = &rest[rest.find(|c| c != '.').unwrap_or(rest.len())..];
            return Some(Ok(None));
        }
        if !rest.starts_with(is_name_code_point) {
            return Some(Err(()));
        }
        let token_len = rest.find(|c| !is_name_code_point(c)).unwrap_or(rest.len());
        let token = &rest[..token_len];
        self.0 = &rest[token_len..];
        Some(Ok(Some(token)))
    }
}

fn is_name_code_point(c: char) -> bool {
    c >= 'A' && c <= 'Z' ||
        c >= 'a' && c <= 'z' ||
        c >= '\u{80}' ||
        c == '_' ||
        c >= '0' && c <= '9' ||
        c == '-'
}

/// This property specifies named grid areas.
///
/// The syntax of this property also provides a visualization of the structure
/// of the grid, making the overall layout of the grid container easier to
/// understand.
#[repr(C, u8)]
#[derive(
    Clone,
    Debug,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
pub enum GridTemplateAreas {
    /// The `none` value.
    None,
    /// The actual value.
    Areas(TemplateAreasArc),
}

impl GridTemplateAreas {
    #[inline]
    /// Get default value as `none`
    pub fn none() -> GridTemplateAreas {
        GridTemplateAreas::None
    }
}

/// A specified value for the `z-index` property.
pub type ZIndex = GenericZIndex<Integer>;
