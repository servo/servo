/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the computed value of
//! [grids](https://drafts.csswg.org/css-grid/)

use cssparser::{Parser, Token, BasicParseError};
use parser::{Parse, ParserContext};
use std::ascii::AsciiExt;
use std::mem;
use style_traits::{ParseError, StyleParseError};
use values::{CSSFloat, CustomIdent};
use values::generics::grid::{GridTemplateComponent, RepeatCount, TrackBreadth, TrackKeyword, TrackRepeat};
use values::generics::grid::{LineNameList, TrackSize, TrackList, TrackListType};
use values::specified::LengthOrPercentage;

/// Parse a single flexible length.
pub fn parse_flex<'i, 't>(input: &mut Parser<'i, 't>) -> Result<CSSFloat, ParseError<'i>> {
    match *input.next()? {
        Token::Dimension { value, ref unit, .. } if unit.eq_ignore_ascii_case("fr") && value.is_sign_positive()
            => Ok(value),
        ref t => Err(BasicParseError::UnexpectedToken(t.clone()).into()),
    }
}

impl Parse for TrackBreadth<LengthOrPercentage> {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if let Ok(lop) = input.try(|i| LengthOrPercentage::parse_non_negative(context, i)) {
            return Ok(TrackBreadth::Breadth(lop))
        }

