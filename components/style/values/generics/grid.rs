/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for the handling of
//! [grids](https://drafts.csswg.org/css-grid/).

use cssparser::{Parser, serialize_identifier};
use parser::{Parse, ParserContext};
use std::{fmt, mem, usize};
use style_traits::ToCss;
use values::{CSSFloat, CustomIdent};
use values::computed::{self, ComputedValueAsSpecified, Context, ToComputedValue};
use values::specified::Integer;

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
    pub line_num: Option<Integer>,
}

impl GridLine {
    /// Check whether this `<grid-line>` represents an `auto` value.
    pub fn is_auto(&self) -> bool {
        self.ident.is_none() && self.line_num.is_none() && !self.is_span
    }
}

impl Default for GridLine {
    fn default() -> Self {
        GridLine {
            is_span: false,
            ident: None,
            line_num: None,
        }
    }
}

impl ToCss for GridLine {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        if self.is_auto() {
            return dest.write_str("auto")
        }

        if self.is_span {
            dest.write_str("span")?;
        }

        if let Some(i) = self.line_num {
            write!(dest, " {}", i.value())?;
        }

        if let Some(ref s) = self.ident {
            dest.write_str(" ")?;
            serialize_identifier(s, dest)?;
        }

        Ok(())
    }
}

impl Parse for GridLine {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let mut grid_line = Default::default();
        if input.try(|i| i.expect_ident_matching("auto")).is_ok() {
            return Ok(grid_line)
        }

        for _ in 0..3 {     // Maximum possible entities for <grid-line>
            if input.try(|i| i.expect_ident_matching("span")).is_ok() {
                if grid_line.is_span || grid_line.line_num.is_some() || grid_line.ident.is_some() {
                    return Err(())      // span (if specified) should be first
                }
                grid_line.is_span = true;       // span (if specified) should be first
            } else if let Ok(i) = input.try(|i| Integer::parse(context, i)) {
                if i.value() == 0 || grid_line.line_num.is_some() {
                    return Err(())
                }
                grid_line.line_num = Some(i);
            } else if let Ok(name) = input.try(|i| i.expect_ident()) {
                if grid_line.ident.is_some() || CustomIdent::from_ident((&*name).into(), &[]).is_err() {
                    return Err(())
                }
                grid_line.ident = Some(name.into_owned());
            } else {
                break
            }
        }

        if grid_line.is_auto() {
            return Err(())
        }

