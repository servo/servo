/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! CSS handling for the specified value of
//! [`position`][position]s
//!
//! [position]: https://drafts.csswg.org/css-backgrounds-3/#position

use crate::hash::FxHashMap;
use crate::parser::{Parse, ParserContext};
use crate::str::HTML_SPACE_CHARACTERS;
use crate::values::computed::LengthPercentage as ComputedLengthPercentage;
use crate::values::computed::{Context, Percentage, ToComputedValue};
use crate::values::generics::position::Position as GenericPosition;
use crate::values::generics::position::ZIndex as GenericZIndex;
use crate::values::specified::transform::OriginComponent;
use crate::values::specified::{AllowQuirks, Integer, LengthPercentage};
use crate::values::{Either, None_};
use crate::Zero;
use cssparser::Parser;
use selectors::parser::SelectorParseErrorKind;
use servo_arc::Arc;
use std::fmt::{self, Write};
use std::ops::Range;
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};

/// The specified value of a CSS `<position>`
pub type Position = GenericPosition<HorizontalPosition, VerticalPosition>;

/// The specified value of a horizontal position.
pub type HorizontalPosition = PositionComponent<X>;

/// The specified value of a vertical position.
pub type VerticalPosition = PositionComponent<Y>;

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
pub enum X {
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
pub enum Y {
    Top,
    Bottom,
}

impl Parse for Position {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_quirky(context, input, AllowQuirks::No)
    }
}

