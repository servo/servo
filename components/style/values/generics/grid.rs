/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for the handling of
//! [grids](https://drafts.csswg.org/css-grid/).

use cssparser::Parser;
use parser::{Parse, ParserContext};
use std::{mem, usize};
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};
use values::{CSSFloat, CustomIdent};
use values::computed::{Context, ToComputedValue};
use values::specified;
use values::specified::grid::parse_line_names;

/// A `<grid-line>` type.
///
/// <https://drafts.csswg.org/css-grid/#typedef-grid-row-start-grid-line>
#[derive(Clone, Debug, Default, MallocSizeOf, PartialEq, SpecifiedValueInfo,
         ToComputedValue)]
pub struct GridLine<Integer> {
    /// Flag to check whether it's a `span` keyword.
    pub is_span: bool,
    /// A custom identifier for named lines.
    ///
    /// <https://drafts.csswg.org/css-grid/#grid-placement-slot>
    pub ident: Option<CustomIdent>,
    /// Denotes the nth grid line from grid item's placement.
    pub line_num: Option<Integer>,
}

impl<Integer> GridLine<Integer> {
    /// The `auto` value.
    pub fn auto() -> Self {
        Self {
            is_span: false,
            line_num: None,
            ident: None,
        }
    }

    /// Check whether this `<grid-line>` represents an `auto` value.
    pub fn is_auto(&self) -> bool {
        self.ident.is_none() && self.line_num.is_none() && !self.is_span
    }
}

impl<Integer> ToCss for GridLine<Integer>
where
    Integer: ToCss,
{
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.is_auto() {
            return dest.write_str("auto");
        }

        if self.is_span {
            dest.write_str("span")?;
        }

        if let Some(ref i) = self.line_num {
            if self.is_span {
                dest.write_str(" ")?;
            }
            i.to_css(dest)?;
        }

        if let Some(ref s) = self.ident {
            if self.is_span || self.line_num.is_some() {
                dest.write_str(" ")?;
            }
            s.to_css(dest)?;
        }

        Ok(())
    }
}

impl Parse for GridLine<specified::Integer> {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let mut grid_line = Self::auto();
        if input.try(|i| i.expect_ident_matching("auto")).is_ok() {
            return Ok(grid_line);
        }

        // <custom-ident> | [ <integer> && <custom-ident>? ] | [ span && [ <integer> || <custom-ident> ] ]
        // This <grid-line> horror is simply,
        // [ span? && [ <custom-ident> || <integer> ] ]
        // And, for some magical reason, "span" should be the first or last value and not in-between.
        let mut val_before_span = false;

        for _ in 0..3 {
            // Maximum possible entities for <grid-line>
            let location = input.current_source_location();
            if input.try(|i| i.expect_ident_matching("span")).is_ok() {
                if grid_line.is_span {
                    return Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                }

                if grid_line.line_num.is_some() || grid_line.ident.is_some() {
                    val_before_span = true;
                }

                grid_line.is_span = true;
            } else if let Ok(i) = input.try(|i| specified::Integer::parse(context, i)) {
                // FIXME(emilio): Probably shouldn't reject if it's calc()...
                if i.value() == 0 || val_before_span || grid_line.line_num.is_some() {
                    return Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                }

                grid_line.line_num = Some(i);
            } else if let Ok(name) = input.try(|i| i.expect_ident_cloned()) {
                if val_before_span || grid_line.ident.is_some() {
                    return Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                }
                grid_line.ident = Some(CustomIdent::from_ident(location, &name, &[])?);
            } else {
                break;
            }
        }

        if grid_line.is_auto() {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        if grid_line.is_span {
            if let Some(i) = grid_line.line_num {
                if i.value() <= 0 {
                    // disallow negative integers for grid spans
                    return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                }
            } else if grid_line.ident.is_none() {
                // integer could be omitted
                return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
            }
        }

        Ok(grid_line)
    }
}

#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, Parse, PartialEq,
         SpecifiedValueInfo, ToComputedValue, ToCss)]
