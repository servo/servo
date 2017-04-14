/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Keyframes: https://drafts.csswg.org/css-animations/#keyframes

#![deny(missing_docs)]

use cssparser::{AtRuleParser, Parser, QualifiedRuleParser, RuleListParser};
use cssparser::{DeclarationListParser, DeclarationParser, parse_one_rule};
use parser::{LengthParsingMode, ParserContext, log_css_error};
use properties::{Importance, PropertyDeclaration, PropertyDeclarationBlock, PropertyId};
use properties::{PropertyDeclarationId, LonghandId, ParsedDeclaration};
use properties::LonghandIdSet;
use properties::animated_properties::TransitionProperty;
use properties::longhands::transition_timing_function::single_value::SpecifiedValue as SpecifiedTimingFunction;
use shared_lock::{SharedRwLock, SharedRwLockReadGuard, Locked, ToCssWithGuard};
use std::fmt;
use std::sync::Arc;
use style_traits::ToCss;
use stylesheets::{CssRuleType, MemoryHoleReporter, Stylesheet};

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
#[derive(Debug, PartialEq)]
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
#[derive(Debug)]
pub struct Keyframe {
    /// The selector this keyframe was specified from.
    pub selector: KeyframeSelector,

    /// The declaration block that was declared inside this keyframe.
    ///
    /// Note that `!important` rules in keyframes don't apply, but we keep this
    /// `Arc` just for convenience.
    pub block: Arc<Locked<PropertyDeclarationBlock>>,
}

impl ToCssWithGuard for Keyframe {
    fn to_css<W>(&self, guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
    where W: fmt::Write {
        let mut iter = self.selector.percentages().iter();
        try!(iter.next().unwrap().to_css(dest));
        for percentage in iter {
            try!(write!(dest, ", "));
            try!(percentage.to_css(dest));
        }
        try!(dest.write_str(" { "));
        try!(self.block.read_with(guard).to_css(dest));
        try!(dest.write_str(" }"));
        Ok(())
    }
}


impl Keyframe {
    /// Parse a CSS keyframe.
    pub fn parse(css: &str, parent_stylesheet: &Stylesheet)
                 -> Result<Arc<Locked<Self>>, ()> {
        let error_reporter = MemoryHoleReporter;
        let context = ParserContext::new(parent_stylesheet.origin,
                                         &parent_stylesheet.url_data,
                                         &error_reporter,
                                         Some(CssRuleType::Keyframe),
                                         LengthParsingMode::Default);
        let mut input = Parser::new(css);

        let mut rule_parser = KeyframeListParser {
            context: &context,
            shared_lock: &parent_stylesheet.shared_lock,
        };
        parse_one_rule(&mut input, &mut rule_parser)
    }
}

/// A keyframes step value. This can be a synthetised keyframes animation, that
/// is, one autogenerated from the current computed values, or a list of
/// declarations to apply.
///
/// TODO: Find a better name for this?
#[derive(Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum KeyframesStepValue {
    /// A step formed by a declaration block specified by the CSS.
    Declarations {
        /// The declaration block per se.
        #[cfg_attr(feature = "servo", ignore_heap_size_of = "Arc")]
        block: Arc<Locked<PropertyDeclarationBlock>>
    },
    /// A synthetic step computed from the current computed values at the time
    /// of the animation.
    ComputedValues,
}

