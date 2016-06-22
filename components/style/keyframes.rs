/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, Delimiter};
use parser::ParserContext;
use properties::animated_properties::TransitionProperty;
use properties::{PropertyDeclaration, parse_property_declaration_list};
use std::sync::Arc;

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
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, HeapSizeOf)]
pub struct KeyframePercentage(pub f32);

impl ::std::cmp::Ord for KeyframePercentage {
    #[inline]
    fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
        // We know we have a number from 0 to 1, so unwrap() here is safe.
        self.0.partial_cmp(&other.0).unwrap()
    }
}

impl ::std::cmp::Eq for KeyframePercentage { }

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
    pub declarations: Arc<Vec<PropertyDeclaration>>,
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

        // NB: Other browsers seem to ignore important declarations in keyframe
        // animations too.
        Ok(Keyframe {
            selector: selector,
            declarations: declarations.normal,
        })
    }
}

/// A single step from a keyframe animation.
#[derive(Debug, Clone, PartialEq, HeapSizeOf)]
pub struct KeyframesStep {
    /// The percentage of the animation duration when this step starts.
    pub start_percentage: KeyframePercentage,
    /// Declarations that will determine the final style during the step.
    pub declarations: Arc<Vec<PropertyDeclaration>>,
}

impl KeyframesStep {
    #[inline]
    fn new(percentage: KeyframePercentage,
           declarations: Arc<Vec<PropertyDeclaration>>) -> Self {
        KeyframesStep {
            start_percentage: percentage,
            declarations: declarations,
        }
    }
}

/// This structure represents a list of animation steps computed from the list
/// of keyframes, in order.
///
/// It only takes into account animable properties.
#[derive(Debug, Clone, PartialEq, HeapSizeOf)]
pub struct KeyframesAnimation {
    pub steps: Vec<KeyframesStep>,
    /// The properties that change in this animation.
    pub properties_changed: Vec<TransitionProperty>,
}

/// Get all the animated properties in a keyframes animation. Note that it's not
/// defined what happens when a property is not on a keyframe, so we only peek
/// the props of the first one.
///
/// In practice, browsers seem to try to do their best job at it, so we might
/// want to go through all the actual keyframes and deduplicate properties.
fn get_animated_properties(keyframe: &Keyframe) -> Vec<TransitionProperty> {
    let mut ret = vec![];
    // NB: declarations are already deduplicated, so we don't have to check for
    // it here.
    for declaration in keyframe.declarations.iter() {
        if let Some(property) = TransitionProperty::from_declaration(&declaration) {
            ret.push(property);
        }
    }

    ret
}

impl KeyframesAnimation {
    pub fn from_keyframes(keyframes: &[Keyframe]) -> Option<Self> {
        debug_assert!(keyframes.len() > 1);
        let mut steps = vec![];

        let animated_properties = get_animated_properties(&keyframes[0]);
        if animated_properties.is_empty() {
            return None;
        }

        for keyframe in keyframes {
            for percentage in keyframe.selector.0.iter() {
                steps.push(KeyframesStep::new(*percentage,
                                              keyframe.declarations.clone()));
            }
        }

        // Sort by the start percentage, so we can easily find a frame.
        steps.sort_by_key(|step| step.start_percentage);

        Some(KeyframesAnimation {
            steps: steps,
            properties_changed: animated_properties,
        })
    }
}

