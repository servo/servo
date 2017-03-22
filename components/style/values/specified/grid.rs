/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Necessary types for [grid](https://drafts.csswg.org/css-grid/).

use cssparser::{Parser, Token};
use parser::{Parse, ParserContext};
use std::ascii::AsciiExt;
use std::fmt;
use style_traits::ToCss;
use values::{CSSFloat, Either, HasViewportPercentage};
use values::computed::{ComputedValueAsSpecified, Context, ToComputedValue};
use values::specified::LengthOrPercentage;

#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// A `<grid-line>` type.
///
/// https://drafts.csswg.org/css-grid/#typedef-grid-row-start-grid-line
pub struct GridLine {
    /// Flag to check whether it's a `span` keyword.
    pub is_span: bool,
    /// A custom identifier for named lines.
    ///
    /// https://drafts.csswg.org/css-grid/#grid-placement-slot
    pub ident: Option<String>,
    /// Denotes the nth grid line from grid item's placement.
    pub integer: Option<i32>,
}

impl Default for GridLine {
    fn default() -> Self {
        GridLine {
            is_span: false,
            ident: None,
            integer: None,
        }
    }
}

impl ToCss for GridLine {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        if !self.is_span && self.ident.is_none() && self.integer.is_none() {
            return dest.write_str("auto")
        }

        if self.is_span {
            dest.write_str("span")?;
        }

        if let Some(i) = self.integer {
            write!(dest, " {}", i)?;
        }

        if let Some(ref s) = self.ident {
            write!(dest, " {}", s)?;
        }

        Ok(())
    }
}

impl Parse for GridLine {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let mut grid_line = Default::default();
        if input.try(|i| i.expect_ident_matching("auto")).is_ok() {
            return Ok(grid_line)
        }

        for _ in 0..3 {     // Maximum possible entities for <grid-line>
            if input.try(|i| i.expect_ident_matching("span")).is_ok() {
                if grid_line.is_span {
                    return Err(())
                }
                grid_line.is_span = true;
            } else if let Ok(i) = input.try(|i| i.expect_integer()) {
                if i == 0 || grid_line.integer.is_some() {
                    return Err(())
                }
                grid_line.integer = Some(i);
            } else if let Ok(name) = input.try(|i| i.expect_ident()) {
                if grid_line.ident.is_some() {
                    return Err(())
                }
                grid_line.ident = Some(name.into_owned());
            } else {
                break
            }
        }

        if grid_line.is_span {
            if let Some(i) = grid_line.integer {
                if i < 0 {      // disallow negative integers for grid spans
                    return Err(())
                }
            } else {
                grid_line.integer = Some(1);
            }
        }

        Ok(grid_line)
    }
}

impl ComputedValueAsSpecified for GridLine {}
no_viewport_percentage!(GridLine);

define_css_keyword_enum!{ TrackKeyword:
    "auto" => Auto,
    "max-content" => MaxContent,
    "min-content" => MinContent
}

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// A track breadth for explicit grid track sizing. It's generic solely to
/// avoid re-implementing it for the computed type.
///
/// https://drafts.csswg.org/css-grid/#typedef-track-breadth
pub enum TrackBreadth<L> {
    /// The generic type is almost always a non-negative `<length-percentage>`
    Breadth(L),
    /// A flex fraction specified in `fr` units.
    Flex(CSSFloat),
    /// One of the track-sizing keywords (`auto`, `min-content`, `max-content`)
    Keyword(TrackKeyword),
}

impl<L> TrackBreadth<L> {
    /// Check whether this is a `<fixed-breadth>` (i.e., it only has `<length-percentage>`)
    ///
    /// https://drafts.csswg.org/css-grid/#typedef-fixed-breadth
    #[inline]
    pub fn is_fixed(&self) -> bool {
        match *self {
            TrackBreadth::Breadth(ref _lop) => true,
            _ => false,
        }
    }
}

/// Parse a single flexible length.
pub fn parse_flex(input: &mut Parser) -> Result<CSSFloat, ()> {
    match input.next()? {
        Token::Dimension(ref value, ref unit) if unit.to_lowercase() == "fr" && value.value.is_sign_positive()
            => Ok(value.value),
        _ => Err(()),
    }
}

impl Parse for TrackBreadth<LengthOrPercentage> {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if let Ok(lop) = input.try(LengthOrPercentage::parse_non_negative) {
            Ok(TrackBreadth::Breadth(lop))
        } else {
            if let Ok(f) = input.try(parse_flex) {
                Ok(TrackBreadth::Flex(f))
            } else {
                TrackKeyword::parse(input).map(TrackBreadth::Keyword)
            }
        }
    }
}