        if grid_line.is_span {
            if let Some(i) = grid_line.line_num {
                if i.value() <= 0 {       // disallow negative integers for grid spans
                    return Err(())
                }
            } else if grid_line.ident.is_some() {       // integer could be omitted
                grid_line.line_num = Some(Integer::new(1));
            } else {
                return Err(())
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

impl<L: ToCss> ToCss for TrackBreadth<L> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            TrackBreadth::Breadth(ref lop) => lop.to_css(dest),
            TrackBreadth::Flex(ref value) => write!(dest, "{}fr", value),
            TrackBreadth::Keyword(ref k) => k.to_css(dest),
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

#[derive(Clone, Debug, HasViewportPercentage, PartialEq)]
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

impl<L: ToComputedValue> ToComputedValue for TrackSize<L> {
    type ComputedValue = TrackSize<L::ComputedValue>;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            TrackSize::Breadth(ref b) => match *b {
                // <flex> outside `minmax()` expands to `mimmax(auto, <flex>)`
                // https://drafts.csswg.org/css-grid/#valdef-grid-template-columns-flex
                TrackBreadth::Flex(f) =>
                    TrackSize::MinMax(TrackBreadth::Keyword(TrackKeyword::Auto), TrackBreadth::Flex(f)),
                _ => TrackSize::Breadth(b.to_computed_value(context)),
            },
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

impl ToCss for RepeatCount {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            RepeatCount::Number(ref c) => c.to_css(dest),
            RepeatCount::AutoFill => dest.write_str("auto-fill"),
            RepeatCount::AutoFit => dest.write_str("auto-fit"),
        }
    }
}

impl Parse for RepeatCount {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if let Ok(i) = input.try(|i| Integer::parse(context, i)) {
            if i.value() > 0 {
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

impl ComputedValueAsSpecified for RepeatCount {}
no_viewport_percentage!(RepeatCount);

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

impl<L: ToComputedValue> ToComputedValue for TrackRepeat<L> {
    type ComputedValue = TrackRepeat<L::ComputedValue>;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        // If the repeat count is numeric, then expand the values and merge accordingly.
        if let RepeatCount::Number(num) = self.count {
            let mut line_names = vec![];
            let mut track_sizes = vec![];
            let mut prev_names = vec![];

            for _ in 0..num.value() {
                let mut names_iter = self.line_names.iter();
                for (size, names) in self.track_sizes.iter().zip(&mut names_iter) {
                    prev_names.extend_from_slice(&names);
                    line_names.push(mem::replace(&mut prev_names, vec![]));
                    track_sizes.push(size.to_computed_value(context));
                }

                if let Some(names) = names_iter.next() {
                    prev_names.extend_from_slice(&names);
                }
            }

            line_names.push(prev_names);
            TrackRepeat {
                count: self.count,
                track_sizes: track_sizes,
                line_names: line_names,
            }

        } else {    // if it's auto-fit/auto-fill, then it's left to the layout.
            TrackRepeat {
                count: self.count,
                track_sizes: self.track_sizes.iter()
                                             .map(|l| l.to_computed_value(context))
                                             .collect(),
                line_names: self.line_names.clone(),
            }
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        TrackRepeat {
            count: computed.count,
            track_sizes: computed.track_sizes.iter()
                                             .map(ToComputedValue::from_computed_value)
                                             .collect(),
            line_names: computed.line_names.clone(),
        }
    }
}

/// The type of a `<track-list>` as determined during parsing.
///
/// https://drafts.csswg.org/css-grid/#typedef-track-list
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum TrackListType {
    /// [`<auto-track-list>`](https://drafts.csswg.org/css-grid/#typedef-auto-track-list)
    ///
    /// If this type exists, then the value at the index in `line_names` field in `TrackList`
    /// has the `<line-names>?` list that comes before `<auto-repeat>`. If it's a specified value,
    /// then the `repeat()` function (that follows the line names list) is also at the given index
    /// in `values` field. On the contrary, if it's a computed value, then the `repeat()` function
    /// is in the `auto_repeat` field.
    Auto(u16),
    /// [`<track-list>`](https://drafts.csswg.org/css-grid/#typedef-track-list)
    Normal,
    /// [`<explicit-track-list>`](https://drafts.csswg.org/css-grid/#typedef-explicit-track-list)
    ///
    /// Note that this is a subset of the normal `<track-list>`, and so it could be used in place
    /// of the latter.
    Explicit,
}

/// A grid `<track-list>` type.
///
/// https://drafts.csswg.org/css-grid/#typedef-track-list
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct TrackList<T> {
    /// The type of this `<track-list>` (auto, explicit or general).
    ///
    /// In order to avoid parsing the same value multiple times, this does a single traversal
    /// and arrives at the type of value it has parsed (or bails out gracefully with an error).
    pub list_type: TrackListType,
    /// A vector of `<track-size> | <track-repeat>` values. In its specified form, it may contain
    /// any value, but once it's computed, it contains only `<track_size>` values.
    ///
    /// Note that this may also contain `<auto-repeat>` at an index. If it exists, it's
    /// given by the index in `TrackListType::Auto`
    pub values: Vec<T>,
    /// `<line-names>` accompanying `<track-size> | <track-repeat>` values.
    ///
    /// If there's no `<line-names>`, then it's represented by an empty vector.
    /// For N values, there will be N+1 `<line-names>`, and so this vector's
    /// length is always one value more than that of the `<track-size>`.
    pub line_names: Vec<Vec<String>>,
    /// `<auto-repeat>` value after computation. This field is necessary, because
    /// the `values` field (after computation) will only contain `<track-size>` values, and
    /// we need something to represent this function.
    pub auto_repeat: Option<TrackRepeat<computed::LengthOrPercentage>>,
}

impl<T: ToCss> ToCss for TrackList<T> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let auto_idx = match self.list_type {
            TrackListType::Auto(i) => i as usize,
            _ => usize::MAX,
        };

        let mut values_iter = self.values.iter().peekable();
        let mut line_names_iter = self.line_names.iter().peekable();

        for idx in 0.. {
            let names = line_names_iter.next().unwrap();    // This should exist!
            concat_serialize_idents("[", "]", names, " ", dest)?;

            match self.auto_repeat {
                Some(ref repeat) if idx == auto_idx => {
                    if !names.is_empty() {
                        dest.write_str(" ")?;
                    }

                    repeat.to_css(dest)?;
                },
                _ => match values_iter.next() {
                    Some(value) => {
                        if !names.is_empty() {
                            dest.write_str(" ")?;
                        }

                        value.to_css(dest)?;
                    },
                    None => break,
                },
            }

            if values_iter.peek().is_some() || line_names_iter.peek().map_or(false, |v| !v.is_empty()) {
                dest.write_str(" ")?;
            }
        }

        Ok(())
    }
}