impl Position {
    /// Parses a `<position>`, with quirks.
    pub fn parse_quirky<'i, 't>(
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
                if let Ok(y_keyword) = input.try(Y::parse) {
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
                if let Ok(y_keyword) = input.try(Y::parse) {
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
        let y_keyword = Y::parse(input)?;
        let lp_and_x_pos: Result<_, ParseError> = input.try(|i| {
            let y_lp = i
                .try(|i| LengthPercentage::parse_quirky(context, i, allow_quirks))
                .ok();
            if let Ok(x_keyword) = i.try(X::parse) {
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

impl<S> PositionComponent<S> {
    /// `0%`
    pub fn zero() -> Self {
        PositionComponent::Length(LengthPercentage::Percentage(Percentage::zero()))
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
                let p = Percentage(1. - length.percentage());
                let l = -length.unclamped_length();
                // We represent `<end-side> <length>` as `calc(100% - <length>)`.
                ComputedLengthPercentage::with_clamping_mode(
                    l,
                    Some(p),
                    length.clamping_mode,
                    /* was_calc = */ true,
                )
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

impl Side for X {
    #[inline]
    fn start() -> Self {
        X::Left
    }

    #[inline]
    fn is_start(&self) -> bool {
        *self == X::Left
    }
}

impl Side for Y {
    #[inline]
    fn start() -> Self {
        Y::Top
    }

    #[inline]
    fn is_start(&self) -> bool {
        *self == Y::Top
    }
}

/// The specified value of a legacy CSS `<position>`
/// Modern position syntax supports 3 and 4-value syntax. That means:
/// If three or four values are given, then each <percentage> or <length> represents an offset
/// and must be preceded by a keyword, which specifies from which edge the offset is given.
/// For example, `bottom 10px right 20px` represents a `10px` vertical
/// offset up from the bottom edge and a `20px` horizontal offset leftward from the right edge.
/// If three values are given, the missing offset is assumed to be zero.
/// But for some historical reasons we need to keep CSS Level 2 syntax which only supports up to
/// 2-value. This type represents this 2-value syntax.
pub type LegacyPosition = GenericPosition<LegacyHPosition, LegacyVPosition>;

/// The specified value of a horizontal position.
pub type LegacyHPosition = OriginComponent<X>;

/// The specified value of a vertical position.
pub type LegacyVPosition = OriginComponent<Y>;

impl Parse for LegacyPosition {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_quirky(context, input, AllowQuirks::No)
    }
}

impl LegacyPosition {
    /// Parses a `<position>`, with quirks.
    pub fn parse_quirky<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_quirks: AllowQuirks,
    ) -> Result<Self, ParseError<'i>> {
        match input.try(|i| OriginComponent::parse(context, i)) {
            Ok(x_pos @ OriginComponent::Center) => {
                if let Ok(y_pos) = input.try(|i| OriginComponent::parse(context, i)) {
                    return Ok(Self::new(x_pos, y_pos));
                }
                let x_pos = input
                    .try(|i| OriginComponent::parse(context, i))
                    .unwrap_or(x_pos);
                let y_pos = OriginComponent::Center;
                return Ok(Self::new(x_pos, y_pos));
            },
            Ok(OriginComponent::Side(x_keyword)) => {
                if let Ok(y_keyword) = input.try(Y::parse) {
                    let x_pos = OriginComponent::Side(x_keyword);
                    let y_pos = OriginComponent::Side(y_keyword);
                    return Ok(Self::new(x_pos, y_pos));
                }
                let x_pos = OriginComponent::Side(x_keyword);
                if let Ok(y_lp) =
                    input.try(|i| LengthPercentage::parse_quirky(context, i, allow_quirks))
                {
                    return Ok(Self::new(x_pos, OriginComponent::Length(y_lp)));
                }
                let _ = input.try(|i| i.expect_ident_matching("center"));
                return Ok(Self::new(x_pos, OriginComponent::Center));
            },
            Ok(x_pos @ OriginComponent::Length(_)) => {
                if let Ok(y_keyword) = input.try(Y::parse) {
                    let y_pos = OriginComponent::Side(y_keyword);
                    return Ok(Self::new(x_pos, y_pos));
                }
                if let Ok(y_lp) =
                    input.try(|i| LengthPercentage::parse_quirky(context, i, allow_quirks))
                {
                    let y_pos = OriginComponent::Length(y_lp);
                    return Ok(Self::new(x_pos, y_pos));
                }
                let _ = input.try(|i| i.expect_ident_matching("center"));
                return Ok(Self::new(x_pos, OriginComponent::Center));
            },
            Err(_) => {},
        }
        let y_keyword = Y::parse(input)?;
        let x_pos: Result<_, ParseError> = input.try(|i| {
            if let Ok(x_keyword) = i.try(X::parse) {
                let x_pos = OriginComponent::Side(x_keyword);
                return Ok(x_pos);
            }
            i.expect_ident_matching("center")?;
            Ok(OriginComponent::Center)
        });
        if let Ok(x_pos) = x_pos {
            let y_pos = OriginComponent::Side(y_keyword);
            return Ok(Self::new(x_pos, y_pos));
        }
        let x_pos = OriginComponent::Center;
        let y_pos = OriginComponent::Side(y_keyword);
        Ok(Self::new(x_pos, y_pos))
    }

    /// `center center`
    #[inline]
    pub fn center() -> Self {
        Self::new(OriginComponent::Center, OriginComponent::Center)
    }
}

impl ToCss for LegacyPosition {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        self.horizontal.to_css(dest)?;
        dest.write_str(" ")?;
        self.vertical.to_css(dest)
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
/// Auto-placement algorithm Option
pub enum AutoFlow {
    /// The auto-placement algorithm places items by filling each row in turn,
    /// adding new rows as necessary.
    Row,
    /// The auto-placement algorithm places items by filling each column in turn,
    /// adding new columns as necessary.
    Column,
}

/// If `dense` is specified, `row` is implied.
fn is_row_dense(autoflow: &AutoFlow, dense: &bool) -> bool {
    *autoflow == AutoFlow::Row && *dense
}

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
/// Controls how the auto-placement algorithm works
/// specifying exactly how auto-placed items get flowed into the grid
pub struct GridAutoFlow {
    /// Specifiy how auto-placement algorithm fills each `row` or `column` in turn
    #[css(contextual_skip_if = "is_row_dense")]
    pub autoflow: AutoFlow,
    /// Specify use `dense` packing algorithm or not
    #[css(represents_keyword)]
    pub dense: bool,
}

impl GridAutoFlow {
    #[inline]
    /// Get default `grid-auto-flow` as `row`
    pub fn row() -> GridAutoFlow {
        GridAutoFlow {
            autoflow: AutoFlow::Row,
            dense: false,
        }
    }
}

impl Parse for GridAutoFlow {
    /// [ row | column ] || dense
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<GridAutoFlow, ParseError<'i>> {
        let mut value = None;
        let mut dense = false;

        while !input.is_exhausted() {
            let location = input.current_source_location();
            let ident = input.expect_ident()?;
            let success = match_ignore_ascii_case! { &ident,
                "row" if value.is_none() => {
                    value = Some(AutoFlow::Row);
                    true
                },
                "column" if value.is_none() => {
                    value = Some(AutoFlow::Column);
                    true
                },
                "dense" if !dense => {
                    dense = true;
                    true
                },
                _ => false
            };
            if !success {
                return Err(location
                    .new_custom_error(SelectorParseErrorKind::UnexpectedIdent(ident.clone())));
            }
        }

        if value.is_some() || dense {
            Ok(GridAutoFlow {
                autoflow: value.unwrap_or(AutoFlow::Row),
                dense: dense,
            })
        } else {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }
}

#[cfg(feature = "gecko")]
impl From<u8> for GridAutoFlow {
    fn from(bits: u8) -> GridAutoFlow {
        use crate::gecko_bindings::structs;

        GridAutoFlow {
            autoflow: if bits & structs::NS_STYLE_GRID_AUTO_FLOW_ROW as u8 != 0 {
                AutoFlow::Row
            } else {
                AutoFlow::Column
            },
            dense: bits & structs::NS_STYLE_GRID_AUTO_FLOW_DENSE as u8 != 0,
        }
    }
}

#[cfg(feature = "gecko")]
impl From<GridAutoFlow> for u8 {
    fn from(v: GridAutoFlow) -> u8 {
        use crate::gecko_bindings::structs;

        let mut result: u8 = match v.autoflow {
            AutoFlow::Row => structs::NS_STYLE_GRID_AUTO_FLOW_ROW as u8,
            AutoFlow::Column => structs::NS_STYLE_GRID_AUTO_FLOW_COLUMN as u8,
        };

        if v.dense {
            result |= structs::NS_STYLE_GRID_AUTO_FLOW_DENSE as u8;
        }
        result
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
/// https://drafts.csswg.org/css-grid/#named-grid-area
pub struct TemplateAreas {
    /// `named area` containing for each template area
    #[css(skip)]
    pub areas: Box<[NamedArea]>,
    /// The original CSS string value of each template area
    #[css(iterable)]
    pub strings: Box<[Box<str>]>,
    /// The number of columns of the grid.
    #[css(skip)]
    pub width: u32,
}

impl TemplateAreas {
    /// Transform `vector` of str into `template area`
    pub fn from_vec(strings: Vec<Box<str>>) -> Result<TemplateAreas, ()> {
        if strings.is_empty() {
            return Err(());
        }
        let mut areas: Vec<NamedArea> = vec![];
        let mut width = 0;
        {
            let mut row = 0u32;
            let mut area_indices = FxHashMap::<&str, usize>::default();
            for string in &strings {
                let mut current_area_index: Option<usize> = None;
                row += 1;
                let mut column = 0u32;
                for token in TemplateAreasTokenizer(string) {
                    column += 1;
                    let token = if let Some(token) = token? {
                        token
                    } else {
                        if let Some(index) = current_area_index.take() {
                            if areas[index].columns.end != column {
                                return Err(());
                            }
                        }
                        continue;
                    };
                    if let Some(index) = current_area_index {
                        if &*areas[index].name == token {
                            if areas[index].rows.start == row {
                                areas[index].columns.end += 1;
                            }
                            continue;
                        }
                        if areas[index].columns.end != column {
                            return Err(());
                        }
                    }
                    if let Some(index) = area_indices.get(token).cloned() {
                        if areas[index].columns.start != column || areas[index].rows.end != row {
                            return Err(());
                        }
                        areas[index].rows.end += 1;
                        current_area_index = Some(index);
                        continue;
                    }
                    let index = areas.len();
                    areas.push(NamedArea {
                        name: token.to_owned().into_boxed_str(),
                        columns: column..(column + 1),
                        rows: row..(row + 1),
                    });
                    assert!(area_indices.insert(token, index).is_none());
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
            areas: areas.into_boxed_slice(),
            strings: strings.into_boxed_slice(),
            width: width,
        })
    }
}

impl Parse for TemplateAreas {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let mut strings = vec![];
        while let Ok(string) = input.try(|i| i.expect_string().map(|s| s.as_ref().into())) {
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

#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToShmem)]
/// Not associated with any particular grid item, but can
/// be referenced from the grid-placement properties.
pub struct NamedArea {
    /// Name of the `named area`
    pub name: Box<str>,
    /// Rows of the `named area`
    pub rows: Range<u32>,
    /// Columns of the `named area`
    pub columns: Range<u32>,
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
/// The syntax of this property also provides a visualization of
/// the structure of the grid, making the overall layout of
/// the grid container easier to understand.
pub type GridTemplateAreas = Either<TemplateAreasArc, None_>;

impl GridTemplateAreas {
    #[inline]
    /// Get default value as `none`
    pub fn none() -> GridTemplateAreas {
        Either::Second(None_)
    }
}

/// A specified value for the `z-index` property.
pub type ZIndex = GenericZIndex<Integer>;
