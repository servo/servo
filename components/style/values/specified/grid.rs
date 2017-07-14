/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the computed value of
//! [grids](https://drafts.csswg.org/css-grid/)

use cssparser::{Parser, Token, BasicParseError};
use parser::{Parse, ParserContext};
use std::ascii::AsciiExt;
use std::mem;
use style_traits::{HasViewportPercentage, ParseError, StyleParseError};
use values::{CSSFloat, CustomIdent};
use values::computed::{self, Context, ToComputedValue};
use values::generics::grid::{GridTemplateComponent, RepeatCount, TrackBreadth, TrackKeyword, TrackRepeat};
use values::generics::grid::{LineNameList, TrackSize, TrackList, TrackListType};
use values::specified::LengthOrPercentage;

/// Parse a single flexible length.
pub fn parse_flex<'i, 't>(input: &mut Parser<'i, 't>) -> Result<CSSFloat, ParseError<'i>> {
    match input.next()? {
        Token::Dimension { value, ref unit, .. } if unit.eq_ignore_ascii_case("fr") && value.is_sign_positive()
            => Ok(value),
        t => Err(BasicParseError::UnexpectedToken(t).into()),
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
pub fn parse_line_names<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Vec<CustomIdent>, ParseError<'i>> {
    input.expect_square_bracket_block()?;
    input.parse_nested_block(|input| {
        let mut values = vec![];
        while let Ok(ident) = input.try(|i| i.expect_ident()) {
            let ident = CustomIdent::from_ident(ident, &["span"])?;
            values.push(ident);
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
                    current_names = input.try(parse_line_names).unwrap_or(vec![]);
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
                            names.push(input.try(parse_line_names).unwrap_or(vec![]));
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

impl Parse for TrackList<LengthOrPercentage> {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        // Merge the line names while parsing values. The resulting values will
        // all be bunch of `<track-size>` and one <auto-repeat>.
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
            current_names.append(&mut input.try(parse_line_names).unwrap_or(vec![]));
            if let Ok(track_size) = input.try(|i| TrackSize::parse(context, i)) {
                if !track_size.is_fixed() {
                    atleast_one_not_fixed = true;
                    if auto_repeat.is_some() {
                        // <auto-track-list> only accepts <fixed-size> and <fixed-repeat>
                        return Err(StyleParseError::UnspecifiedError.into())
                    }
                }

                names.push(mem::replace(&mut current_names, vec![]));
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
                        names.push(mem::replace(&mut current_names, vec![]));
                        continue
                    },
                    RepeatType::Fixed => (),
                }

                // If the repeat count is numeric, we axpand and merge the values.
                let mut repeat = repeat.expand();
                let mut repeat_names_iter = repeat.line_names.drain(..);
                for (size, repeat_names) in repeat.track_sizes.drain(..).zip(&mut repeat_names_iter) {
                    current_names.extend_from_slice(&repeat_names);
                    names.push(mem::replace(&mut current_names, vec![]));
                    values.push(size);
                }

                if let Some(names) = repeat_names_iter.next() {
                    current_names.extend_from_slice(&names);
                }
            } else {
                if values.is_empty() && auto_repeat.is_none() {
                    return Err(StyleParseError::UnspecifiedError.into())
                }

                names.push(current_names);
                break
            }
        }

        Ok(TrackList {
            list_type: list_type,
            values: values,
            line_names: names,
            auto_repeat: auto_repeat,
        })
    }
}

impl HasViewportPercentage for TrackList<LengthOrPercentage> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool {
        self.values.iter().any(|ref v| v.has_viewport_percentage())
    }
}


impl ToComputedValue for TrackList<LengthOrPercentage> {
    type ComputedValue = TrackList<computed::LengthOrPercentage>;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        let mut values = Vec::with_capacity(self.values.len() + 1);
        for value in self.values.iter().map(|val| val.to_computed_value(context)) {
            values.push(value);
        }

        TrackList {
            list_type: self.list_type.to_computed_value(context),
            values: values,
            line_names: self.line_names.clone(),
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

impl HasViewportPercentage for GridTemplateComponent<LengthOrPercentage> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            GridTemplateComponent::TrackList(ref l) => l.has_viewport_percentage(),
            _ => false,
        }
    }
}

impl ToComputedValue for GridTemplateComponent<LengthOrPercentage> {
    type ComputedValue = GridTemplateComponent<computed::LengthOrPercentage>;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            GridTemplateComponent::None => GridTemplateComponent::None,
            GridTemplateComponent::TrackList(ref l) => GridTemplateComponent::TrackList(l.to_computed_value(context)),
            GridTemplateComponent::Subgrid(ref n) => GridTemplateComponent::Subgrid(n.to_computed_value(context)),
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            GridTemplateComponent::None => GridTemplateComponent::None,
            GridTemplateComponent::TrackList(ref l) =>
                GridTemplateComponent::TrackList(ToComputedValue::from_computed_value(l)),
            GridTemplateComponent::Subgrid(ref n) =>
                GridTemplateComponent::Subgrid(ToComputedValue::from_computed_value(n)),
        }
    }
}