impl<L: ToCss> ToCss for TrackBreadth<L> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            TrackBreadth::Breadth(ref lop) => lop.to_css(dest),
            TrackBreadth::Flex(ref value) => write!(dest, "{}fr", value),
            TrackBreadth::Keyword(ref k) => k.to_css(dest),
        }
    }
}

impl HasViewportPercentage for TrackBreadth<LengthOrPercentage> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool {
        if let TrackBreadth::Breadth(ref lop) = *self {
            lop.has_viewport_percentage()
        } else {
            false
        }
    }
}

impl<L: ToComputedValue> ToComputedValue for TrackBreadth<L> {
    type ComputedValue = TrackBreadth<L::ComputedValue>;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            TrackBreadth::Breadth(ref lop) => TrackBreadth::Breadth(lop.to_computed_value(context)),
            TrackBreadth::Flex(fr) => TrackBreadth::Flex(fr),
            TrackBreadth::Keyword(k) => TrackBreadth::Keyword(k),
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            TrackBreadth::Breadth(ref lop) =>
                TrackBreadth::Breadth(ToComputedValue::from_computed_value(lop)),
            TrackBreadth::Flex(fr) => TrackBreadth::Flex(fr),
            TrackBreadth::Keyword(k) => TrackBreadth::Keyword(k),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// A `<track-size>` type for explicit grid track sizing. Like `<track-breadth>`, this is
/// generic only to avoid code bloat. It only takes `<length-percentage>`
///
/// https://drafts.csswg.org/css-grid/#typedef-track-size
pub enum TrackSize<L> {
    /// A flexible `<track-breadth>`
    Breadth(TrackBreadth<L>),
    /// A `minmax` function for a range over an inflexible `<track-breadth>`
    /// and a flexible `<track-breadth>`
    ///
    /// https://drafts.csswg.org/css-grid/#valdef-grid-template-columns-minmax
    MinMax(TrackBreadth<L>, TrackBreadth<L>),
    /// A `fit-content` function.
    ///
    /// https://drafts.csswg.org/css-grid/#valdef-grid-template-columns-fit-content
    FitContent(L),
}

impl<L> TrackSize<L> {
    /// Check whether this is a `<fixed-size>`
    ///
    /// https://drafts.csswg.org/css-grid/#typedef-fixed-size
    pub fn is_fixed(&self) -> bool {
        match *self {
            TrackSize::Breadth(ref breadth) => breadth.is_fixed(),
            // For minmax function, it could be either
            // minmax(<fixed-breadth>, <track-breadth>) or minmax(<inflexible-breadth>, <fixed-breadth>),
            // and since both variants are a subset of minmax(<inflexible-breadth>, <track-breadth>), we only
            // need to make sure that they're fixed. So, we don't have to modify the parsing function.
            TrackSize::MinMax(ref breadth_1, ref breadth_2) => {
                if breadth_1.is_fixed() {
                    return true     // the second value is always a <track-breadth>
                }

                match *breadth_1 {
                    TrackBreadth::Flex(_) => false,     // should be <inflexible-breadth> at this point
                    _ => breadth_2.is_fixed(),
                }
            },
            TrackSize::FitContent(_) => false,
        }
    }
}

impl<L> Default for TrackSize<L> {
    fn default() -> Self {
        TrackSize::Breadth(TrackBreadth::Keyword(TrackKeyword::Auto))
    }
}

impl Parse for TrackSize<LengthOrPercentage> {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if let Ok(b) = input.try(|i| TrackBreadth::parse(context, i)) {
            Ok(TrackSize::Breadth(b))
        } else {
            if input.try(|i| i.expect_function_matching("minmax")).is_ok() {
                input.parse_nested_block(|input| {
                    let inflexible_breadth = if let Ok(lop) = input.try(LengthOrPercentage::parse_non_negative) {
                        Ok(TrackBreadth::Breadth(lop))
                    } else {
                        TrackKeyword::parse(input).map(TrackBreadth::Keyword)
                    };

                    input.expect_comma()?;
                    Ok(TrackSize::MinMax(inflexible_breadth?, TrackBreadth::parse(context, input)?))
                })
            } else {
                input.expect_function_matching("fit-content")?;
                input.parse_nested_block(|i| LengthOrPercentage::parse(context, i).map(TrackSize::FitContent))
            }
        }
    }
}

