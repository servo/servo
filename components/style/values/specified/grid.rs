/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! CSS handling for the computed value of
//! [grids](https://drafts.csswg.org/css-grid/)

use crate::parser::{Parse, ParserContext};
use crate::values::generics::grid::{GridTemplateComponent, ImplicitGridTracks, RepeatCount};
use crate::values::generics::grid::{LineNameList, TrackBreadth, TrackRepeat, TrackSize};
use crate::values::generics::grid::{TrackList, TrackListValue};
use crate::values::specified::{Integer, LengthPercentage};
use crate::values::{CSSFloat, CustomIdent};
use cssparser::{ParseError as CssParseError, Parser, Token};
use std::mem;
use style_traits::{ParseError, StyleParseErrorKind};

/// Parse a single flexible length.
pub fn parse_flex<'i, 't>(input: &mut Parser<'i, 't>) -> Result<CSSFloat, ParseError<'i>> {
    let location = input.current_source_location();
    match *input.next()? {
        Token::Dimension {
            value, ref unit, ..
        } if unit.eq_ignore_ascii_case("fr") && value.is_sign_positive() => Ok(value),
        ref t => Err(location.new_unexpected_token_error(t.clone())),
    }
}

impl<L> TrackBreadth<L> {
    fn parse_keyword<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        #[derive(Parse)]
        enum TrackKeyword {
            Auto,
            MaxContent,
            MinContent,
        }

        Ok(match TrackKeyword::parse(input)? {
            TrackKeyword::Auto => TrackBreadth::Auto,
            TrackKeyword::MaxContent => TrackBreadth::MaxContent,
            TrackKeyword::MinContent => TrackBreadth::MinContent,
        })
    }
}

impl Parse for TrackBreadth<LengthPercentage> {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        // FIXME: This and other callers in this file should use
        // NonNegativeLengthPercentage instead.
        //
        // Though it seems these cannot be animated so it's ~ok.
        if let Ok(lp) = input.try(|i| LengthPercentage::parse_non_negative(context, i)) {
            return Ok(TrackBreadth::Breadth(lp));
        }

        if let Ok(f) = input.try(parse_flex) {
            return Ok(TrackBreadth::Fr(f));
        }

        Self::parse_keyword(input)
    }
}

impl Parse for TrackSize<LengthPercentage> {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(b) = input.try(|i| TrackBreadth::parse(context, i)) {
            return Ok(TrackSize::Breadth(b));
        }

        if input.try(|i| i.expect_function_matching("minmax")).is_ok() {
            return input.parse_nested_block(|input| {
                let inflexible_breadth =
                    match input.try(|i| LengthPercentage::parse_non_negative(context, i)) {
                        Ok(lp) => TrackBreadth::Breadth(lp),
                        Err(..) => TrackBreadth::parse_keyword(input)?,
                    };

                input.expect_comma()?;
                Ok(TrackSize::Minmax(
                    inflexible_breadth,
                    TrackBreadth::parse(context, input)?,
                ))
            });
        }

        input.expect_function_matching("fit-content")?;
        let lp = input.parse_nested_block(|i| LengthPercentage::parse_non_negative(context, i))?;
        Ok(TrackSize::FitContent(TrackBreadth::Breadth(lp)))
    }
}

impl Parse for ImplicitGridTracks<TrackSize<LengthPercentage>> {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        use style_traits::{Separator, Space};
        let track_sizes = Space::parse(input, |i| TrackSize::parse(context, i))?;
        if track_sizes.len() == 1 && track_sizes[0].is_initial() {
            // A single track with the initial value is always represented by an empty slice.
            return Ok(Default::default());
        }
        return Ok(ImplicitGridTracks(track_sizes.into()));
    }
}

/// Parse the grid line names into a vector of owned strings.
///
/// <https://drafts.csswg.org/css-grid/#typedef-line-names>
pub fn parse_line_names<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<crate::OwnedSlice<CustomIdent>, ParseError<'i>> {
    input.expect_square_bracket_block()?;
    input.parse_nested_block(|input| {
        let mut values = vec![];
        while let Ok((loc, ident)) = input.try(|i| -> Result<_, CssParseError<()>> {
            Ok((i.current_source_location(), i.expect_ident_cloned()?))
        }) {
            let ident = CustomIdent::from_ident(loc, &ident, &["span", "auto"])?;
            values.push(ident);
        }

        Ok(values.into())
    })
}

/// The type of `repeat` function (only used in parsing).
///
/// <https://drafts.csswg.org/css-grid/#typedef-track-repeat>
#[derive(Clone, Copy, Debug, PartialEq, SpecifiedValueInfo)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
enum RepeatType {
    /// [`<auto-repeat>`](https://drafts.csswg.org/css-grid/#typedef-auto-repeat)
    Auto,
    /// [`<track-repeat>`](https://drafts.csswg.org/css-grid/#typedef-track-repeat)
    Normal,
    /// [`<fixed-repeat>`](https://drafts.csswg.org/css-grid/#typedef-fixed-repeat)
    Fixed,
}