pub enum TrackKeyword {
    Auto,
    MaxContent,
    MinContent,
}

/// A track breadth for explicit grid track sizing. It's generic solely to
/// avoid re-implementing it for the computed type.
///
/// <https://drafts.csswg.org/css-grid/#typedef-track-breadth>
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo,
         ToComputedValue, ToCss)]
pub enum TrackBreadth<L> {
    /// The generic type is almost always a non-negative `<length-percentage>`
    Breadth(L),
    /// A flex fraction specified in `fr` units.
    #[css(dimension)]
    Fr(CSSFloat),
    /// One of the track-sizing keywords (`auto`, `min-content`, `max-content`)
    Keyword(TrackKeyword),
}

impl<L> TrackBreadth<L> {
    /// Check whether this is a `<fixed-breadth>` (i.e., it only has `<length-percentage>`)
    ///
    /// <https://drafts.csswg.org/css-grid/#typedef-fixed-breadth>
    #[inline]
    pub fn is_fixed(&self) -> bool {
        match *self {
            TrackBreadth::Breadth(ref _lop) => true,
            _ => false,
        }
    }
}

/// A `<track-size>` type for explicit grid track sizing. Like `<track-breadth>`, this is
/// generic only to avoid code bloat. It only takes `<length-percentage>`
///
/// <https://drafts.csswg.org/css-grid/#typedef-track-size>
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo)]
pub enum TrackSize<L> {
    /// A flexible `<track-breadth>`
    Breadth(TrackBreadth<L>),
    /// A `minmax` function for a range over an inflexible `<track-breadth>`
    /// and a flexible `<track-breadth>`
    ///
    /// <https://drafts.csswg.org/css-grid/#valdef-grid-template-columns-minmax>
    #[css(function)]
    Minmax(TrackBreadth<L>, TrackBreadth<L>),
    /// A `fit-content` function.
    ///
    /// <https://drafts.csswg.org/css-grid/#valdef-grid-template-columns-fit-content>
    #[css(function)]
    FitContent(L),
}