impl<L: ToCss> ToCss for TrackSize<L> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            TrackSize::Breadth(ref b) => b.to_css(dest),
            TrackSize::MinMax(ref infexible, ref flexible) => {
                dest.write_str("minmax(")?;
                infexible.to_css(dest)?;
                dest.write_str(", ")?;
                flexible.to_css(dest)?;
                dest.write_str(")")
            },
            TrackSize::FitContent(ref lop) => {
                dest.write_str("fit-content(")?;
                lop.to_css(dest)?;
                dest.write_str(")")
            },
        }
    }
}

impl HasViewportPercentage for TrackSize<LengthOrPercentage> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            TrackSize::Breadth(ref b) => b.has_viewport_percentage(),
            TrackSize::MinMax(ref inf_b, ref b) => inf_b.has_viewport_percentage() || b.has_viewport_percentage(),
            TrackSize::FitContent(ref lop) => lop.has_viewport_percentage(),
        }
    }
}

impl<L: ToComputedValue> ToComputedValue for TrackSize<L> {
    type ComputedValue = TrackSize<L::ComputedValue>;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            TrackSize::Breadth(ref b) => TrackSize::Breadth(b.to_computed_value(context)),
            TrackSize::MinMax(ref b_1, ref b_2) =>
                TrackSize::MinMax(b_1.to_computed_value(context), b_2.to_computed_value(context)),
            TrackSize::FitContent(ref lop) => TrackSize::FitContent(lop.to_computed_value(context)),
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            TrackSize::Breadth(ref b) =>
                TrackSize::Breadth(ToComputedValue::from_computed_value(b)),
            TrackSize::MinMax(ref b_1, ref b_2) =>
                TrackSize::MinMax(ToComputedValue::from_computed_value(b_1),
                                  ToComputedValue::from_computed_value(b_2)),
            TrackSize::FitContent(ref lop) =>
                TrackSize::FitContent(ToComputedValue::from_computed_value(lop)),
        }
    }
}

/// Parse the grid line names into a vector of owned strings.
///
/// https://drafts.csswg.org/css-grid/#typedef-line-names
pub fn parse_line_names(input: &mut Parser) -> Result<Vec<String>, ()> {
    input.expect_square_bracket_block()?;
    input.parse_nested_block(|input| {
        let mut values = vec![];
        loop {
            let ident = input.expect_ident()?;
            if ident.eq_ignore_ascii_case("span") {
                return Err(())
            }

            values.push(ident.into_owned());
            if input.try(|i| i.expect_whitespace()).is_err() {
                break
            }
        }

        Ok(values)
    })
}

/// The initial argument of the `repeat` function.
///
/// https://drafts.csswg.org/css-grid/#typedef-track-repeat
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum RepeatCount {
    /// A positive integer. This is allowed only for `<track-repeat>` and `<fixed-repeat>`
    Number(u32),
    /// An `<auto-fill>` keyword allowed only for `<auto-repeat>`
    AutoFill,
    /// An `<auto-fit>` keyword allowed only for `<auto-repeat>`
    AutoFit,
}

impl Parse for RepeatCount {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if let Ok(i) = input.try(|i| i.expect_integer()) {
            if i > 0 {      // only positive integers are allowed
                Ok(RepeatCount::Number(i as u32))
            } else {
                Err(())
            }
        } else {
            match_ignore_ascii_case! { &input.expect_ident()?,
                "auto-fill" => Ok(RepeatCount::AutoFill),
                "auto-fit" => Ok(RepeatCount::AutoFit),
                _ => Err(()),
            }
        }
    }
}

impl ToCss for RepeatCount {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            RepeatCount::Number(ref c) => c.to_css(dest),
            RepeatCount::AutoFill => dest.write_str("auto-fill"),
            RepeatCount::AutoFit => dest.write_str("auto-fit"),
        }
    }
}

impl ComputedValueAsSpecified for RepeatCount {}
no_viewport_percentage!(RepeatCount);

/// The type of `repeat` function (only used in parsing).
///
/// https://drafts.csswg.org/css-grid/#typedef-track-repeat
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
enum RepeatType {
    /// [`<track-repeat>`](https://drafts.csswg.org/css-grid/#typedef-track-repeat)
    Auto,
    /// [`<auto-repeat>`](https://drafts.csswg.org/css-grid/#typedef-auto-repeat)
    Normal,
    /// [`<fixed-repeat>`](https://drafts.csswg.org/css-grid/#typedef-fixed-repeat)
    Fixed,
}

