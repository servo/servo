/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A [`@container`][container] rule.
//!
//! [container]: https://drafts.csswg.org/css-contain-3/#container-rule

use crate::logical_geometry::{WritingMode, LogicalSize};
use crate::dom::TElement;
use crate::media_queries::Device;
use crate::parser::ParserContext;
use crate::queries::{QueryCondition, FeatureType};
use crate::queries::feature::{AllowsRanges, Evaluator, FeatureFlags, QueryFeatureDescription};
use crate::queries::values::Orientation;
use crate::str::CssStringWriter;
use crate::shared_lock::{
    DeepCloneParams, DeepCloneWithLock, Locked, SharedRwLock, SharedRwLockReadGuard, ToCssWithGuard,
};
use crate::values::specified::ContainerName;
use crate::values::computed::{Context, CSSPixelLength, Ratio};
use crate::properties::ComputedValues;
use crate::stylesheets::CssRules;
use app_units::Au;
use cssparser::{SourceLocation, Parser};
use euclid::default::Size2D;
#[cfg(feature = "gecko")]
use malloc_size_of::{MallocSizeOfOps, MallocUnconditionalShallowSizeOf};
use servo_arc::Arc;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss, ParseError};

/// A container rule.
#[derive(Debug, ToShmem)]
pub struct ContainerRule {
    /// The container query and name.
    pub condition: Arc<ContainerCondition>,
    /// The nested rules inside the block.
    pub rules: Arc<Locked<CssRules>>,
    /// The source position where this rule was found.
    pub source_location: SourceLocation,
}

impl ContainerRule {
    /// Returns the query condition.
    pub fn query_condition(&self) -> &QueryCondition {
        &self.condition.condition
    }

    /// Returns the query name filter.
    pub fn container_name(&self) -> &ContainerName {
        &self.condition.name
    }

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
            if !self.condition.name.is_none() {
                self.condition.name.to_css(&mut writer)?;
                writer.write_char(' ')?;
            }
            self.condition.condition.to_css(&mut writer)?;
        }
        self.rules.read_with(guard).to_css_block(guard, dest)
    }
}

/// A container condition and filter, combined.
#[derive(Debug, ToShmem, ToCss)]
pub struct ContainerCondition {
    #[css(skip_if = "ContainerName::is_none")]
    name: ContainerName,
    condition: QueryCondition,
    #[css(skip)]
    flags: FeatureFlags,
}

/// The result of a successful container query lookup.
pub struct ContainerLookupResult<E> {
    /// The relevant container.
    pub element: E,
    /// The sizing / writing-mode information of the container.
    pub info: ContainerInfo,
    /// The style of the element.
    pub style: Arc<ComputedValues>,
}

impl ContainerCondition {
    /// Parse a container condition.
    pub fn parse<'a>(
        context: &ParserContext,
        input: &mut Parser<'a, '_>,
    ) -> Result<Self, ParseError<'a>> {
        use crate::parser::Parse;

        // FIXME: This is a bit ambiguous:
        // https://github.com/w3c/csswg-drafts/issues/7203
        let name = input.try_parse(|input| {
            ContainerName::parse(context, input)
        }).ok().unwrap_or_else(ContainerName::none);
        let condition = QueryCondition::parse(context, input, FeatureType::Container)?;
        let flags = condition.cumulative_flags();
        Ok(Self { name, condition, flags })
    }

    fn valid_container_info<E>(&self, potential_container: E) -> Option<ContainerLookupResult<E>>
    where
        E: TElement,
    {
        use crate::values::computed::ContainerType;

        fn container_type_axes(ty_: ContainerType, wm: WritingMode) -> FeatureFlags {
            if ty_.contains(ContainerType::SIZE) {
                return FeatureFlags::all_container_axes()
            }
            if ty_.contains(ContainerType::INLINE_SIZE) {
                let physical_axis = if wm.is_vertical() {
                    FeatureFlags::CONTAINER_REQUIRES_HEIGHT_AXIS
                } else {
                    FeatureFlags::CONTAINER_REQUIRES_WIDTH_AXIS
                };
                return FeatureFlags::CONTAINER_REQUIRES_INLINE_AXIS | physical_axis
            }
            FeatureFlags::empty()
        }

        let data = match potential_container.borrow_data() {
            Some(data) => data,
            None => return None,
        };
        let style = data.styles.primary();
        let wm = style.writing_mode;
        let box_style = style.get_box();

        // Filter by container-type.
        let container_type = box_style.clone_container_type();
        let available_axes = container_type_axes(container_type, wm);
        if !available_axes.contains(self.flags.container_axes()) {
            return None;
        }

        // Filter by container-name.
        let container_name = box_style.clone_container_name();
        for filter_name in self.name.0.iter() {
            if !container_name.0.contains(filter_name) {
                return None;
            }
        }

        let size = potential_container.primary_box_size();
        let style = style.clone();
        Some(ContainerLookupResult {
            element: potential_container,
            info: ContainerInfo { size, wm },
            style,
        })
    }

    /// Performs container lookup for a given element.
    pub fn find_container<E>(&self, mut e: E) -> Option<ContainerLookupResult<E>>
    where
        E: TElement,
    {
        while let Some(element) = e.traversal_parent() {
            if let Some(result) = self.valid_container_info(element) {
                return Some(result);
            }
            e = element;
        }

        None
    }

    /// Tries to match a container query condition for a given element.
    pub(crate) fn matches<E>(&self, device: &Device, element: E) -> bool
    where
        E: TElement,
    {
        let result = self.find_container(element);
        let info = result.map(|r| (r.info, r.style));
        Context::for_container_query_evaluation(device, info, |context| {
            self.condition.matches(context)
        })
    }
}