/// A single step from a keyframe animation.
#[derive(Debug)]
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
           value: KeyframesStepValue,
           guard: &SharedRwLockReadGuard) -> Self {
        let declared_timing_function = match value {
            KeyframesStepValue::Declarations { ref block } => {
                block.read_with(guard).declarations().iter().any(|&(ref prop_decl, _)| {
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

    /// Return specified TransitionTimingFunction if this KeyframesSteps has 'animation-timing-function'.
    pub fn get_animation_timing_function(&self, guard: &SharedRwLockReadGuard)
                                         -> Option<SpecifiedTimingFunction> {
        if !self.declared_timing_function {
            return None;
        }
        match self.value {
            KeyframesStepValue::Declarations { ref block } => {
                let guard = block.read_with(guard);
                let &(ref declaration, _) =
                    guard.get(PropertyDeclarationId::Longhand(LonghandId::AnimationTimingFunction)).unwrap();
                match *declaration {
                    PropertyDeclaration::AnimationTimingFunction(ref value) => {
                        // Use the first value.
                        Some(value.0[0])
                    },
                    PropertyDeclaration::CSSWideKeyword(..) => None,
                    PropertyDeclaration::WithVariables(..) => None,
                    _ => panic!(),
                }
            },
            KeyframesStepValue::ComputedValues => {
                panic!("Shouldn't happen to set animation-timing-function in missing keyframes")
            },
        }
    }
}

/// This structure represents a list of animation steps computed from the list
/// of keyframes, in order.
///
/// It only takes into account animable properties.
#[derive(Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct KeyframesAnimation {
    /// The difference steps of the animation.
    pub steps: Vec<KeyframesStep>,
    /// The properties that change in this animation.
    pub properties_changed: Vec<TransitionProperty>,
}

/// Get all the animated properties in a keyframes animation.
fn get_animated_properties(keyframes: &[Arc<Locked<Keyframe>>], guard: &SharedRwLockReadGuard)
                           -> Vec<TransitionProperty> {
    let mut ret = vec![];
    let mut seen = LonghandIdSet::new();
    // NB: declarations are already deduplicated, so we don't have to check for
    // it here.
    for keyframe in keyframes {
        let keyframe = keyframe.read_with(&guard);
        let block = keyframe.block.read_with(guard);
        for &(ref declaration, importance) in block.declarations().iter() {
            assert!(!importance.important());

            if let Some(property) = TransitionProperty::from_declaration(declaration) {
                if !seen.has_transition_property_bit(&property) {
                    ret.push(property);
                    seen.set_transition_property_bit(&property);
                }
            }
        }
    }

    ret
}

impl KeyframesAnimation {
    /// Create a keyframes animation from a given list of keyframes.
    ///
    /// This will return a keyframe animation with empty steps and
    /// properties_changed if the list of keyframes is empty, or there are no
    //  animated properties obtained from the keyframes.
    ///
    /// Otherwise, this will compute and sort the steps used for the animation,
    /// and return the animation object.
    pub fn from_keyframes(keyframes: &[Arc<Locked<Keyframe>>], guard: &SharedRwLockReadGuard)
                          -> Self {
        let mut result = KeyframesAnimation {
            steps: vec![],
            properties_changed: vec![],
        };

        if keyframes.is_empty() {
            return result;
        }

        result.properties_changed = get_animated_properties(keyframes, guard);
        if result.properties_changed.is_empty() {
            return result;
        }

        for keyframe in keyframes {
            let keyframe = keyframe.read_with(&guard);
            for percentage in keyframe.selector.0.iter() {
                result.steps.push(KeyframesStep::new(*percentage, KeyframesStepValue::Declarations {
                    block: keyframe.block.clone(),
                }, guard));
            }
        }

        // Sort by the start percentage, so we can easily find a frame.
        result.steps.sort_by_key(|step| step.start_percentage);

        // Prepend autogenerated keyframes if appropriate.
        if result.steps[0].start_percentage.0 != 0. {
            result.steps.insert(0, KeyframesStep::new(KeyframePercentage::new(0.),
                                                      KeyframesStepValue::ComputedValues,
                                                      guard));
        }

        if result.steps.last().unwrap().start_percentage.0 != 1. {
            result.steps.push(KeyframesStep::new(KeyframePercentage::new(1.),
                                                 KeyframesStepValue::ComputedValues,
                                                 guard));
        }

        result
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
    shared_lock: &'a SharedRwLock,
}

/// Parses a keyframe list from CSS input.
pub fn parse_keyframe_list(context: &ParserContext, input: &mut Parser, shared_lock: &SharedRwLock)
                           -> Vec<Arc<Locked<Keyframe>>> {
    RuleListParser::new_for_nested_rule(input, KeyframeListParser {
        context: context,
        shared_lock: shared_lock,
    }).filter_map(Result::ok).collect()
}

enum Void {}
impl<'a> AtRuleParser for KeyframeListParser<'a> {
    type Prelude = Void;
    type AtRule = Arc<Locked<Keyframe>>;
}

impl<'a> QualifiedRuleParser for KeyframeListParser<'a> {
    type Prelude = KeyframeSelector;
    type QualifiedRule = Arc<Locked<Keyframe>>;

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
        let context = ParserContext::new_with_rule_type(self.context, Some(CssRuleType::Keyframe));
        let parser = KeyframeDeclarationParser {
            context: &context,
        };
        let mut iter = DeclarationListParser::new(input, parser);
        let mut block = PropertyDeclarationBlock::new();
        while let Some(declaration) = iter.next() {
            match declaration {
                Ok(parsed) => parsed.expand_push_into(&mut block, Importance::Normal),
                Err(range) => {
                    let pos = range.start;
                    let message = format!("Unsupported keyframe property declaration: '{}'",
                                          iter.input.slice(range));
                    log_css_error(iter.input, pos, &*message, &context);
                }
            }
            // `parse_important` is not called here, `!important` is not allowed in keyframe blocks.
        }
        Ok(Arc::new(self.shared_lock.wrap(Keyframe {
            selector: prelude,
            block: Arc::new(self.shared_lock.wrap(block)),
        })))
    }
}

struct KeyframeDeclarationParser<'a, 'b: 'a> {
    context: &'a ParserContext<'b>,
}

/// Default methods reject all at rules.
impl<'a, 'b> AtRuleParser for KeyframeDeclarationParser<'a, 'b> {
    type Prelude = ();
    type AtRule = ParsedDeclaration;
}

impl<'a, 'b> DeclarationParser for KeyframeDeclarationParser<'a, 'b> {
    type Declaration = ParsedDeclaration;

    fn parse_value(&mut self, name: &str, input: &mut Parser) -> Result<ParsedDeclaration, ()> {
        let id = try!(PropertyId::parse(name.into()));
        match ParsedDeclaration::parse(id, self.context, input) {
            Ok(parsed) => {
                // In case there is still unparsed text in the declaration, we should roll back.
                if !input.is_exhausted() {
                    Err(())
                } else {
                    Ok(parsed)
                }
            }
            Err(_) => Err(())
        }
    }
}