/// The structure corresponding to the various `repeat` functions.
///
/// https://drafts.csswg.org/css-grid/#typedef-track-repeat
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct TrackRepeat<L> {
    /// The number of times for the value to be repeated (could also be `auto-fit` or `auto-fill`)
    pub count: RepeatCount,
    /// `[ <line-names>? <track-size> ]+` in the form of a vector of 2-tuples.
    pub repeat_variants: Vec<(Vec<String>, TrackSize<L>)>,
    /// Final `<line-names>`
    pub line_names_last: Vec<String>,
}

impl TrackRepeat<LengthOrPercentage> {
    fn parse_with_repeat_type(context: &ParserContext, input: &mut Parser)
                              -> Result<(TrackRepeat<LengthOrPercentage>, RepeatType), ()> {
        input.try(|i| i.expect_function_matching("repeat")).and_then(|_| {
            input.parse_nested_block(|input| {
                let count = RepeatCount::parse(context, input)?;
                input.expect_comma()?;

                let is_auto = count == RepeatCount::AutoFit || count == RepeatCount::AutoFill;
                let mut repeat_type = if is_auto {
                    RepeatType::Auto
                } else {    // <fixed-size> is a subset of <track_size>, so it should work for both
                    RepeatType::Fixed
                };

                let mut current_names;
                let mut variants = vec![];

                loop {
                    current_names = input.try(parse_line_names).unwrap_or(vec![]);
                    if let Ok(track_size) = input.try(|i| TrackSize::parse(context, i)) {
                        if !track_size.is_fixed() {
                            if is_auto {
                                return Err(())      // should be <fixed-size> for <auto-repeat>
                            }

                            if repeat_type == RepeatType::Fixed {
                                repeat_type = RepeatType::Normal       // <track-size> for sure
                            }
                        }

                        variants.push((current_names, track_size));
                    } else {
                        if variants.is_empty() {
                            return Err(())      // expecting at least one <track-size>
                        }

                        break       // no more <track-size>, breaking
                    }
                }

                let repeat = TrackRepeat {
                    count: count,
                    repeat_variants: variants,
                    line_names_last: current_names,    // last set of <line-names>
                };

                Ok((repeat, repeat_type))
            })
        })
    }
}

impl<L: ToCss> ToCss for TrackRepeat<L> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        dest.write_str("repeat(")?;
        self.count.to_css(dest)?;
        dest.write_str(", ")?;

        let total = self.repeat_variants.len();
        for (i, &(ref names, ref size)) in self.repeat_variants.iter().enumerate() {
            if !names.is_empty() {
                dest.write_str("[")?;
                dest.write_str(&names.join(" "))?;
                dest.write_str("] ")?;
            }

            size.to_css(dest)?;
            if i < total - 1 {
                dest.write_str(" ")?;
            }
        }

        if !self.line_names_last.is_empty() {
            dest.write_str(" [")?;
            dest.write_str(&self.line_names_last.join(" "))?;
            dest.write_str("]")?;
        }

        dest.write_str(")")
    }
}

impl HasViewportPercentage for TrackRepeat<LengthOrPercentage> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool {
        self.repeat_variants.iter().any(|&(_, ref v)| v.has_viewport_percentage())
    }
}

impl<L: ToComputedValue> ToComputedValue for TrackRepeat<L> {
    type ComputedValue = TrackRepeat<L::ComputedValue>;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        TrackRepeat {
            count: self.count,
            repeat_variants: self.repeat_variants.iter().map(|&(ref names, ref size)| {
                (names.clone(), size.to_computed_value(context))
            }).collect(),
            line_names_last: self.line_names_last.clone(),
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        TrackRepeat {
            count: computed.count,
            repeat_variants: computed.repeat_variants.iter().map(|&(ref names, ref size)| {
                (names.clone(), ToComputedValue::from_computed_value(size))
            }).collect(),
            line_names_last: computed.line_names_last.clone(),
        }
    }
}

/// The type of a `<track-list>`
///
/// https://drafts.csswg.org/css-grid/#typedef-track-list
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum TrackListType {
    /// [`<auto-track-list>`](https://drafts.csswg.org/css-grid/#typedef-auto-track-list)
    ///
    /// If it exists, the value at this index in `TrackList`, is the `<line-names>?`
    /// along with `<auto-repeat>`, subsequently followed by the `<line-names>?` and
    /// `<fixed-size> | <fixed-repeat>` values.
    Auto(u32),
    /// [`<track-list>`](https://drafts.csswg.org/css-grid/#typedef-track-list)
    Normal,
    /// [`<explicit-track-list>`](https://drafts.csswg.org/css-grid/#typedef-explicit-track-list)
    Explicit,
}