        if let Ok(f) = input.try(parse_flex) {
            return Ok(TrackBreadth::Flex(f))
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
/// https://drafts.csswg.org/css-grid/#typedef-line-names
pub fn parse_line_names<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Box<[CustomIdent]>, ParseError<'i>> {
    input.expect_square_bracket_block()?;
    input.parse_nested_block(|input| {
        let mut values = vec![];
        while let Ok(ident) = input.try(|i| i.expect_ident_cloned()) {
            let ident = CustomIdent::from_ident(&ident, &["span"])?;
            values.push(ident);
        }

        Ok(values.into_boxed_slice())
    })
}

/// The type of `repeat` function (only used in parsing).
///
/// https://drafts.csswg.org/css-grid/#typedef-track-repeat
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
enum RepeatType {
    /// [`<auto-repeat>`](https://drafts.csswg.org/css-grid/#typedef-auto-repeat)
    Auto,
    /// [`<track-repeat>`](https://drafts.csswg.org/css-grid/#typedef-track-repeat)
    Normal,
    /// [`<fixed-repeat>`](https://drafts.csswg.org/css-grid/#typedef-fixed-repeat)
    Fixed,
}

impl TrackRepeat<LengthOrPercentage> {
    fn parse_with_repeat_type<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                                      -> Result<(TrackRepeat<LengthOrPercentage>, RepeatType), ParseError<'i>> {
        input.try(|i| i.expect_function_matching("repeat").map_err(|e| e.into())).and_then(|_| {
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
                    current_names = input.try(parse_line_names).unwrap_or(vec![].into_boxed_slice());
                    if let Ok(track_size) = input.try(|i| TrackSize::parse(context, i)) {
                        if !track_size.is_fixed() {
                            if is_auto {
                                // should be <fixed-size> for <auto-repeat>
                                return Err(StyleParseError::UnspecifiedError.into())
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
                            return Err(StyleParseError::UnspecifiedError.into())
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

impl Parse for TrackList<LengthOrPercentage> {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        // Merge the line names while parsing values. The resulting values will
        // all be bunch of `<track-size>` and one <auto-repeat>.
        // FIXME: We need to decide which way is better for repeat function in
        // https://bugzilla.mozilla.org/show_bug.cgi?id=1382369.
        //
        // For example,
        // `[a b] 100px [c d] repeat(1, 30px [g]) [h]` will be merged as `[a b] 100px [c d] 30px [g h]`
        //  whereas, `[a b] repeat(2, [c] 50px [d]) [e f] repeat(auto-fill, [g] 12px) 10px [h]` will be merged as
        // `[a b c] 50px [d c] 50px [d e f] repeat(auto-fill, [g] 12px) 10px [h]`, with the `<auto-repeat>` value
        // set in the `auto_repeat` field, and the `idx` in TrackListType::Auto pointing to the values after
        // `<auto-repeat>` (in this case, `10px [h]`).
        let mut current_names = vec![];
        let mut names = vec![];
        let mut values = vec![];

        let mut list_type = TrackListType::Explicit;    // assume it's the simplest case
        // holds <auto-repeat> value. It can only be only one in a TrackList.
        let mut auto_repeat = None;
        // assume that everything is <fixed-size>. This flag is useful when we encounter <auto-repeat>
        let mut atleast_one_not_fixed = false;

        loop {
            current_names.extend_from_slice(&mut input.try(parse_line_names).unwrap_or(vec![].into_boxed_slice()));
            if let Ok(track_size) = input.try(|i| TrackSize::parse(context, i)) {
                if !track_size.is_fixed() {
                    atleast_one_not_fixed = true;
                    if auto_repeat.is_some() {
                        // <auto-track-list> only accepts <fixed-size> and <fixed-repeat>
                        return Err(StyleParseError::UnspecifiedError.into())
                    }
                }

                let vec = mem::replace(&mut current_names, vec![]);
                names.push(vec.into_boxed_slice());
                values.push(track_size);
            } else if let Ok((repeat, type_)) = input.try(|i| TrackRepeat::parse_with_repeat_type(context, i)) {
                if list_type == TrackListType::Explicit {
                    list_type = TrackListType::Normal;      // <explicit-track-list> doesn't contain repeat()
                }

                match type_ {
                    RepeatType::Normal => {
                        atleast_one_not_fixed = true;
                        if auto_repeat.is_some() { // only <fixed-repeat>
                            return Err(StyleParseError::UnspecifiedError.into())
                        }
                    },
                    RepeatType::Auto => {
                        if auto_repeat.is_some() || atleast_one_not_fixed {
                            // We've either seen <auto-repeat> earlier, or there's at least one non-fixed value
                            return Err(StyleParseError::UnspecifiedError.into())
                        }

                        list_type = TrackListType::Auto(values.len() as u16);
                        auto_repeat = Some(repeat);
                        let vec = mem::replace(&mut current_names, vec![]);
                        names.push(vec.into_boxed_slice());
                        continue
                    },
                    RepeatType::Fixed => (),
                }

                // If the repeat count is numeric, we axpand and merge the values.
                let mut repeat = repeat.expand();
                let mut repeat_names_iter = repeat.line_names.iter();
                for (size, repeat_names) in repeat.track_sizes.drain(..).zip(&mut repeat_names_iter) {
                    current_names.extend_from_slice(&repeat_names);
                    let vec = mem::replace(&mut current_names, vec![]);
                    names.push(vec.into_boxed_slice());
                    values.push(size);
                }

                if let Some(names) = repeat_names_iter.next() {
                    current_names.extend_from_slice(&names);
                }
            } else {
                if values.is_empty() && auto_repeat.is_none() {
                    return Err(StyleParseError::UnspecifiedError.into())
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

impl Parse for GridTemplateComponent<LengthOrPercentage> {
    // FIXME: Derive Parse (probably with None_)
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(GridTemplateComponent::None)
        }

        Self::parse_without_none(context, input)
    }
}

impl GridTemplateComponent<LengthOrPercentage> {
    /// Parses a `GridTemplateComponent<LengthOrPercentage>` except `none` keyword.
    pub fn parse_without_none<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                                      -> Result<Self, ParseError<'i>> {
        if let Ok(t) = input.try(|i| TrackList::parse(context, i)) {
            return Ok(GridTemplateComponent::TrackList(t))
        }

        LineNameList::parse(context, input).map(GridTemplateComponent::Subgrid)
    }
}
