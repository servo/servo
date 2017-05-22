/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the computed value of
//! [grids](https://drafts.csswg.org/css-grid/)

use cssparser::{Parser, Token};
use parser::{Parse, ParserContext};
use std::{mem, usize};
use std::ascii::AsciiExt;
use style_traits::HasViewportPercentage;
use values::{CSSFloat, CustomIdent, Either};
use values::computed::{self, Context, ToComputedValue};
use values::generics::grid::{RepeatCount, TrackBreadth, TrackKeyword, TrackRepeat};
use values::generics::grid::{TrackSize, TrackList, TrackListType};
use values::specified::LengthOrPercentage;

/// Parse a single flexible length.
pub fn parse_flex(input: &mut Parser) -> Result<CSSFloat, ()> {
    match input.next()? {
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
                            let keyword = TrackKeyword::parse(input)?;
                            TrackBreadth::Keyword(keyword)
                        }
                    };

                input.expect_comma()?;
                Ok(TrackSize::MinMax(inflexible_breadth, TrackBreadth::parse(context, input)?))
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

impl HasViewportPercentage for TrackRepeat<LengthOrPercentage> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool {
        self.track_sizes.iter().any(|ref v| v.has_viewport_percentage())
    }
}

/// Either a `<track-size>` or `<track-repeat>` component of `<track-list>`
///
/// This is required only for the specified form of `<track-list>`, and will become
/// `TrackSize<LengthOrPercentage>` in its computed form.
pub type TrackSizeOrRepeat = Either<TrackSize<LengthOrPercentage>, TrackRepeat<LengthOrPercentage>>;

impl Parse for TrackList<TrackSizeOrRepeat> {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let mut current_names;
        let mut names = vec![];
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

                names.push(current_names);
                values.push(Either::First(track_size));
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
                        list_type = TrackListType::Auto(values.len() as u16);
                    },
                    RepeatType::Fixed => (),
                }

                names.push(current_names);
                values.push(Either::Second(repeat));
            } else {
                if values.is_empty() {
                    return Err(())
                }

                names.push(current_names);
                break
            }
        }

        Ok(TrackList {
            list_type: list_type,
            values: values,
            line_names: names,
            auto_repeat: None,      // filled only in computation
        })
    }
}

impl HasViewportPercentage for TrackList<TrackSizeOrRepeat> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool {
        self.values.iter().any(|ref v| v.has_viewport_percentage())
    }
}

impl ToComputedValue for TrackList<TrackSizeOrRepeat> {
    type ComputedValue = TrackList<TrackSize<computed::LengthOrPercentage>>;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        // Merge the line names while computing values. The resulting values will
        // all be a bunch of `<track-size>`.
        //
        // For example,
        // `[a b] 100px [c d] repeat(1, 30px [g]) [h]` will be merged as `[a b] 100px [c d] 30px [g h]`
        //  whereas, `[a b] repeat(2, [c] 50px [d]) [e f] repeat(auto-fill, [g] 12px) 10px [h]` will be merged as
        // `[a b c] 50px [d c] 50px [d e f] repeat(auto-fill, [g] 12px) 10px [h]`, with the `<auto-repeat>` value
        // set in the `auto_repeat` field, and the `idx` in TrackListType::Auto pointing to the values after
        // `<auto-repeat>` (in this case, `10px [h]`).
        let mut line_names = vec![];
        let mut list_type = self.list_type;
        let mut values = vec![];
        let mut prev_names = vec![];
        let mut auto_repeat = None;

        let mut names_iter = self.line_names.iter();
        for (size_or_repeat, names) in self.values.iter().zip(&mut names_iter) {
            prev_names.extend_from_slice(names);

            match *size_or_repeat {
                Either::First(ref size) => values.push(size.to_computed_value(context)),
                Either::Second(ref repeat) => {
                    let mut computed = repeat.to_computed_value(context);
                    if computed.count == RepeatCount::AutoFit || computed.count == RepeatCount::AutoFill {
                        line_names.push(mem::replace(&mut prev_names, vec![]));     // don't merge for auto
                        list_type = TrackListType::Auto(values.len() as u16);
                        auto_repeat = Some(computed);
                        continue
                    }

                    let mut repeat_names_iter = computed.line_names.drain(..);
                    for (size, mut names) in computed.track_sizes.drain(..).zip(&mut repeat_names_iter) {
                        prev_names.append(&mut names);
                        line_names.push(mem::replace(&mut prev_names, vec![]));
                        values.push(size);
                    }

                    if let Some(mut names) = repeat_names_iter.next() {
                        prev_names.append(&mut names);
                    }

                    continue    // last `<line-names>` in repeat() may merge with the next set
                }
            }

            line_names.push(mem::replace(&mut prev_names, vec![]));
        }

        if let Some(names) = names_iter.next() {
            prev_names.extend_from_slice(names);
        }

        line_names.push(mem::replace(&mut prev_names, vec![]));

        TrackList {
            list_type: list_type,
            values: values,
            line_names: line_names,
            auto_repeat: auto_repeat,
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        let auto_idx = if let TrackListType::Auto(idx) = computed.list_type {
            idx as usize
        } else {
            usize::MAX
        };

        let mut values = Vec::with_capacity(computed.values.len() + 1);
        for (i, value) in computed.values.iter().map(ToComputedValue::from_computed_value).enumerate() {
            if i == auto_idx {
                let value = TrackRepeat::from_computed_value(computed.auto_repeat.as_ref().unwrap());
                values.push(Either::Second(value));
            }

            values.push(Either::First(value));
        }

        TrackList {
            list_type: computed.list_type,
            values: values,
            line_names: computed.line_names.clone(),
            auto_repeat: None,
        }
    }
}