impl<L> TrackSize<L> {
    /// Check whether this is a `<fixed-size>`
    ///
    /// <https://drafts.csswg.org/css-grid/#typedef-fixed-size>
    pub fn is_fixed(&self) -> bool {
        match *self {
            TrackSize::Breadth(ref breadth) => breadth.is_fixed(),
            // For minmax function, it could be either
            // minmax(<fixed-breadth>, <track-breadth>) or minmax(<inflexible-breadth>, <fixed-breadth>),
            // and since both variants are a subset of minmax(<inflexible-breadth>, <track-breadth>), we only
            // need to make sure that they're fixed. So, we don't have to modify the parsing function.
            TrackSize::Minmax(ref breadth_1, ref breadth_2) => {
                if breadth_1.is_fixed() {
                    return true; // the second value is always a <track-breadth>
                }

                match *breadth_1 {
                    TrackBreadth::Fr(_) => false, // should be <inflexible-breadth> at this point
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

impl<L: PartialEq> TrackSize<L> {
    /// Returns true if current TrackSize is same as default.
    pub fn is_default(&self) -> bool {
        *self == TrackSize::default()
    }
}

impl<L: ToCss> ToCss for TrackSize<L> {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match *self {
            TrackSize::Breadth(ref breadth) => breadth.to_css(dest),
            TrackSize::Minmax(ref min, ref max) => {
                // According to gecko minmax(auto, <flex>) is equivalent to <flex>,
                // and both are serialized as <flex>.
                if let TrackBreadth::Keyword(TrackKeyword::Auto) = *min {
                    if let TrackBreadth::Fr(_) = *max {
                        return max.to_css(dest);
                    }
                }

                dest.write_str("minmax(")?;
                min.to_css(dest)?;
                dest.write_str(", ")?;
                max.to_css(dest)?;
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
            TrackSize::Breadth(TrackBreadth::Fr(ref f)) => {
                // <flex> outside `minmax()` expands to `mimmax(auto, <flex>)`
                // https://drafts.csswg.org/css-grid/#valdef-grid-template-columns-flex
                // FIXME(nox): This sounds false, the spec just says that <flex>
                // implies `minmax(auto, <flex>)`, not that it should be changed
                // into `minmax` at computed value time.
                TrackSize::Minmax(
                    TrackBreadth::Keyword(TrackKeyword::Auto),
                    TrackBreadth::Fr(f.to_computed_value(context)),
                )
            },
            TrackSize::Breadth(ref b) => TrackSize::Breadth(b.to_computed_value(context)),
            TrackSize::Minmax(ref b1, ref b2) => {
                TrackSize::Minmax(b1.to_computed_value(context), b2.to_computed_value(context))
            },
            TrackSize::FitContent(ref lop) => TrackSize::FitContent(lop.to_computed_value(context)),
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            TrackSize::Breadth(ref b) => {
                TrackSize::Breadth(ToComputedValue::from_computed_value(b))
            },
            TrackSize::Minmax(ref b1, ref b2) => TrackSize::Minmax(
                ToComputedValue::from_computed_value(b1),
                ToComputedValue::from_computed_value(b2),
            ),
            TrackSize::FitContent(ref lop) => {
                TrackSize::FitContent(ToComputedValue::from_computed_value(lop))
            },
        }
    }
}

/// Helper function for serializing identifiers with a prefix and suffix, used
/// for serializing <line-names> (in grid).
pub fn concat_serialize_idents<W>(
    prefix: &str,
    suffix: &str,
    slice: &[CustomIdent],
    sep: &str,
    dest: &mut CssWriter<W>,
) -> fmt::Result
where
    W: Write,
{
    if let Some((ref first, rest)) = slice.split_first() {
        dest.write_str(prefix)?;
        first.to_css(dest)?;
        for thing in rest {
            dest.write_str(sep)?;
            thing.to_css(dest)?;
        }

        dest.write_str(suffix)?;
    }

    Ok(())
}

/// The initial argument of the `repeat` function.
///
/// <https://drafts.csswg.org/css-grid/#typedef-track-repeat>
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToComputedValue, ToCss)]
pub enum RepeatCount<Integer> {
    /// A positive integer. This is allowed only for `<track-repeat>` and `<fixed-repeat>`
    Number(Integer),
    /// An `<auto-fill>` keyword allowed only for `<auto-repeat>`
    AutoFill,
    /// An `<auto-fit>` keyword allowed only for `<auto-repeat>`
    AutoFit,
}

impl Parse for RepeatCount<specified::Integer> {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        // Maximum number of repeat is 10000. The greater numbers should be clamped.
        const MAX_LINE: i32 = 10000;
        if let Ok(mut i) = input.try(|i| specified::Integer::parse_positive(context, i)) {
            if i.value() > MAX_LINE {
                i = specified::Integer::new(MAX_LINE);
            }
            Ok(RepeatCount::Number(i))
        } else {
            try_match_ident_ignore_ascii_case! { input,
                "auto-fill" => Ok(RepeatCount::AutoFill),
                "auto-fit" => Ok(RepeatCount::AutoFit),
            }
        }
    }
}

/// The structure containing `<line-names>` and `<track-size>` values.
///
/// It can also hold `repeat()` function parameters, which expands into the respective
/// values in its computed form.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo,
         ToComputedValue)]
#[css(function = "repeat")]
pub struct TrackRepeat<L, I> {
    /// The number of times for the value to be repeated (could also be `auto-fit` or `auto-fill`)
    pub count: RepeatCount<I>,
    /// `<line-names>` accompanying `<track_size>` values.
    ///
    /// If there's no `<line-names>`, then it's represented by an empty vector.
    /// For N `<track-size>` values, there will be N+1 `<line-names>`, and so this vector's
    /// length is always one value more than that of the `<track-size>`.
    pub line_names: Box<[Box<[CustomIdent]>]>,
    /// `<track-size>` values.
    pub track_sizes: Vec<TrackSize<L>>,
}

