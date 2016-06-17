/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, Delimiter};
use parser::ParserContext;
use properties::{ComputedValues, PropertyDeclarationBlock, parse_property_declaration_list};
use properties::animated_properties::AnimatedProperty;

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

    if keyframes.len() < 2 {
        return Err(())
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

/// A single step from a keyframe animation.
#[derive(Debug, Clone, PartialEq, HeapSizeOf)]
pub struct ComputedKeyframesStep<C: ComputedValues> {
    /// The percentage of the animation duration that should be taken for this
    /// step.
    duration_percentage: KeyframePercentage,
    // XXX: Can we optimise this? Probably not such a show-stopper... Probably
    // storing declared values could work/should be the thing to do?
    /// The computed values at the beginning of the step.
    begin: C,
    /// The computed values at the end of the step.
    end: C,
}

/// This structure represents a list of animation steps computed from the list
/// of keyframes, in order.
///
/// It only takes into account animable properties.
#[derive(Debug, Clone, PartialEq, HeapSizeOf)]
pub struct ComputedKeyframesAnimation<C: ComputedValues> {
    steps: Vec<ComputedKeyframesStep<C>>,
    /// The properties that change in this animation.
    properties_changed: Vec<AnimatedProperty>,
}

fn get_animated_properties(keyframe: &Keyframe) -> Vec<AnimatedProperty> {
    // TODO
    vec![]
}

impl<C: ComputedValues> ComputedKeyframesAnimation<C> {
    pub fn from_keyframes(keyframes: &[Keyframe]) -> Option<ComputedKeyframesAnimation<C>> {
        debug_assert!(keyframes.len() > 1);
        let mut steps = vec![];

        // NB: we do two passes, first storing the steps in the order of
        // appeareance, then sorting them, then updating with the real
        // "duration_percentage".
        let mut animated_properties = get_animated_properties(&keyframes[0]);

        if animated_properties.is_empty() {
            return None;
        }

        for keyframe in keyframes {
            for step in keyframe.selector.0.iter() {

            }
        }

        Some(ComputedKeyframesAnimation {
            steps: steps,
            properties_changed: animated_properties,
        })
    }
}
