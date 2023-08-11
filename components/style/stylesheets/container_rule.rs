/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A [`@container`][container] rule.
//!
//! [container]: https://drafts.csswg.org/css-contain-3/#container-rule

use crate::logical_geometry::{WritingMode, LogicalSize};
use crate::queries::QueryCondition;
use crate::shared_lock::{
    DeepCloneParams, DeepCloneWithLock, Locked, SharedRwLock, SharedRwLockReadGuard, ToCssWithGuard,
};
use crate::values::specified::ContainerName;
use crate::values::computed::{Context, CSSPixelLength, Ratio};
use crate::str::CssStringWriter;
use crate::stylesheets::CssRules;
use crate::queries::feature::{AllowsRanges, Evaluator, ParsingRequirements, QueryFeatureDescription};
use crate::queries::values::Orientation;
use app_units::Au;
use cssparser::SourceLocation;
use euclid::default::Size2D;
#[cfg(feature = "gecko")]
use malloc_size_of::{MallocSizeOfOps, MallocUnconditionalShallowSizeOf};
use servo_arc::Arc;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};

/// A container rule.
#[derive(Debug, ToShmem)]
pub struct ContainerRule {
    /// The container name.
    pub name: ContainerName,
    /// The container query.
    pub condition: ContainerCondition,
    /// The nested rules inside the block.
    pub rules: Arc<Locked<CssRules>>,
    /// The source position where this rule was found.
    pub source_location: SourceLocation,
}

impl ContainerRule {
    /// Measure heap usage.
    #[cfg(feature = "gecko")]
    pub fn size_of(&self, guard: &SharedRwLockReadGuard, ops: &mut MallocSizeOfOps) -> usize {
        // Measurement of other fields may be added later.
        self.rules.unconditional_shallow_size_of(ops)
            + self.rules.read_with(guard).size_of(guard, ops)
    }
}

impl DeepCloneWithLock for ContainerRule {
    fn deep_clone_with_lock(
        &self,
        lock: &SharedRwLock,
        guard: &SharedRwLockReadGuard,
        params: &DeepCloneParams,
    ) -> Self {
        let rules = self.rules.read_with(guard);
        Self {
            name: self.name.clone(),
            condition: self.condition.clone(),
            rules: Arc::new(lock.wrap(rules.deep_clone_with_lock(lock, guard, params))),
            source_location: self.source_location.clone(),
        }
    }
}

impl ToCssWithGuard for ContainerRule {
    fn to_css(&self, guard: &SharedRwLockReadGuard, dest: &mut CssStringWriter) -> fmt::Result {
        dest.write_str("@container ")?;
        {
            let mut writer = CssWriter::new(dest);
            if !self.name.is_none() {
                self.name.to_css(&mut writer)?;
                writer.write_char(' ')?;
            }
            self.condition.to_css(&mut writer)?;
        }
        self.rules.read_with(guard).to_css_block(guard, dest)
    }
}

/// TODO: Factor out the media query code to work with containers.
pub type ContainerCondition = QueryCondition;

fn get_container(_context: &Context) -> (Size2D<Au>, WritingMode) {
    unimplemented!("TODO: implement container matching");
}

fn eval_width(context: &Context) -> CSSPixelLength {
    let (size, _wm) = get_container(context);
    CSSPixelLength::new(size.width.to_f32_px())
}

fn eval_height(context: &Context) -> CSSPixelLength {
    let (size, _wm) = get_container(context);
    CSSPixelLength::new(size.height.to_f32_px())
}

fn eval_inline_size(context: &Context) -> CSSPixelLength {
    let (size, wm) = get_container(context);
    CSSPixelLength::new(LogicalSize::from_physical(wm, size).inline.to_f32_px())
}

fn eval_block_size(context: &Context) -> CSSPixelLength {
    let (size, wm) = get_container(context);
    CSSPixelLength::new(LogicalSize::from_physical(wm, size).block.to_f32_px())
}

fn eval_aspect_ratio(context: &Context) -> Ratio {
    let (size, _wm) = get_container(context);
    Ratio::new(size.width.0 as f32, size.height.0 as f32)
}

fn eval_orientation(context: &Context, value: Option<Orientation>) -> bool {
    let (size, _wm) = get_container(context);
    Orientation::eval(size, value)
}

/// https://drafts.csswg.org/css-contain-3/#container-features
///
/// TODO: Support style queries, perhaps.
pub static CONTAINER_FEATURES: [QueryFeatureDescription; 6] = [
    feature!(
        atom!("width"),
        AllowsRanges::Yes,
        Evaluator::Length(eval_width),
        ParsingRequirements::empty(),
    ),
    feature!(
        atom!("height"),
        AllowsRanges::Yes,
        Evaluator::Length(eval_height),
        ParsingRequirements::empty(),
    ),
    feature!(
        atom!("inline-size"),
        AllowsRanges::Yes,
        Evaluator::Length(eval_inline_size),
        ParsingRequirements::empty(),
    ),
    feature!(
        atom!("block-size"),
        AllowsRanges::Yes,
        Evaluator::Length(eval_block_size),
        ParsingRequirements::empty(),
    ),
    feature!(
        atom!("aspect-ratio"),
        AllowsRanges::Yes,
        Evaluator::NumberRatio(eval_aspect_ratio),
        ParsingRequirements::empty(),
    ),
    feature!(
        atom!("orientation"),
        AllowsRanges::No,
        keyword_evaluator!(eval_orientation, Orientation),
        ParsingRequirements::empty(),
    ),
];
