/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A [`@container`][container] rule.
//!
//! [container]: https://drafts.csswg.org/css-contain-3/#container-rule

use crate::computed_value_flags::ComputedValueFlags;
use crate::dom::TElement;
use crate::logical_geometry::{LogicalSize, WritingMode};
use crate::media_queries::Device;
use crate::parser::ParserContext;
use crate::properties::ComputedValues;
use crate::queries::condition::KleeneValue;
use crate::queries::feature::{AllowsRanges, Evaluator, FeatureFlags, QueryFeatureDescription};
use crate::queries::values::Orientation;
use crate::queries::{FeatureType, QueryCondition};
use crate::shared_lock::{
    DeepCloneParams, DeepCloneWithLock, Locked, SharedRwLock, SharedRwLockReadGuard, ToCssWithGuard,
};
use crate::str::CssStringWriter;
use crate::stylesheets::CssRules;
use crate::values::computed::{CSSPixelLength, ContainerType, Context, Ratio};
use crate::values::specified::ContainerName;
use app_units::Au;
use cssparser::{Parser, SourceLocation};
use euclid::default::Size2D;
#[cfg(feature = "gecko")]
use malloc_size_of::{MallocSizeOfOps, MallocUnconditionalShallowSizeOf};
use servo_arc::Arc;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, ToCss};

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
        self.rules.unconditional_shallow_size_of(ops) +
            self.rules.read_with(guard).size_of(guard, ops)
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

fn container_type_axes(ty_: ContainerType, wm: WritingMode) -> FeatureFlags {
    match ty_ {
        ContainerType::Size => FeatureFlags::all_container_axes(),
        ContainerType::InlineSize => {
            let physical_axis = if wm.is_vertical() {
                FeatureFlags::CONTAINER_REQUIRES_HEIGHT_AXIS
            } else {
                FeatureFlags::CONTAINER_REQUIRES_WIDTH_AXIS
            };
            FeatureFlags::CONTAINER_REQUIRES_INLINE_AXIS | physical_axis
        },
        ContainerType::Normal => FeatureFlags::empty(),
    }
}

enum TraversalResult<T> {
    InProgress,
    StopTraversal,
    Done(T),
}

fn traverse_container<E, F, R>(
    mut e: E,
    originating_element_style: Option<&ComputedValues>,
    evaluator: F,
) -> Option<(E, R)>
where
    E: TElement,
    F: Fn(E, Option<&ComputedValues>) -> TraversalResult<R>,
{
    if originating_element_style.is_some() {
        match evaluator(e, originating_element_style) {
            TraversalResult::InProgress => {},
            TraversalResult::StopTraversal => return None,
            TraversalResult::Done(result) => return Some((e, result)),
        }
    }
    while let Some(element) = e.traversal_parent() {
        match evaluator(element, None) {
            TraversalResult::InProgress => {},
            TraversalResult::StopTraversal => return None,
            TraversalResult::Done(result) => return Some((element, result)),
        }
        e = element;
    }

    None
}

