/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the computed value of
//! [grids](https://drafts.csswg.org/css-grid/)

use cssparser::{Parser, Token, ParseError as CssParseError};
use parser::{Parse, ParserContext};
#[allow(unused_imports)] use std::ascii::AsciiExt;
use std::mem;
use style_traits::{ParseError, StyleParseErrorKind};
use values::{CSSFloat, CustomIdent};
use values::computed::{self, Context, ToComputedValue};
use values::generics::grid::{GridTemplateComponent, RepeatCount, TrackBreadth, TrackKeyword, TrackRepeat};
use values::generics::grid::{LineNameList, TrackSize, TrackList, TrackListType, TrackListValue};
use values::specified::{LengthOrPercentage, Integer};

/// Parse a single flexible length.
pub fn parse_flex<'i, 't>(input: &mut Parser<'i, 't>) -> Result<CSSFloat, ParseError<'i>> {
    let location = input.current_source_location();
    match *input.next()? {
        Token::Dimension { value, ref unit, .. } if unit.eq_ignore_ascii_case("fr") && value.is_sign_positive()
            => Ok(value),
        ref t => Err(location.new_unexpected_token_error(t.clone())),
    }
}

impl Parse for TrackBreadth<LengthOrPercentage> {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if let Ok(lop) = input.try(|i| LengthOrPercentage::parse_non_negative(context, i)) {
            return Ok(TrackBreadth::Breadth(lop))
        }

        if let Ok(f) = input.try(parse_flex) {
            return Ok(TrackBreadth::Fr(f))
        }

        TrackKeyword::parse(input).map(TrackBreadth::Keyword)
    }
}

impl Parse for TrackSize<LengthOrPercentage> {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if let Ok(b) = input.try(|i| TrackBreadth::parse(context, i)) {
            return Ok(TrackSize::Breadth(b))
        }

        if input.try(|i| i.expect_function_matching("minmax")).is_ok() {
            return input.parse_nested_block(|input| {
                let inflexible_breadth =
                    match input.try(|i| LengthOrPercentage::parse_non_negative(context, i)) {
                        Ok(lop) => TrackBreadth::Breadth(lop),
                        Err(..) => {
                            let keyword = TrackKeyword::parse(input)?;
                            TrackBreadth::Keyword(keyword)
                        }
                    };

                input.expect_comma()?;
                Ok(TrackSize::Minmax(inflexible_breadth, TrackBreadth::parse(context, input)?))
            });
        }

        input.expect_function_matching("fit-content")?;
        let lop = input.parse_nested_block(|i| LengthOrPercentage::parse_non_negative(context, i))?;
        Ok(TrackSize::FitContent(lop))
    }
}

/// Parse the grid line names into a vector of owned strings.
///
/// <https://drafts.csswg.org/css-grid/#typedef-line-names>
pub fn parse_line_names<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Box<[CustomIdent]>, ParseError<'i>> {
    input.expect_square_bracket_block()?;
    input.parse_nested_block(|input| {
        let mut values = vec![];
        while let Ok((loc, ident)) = input.try(|i| -> Result<_, CssParseError<()>> {
             Ok((i.current_source_location(), i.expect_ident_cloned()?))
        }) {
            let ident = CustomIdent::from_ident(loc, &ident, &["span"])?;
            values.push(ident);
        }

        Ok(values.into_boxed_slice())
    })
}

/// The type of `repeat` function (only used in parsing).
///
/// <https://drafts.csswg.org/css-grid/#typedef-track-repeat>
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
enum RepeatType {
    /// [`<auto-repeat>`](https://drafts.csswg.org/css-grid/#typedef-auto-repeat)
    Auto,
    /// [`<track-repeat>`](https://drafts.csswg.org/css-grid/#typedef-track-repeat)
    Normal,
    /// [`<fixed-repeat>`](https://drafts.csswg.org/css-grid/#typedef-fixed-repeat)
    Fixed,
}