impl<L: ToCss, I: ToCss> ToCss for TrackRepeat<L, I> {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        dest.write_str("repeat(")?;
        self.count.to_css(dest)?;
        dest.write_str(", ")?;

        let mut line_names_iter = self.line_names.iter();
        for (i, (ref size, ref names)) in self.track_sizes
            .iter()
            .zip(&mut line_names_iter)
            .enumerate()
        {
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
impl<L: Clone> TrackRepeat<L, specified::Integer> {
    /// If the repeat count is numeric, then expand the values and merge accordingly.
    pub fn expand(&self) -> Self {
        if let RepeatCount::Number(num) = self.count {
            let mut line_names = vec![];
            let mut track_sizes = vec![];
            let mut prev_names = vec![];

            for _ in 0..num.value() {
                let mut names_iter = self.line_names.iter();
                for (size, names) in self.track_sizes.iter().zip(&mut names_iter) {
                    prev_names.extend_from_slice(&names);
                    let vec = mem::replace(&mut prev_names, vec![]);
                    line_names.push(vec.into_boxed_slice());
                    track_sizes.push(size.clone());
                }

                if let Some(names) = names_iter.next() {
                    prev_names.extend_from_slice(&names);
                }
            }

            line_names.push(prev_names.into_boxed_slice());
            TrackRepeat {
                count: self.count,
                track_sizes: track_sizes,
                line_names: line_names.into_boxed_slice(),
            }
        } else {
            // if it's auto-fit/auto-fill, then it's left to the layout.
            TrackRepeat {
                count: self.count,
                track_sizes: self.track_sizes.clone(),
                line_names: self.line_names.clone(),
            }
        }
    }
}

/// Track list values. Can be <track-size> or <track-repeat>
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo,
         ToComputedValue, ToCss)]
pub enum TrackListValue<LengthOrPercentage, Integer> {
    /// A <track-size> value.
    TrackSize(TrackSize<LengthOrPercentage>),
    /// A <track-repeat> value.
    TrackRepeat(TrackRepeat<LengthOrPercentage, Integer>),
}

/// The type of a `<track-list>` as determined during parsing.
///
/// <https://drafts.csswg.org/css-grid/#typedef-track-list>
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToComputedValue)]
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
/// <https://drafts.csswg.org/css-grid/#typedef-track-list>
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo)]
pub struct TrackList<LengthOrPercentage, Integer> {
    /// The type of this `<track-list>` (auto, explicit or general).
    ///
    /// In order to avoid parsing the same value multiple times, this does a single traversal
    /// and arrives at the type of value it has parsed (or bails out gracefully with an error).
    #[css(skip)]
    pub list_type: TrackListType,
    /// A vector of `<track-size> | <track-repeat>` values.
    pub values: Vec<TrackListValue<LengthOrPercentage, Integer>>,
    /// `<line-names>` accompanying `<track-size> | <track-repeat>` values.
    ///
    /// If there's no `<line-names>`, then it's represented by an empty vector.
    /// For N values, there will be N+1 `<line-names>`, and so this vector's
    /// length is always one value more than that of the `<track-size>`.
    pub line_names: Box<[Box<[CustomIdent]>]>,
    /// `<auto-repeat>` value. There can only be one `<auto-repeat>` in a TrackList.
    pub auto_repeat: Option<TrackRepeat<LengthOrPercentage, Integer>>,
}

impl<L: ToCss, I: ToCss> ToCss for TrackList<L, I> {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        let auto_idx = match self.list_type {
            TrackListType::Auto(i) => i as usize,
            _ => usize::MAX,
        };

        let mut values_iter = self.values.iter().peekable();
        let mut line_names_iter = self.line_names.iter().peekable();