impl ContainerCondition {
    /// Parse a container condition.
    pub fn parse<'a>(
        context: &ParserContext,
        input: &mut Parser<'a, '_>,
    ) -> Result<Self, ParseError<'a>> {
        let name = input
            .try_parse(|input| ContainerName::parse_for_query(context, input))
            .ok()
            .unwrap_or_else(ContainerName::none);
        let condition = QueryCondition::parse(context, input, FeatureType::Container)?;
        let flags = condition.cumulative_flags();
        Ok(Self {
            name,
            condition,
            flags,
        })
    }

    fn valid_container_info<E>(
        &self,
        potential_container: E,
        originating_element_style: Option<&ComputedValues>,
    ) -> TraversalResult<ContainerLookupResult<E>>
    where
        E: TElement,
    {
        let data;
        let style = match originating_element_style {
            Some(s) => s,
            None => {
                data = match potential_container.borrow_data() {
                    Some(d) => d,
                    None => return TraversalResult::InProgress,
                };
                &**data.styles.primary()
            },
        };
        let wm = style.writing_mode;
        let box_style = style.get_box();

        // Filter by container-type.
        let container_type = box_style.clone_container_type();
        let available_axes = container_type_axes(container_type, wm);
        if !available_axes.contains(self.flags.container_axes()) {
            return TraversalResult::InProgress;
        }

        // Filter by container-name.
        let container_name = box_style.clone_container_name();
        for filter_name in self.name.0.iter() {
            if !container_name.0.contains(filter_name) {
                return TraversalResult::InProgress;
            }
        }

        let size = potential_container.query_container_size(&box_style.clone_display());
        let style = style.to_arc();
        TraversalResult::Done(ContainerLookupResult {
            element: potential_container,
            info: ContainerInfo { size, wm },
            style,
        })
    }

    /// Performs container lookup for a given element.
    pub fn find_container<E>(
        &self,
        e: E,
        originating_element_style: Option<&ComputedValues>,
    ) -> Option<ContainerLookupResult<E>>
    where
        E: TElement,
    {
        match traverse_container(
            e,
            originating_element_style,
            |element, originating_element_style| {
                self.valid_container_info(element, originating_element_style)
            },
        ) {
            Some((_, result)) => Some(result),
            None => None,
        }
    }

    /// Tries to match a container query condition for a given element.
    pub(crate) fn matches<E>(
        &self,
        device: &Device,
        element: E,
        originating_element_style: Option<&ComputedValues>,
        invalidation_flags: &mut ComputedValueFlags,
    ) -> KleeneValue
    where
        E: TElement,
    {
        let result = self.find_container(element, originating_element_style);
        let (container, info) = match result {
            Some(r) => (Some(r.element), Some((r.info, r.style))),
            None => (None, None),
        };
        // Set up the lookup for the container in question, as the condition may be using container query lengths.
        let size_query_container_lookup = ContainerSizeQuery::for_option_element(container, None);
        Context::for_container_query_evaluation(
            device,
            info,
            size_query_container_lookup,
            |context| {
                let matches = self.condition.matches(context);
                if context
                    .style()
                    .flags()
                    .contains(ComputedValueFlags::USES_VIEWPORT_UNITS)
                {
                    // TODO(emilio): Might need something similar to improve
                    // invalidation of font relative container-query lengths.
                    invalidation_flags
                        .insert(ComputedValueFlags::USES_VIEWPORT_UNITS_ON_CONTAINER_QUERIES);
                }
                matches
            },
        )
    }
}

/// Information needed to evaluate an individual container query.
#[derive(Copy, Clone)]
pub struct ContainerInfo {
    size: Size2D<Option<Au>>,
    wm: WritingMode,
}

impl ContainerInfo {
    fn size(&self) -> Option<Size2D<Au>> {
        Some(Size2D::new(self.size.width?, self.size.height?))
    }
}

fn eval_width(context: &Context) -> Option<CSSPixelLength> {
    let info = context.container_info.as_ref()?;
    Some(CSSPixelLength::new(info.size.width?.to_f32_px()))
}

fn eval_height(context: &Context) -> Option<CSSPixelLength> {
    let info = context.container_info.as_ref()?;
    Some(CSSPixelLength::new(info.size.height?.to_f32_px()))
}

fn eval_inline_size(context: &Context) -> Option<CSSPixelLength> {
    let info = context.container_info.as_ref()?;
    Some(CSSPixelLength::new(
        LogicalSize::from_physical(info.wm, info.size)
            .inline?
            .to_f32_px(),
    ))
}

fn eval_block_size(context: &Context) -> Option<CSSPixelLength> {
    let info = context.container_info.as_ref()?;
    Some(CSSPixelLength::new(
        LogicalSize::from_physical(info.wm, info.size)
            .block?
            .to_f32_px(),
    ))
}

fn eval_aspect_ratio(context: &Context) -> Option<Ratio> {
    let info = context.container_info.as_ref()?;
    Some(Ratio::new(
        info.size.width?.0 as f32,
        info.size.height?.0 as f32,
    ))
}

fn eval_orientation(context: &Context, value: Option<Orientation>) -> KleeneValue {
    let size = match context.container_info.as_ref().and_then(|info| info.size()) {
        Some(size) => size,
        None => return KleeneValue::Unknown,
    };
    KleeneValue::from(Orientation::eval(size, value))
}