/// Information needed to evaluate an individual container query.
#[derive(Copy, Clone)]
pub struct ContainerInfo {
    size: Size2D<Au>,
    wm: WritingMode,
}

fn get_container(context: &Context) -> ContainerInfo {
    if let Some(ref info) = context.container_info {
        return info.clone()
    }
    ContainerInfo {
        size: context.device().au_viewport_size(),
        wm: WritingMode::horizontal_tb(),
    }
}

fn eval_width(context: &Context) -> CSSPixelLength {
    let info = get_container(context);
    CSSPixelLength::new(info.size.width.to_f32_px())
}

fn eval_height(context: &Context) -> CSSPixelLength {
    let info = get_container(context);
    CSSPixelLength::new(info.size.height.to_f32_px())
}

fn eval_inline_size(context: &Context) -> CSSPixelLength {
    let info = get_container(context);
    CSSPixelLength::new(LogicalSize::from_physical(info.wm, info.size).inline.to_f32_px())
}

fn eval_block_size(context: &Context) -> CSSPixelLength {
    let info = get_container(context);
    CSSPixelLength::new(LogicalSize::from_physical(info.wm, info.size).block.to_f32_px())
}

fn eval_aspect_ratio(context: &Context) -> Ratio {
    let info = get_container(context);
    Ratio::new(info.size.width.0 as f32, info.size.height.0 as f32)
}

fn eval_orientation(context: &Context, value: Option<Orientation>) -> bool {
    let info = get_container(context);
    Orientation::eval(info.size, value)
}

/// https://drafts.csswg.org/css-contain-3/#container-features
///
/// TODO: Support style queries, perhaps.
pub static CONTAINER_FEATURES: [QueryFeatureDescription; 6] = [
    feature!(
        atom!("width"),
        AllowsRanges::Yes,
        Evaluator::Length(eval_width),
        FeatureFlags::CONTAINER_REQUIRES_WIDTH_AXIS,
    ),
    feature!(
        atom!("height"),
        AllowsRanges::Yes,
        Evaluator::Length(eval_height),
        FeatureFlags::CONTAINER_REQUIRES_HEIGHT_AXIS,
    ),
    feature!(
        atom!("inline-size"),
        AllowsRanges::Yes,
        Evaluator::Length(eval_inline_size),
        FeatureFlags::CONTAINER_REQUIRES_INLINE_AXIS,
    ),
    feature!(
        atom!("block-size"),
        AllowsRanges::Yes,
        Evaluator::Length(eval_block_size),
        FeatureFlags::CONTAINER_REQUIRES_BLOCK_AXIS,
    ),
    feature!(
        atom!("aspect-ratio"),
        AllowsRanges::Yes,
        Evaluator::NumberRatio(eval_aspect_ratio),
        // XXX from_bits_truncate is const, but the pipe operator isn't, so this
        // works around it.
        FeatureFlags::from_bits_truncate(FeatureFlags::CONTAINER_REQUIRES_BLOCK_AXIS.bits() | FeatureFlags::CONTAINER_REQUIRES_INLINE_AXIS.bits()),
    ),
    feature!(
        atom!("orientation"),
        AllowsRanges::No,
        keyword_evaluator!(eval_orientation, Orientation),
        FeatureFlags::from_bits_truncate(FeatureFlags::CONTAINER_REQUIRES_BLOCK_AXIS.bits() | FeatureFlags::CONTAINER_REQUIRES_INLINE_AXIS.bits()),
    ),
];