/// A grid `<track-list>` type.
///
/// https://drafts.csswg.org/css-grid/#typedef-track-list
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct TrackList<L> {
    /// The type of this `<track-list>` (auto, explicit or general).
    ///
    /// In order to avoid parsing the same value multiple times, this does a single traversal
    /// and arrives at the type of value it has parsed (or bails out gracefully with an error).
    pub list_type: TrackListType,
    /// The list of values parsed into a vector of 2-tuples of `<line-names>` and
    /// `<track-size> | <track-repeat>`
    ///
    /// Note that this may also contain `<auto-repeat>` at an index. If it exists, it's
    /// given by the index in `TrackListType::Auto`
    pub values_list: Vec<(Vec<String>, Either<TrackSize<L>, TrackRepeat<L>>)>,
    /// The final set of `<line-names>` for any kind of `<track-list>`
    pub line_names_last: Vec<String>,
}

impl Parse for TrackList<LengthOrPercentage> {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let mut current_names;
        let mut values = vec![];

        let mut list_type = TrackListType::Explicit;    // assume it's the simplest case
        // marker to check whether we've already encountered <auto-repeat> along the way
        let mut is_auto = false;
        // assume that everything is <fixed-size>. This flag is useful when we encounter <auto-repeat>
        let mut atleast_one_not_fixed = false;

        loop {
            current_names = input.try(parse_line_names).unwrap_or(vec![]);
            if let Ok(track_size) = input.try(|i| TrackSize::parse(context, i)) {
                if !track_size.is_fixed() {
                    atleast_one_not_fixed = true;
                    if is_auto {
                        return Err(())      // <auto-track-list> only accepts <fixed-size> and <fixed-repeat>
                    }
                }

                values.push((current_names, Either::First(track_size)));
            } else if let Ok((repeat, type_)) = input.try(|i| TrackRepeat::parse_with_repeat_type(context, i)) {
                if list_type == TrackListType::Explicit {
                    list_type = TrackListType::Normal;      // <explicit-track-list> doesn't contain repeat()
                }

                match type_ {
                    RepeatType::Normal => {
                        atleast_one_not_fixed = true;
                        if is_auto {            // only <fixed-repeat>
                            return Err(())
                        }
                    },
                    RepeatType::Auto => {
                        if is_auto || atleast_one_not_fixed {
                            // We've either seen <auto-repeat> earlier, or there's at least one non-fixed value
                            return Err(())
                        }

                        is_auto = true;
                        list_type = TrackListType::Auto(values.len() as u32);
                    },
                    RepeatType::Fixed => (),
                }

                values.push((current_names, Either::Second(repeat)));
            } else {
                if values.is_empty() {
                    return Err(())
                }

                break
            }
        }

        Ok(TrackList {
            list_type: list_type,
            values_list: values,
            line_names_last: current_names,
        })
    }
}

impl<L: ToCss> ToCss for TrackList<L> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        for (i, &(ref names, ref value)) in self.values_list.iter().enumerate() {
            if i > 0 {
                dest.write_str(" ")?;
            }

            if !names.is_empty() {
                dest.write_str("[")?;
                dest.write_str(&names.join(" "))?;
                dest.write_str("] ")?;
            }

            value.to_css(dest)?;
        }

        if !self.line_names_last.is_empty() {
            dest.write_str(" [")?;
            dest.write_str(&self.line_names_last.join(" "))?;
            dest.write_str("]")?;
        }

        Ok(())
    }
}

impl HasViewportPercentage for TrackList<LengthOrPercentage> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool {
        self.values_list.iter().any(|&(_, ref v)| v.has_viewport_percentage())
    }
}

impl<L: ToComputedValue> ToComputedValue for TrackList<L> {
    type ComputedValue = TrackList<L::ComputedValue>;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        TrackList {
            list_type: self.list_type,
            values_list: self.values_list.iter().map(|&(ref names, ref value)| {
                (names.clone(), value.to_computed_value(context))
            }).collect(),
            line_names_last: self.line_names_last.clone(),
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        TrackList {
            list_type: computed.list_type,
            values_list: computed.values_list.iter().map(|&(ref names, ref value)| {
                (names.clone(), ToComputedValue::from_computed_value(value))
            }).collect(),
            line_names_last: computed.line_names_last.clone(),
        }
    }
}