/// https://drafts.csswg.org/css-contain-3/#container-features
///
/// TODO: Support style queries, perhaps.
pub static CONTAINER_FEATURES: [QueryFeatureDescription; 6] = [
    feature!(
        atom!("width"),
        AllowsRanges::Yes,
        Evaluator::OptionalLength(eval_width),
        FeatureFlags::CONTAINER_REQUIRES_WIDTH_AXIS,
    ),
    feature!(
        atom!("height"),
        AllowsRanges::Yes,
        Evaluator::OptionalLength(eval_height),
        FeatureFlags::CONTAINER_REQUIRES_HEIGHT_AXIS,
    ),
    feature!(
        atom!("inline-size"),
        AllowsRanges::Yes,
        Evaluator::OptionalLength(eval_inline_size),
        FeatureFlags::CONTAINER_REQUIRES_INLINE_AXIS,
    ),
    feature!(
        atom!("block-size"),
        AllowsRanges::Yes,
        Evaluator::OptionalLength(eval_block_size),
        FeatureFlags::CONTAINER_REQUIRES_BLOCK_AXIS,
    ),
    feature!(
        atom!("aspect-ratio"),
        AllowsRanges::Yes,
        Evaluator::OptionalNumberRatio(eval_aspect_ratio),
        // XXX from_bits_truncate is const, but the pipe operator isn't, so this
        // works around it.
        FeatureFlags::from_bits_truncate(
            FeatureFlags::CONTAINER_REQUIRES_BLOCK_AXIS.bits() |
                FeatureFlags::CONTAINER_REQUIRES_INLINE_AXIS.bits()
        ),
    ),
    feature!(
        atom!("orientation"),
        AllowsRanges::No,
        keyword_evaluator!(eval_orientation, Orientation),
        FeatureFlags::from_bits_truncate(
            FeatureFlags::CONTAINER_REQUIRES_BLOCK_AXIS.bits() |
                FeatureFlags::CONTAINER_REQUIRES_INLINE_AXIS.bits()
        ),
    ),
];

/// Result of a container size query, signifying the hypothetical containment boundary in terms of physical axes.
/// Defined by up to two size containers. Queries on logical axes are resolved with respect to the querying
/// element's writing mode.
#[derive(Copy, Clone, Default)]
pub struct ContainerSizeQueryResult {
    width: Option<Au>,
    height: Option<Au>,
}

impl ContainerSizeQueryResult {
    fn get_viewport_size(context: &Context) -> Size2D<Au> {
        use crate::values::specified::ViewportVariant;
        context.viewport_size_for_viewport_unit_resolution(ViewportVariant::Small)
    }

    fn get_logical_viewport_size(context: &Context) -> LogicalSize<Au> {
        LogicalSize::from_physical(
            context.builder.writing_mode,
            Self::get_viewport_size(context),
        )
    }

    /// Get the inline-size of the query container.
    pub fn get_container_inline_size(&self, context: &Context) -> Au {
        if context.builder.writing_mode.is_horizontal() {
            if let Some(w) = self.width {
                return w;
            }
        } else {
            if let Some(h) = self.height {
                return h;
            }
        }
        Self::get_logical_viewport_size(context).inline
    }

    /// Get the block-size of the query container.
    pub fn get_container_block_size(&self, context: &Context) -> Au {
        if context.builder.writing_mode.is_horizontal() {
            self.get_container_height(context)
        } else {
            self.get_container_width(context)
        }
    }

    /// Get the width of the query container.
    pub fn get_container_width(&self, context: &Context) -> Au {
        if let Some(w) = self.width {
            return w;
        }
        Self::get_viewport_size(context).width
    }

    /// Get the height of the query container.
    pub fn get_container_height(&self, context: &Context) -> Au {
        if let Some(h) = self.height {
            return h;
        }
        Self::get_viewport_size(context).height
    }

    // Merge the result of a subsequent lookup, preferring the initial result.
    fn merge(self, new_result: Self) -> Self {
        let mut result = self;
        if let Some(width) = new_result.width {
            result.width.get_or_insert(width);
        }
        if let Some(height) = new_result.height {
            result.height.get_or_insert(height);
        }
        result
    }

    fn is_complete(&self) -> bool {
        self.width.is_some() && self.height.is_some()
    }
}

/// Unevaluated lazy container size query.
pub enum ContainerSizeQuery<'a> {
    /// Query prior to evaluation.
    NotEvaluated(Box<dyn Fn() -> ContainerSizeQueryResult + 'a>),
    /// Cached evaluated result.
    Evaluated(ContainerSizeQueryResult),
}

