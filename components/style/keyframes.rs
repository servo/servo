/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Keyframes: https://drafts.csswg.org/css-animations/#keyframes

#![deny(missing_docs)]

use cssparser::{AtRuleParser, Parser, QualifiedRuleParser, RuleListParser};
use cssparser::{DeclarationListParser, DeclarationParser, parse_one_rule};
use parking_lot::RwLock;
use parser::{ParserContext, ParserContextExtraData, log_css_error};
use properties::{Importance, PropertyDeclaration, PropertyDeclarationBlock, PropertyId};
use properties::PropertyDeclarationParseResult;
use properties::animated_properties::TransitionProperty;
use std::fmt;
use std::sync::Arc;
use style_traits::ToCss;
use stylesheets::{MemoryHoleReporter, Stylesheet};

/// A number from 0 to 1, indicating the percentage of the animation when this
/// keyframe should run.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct KeyframePercentage(pub f32);

impl ::std::cmp::Ord for KeyframePercentage {
    #[inline]
    fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
        // We know we have a number from 0 to 1, so unwrap() here is safe.
        self.0.partial_cmp(&other.0).unwrap()
    }
}

impl ::std::cmp::Eq for KeyframePercentage { }

impl ToCss for KeyframePercentage {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        write!(dest, "{}%", self.0 * 100.0)
    }
}

impl KeyframePercentage {
    /// Trivially constructs a new `KeyframePercentage`.
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
            let percentage = try!(input.expect_percentage());
            if percentage >= 0. && percentage <= 1. {
                KeyframePercentage::new(percentage)
            } else {
                return Err(());
            }
        };

        Ok(percentage)
    }
}

/// A keyframes selector is a list of percentages or from/to symbols, which are
/// converted at parse time to percentages.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct KeyframeSelector(Vec<KeyframePercentage>);
impl KeyframeSelector {
    /// Return the list of percentages this selector contains.
    #[inline]
    pub fn percentages(&self) -> &[KeyframePercentage] {
        &self.0
    }

    /// A dummy public function so we can write a unit test for this.
    pub fn new_for_unit_testing(percentages: Vec<KeyframePercentage>) -> KeyframeSelector {
        KeyframeSelector(percentages)
    }

    /// Parse a keyframe selector from CSS input.
    pub fn parse(input: &mut Parser) -> Result<Self, ()> {
        input.parse_comma_separated(KeyframePercentage::parse)
             .map(KeyframeSelector)
    }
}

/// A keyframe.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Keyframe {
    /// The selector this keyframe was specified from.
    pub selector: KeyframeSelector,

    /// The declaration block that was declared inside this keyframe.
    ///
    /// Note that `!important` rules in keyframes don't apply, but we keep this
    /// `Arc` just for convenience.
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "Arc")]
    pub block: Arc<RwLock<PropertyDeclarationBlock>>,
}

impl ToCss for Keyframe {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let mut iter = self.selector.percentages().iter();
        try!(iter.next().unwrap().to_css(dest));
        for percentage in iter {
            try!(write!(dest, ", "));
            try!(percentage.to_css(dest));
        }
        try!(dest.write_str(" { "));
        try!(self.block.read().to_css(dest));
        try!(dest.write_str(" }"));
        Ok(())
    }
}


impl Keyframe {
    /// Parse a CSS keyframe.
    pub fn parse(css: &str,
                 parent_stylesheet: &Stylesheet,
                 extra_data: ParserContextExtraData)
                 -> Result<Arc<RwLock<Self>>, ()> {
        let error_reporter = Box::new(MemoryHoleReporter);
        let context = ParserContext::new_with_extra_data(parent_stylesheet.origin,
                                                         &parent_stylesheet.base_url,
                                                         error_reporter,
                                                         extra_data);
        let mut input = Parser::new(css);

        let mut rule_parser = KeyframeListParser {
            context: &context,
        };
        parse_one_rule(&mut input, &mut rule_parser)
    }
}

/// A keyframes step value. This can be a synthetised keyframes animation, that
/// is, one autogenerated from the current computed values, or a list of
/// declarations to apply.
///
/// TODO: Find a better name for this?
#[derive(Debug, Clone)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum KeyframesStepValue {
    /// A step formed by a declaration block specified by the CSS.
    Declarations {
        /// The declaration block per se.
        #[cfg_attr(feature = "servo", ignore_heap_size_of = "Arc")]
        block: Arc<RwLock<PropertyDeclarationBlock>>
    },
    /// A synthetic step computed from the current computed values at the time
    /// of the animation.
    ComputedValues,
}

/// A single step from a keyframe animation.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct KeyframesStep {
    /// The percentage of the animation duration when this step starts.
    pub start_percentage: KeyframePercentage,
    /// Declarations that will determine the final style during the step, or
    /// `ComputedValues` if this is an autogenerated step.
    pub value: KeyframesStepValue,
    /// Wether a animation-timing-function declaration exists in the list of
    /// declarations.
    ///
    /// This is used to know when to override the keyframe animation style.
    pub declared_timing_function: bool,
}

impl KeyframesStep {
    #[inline]
    fn new(percentage: KeyframePercentage,
           value: KeyframesStepValue) -> Self {
        let declared_timing_function = match value {
            KeyframesStepValue::Declarations { ref block } => {
                block.read().declarations.iter().any(|&(ref prop_decl, _)| {
                    match *prop_decl {
                        PropertyDeclaration::AnimationTimingFunction(..) => true,
                        _ => false,
                    }
                })
            }
            _ => false,
        };

        KeyframesStep {
            start_percentage: percentage,
            value: value,
            declared_timing_function: declared_timing_function,
        }
    }
}

