/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, Delimiter};
use parser::ParserContext;
use properties::{PropertyDeclarationBlock, parse_property_declaration_list};

/// Parses a keyframes list, like:
/// 0%, 50% {
///     width: 50%;
/// }
///
/// 40%, 60%, 100% {
///     width: 100%;
/// }
pub fn parse_keyframe_list(context: &ParserContext, input: &mut Parser) -> Result<Vec<Keyframe>, ()> {
    let mut keyframes = vec![];
    while !input.is_exhausted() {
        keyframes.push(try!(Keyframe::parse(context, input)));
    }
    Ok(keyframes)
}

/// A number from 1 to 100, indicating the percentage of the animation where
/// this keyframe should run.
#[derive(Debug, Copy, Clone, PartialEq, HeapSizeOf)]
pub struct KeyframePercentage(f32);

impl KeyframePercentage {
    #[inline]
    pub fn new(value: f32) -> KeyframePercentage {
        debug_assert!(value >= 0. && value <= 1.);
        KeyframePercentage(value)
    }

    fn parse(input: &mut Parser) -> Result<KeyframePercentage, ()> {
        let percentage = if input.try(|input| input.expect_ident_matching("from")).is_ok() {
            KeyframePercentage::new(0.)
        } else if input.try(|input| input.expect_ident_matching("to")).is_ok() {
            KeyframePercentage::new(1.)
        } else {
            KeyframePercentage::new(try!(input.expect_percentage()))
        };

        Ok(percentage)
    }
}

/// A keyframes selector is a list of percentages or from/to symbols, which are
/// converted at parse time to percentages.
#[derive(Debug, Clone, PartialEq, HeapSizeOf)]
pub struct KeyframeSelector(Vec<KeyframePercentage>);
impl KeyframeSelector {
    #[inline]
    pub fn percentages(&self) -> &[KeyframePercentage] {
        &self.0
    }

    /// A dummy public function so we can write a unit test for this.
    pub fn new_for_unit_testing(percentages: Vec<KeyframePercentage>) -> KeyframeSelector {
        KeyframeSelector(percentages)
    }
}

/// A keyframe.
#[derive(Debug, Clone, PartialEq, HeapSizeOf)]
pub struct Keyframe {
    pub selector: KeyframeSelector,
    pub declarations: PropertyDeclarationBlock,
}

impl Keyframe {
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<Keyframe, ()> {
        let percentages = try!(input.parse_until_before(Delimiter::CurlyBracketBlock, |input| {
            input.parse_comma_separated(|input| KeyframePercentage::parse(input))
        }));
        let selector = KeyframeSelector(percentages);

        try!(input.expect_curly_bracket_block());

        let declarations = input.parse_nested_block(|input| {
            Ok(parse_property_declaration_list(context, input))
        }).unwrap();

        Ok(Keyframe {
            selector: selector,
            declarations: declarations,
        })
    }
}
