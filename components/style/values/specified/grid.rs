/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Necessary types for [grid](https://drafts.csswg.org/css-grid/).

use cssparser::{Parser, Token, serialize_identifier};
use parser::{Parse, ParserContext};
use std::ascii::AsciiExt;
use std::fmt;
use style_traits::ToCss;
use values::{CSSFloat, CustomIdent, HasViewportPercentage};
use values::computed::{ComputedValueAsSpecified, Context, ToComputedValue};
use values::specified::{Integer, LengthOrPercentage};

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
            try!(dest.write_str("span"));
        }

        if let Some(i) = self.integer {
            try!(write!(dest, " {}", i));
        }

        if let Some(ref s) = self.ident {
            try!(write!(dest, " {}", s));
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
    match try!(input.next()) {
        Token::Dimension(ref value, ref unit) if unit.eq_ignore_ascii_case("fr") && value.value.is_sign_positive()
            => Ok(value.value),
        _ => Err(()),
    }
}

impl Parse for TrackBreadth<LengthOrPercentage> {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if let Ok(lop) = input.try(|i| LengthOrPercentage::parse_non_negative(context, i)) {
            return Ok(TrackBreadth::Breadth(lop))
        }

        if let Ok(f) = input.try(parse_flex) {
            return Ok(TrackBreadth::Flex(f))
        }

        TrackKeyword::parse(input).map(TrackBreadth::Keyword)
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
            return Ok(TrackSize::Breadth(b))
        }

        if input.try(|i| i.expect_function_matching("minmax")).is_ok() {
            return input.parse_nested_block(|input| {
                let inflexible_breadth =
                    match input.try(|i| LengthOrPercentage::parse_non_negative(context, i)) {
                        Ok(lop) => TrackBreadth::Breadth(lop),
                        Err(..) => {
                            let keyword = try!(TrackKeyword::parse(input));
                            TrackBreadth::Keyword(keyword)
                        }
                    };

                try!(input.expect_comma());
                Ok(TrackSize::MinMax(inflexible_breadth, try!(TrackBreadth::parse(context, input))))
            });
        }

        try!(input.expect_function_matching("fit-content"));
        // FIXME(emilio): This needs a parse_nested_block, doesn't it?
        Ok(try!(LengthOrPercentage::parse(context, input).map(TrackSize::FitContent)))
    }
}

impl<L: ToCss> ToCss for TrackSize<L> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            TrackSize::Breadth(ref b) => b.to_css(dest),
            TrackSize::MinMax(ref infexible, ref flexible) => {
                try!(dest.write_str("minmax("));
                try!(infexible.to_css(dest));
                try!(dest.write_str(", "));
                try!(flexible.to_css(dest));
                dest.write_str(")")
            },
            TrackSize::FitContent(ref lop) => {
                try!(dest.write_str("fit-content("));
                try!(lop.to_css(dest));
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
        while let Ok(ident) = input.try(|i| i.expect_ident()) {
            if CustomIdent::from_ident((&*ident).into(), &["span"]).is_err() {
                return Err(())
            }

            values.push(ident.into_owned());
        }

        Ok(values)
    })
}

fn concat_serialize_idents<W>(prefix: &str, suffix: &str,
                              slice: &[String], sep: &str, dest: &mut W) -> fmt::Result
    where W: fmt::Write
{
    if let Some((ref first, rest)) = slice.split_first() {
        dest.write_str(prefix)?;
        serialize_identifier(first, dest)?;
        for thing in rest {
            dest.write_str(sep)?;
            serialize_identifier(thing, dest)?;
        }

        dest.write_str(suffix)?;
    }

    Ok(())
}

/// The initial argument of the `repeat` function.
///
/// https://drafts.csswg.org/css-grid/#typedef-track-repeat
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum RepeatCount {
    /// A positive integer. This is allowed only for `<track-repeat>` and `<fixed-repeat>`
    Number(Integer),
    /// An `<auto-fill>` keyword allowed only for `<auto-repeat>`
    AutoFill,
    /// An `<auto-fit>` keyword allowed only for `<auto-repeat>`
    AutoFit,
}

impl Parse for RepeatCount {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if let Ok(i) = input.try(|i| Integer::parse(context, i)) {
            if i.value > 0 {
                Ok(RepeatCount::Number(i))
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
    /// [`<auto-repeat>`](https://drafts.csswg.org/css-grid/#typedef-auto-repeat)
    Auto,
    /// [`<track-repeat>`](https://drafts.csswg.org/css-grid/#typedef-track-repeat)
    Normal,
    /// [`<fixed-repeat>`](https://drafts.csswg.org/css-grid/#typedef-fixed-repeat)
    Fixed,
}

/// The structure containing `<line-names>` and `<track-size>` values.
///
/// It can also hold `repeat()` function parameters, which expands into the respective
/// values in its computed form.
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct TrackRepeat<L> {
    /// The number of times for the value to be repeated (could also be `auto-fit` or `auto-fill`)
    pub count: RepeatCount,
    /// `<line-names>` accompanying `<track_size>` values.
    ///
    /// If there's no `<line-names>`, then it's represented by an empty vector.
    /// For N `<track-size>` values, there will be N+1 `<line-names>`, and so this vector's
    /// length is always one value more than that of the `<track-size>`.
    pub line_names: Vec<Vec<String>>,
    /// `<track-size>` values.
    pub track_sizes: Vec<TrackSize<L>>,
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

                let mut names = vec![];
                let mut values = vec![];
                let mut current_names;

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

                        values.push(track_size);
                        names.push(current_names);
                    } else {
                        if values.is_empty() {
                            return Err(())      // expecting at least one <track-size>
                        }

                        names.push(current_names);      // final `<line-names>`
                        break       // no more <track-size>, breaking
                    }
                }

                let repeat = TrackRepeat {
                    count: count,
                    track_sizes: values,
                    line_names: names,
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

        let mut line_names_iter = self.line_names.iter();
        for (i, (ref size, ref names)) in self.track_sizes.iter()
                                              .zip(&mut line_names_iter).enumerate() {
            if i > 0 {
                dest.write_str(" ")?;
            }

            concat_serialize_idents("[", "] ", names, " ", dest)?;
            size.to_css(dest)?;
        }

        if let Some(line_names_last) = line_names_iter.next() {
            concat_serialize_idents(" [", "]", line_names_last, " ", dest)?;
        }

        dest.write_str(")")?;
        Ok(())
    }
}

impl HasViewportPercentage for TrackRepeat<LengthOrPercentage> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool {
        self.track_sizes.iter().any(|ref v| v.has_viewport_percentage())
    }
}