impl<'a> ContainerSizeQuery<'a> {
    fn evaluate_potential_size_container<E>(
        e: E,
        originating_element_style: Option<&ComputedValues>,
    ) -> TraversalResult<ContainerSizeQueryResult>
    where
        E: TElement,
    {
        let data;
        let style = match originating_element_style {
            Some(s) => s,
            None => {
                data = match e.borrow_data() {
                    Some(d) => d,
                    None => return TraversalResult::InProgress,
                };
                &**data.styles.primary()
            },
        };
        if !style
            .flags
            .contains(ComputedValueFlags::SELF_OR_ANCESTOR_HAS_SIZE_CONTAINER_TYPE)
        {
            // We know we won't find a size container.
            return TraversalResult::StopTraversal;
        }

        let wm = style.writing_mode;
        let box_style = style.get_box();

        let container_type = box_style.clone_container_type();
        let size = e.query_container_size(&box_style.clone_display());
        match container_type {
            ContainerType::Size => TraversalResult::Done(ContainerSizeQueryResult {
                width: size.width,
                height: size.height,
            }),
            ContainerType::InlineSize => {
                if wm.is_horizontal() {
                    TraversalResult::Done(ContainerSizeQueryResult {
                        width: size.width,
                        height: None,
                    })
                } else {
                    TraversalResult::Done(ContainerSizeQueryResult {
                        width: None,
                        height: size.height,
                    })
                }
            },
            ContainerType::Normal => TraversalResult::InProgress,
        }
    }

    /// Find the query container size for a given element. Meant to be used as a callback for new().
    fn lookup<E>(
        element: E,
        originating_element_style: Option<&ComputedValues>,
    ) -> ContainerSizeQueryResult
    where
        E: TElement + 'a,
    {
        match traverse_container(
            element,
            originating_element_style,
            |e, originating_element_style| {
                Self::evaluate_potential_size_container(e, originating_element_style)
            },
        ) {
            Some((container, result)) => {
                if result.is_complete() {
                    result
                } else {
                    // Traverse up from the found size container to see if we can get a complete containment.
                    result.merge(Self::lookup(container, None))
                }
            },
            None => ContainerSizeQueryResult::default(),
        }
    }

    /// Create a new instance of the container size query for given element, with a deferred lookup callback.
    pub fn for_element<E>(element: E, originating_element_style: Option<&'a ComputedValues>) -> Self
    where
        E: TElement + 'a,
    {
        let parent;
        let data;
        let style = match originating_element_style {
            Some(s) => Some(s),
            None => {
                // No need to bother if we're the top element.
                parent = match element.traversal_parent() {
                    Some(parent) => parent,
                    None => return Self::none(),
                };
                data = parent.borrow_data();
                data.as_ref().map(|data| &**data.styles.primary())
            },
        };
        let should_traverse = match style {
            Some(style) => style
                .flags
                .contains(ComputedValueFlags::SELF_OR_ANCESTOR_HAS_SIZE_CONTAINER_TYPE),
            None => true, // `display: none`, still want to show a correct computed value, so give it a try.
        };
        if should_traverse {
            return Self::NotEvaluated(Box::new(move || {
                Self::lookup(element, originating_element_style)
            }));
        }
        Self::none()
    }

    /// Create a new instance, but with optional element.
    pub fn for_option_element<E>(
        element: Option<E>,
        originating_element_style: Option<&'a ComputedValues>,
    ) -> Self
    where
        E: TElement + 'a,
    {
        if let Some(e) = element {
            Self::for_element(e, originating_element_style)
        } else {
            Self::none()
        }
    }

    /// Create a query that evaluates to empty, for cases where container size query is not required.
    pub fn none() -> Self {
        ContainerSizeQuery::Evaluated(ContainerSizeQueryResult::default())
    }

    /// Get the result of the container size query, doing the lookup if called for the first time.
    pub fn get(&mut self) -> ContainerSizeQueryResult {
        match self {
            Self::NotEvaluated(lookup) => {
                *self = Self::Evaluated((lookup)());
                match self {
                    Self::Evaluated(info) => *info,
                    _ => unreachable!("Just evaluated but not set?"),
                }
            },
            Self::Evaluated(info) => *info,
        }
    }
}