/// This structure represents a list of animation steps computed from the list
/// of keyframes, in order.
///
/// It only takes into account animable properties.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct KeyframesAnimation {
    /// The difference steps of the animation.
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
    for &(ref declaration, _) in keyframe.block.read().declarations.iter() {
        if let Some(property) = TransitionProperty::from_declaration(declaration) {
            ret.push(property);
        }
    }

    ret
}

impl KeyframesAnimation {
    /// Create a keyframes animation from a given list of keyframes.
    ///
    /// This will return `None` if the list of keyframes is empty, or there are
    /// no animated properties obtained from the keyframes.
    ///
    /// Otherwise, this will compute and sort the steps used for the animation,
    /// and return the animation object.
    pub fn from_keyframes(keyframes: &[Arc<RwLock<Keyframe>>]) -> Option<Self> {
        if keyframes.is_empty() {
            return None;
        }

        let animated_properties = get_animated_properties(&keyframes[0].read());
        if animated_properties.is_empty() {
            return None;
        }

        let mut steps = vec![];

        for keyframe in keyframes {
            let keyframe = keyframe.read();
            for percentage in keyframe.selector.0.iter() {
                steps.push(KeyframesStep::new(*percentage, KeyframesStepValue::Declarations {
                    block: keyframe.block.clone(),
                }));
            }
        }

        // Sort by the start percentage, so we can easily find a frame.
        steps.sort_by_key(|step| step.start_percentage);

        // Prepend autogenerated keyframes if appropriate.
        if steps[0].start_percentage.0 != 0. {
            steps.insert(0, KeyframesStep::new(KeyframePercentage::new(0.),
                                               KeyframesStepValue::ComputedValues));
        }

        if steps.last().unwrap().start_percentage.0 != 1. {
            steps.push(KeyframesStep::new(KeyframePercentage::new(0.),
                                          KeyframesStepValue::ComputedValues));
        }

        Some(KeyframesAnimation {
            steps: steps,
            properties_changed: animated_properties,
        })
    }
}

/// Parses a keyframes list, like:
/// 0%, 50% {
///     width: 50%;
/// }
///
/// 40%, 60%, 100% {
///     width: 100%;
/// }
struct KeyframeListParser<'a> {
    context: &'a ParserContext<'a>,
}

/// Parses a keyframe list from CSS input.
pub fn parse_keyframe_list(context: &ParserContext, input: &mut Parser) -> Vec<Arc<RwLock<Keyframe>>> {
    RuleListParser::new_for_nested_rule(input, KeyframeListParser { context: context })
        .filter_map(Result::ok)
        .collect()
}

enum Void {}
impl<'a> AtRuleParser for KeyframeListParser<'a> {
    type Prelude = Void;
    type AtRule = Arc<RwLock<Keyframe>>;
}

impl<'a> QualifiedRuleParser for KeyframeListParser<'a> {
    type Prelude = KeyframeSelector;
    type QualifiedRule = Arc<RwLock<Keyframe>>;

    fn parse_prelude(&mut self, input: &mut Parser) -> Result<Self::Prelude, ()> {
        let start = input.position();
        match KeyframeSelector::parse(input) {
            Ok(sel) => Ok(sel),
            Err(()) => {
                let message = format!("Invalid keyframe rule: '{}'", input.slice_from(start));
                log_css_error(input, start, &message, self.context);
                Err(())
            }
        }
    }

    fn parse_block(&mut self, prelude: Self::Prelude, input: &mut Parser)
                   -> Result<Self::QualifiedRule, ()> {
        let mut declarations = Vec::new();
        let parser = KeyframeDeclarationParser {
            context: self.context,
        };
        let mut iter = DeclarationListParser::new(input, parser);
        while let Some(declaration) = iter.next() {
            match declaration {
                Ok(d) => declarations.extend(d.into_iter().map(|d| (d, Importance::Normal))),
                Err(range) => {
                    let pos = range.start;
                    let message = format!("Unsupported keyframe property declaration: '{}'",
                                          iter.input.slice(range));
                    log_css_error(iter.input, pos, &*message, self.context);
                }
            }
            // `parse_important` is not called here, `!important` is not allowed in keyframe blocks.
        }
        Ok(Arc::new(RwLock::new(Keyframe {
            selector: prelude,
            block: Arc::new(RwLock::new(PropertyDeclarationBlock {
                declarations: declarations,
                important_count: 0,
            })),
        })))
    }
}

struct KeyframeDeclarationParser<'a, 'b: 'a> {
    context: &'a ParserContext<'b>,
}

/// Default methods reject all at rules.
impl<'a, 'b> AtRuleParser for KeyframeDeclarationParser<'a, 'b> {
    type Prelude = ();
    type AtRule = Vec<PropertyDeclaration>;
}

impl<'a, 'b> DeclarationParser for KeyframeDeclarationParser<'a, 'b> {
    type Declaration = Vec<PropertyDeclaration>;

    fn parse_value(&mut self, name: &str, input: &mut Parser) -> Result<Vec<PropertyDeclaration>, ()> {
        let id = try!(PropertyId::parse(name.into()));
        let mut results = Vec::new();
        match PropertyDeclaration::parse(id, self.context, input, &mut results, true) {
            PropertyDeclarationParseResult::ValidOrIgnoredDeclaration => {}
            _ => return Err(())
        }
        Ok(results)
    }
}