impl TrackRepeat<LengthPercentage, Integer> {
    fn parse_with_repeat_type<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<(Self, RepeatType), ParseError<'i>> {
        input
            .try(|i| i.expect_function_matching("repeat").map_err(|e| e.into()))
            .and_then(|_| {
                input.parse_nested_block(|input| {
                    let count = RepeatCount::parse(context, input)?;
                    input.expect_comma()?;

                    let is_auto = count == RepeatCount::AutoFit || count == RepeatCount::AutoFill;
                    let mut repeat_type = if is_auto {
                        RepeatType::Auto
                    } else {
                        // <fixed-size> is a subset of <track-size>, so it should work for both
                        RepeatType::Fixed
                    };

                    let mut names = vec![];
                    let mut values = vec![];
                    let mut current_names;

                    loop {
                        current_names = input.try(parse_line_names).unwrap_or_default();
                        if let Ok(track_size) = input.try(|i| TrackSize::parse(context, i)) {
                            if !track_size.is_fixed() {
                                if is_auto {
                                    // should be <fixed-size> for <auto-repeat>
                                    return Err(input
                                        .new_custom_error(StyleParseErrorKind::UnspecifiedError));
                                }

                                if repeat_type == RepeatType::Fixed {
                                    repeat_type = RepeatType::Normal // <track-size> for sure
                                }
                            }

                            values.push(track_size);
                            names.push(current_names);
                        } else {
                            if values.is_empty() {
                                // expecting at least one <track-size>
                                return Err(
                                    input.new_custom_error(StyleParseErrorKind::UnspecifiedError)
                                );
                            }

                            names.push(current_names); // final `<line-names>`
                            break; // no more <track-size>, breaking
                        }
                    }

                    let repeat = TrackRepeat {
                        count,
                        track_sizes: values.into(),
                        line_names: names.into(),
                    };

                    Ok((repeat, repeat_type))
                })
            })
    }
}

impl Parse for TrackList<LengthPercentage, Integer> {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let mut current_names = vec![];
        let mut names = vec![];
        let mut values = vec![];

        // Whether we've parsed an `<auto-repeat>` value.
        let mut auto_repeat_index = None;
        // assume that everything is <fixed-size>. This flag is useful when we encounter <auto-repeat>
        let mut at_least_one_not_fixed = false;
        loop {
            current_names.extend_from_slice(&mut input.try(parse_line_names).unwrap_or_default());
            if let Ok(track_size) = input.try(|i| TrackSize::parse(context, i)) {
                if !track_size.is_fixed() {
                    at_least_one_not_fixed = true;
                    if auto_repeat_index.is_some() {
                        // <auto-track-list> only accepts <fixed-size> and <fixed-repeat>
                        return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                    }
                }

                let vec = mem::replace(&mut current_names, vec![]);
                names.push(vec.into());
                values.push(TrackListValue::TrackSize(track_size));
            } else if let Ok((repeat, type_)) =
                input.try(|i| TrackRepeat::parse_with_repeat_type(context, i))
            {
                match type_ {
                    RepeatType::Normal => {
                        at_least_one_not_fixed = true;
                        if auto_repeat_index.is_some() {
                            // only <fixed-repeat>
                            return Err(
                                input.new_custom_error(StyleParseErrorKind::UnspecifiedError)
                            );
                        }
                    },
                    RepeatType::Auto => {
                        if auto_repeat_index.is_some() || at_least_one_not_fixed {
                            // We've either seen <auto-repeat> earlier, or there's at least one non-fixed value
                            return Err(
                                input.new_custom_error(StyleParseErrorKind::UnspecifiedError)
                            );
                        }
                        auto_repeat_index = Some(values.len());
                    },
                    RepeatType::Fixed => {},
                }

                let vec = mem::replace(&mut current_names, vec![]);
                names.push(vec.into());
                values.push(TrackListValue::TrackRepeat(repeat));
            } else {
                if values.is_empty() && auto_repeat_index.is_none() {
                    return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                }

                names.push(current_names.into());
                break;
            }
        }

        Ok(TrackList {
            auto_repeat_index: auto_repeat_index.unwrap_or(std::usize::MAX),
            values: values.into(),
            line_names: names.into(),
        })
    }
}

#[cfg(feature = "gecko")]
#[inline]
fn allow_grid_template_subgrids() -> bool {
    static_prefs::pref!("layout.css.grid-template-subgrid-value.enabled")
}

#[cfg(feature = "servo")]
#[inline]
fn allow_grid_template_subgrids() -> bool {
    false
}

#[cfg(feature = "gecko")]
#[inline]
fn allow_grid_template_masonry() -> bool {
    static_prefs::pref!("layout.css.grid-template-masonry-value.enabled")
}

#[cfg(feature = "servo")]
#[inline]
fn allow_grid_template_masonry() -> bool {
    false
}

impl Parse for GridTemplateComponent<LengthPercentage, Integer> {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(GridTemplateComponent::None);
        }

        Self::parse_without_none(context, input)
    }
}

impl GridTemplateComponent<LengthPercentage, Integer> {
    /// Parses a `GridTemplateComponent<LengthPercentage>` except `none` keyword.
    pub fn parse_without_none<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if allow_grid_template_subgrids() {
            if let Ok(t) = input.try(|i| LineNameList::parse(context, i)) {
                return Ok(GridTemplateComponent::Subgrid(Box::new(t)));
            }
        }
        if allow_grid_template_masonry() {
            if input.try(|i| i.expect_ident_matching("masonry")).is_ok() {
                return Ok(GridTemplateComponent::Masonry);
            }
        }
        let track_list = TrackList::parse(context, input)?;
        Ok(GridTemplateComponent::TrackList(Box::new(track_list)))
    }
}