        for idx in 0.. {
            let names = line_names_iter.next().unwrap(); // This should exist!
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

            if values_iter.peek().is_some() ||
                line_names_iter.peek().map_or(false, |v| !v.is_empty()) ||
                (idx + 1 == auto_idx)
            {
                dest.write_str(" ")?;
            }
        }

        Ok(())
    }
}

/// The `<line-name-list>` for subgrids.
///
/// `subgrid [ <line-names> | repeat(<positive-integer> | auto-fill, <line-names>+) ]+`
/// Old spec: https://www.w3.org/TR/2015/WD-css-grid-1-20150917/#typedef-line-name-list
#[derive(Clone, Debug, Default, MallocSizeOf, PartialEq, SpecifiedValueInfo,
         ToComputedValue)]
pub struct LineNameList {
    /// The optional `<line-name-list>`
    pub names: Box<[Box<[CustomIdent]>]>,
    /// Indicates the line name that requires `auto-fill`
    pub fill_idx: Option<u32>,
}

impl Parse for LineNameList {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        input.expect_ident_matching("subgrid")?;
        let mut line_names = vec![];
        let mut fill_idx = None;

        loop {
            let repeat_parse_result = input.try(|input| {
                input.expect_function_matching("repeat")?;
                input.parse_nested_block(|input| {
                    let count = RepeatCount::parse(context, input)?;
                    input.expect_comma()?;
                    let mut names_list = vec![];
                    names_list.push(parse_line_names(input)?); // there should be at least one
                    while let Ok(names) = input.try(parse_line_names) {
                        names_list.push(names);
                    }

                    Ok((names_list, count))
                })
            });

            if let Ok((mut names_list, count)) = repeat_parse_result {
                match count {
                    RepeatCount::Number(num) => line_names.extend(
                        names_list
                            .iter()
                            .cloned()
                            .cycle()
                            .take(num.value() as usize * names_list.len()),
                    ),
                    RepeatCount::AutoFill if fill_idx.is_none() => {
                        // `repeat(autof-fill, ..)` should have just one line name.
                        if names_list.len() != 1 {
                            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                        }
                        let names = names_list.pop().unwrap();

                        line_names.push(names);
                        fill_idx = Some(line_names.len() as u32 - 1);
                    },
                    _ => return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError)),
                }
            } else if let Ok(names) = input.try(parse_line_names) {
                line_names.push(names);
            } else {
                break;
            }
        }

        Ok(LineNameList {
            names: line_names.into_boxed_slice(),
            fill_idx: fill_idx,
        })
    }
}

impl ToCss for LineNameList {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        dest.write_str("subgrid")?;
        let fill_idx = self.fill_idx.map(|v| v as usize).unwrap_or(usize::MAX);
        for (i, names) in self.names.iter().enumerate() {
            if i == fill_idx {
                dest.write_str(" repeat(auto-fill,")?;
            }

            dest.write_str(" [")?;

            if let Some((ref first, rest)) = names.split_first() {
                first.to_css(dest)?;
                for name in rest {
                    dest.write_str(" ")?;
                    name.to_css(dest)?;
                }
            }

            dest.write_str("]")?;
            if i == fill_idx {
                dest.write_str(")")?;
            }
        }

        Ok(())
    }
}

/// Variants for `<grid-template-rows> | <grid-template-columns>`
/// Subgrid deferred to Level 2 spec due to lack of implementation.
/// But it's implemented in gecko, so we have to as well.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo,
         ToComputedValue, ToCss)]
pub enum GridTemplateComponent<L, I> {
    /// `none` value.
    None,
    /// The grid `<track-list>`
    TrackList(#[compute(field_bound)] TrackList<L, I>),
    /// A `subgrid <line-name-list>?`
    Subgrid(LineNameList),
}

impl<L, I> GridTemplateComponent<L, I> {
    /// Returns length of the <track-list>s <track-size>
    pub fn track_list_len(&self) -> usize {
        match *self {
            GridTemplateComponent::TrackList(ref tracklist) => tracklist.values.len(),
            _ => 0,
        }
    }
}