impl TrackRepeat<LengthOrPercentage, Integer> {
    fn parse_with_repeat_type<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<(Self, RepeatType), ParseError<'i>> {
        input.try(|i| i.expect_function_matching("repeat").map_err(|e| e.into())).and_then(|_| {
            input.parse_nested_block(|input| {
                let count = RepeatCount::parse(context, input)?;
                input.expect_comma()?;

                let is_auto = count == RepeatCount::AutoFit || count == RepeatCount::AutoFill;
                let mut repeat_type = if is_auto {
                    RepeatType::Auto
                } else {    // <fixed-size> is a subset of <track-size>, so it should work for both
                    RepeatType::Fixed
                };

                let mut names = vec![];
                let mut values = vec![];
                let mut current_names;

                loop {
                    current_names = input.try(parse_line_names).unwrap_or(vec![].into_boxed_slice());
                    if let Ok(track_size) = input.try(|i| TrackSize::parse(context, i)) {
                        if !track_size.is_fixed() {
                            if is_auto {
                                // should be <fixed-size> for <auto-repeat>
                                return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
                            }

                            if repeat_type == RepeatType::Fixed {
                                repeat_type = RepeatType::Normal       // <track-size> for sure
                            }
                        }

                        values.push(track_size);
                        names.push(current_names);
                        if is_auto {
                            // FIXME: In the older version of the spec
                            // (https://www.w3.org/TR/2015/WD-css-grid-1-20150917/#typedef-auto-repeat),
                            // if the repeat type is `<auto-repeat>` we shouldn't try to parse more than
                            // one `TrackSize`. But in current version of the spec, this is deprecated
                            // but we are adding this for gecko parity. We should remove this when
                            // gecko implements new spec.
                            names.push(input.try(parse_line_names).unwrap_or(vec![].into_boxed_slice()));
                            break
                        }
                    } else {
                        if values.is_empty() {
                            // expecting at least one <track-size>
                            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
                        }

                        names.push(current_names);      // final `<line-names>`
                        break       // no more <track-size>, breaking
                    }
                }

                let repeat = TrackRepeat {
                    count: count,
                    track_sizes: values,
                    line_names: names.into_boxed_slice(),
                };

                Ok((repeat, repeat_type))
            })
        })
    }
}

impl Parse for TrackList<LengthOrPercentage, Integer> {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        let mut current_names = vec![];
        let mut names = vec![];
        let mut values = vec![];

        let mut list_type = TrackListType::Explicit;    // assume it's the simplest case
        // holds <auto-repeat> value. It can only be only one in a TrackList.
        let mut auto_repeat = None;
        // if there is any <auto-repeat> the list will be of type TrackListType::Auto(idx)
        // where idx points to the position of the <auto-repeat> in the track list. If there
        // is any repeat before <auto-repeat>, we need to take the number of repetitions into
        // account to set the position of <auto-repeat> so it remains the same while computing
        // values.
        let mut auto_offset = 0;
        // assume that everything is <fixed-size>. This flag is useful when we encounter <auto-repeat>
        let mut atleast_one_not_fixed = false;
        loop {
            current_names.extend_from_slice(&mut input.try(parse_line_names).unwrap_or(vec![].into_boxed_slice()));
            if let Ok(track_size) = input.try(|i| TrackSize::parse(context, i)) {
                if !track_size.is_fixed() {
                    atleast_one_not_fixed = true;
                    if auto_repeat.is_some() {
                        // <auto-track-list> only accepts <fixed-size> and <fixed-repeat>
                        return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
                    }
                }

                let vec = mem::replace(&mut current_names, vec![]);
                names.push(vec.into_boxed_slice());
                values.push(TrackListValue::TrackSize(track_size));
            } else if let Ok((repeat, type_)) = input.try(|i| TrackRepeat::parse_with_repeat_type(context, i)) {
                if list_type == TrackListType::Explicit {
                    list_type = TrackListType::Normal;      // <explicit-track-list> doesn't contain repeat()
                }

                match type_ {
                    RepeatType::Normal => {
                        atleast_one_not_fixed = true;
                        if auto_repeat.is_some() { // only <fixed-repeat>
                            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
                        }
                    },
                    RepeatType::Auto => {
                        if auto_repeat.is_some() || atleast_one_not_fixed {
                            // We've either seen <auto-repeat> earlier, or there's at least one non-fixed value
                            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
                        }

                        list_type = TrackListType::Auto(values.len() as u16 + auto_offset);
                        auto_repeat = Some(repeat);
                        let vec = mem::replace(&mut current_names, vec![]);
                        names.push(vec.into_boxed_slice());
                        continue;
                    },
                    RepeatType::Fixed => (),
                }

                let vec = mem::replace(&mut current_names, vec![]);
                names.push(vec.into_boxed_slice());
                if let RepeatCount::Number(num) = repeat.count {
                    auto_offset += (num.value() - 1) as u16;
                }
                values.push(TrackListValue::TrackRepeat(repeat));
            } else {
                if values.is_empty() && auto_repeat.is_none() {
                    return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
                }

                names.push(current_names.into_boxed_slice());
                break
            }
        }

        Ok(TrackList {
            list_type: list_type,
            values: values,
            line_names: names.into_boxed_slice(),
            auto_repeat: auto_repeat,
        })
    }
}

impl ToComputedValue for TrackList<LengthOrPercentage, Integer> {
    type ComputedValue = TrackList<computed::LengthOrPercentage, computed::Integer>;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        // Merge the line names while computing values. The resulting values will
        // all be bunch of `<track-size>` and one <auto-repeat>.
        //
        // For example,
        // `[a b] 100px [c d] repeat(1, 30px [g]) [h]` will be merged as `[a b] 100px [c d] 30px [g h]`
        //  whereas, `[a b] repeat(2, [c] 50px [d]) [e f] repeat(auto-fill, [g] 12px) 10px [h]` will be merged as
        // `[a b c] 50px [d c] 50px [d e f] repeat(auto-fill, [g] 12px) 10px [h]`, with the `<auto-repeat>` value
        // set in the `auto_repeat` field, and the `idx` in TrackListType::Auto pointing to the values after
        // `<auto-repeat>` (in this case, `10px [h]`).
        let mut prev_names = vec![];
        let mut line_names = Vec::with_capacity(self.line_names.len() + 1);
        let mut values = Vec::with_capacity(self.values.len() + 1);
        for (pos, names) in self.line_names.iter().enumerate() {
            prev_names.extend_from_slice(&names);
            if pos >= self.values.len() {
                let vec = mem::replace(&mut prev_names, vec![]);
                line_names.push(vec.into_boxed_slice());
                continue;
            }

            match self.values[pos] {
                TrackListValue::TrackSize(ref size) => {
                    let vec = mem::replace(&mut prev_names, vec![]);
                    line_names.push(vec.into_boxed_slice());
                    values.push(TrackListValue::TrackSize(size.to_computed_value(context)));
                },
                TrackListValue::TrackRepeat(ref repeat) => {
                    // If the repeat count is numeric, we expand and merge the values.
                    let mut repeat = repeat.expand();
                    let mut repeat_names_iter = repeat.line_names.iter();
                    for (size, repeat_names) in repeat.track_sizes.drain(..).zip(&mut repeat_names_iter) {
                        prev_names.extend_from_slice(&repeat_names);
                        let vec = mem::replace(&mut prev_names, vec![]);
                        line_names.push(vec.into_boxed_slice());
                        values.push(TrackListValue::TrackSize(size.to_computed_value(context)));
                    }

                    if let Some(names) = repeat_names_iter.next() {
                        prev_names.extend_from_slice(&names);
                    }
                },
            }
        }

        TrackList {
            list_type: self.list_type.to_computed_value(context),
            values: values,
            line_names: line_names.into_boxed_slice(),
            auto_repeat: self.auto_repeat.clone().map(|repeat| repeat.to_computed_value(context)),
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        let mut values = Vec::with_capacity(computed.values.len() + 1);
        for value in computed.values.iter().map(ToComputedValue::from_computed_value) {
            values.push(value);
        }

        TrackList {
            list_type: computed.list_type,
            values: values,
            line_names: computed.line_names.clone(),
            auto_repeat: computed.auto_repeat.clone().map(|ref repeat| TrackRepeat::from_computed_value(repeat)),
        }
    }
}

#[cfg(feature = "gecko")]
#[inline]
fn allow_grid_template_subgrids() -> bool {
    use gecko_bindings::structs::mozilla;
    unsafe { mozilla::StylePrefs_sGridTemplateSubgridValueEnabled }
}

#[cfg(feature = "servo")]
#[inline]
fn allow_grid_template_subgrids() -> bool {
    false
}

impl Parse for GridTemplateComponent<LengthOrPercentage, Integer> {
    // FIXME: Derive Parse (probably with None_)
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(GridTemplateComponent::None)
        }

        Self::parse_without_none(context, input)
    }
}

impl GridTemplateComponent<LengthOrPercentage, Integer> {
    /// Parses a `GridTemplateComponent<LengthOrPercentage>` except `none` keyword.
    pub fn parse_without_none<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if allow_grid_template_subgrids() {
            if let Ok(t) = input.try(|i| LineNameList::parse(context, i)) {
                return Ok(GridTemplateComponent::Subgrid(t))
            }
        }

        TrackList::parse(context, input).map(GridTemplateComponent::TrackList)
    }
}
