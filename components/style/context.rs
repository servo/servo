/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The context within which style is calculated.

#[cfg(feature = "servo")] use animation::Animation;
use app_units::Au;
use bloom::StyleBloom;
use data::{EagerPseudoStyles, ElementData};
use dom::{TElement, SendElement};
#[cfg(feature = "servo")] use dom::OpaqueNode;
use euclid::Size2D;
use euclid::TypedScale;
use fnv::FnvHashMap;
use font_metrics::FontMetricsProvider;
#[cfg(feature = "gecko")] use gecko_bindings::structs;
use parallel::{STACK_SAFETY_MARGIN_KB, STYLE_THREAD_STACK_SIZE_KB};
#[cfg(feature = "servo")] use parking_lot::RwLock;
use properties::ComputedValues;
#[cfg(feature = "servo")] use properties::PropertyId;
use rule_cache::RuleCache;
use rule_tree::StrongRuleNode;
use selector_parser::{EAGER_PSEUDO_COUNT, SnapshotMap};
use selectors::NthIndexCache;
use selectors::matching::ElementSelectorFlags;
use servo_arc::Arc;
#[cfg(feature = "servo")] use servo_atoms::Atom;
use shared_lock::StylesheetGuards;
use sharing::StyleSharingCache;
use std::fmt;
use std::ops;
#[cfg(feature = "servo")] use std::sync::Mutex;
#[cfg(feature = "servo")] use std::sync::mpsc::Sender;
use style_traits::CSSPixel;
use style_traits::DevicePixel;
#[cfg(feature = "servo")] use style_traits::SpeculativePainter;
use stylist::Stylist;
use thread_state::{self, ThreadState};
use time;
use timer::Timer;
use traversal::DomTraversal;
use traversal_flags::TraversalFlags;
use uluru::{Entry, LRUCache};

pub use selectors::matching::QuirksMode;

/// This structure is used to create a local style context from a shared one.
#[cfg(feature = "servo")]
pub struct ThreadLocalStyleContextCreationInfo {
    new_animations_sender: Sender<Animation>,
}

#[cfg(feature = "servo")]
impl ThreadLocalStyleContextCreationInfo {
    /// Trivially constructs a `ThreadLocalStyleContextCreationInfo`.
    pub fn new(animations_sender: Sender<Animation>) -> Self {
        ThreadLocalStyleContextCreationInfo {
            new_animations_sender: animations_sender,
        }
    }
}

/// A global options structure for the style system. We use this instead of
/// opts to abstract across Gecko and Servo.
#[derive(Clone)]
pub struct StyleSystemOptions {
    /// Whether the style sharing cache is disabled.
    pub disable_style_sharing_cache: bool,
    /// Whether we should dump statistics about the style system.
    pub dump_style_statistics: bool,
    /// The minimum number of elements that must be traversed to trigger a dump
    /// of style statistics.
    pub style_statistics_threshold: usize,
}

#[cfg(feature = "gecko")]
fn get_env_bool(name: &str) -> bool {
    use std::env;
    match env::var(name) {
        Ok(s) => !s.is_empty(),
        Err(_) => false,
    }
}

const DEFAULT_STATISTICS_THRESHOLD: usize = 50;

#[cfg(feature = "gecko")]
fn get_env_usize(name: &str) -> Option<usize> {
    use std::env;
    env::var(name).ok().map(|s| {
        s.parse::<usize>().expect("Couldn't parse environmental variable as usize")
    })
}

impl Default for StyleSystemOptions {
    #[cfg(feature = "servo")]
    fn default() -> Self {
        use servo_config::opts;

        StyleSystemOptions {
            disable_style_sharing_cache: opts::get().disable_share_style_cache,
            dump_style_statistics: opts::get().style_sharing_stats,
            style_statistics_threshold: DEFAULT_STATISTICS_THRESHOLD,
        }
    }

    #[cfg(feature = "gecko")]
    fn default() -> Self {
        StyleSystemOptions {
            disable_style_sharing_cache: get_env_bool("DISABLE_STYLE_SHARING_CACHE"),
            dump_style_statistics: get_env_bool("DUMP_STYLE_STATISTICS"),
            style_statistics_threshold: get_env_usize("STYLE_STATISTICS_THRESHOLD")
                                          .unwrap_or(DEFAULT_STATISTICS_THRESHOLD),
        }
    }
}

impl StyleSystemOptions {
    #[cfg(feature = "servo")]
    /// On Gecko's nightly build?
    pub fn is_nightly(&self) -> bool {
        false
    }

    #[cfg(feature = "gecko")]
    /// On Gecko's nightly build?
    #[inline]
    pub fn is_nightly(&self) -> bool {
        structs::GECKO_IS_NIGHTLY
    }
}

/// A shared style context.
///
/// There's exactly one of these during a given restyle traversal, and it's
/// shared among the worker threads.
pub struct SharedStyleContext<'a> {
    /// The CSS selector stylist.
    pub stylist: &'a Stylist,

    /// Whether visited styles are enabled.
    ///
    /// They may be disabled when Gecko's pref layout.css.visited_links_enabled
    /// is false, or when in private browsing mode.
    pub visited_styles_enabled: bool,

    /// Configuration options.
    pub options: StyleSystemOptions,

    /// Guards for pre-acquired locks
    pub guards: StylesheetGuards<'a>,

    /// The current timer for transitions and animations. This is needed to test
    /// them.
    pub timer: Timer,

    /// Flags controlling how we traverse the tree.
    pub traversal_flags: TraversalFlags,

    /// A map with our snapshots in order to handle restyle hints.
    pub snapshot_map: &'a SnapshotMap,

    /// The animations that are currently running.
    #[cfg(feature = "servo")]
    pub running_animations: Arc<RwLock<FnvHashMap<OpaqueNode, Vec<Animation>>>>,

    /// The list of animations that have expired since the last style recalculation.
    #[cfg(feature = "servo")]
    pub expired_animations: Arc<RwLock<FnvHashMap<OpaqueNode, Vec<Animation>>>>,

    /// Paint worklets
    #[cfg(feature = "servo")]
    pub registered_speculative_painters: &'a RegisteredSpeculativePainters,

    /// Data needed to create the thread-local style context from the shared one.
    #[cfg(feature = "servo")]
    pub local_context_creation_data: Mutex<ThreadLocalStyleContextCreationInfo>,

}

impl<'a> SharedStyleContext<'a> {
    /// Return a suitable viewport size in order to be used for viewport units.
    pub fn viewport_size(&self) -> Size2D<Au> {
        self.stylist.device().au_viewport_size()
    }

    /// The device pixel ratio
    pub fn device_pixel_ratio(&self) -> TypedScale<f32, CSSPixel, DevicePixel> {
        self.stylist.device().device_pixel_ratio()
    }

    /// The quirks mode of the document.
    pub fn quirks_mode(&self) -> QuirksMode {
        self.stylist.quirks_mode()
    }
}

/// The structure holds various intermediate inputs that are eventually used by
/// by the cascade.
///
/// The matching and cascading process stores them in this format temporarily
/// within the `CurrentElementInfo`. At the end of the cascade, they are folded
/// down into the main `ComputedValues` to reduce memory usage per element while
/// still remaining accessible.
#[derive(Clone, Debug, Default)]
pub struct CascadeInputs {
    /// The rule node representing the ordered list of rules matched for this
    /// node.
    pub rules: Option<StrongRuleNode>,

    /// The rule node representing the ordered list of rules matched for this
    /// node if visited, only computed if there's a relevant link for this
    /// element. A element's "relevant link" is the element being matched if it
    /// is a link or the nearest ancestor link.
    pub visited_rules: Option<StrongRuleNode>,
}

impl CascadeInputs {
    /// Construct inputs from previous cascade results, if any.
    pub fn new_from_style(style: &ComputedValues) -> Self {
        CascadeInputs {
            rules: style.rules.clone(),
            visited_rules: style.visited_style().and_then(|v| v.rules.clone()),
        }
    }
}

/// A list of cascade inputs for eagerly-cascaded pseudo-elements.
/// The list is stored inline.
#[derive(Debug)]
pub struct EagerPseudoCascadeInputs(Option<[Option<CascadeInputs>; EAGER_PSEUDO_COUNT]>);

// Manually implement `Clone` here because the derived impl of `Clone` for
// array types assumes the value inside is `Copy`.
impl Clone for EagerPseudoCascadeInputs {
    fn clone(&self) -> Self {
        if self.0.is_none() {
            return EagerPseudoCascadeInputs(None)
        }
        let self_inputs = self.0.as_ref().unwrap();
        let mut inputs: [Option<CascadeInputs>; EAGER_PSEUDO_COUNT] = Default::default();
        for i in 0..EAGER_PSEUDO_COUNT {
            inputs[i] = self_inputs[i].clone();
        }
        EagerPseudoCascadeInputs(Some(inputs))
    }
}

impl EagerPseudoCascadeInputs {
    /// Construct inputs from previous cascade results, if any.
    fn new_from_style(styles: &EagerPseudoStyles) -> Self {
        EagerPseudoCascadeInputs(styles.as_optional_array().map(|styles| {
            let mut inputs: [Option<CascadeInputs>; EAGER_PSEUDO_COUNT] = Default::default();
            for i in 0..EAGER_PSEUDO_COUNT {
                inputs[i] = styles[i].as_ref().map(|s| CascadeInputs::new_from_style(s));
            }
            inputs
        }))
    }

    /// Returns the list of rules, if they exist.
    pub fn into_array(self) -> Option<[Option<CascadeInputs>; EAGER_PSEUDO_COUNT]> {
        self.0
    }
}

/// The cascade inputs associated with a node, including those for any
/// pseudo-elements.
///
/// The matching and cascading process stores them in this format temporarily
/// within the `CurrentElementInfo`. At the end of the cascade, they are folded
/// down into the main `ComputedValues` to reduce memory usage per element while
/// still remaining accessible.
#[derive(Clone, Debug)]
pub struct ElementCascadeInputs {
    /// The element's cascade inputs.
    pub primary: CascadeInputs,
    /// A list of the inputs for the element's eagerly-cascaded pseudo-elements.
    pub pseudos: EagerPseudoCascadeInputs,
}

impl ElementCascadeInputs {
    /// Construct inputs from previous cascade results, if any.
    pub fn new_from_element_data(data: &ElementData) -> Self {
        debug_assert!(data.has_styles());
        ElementCascadeInputs {
            primary: CascadeInputs::new_from_style(data.styles.primary()),
            pseudos: EagerPseudoCascadeInputs::new_from_style(&data.styles.pseudos),
        }
    }
}

/// Statistics gathered during the traversal. We gather statistics on each
/// thread and then combine them after the threads join via the Add
/// implementation below.
#[derive(Default)]
pub struct TraversalStatistics {
    /// The total number of elements traversed.
    pub elements_traversed: u32,
    /// The number of elements where has_styles() went from false to true.
    pub elements_styled: u32,
    /// The number of elements for which we performed selector matching.
    pub elements_matched: u32,
    /// The number of cache hits from the StyleSharingCache.
    pub styles_shared: u32,
    /// The number of styles reused via rule node comparison from the
    /// StyleSharingCache.
    pub styles_reused: u32,
    /// The number of selectors in the stylist.
    pub selectors: u32,
    /// The number of revalidation selectors.
    pub revalidation_selectors: u32,
    /// The number of state/attr dependencies in the dependency set.
    pub dependency_selectors: u32,
    /// The number of declarations in the stylist.
    pub declarations: u32,
    /// The number of times the stylist was rebuilt.
    pub stylist_rebuilds: u32,
    /// Time spent in the traversal, in milliseconds.
    pub traversal_time_ms: f64,
    /// Whether this was a parallel traversal.
    pub is_parallel: Option<bool>,
    /// Whether this is a "large" traversal.
    pub is_large: Option<bool>,
}

/// Implementation of Add to aggregate statistics across different threads.
impl<'a> ops::Add for &'a TraversalStatistics {
    type Output = TraversalStatistics;
    fn add(self, other: Self) -> TraversalStatistics {
        debug_assert!(self.traversal_time_ms == 0.0 && other.traversal_time_ms == 0.0,
                      "traversal_time_ms should be set at the end by the caller");
        debug_assert!(self.selectors == 0, "set at the end");
        debug_assert!(self.revalidation_selectors == 0, "set at the end");
        debug_assert!(self.dependency_selectors == 0, "set at the end");
        debug_assert!(self.declarations == 0, "set at the end");
        debug_assert!(self.stylist_rebuilds == 0, "set at the end");
        TraversalStatistics {
            elements_traversed: self.elements_traversed + other.elements_traversed,
            elements_styled: self.elements_styled + other.elements_styled,
            elements_matched: self.elements_matched + other.elements_matched,
            styles_shared: self.styles_shared + other.styles_shared,
            styles_reused: self.styles_reused + other.styles_reused,
            selectors: 0,
            revalidation_selectors: 0,
            dependency_selectors: 0,
            declarations: 0,
            stylist_rebuilds: 0,
            traversal_time_ms: 0.0,
            is_parallel: None,
            is_large: None,
        }
    }
}

/// Format the statistics in a way that the performance test harness understands.
/// See https://bugzilla.mozilla.org/show_bug.cgi?id=1331856#c2
impl fmt::Display for TraversalStatistics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        debug_assert!(self.traversal_time_ms != 0.0, "should have set traversal time");
        writeln!(f, "[PERF] perf block start")?;
        writeln!(f, "[PERF],traversal,{}", if self.is_parallel.unwrap() {
            "parallel"
        } else {
            "sequential"
        })?;
        writeln!(f, "[PERF],elements_traversed,{}", self.elements_traversed)?;
        writeln!(f, "[PERF],elements_styled,{}", self.elements_styled)?;
        writeln!(f, "[PERF],elements_matched,{}", self.elements_matched)?;
        writeln!(f, "[PERF],styles_shared,{}", self.styles_shared)?;
        writeln!(f, "[PERF],styles_reused,{}", self.styles_reused)?;
        writeln!(f, "[PERF],selectors,{}", self.selectors)?;
        writeln!(f, "[PERF],revalidation_selectors,{}", self.revalidation_selectors)?;
        writeln!(f, "[PERF],dependency_selectors,{}", self.dependency_selectors)?;
        writeln!(f, "[PERF],declarations,{}", self.declarations)?;
        writeln!(f, "[PERF],stylist_rebuilds,{}", self.stylist_rebuilds)?;
        writeln!(f, "[PERF],traversal_time_ms,{}", self.traversal_time_ms)?;
        writeln!(f, "[PERF] perf block end")
    }
}

impl TraversalStatistics {
    /// Computes the traversal time given the start time in seconds.
    pub fn finish<E, D>(&mut self, traversal: &D, parallel: bool, start: f64)
        where E: TElement,
              D: DomTraversal<E>,
    {
        let threshold = traversal.shared_context().options.style_statistics_threshold;
        let stylist = traversal.shared_context().stylist;

        self.is_parallel = Some(parallel);
        self.is_large = Some(self.elements_traversed as usize >= threshold);
        self.traversal_time_ms = (time::precise_time_s() - start) * 1000.0;
        self.selectors = stylist.num_selectors() as u32;
        self.revalidation_selectors = stylist.num_revalidation_selectors() as u32;
        self.dependency_selectors = stylist.num_invalidations() as u32;
        self.declarations = stylist.num_declarations() as u32;
        self.stylist_rebuilds = stylist.num_rebuilds() as u32;
    }

    /// Returns whether this traversal is 'large' in order to avoid console spam
    /// from lots of tiny traversals.
    pub fn is_large_traversal(&self) -> bool {
        self.is_large.unwrap()
    }
}

#[cfg(feature = "gecko")]
bitflags! {
    /// Represents which tasks are performed in a SequentialTask of
    /// UpdateAnimations which is a result of normal restyle.
    pub struct UpdateAnimationsTasks: u8 {
        /// Update CSS Animations.
        const CSS_ANIMATIONS = structs::UpdateAnimationsTasks_CSSAnimations;
        /// Update CSS Transitions.
        const CSS_TRANSITIONS = structs::UpdateAnimationsTasks_CSSTransitions;
        /// Update effect properties.
        const EFFECT_PROPERTIES = structs::UpdateAnimationsTasks_EffectProperties;
        /// Update animation cacade results for animations running on the compositor.
        const CASCADE_RESULTS = structs::UpdateAnimationsTasks_CascadeResults;
    }
}

#[cfg(feature = "gecko")]
bitflags! {
    /// Represents which tasks are performed in a SequentialTask as a result of
    /// animation-only restyle.
    pub struct PostAnimationTasks: u8 {
        /// Display property was changed from none in animation-only restyle so
        /// that we need to resolve styles for descendants in a subsequent
        /// normal restyle.
        const DISPLAY_CHANGED_FROM_NONE_FOR_SMIL = 0x01;
    }
}


/// A task to be run in sequential mode on the parent (non-worker) thread. This
/// is used by the style system to queue up work which is not safe to do during
/// the parallel traversal.
pub enum SequentialTask<E: TElement> {
    /// Entry to avoid an unused type parameter error on servo.
    Unused(SendElement<E>),

    /// Performs one of a number of possible tasks related to updating animations based on the
    /// |tasks| field. These include updating CSS animations/transitions that changed as part
    /// of the non-animation style traversal, and updating the computed effect properties.
    #[cfg(feature = "gecko")]
    UpdateAnimations {
        /// The target element or pseudo-element.
        el: SendElement<E>,
        /// The before-change style for transitions. We use before-change style as the initial
        /// value of its Keyframe. Required if |tasks| includes CSSTransitions.
        before_change_style: Option<Arc<ComputedValues>>,
        /// The tasks which are performed in this SequentialTask.
        tasks: UpdateAnimationsTasks
    },

    /// Performs one of a number of possible tasks as a result of animation-only restyle.
    /// Currently we do only process for resolving descendant elements that were display:none
    /// subtree for SMIL animation.
    #[cfg(feature = "gecko")]
    PostAnimation {
        /// The target element.
        el: SendElement<E>,
        /// The tasks which are performed in this SequentialTask.
        tasks: PostAnimationTasks
    },
}

impl<E: TElement> SequentialTask<E> {
    /// Executes this task.
    pub fn execute(self) {
        use self::SequentialTask::*;
        debug_assert!(thread_state::get() == ThreadState::LAYOUT);
        match self {
            Unused(_) => unreachable!(),
            #[cfg(feature = "gecko")]
            UpdateAnimations { el, before_change_style, tasks } => {
                el.update_animations(before_change_style, tasks);
            }
            #[cfg(feature = "gecko")]
            PostAnimation { el, tasks } => {
                el.process_post_animation(tasks);
            }
        }
    }

    /// Creates a task to update various animation-related state on
    /// a given (pseudo-)element.
    #[cfg(feature = "gecko")]
    pub fn update_animations(el: E,
                             before_change_style: Option<Arc<ComputedValues>>,
                             tasks: UpdateAnimationsTasks) -> Self {
        use self::SequentialTask::*;
        UpdateAnimations {
            el: unsafe { SendElement::new(el) },
            before_change_style: before_change_style,
            tasks: tasks,
        }
    }

    /// Creates a task to do post-process for a given element as a result of
    /// animation-only restyle.
    #[cfg(feature = "gecko")]
    pub fn process_post_animation(el: E, tasks: PostAnimationTasks) -> Self {
        use self::SequentialTask::*;
        PostAnimation {
            el: unsafe { SendElement::new(el) },
            tasks: tasks,
        }
    }
}

type CacheItem<E> = (SendElement<E>, ElementSelectorFlags);

/// Map from Elements to ElementSelectorFlags. Used to defer applying selector
/// flags until after the traversal.
pub struct SelectorFlagsMap<E: TElement> {
    /// The hashmap storing the flags to apply.
    map: FnvHashMap<SendElement<E>, ElementSelectorFlags>,
    /// An LRU cache to avoid hashmap lookups, which can be slow if the map
    /// gets big.
    cache: LRUCache<[Entry<CacheItem<E>>; 4 + 1]>,
}

#[cfg(debug_assertions)]
impl<E: TElement> Drop for SelectorFlagsMap<E> {
    fn drop(&mut self) {
        debug_assert!(self.map.is_empty());
    }
}

impl<E: TElement> SelectorFlagsMap<E> {
    /// Creates a new empty SelectorFlagsMap.
    pub fn new() -> Self {
        SelectorFlagsMap {
            map: FnvHashMap::default(),
            cache: LRUCache::default(),
        }
    }

    /// Inserts some flags into the map for a given element.
    pub fn insert_flags(&mut self, element: E, flags: ElementSelectorFlags) {
        let el = unsafe { SendElement::new(element) };
        // Check the cache. If the flags have already been noted, we're done.
        if let Some(item) = self.cache.find(|x| x.0 == el) {
            if !item.1.contains(flags) {
                item.1.insert(flags);
                self.map.get_mut(&el).unwrap().insert(flags);
            }
            return;
        }

        let f = self.map.entry(el).or_insert(ElementSelectorFlags::empty());
        *f |= flags;

        self.cache.insert((unsafe { SendElement::new(element) }, *f))
    }

    /// Applies the flags. Must be called on the main thread.
    fn apply_flags(&mut self) {
        debug_assert!(thread_state::get() == ThreadState::LAYOUT);
        self.cache.evict_all();
        for (el, flags) in self.map.drain() {
            unsafe { el.set_selector_flags(flags); }
        }
    }
}

/// A list of SequentialTasks that get executed on Drop.
pub struct SequentialTaskList<E>(Vec<SequentialTask<E>>)
where
    E: TElement;

impl<E> ops::Deref for SequentialTaskList<E>
where
    E: TElement,
{
    type Target = Vec<SequentialTask<E>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> ops::DerefMut for SequentialTaskList<E>
where
    E: TElement,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<E> Drop for SequentialTaskList<E>
where
    E: TElement,
{
    fn drop(&mut self) {
        debug_assert!(thread_state::get() == ThreadState::LAYOUT);
        for task in self.0.drain(..) {
            task.execute()
        }
    }
}


/// A helper type for stack limit checking.  This assumes that stacks grow
/// down, which is true for all non-ancient CPU architectures.
pub struct StackLimitChecker {
   lower_limit: usize
}

impl StackLimitChecker {
    /// Create a new limit checker, for this thread, allowing further use
    /// of up to |stack_size| bytes beyond (below) the current stack pointer.
    #[inline(never)]
    pub fn new(stack_size_limit: usize) -> Self {
        StackLimitChecker {
            lower_limit: StackLimitChecker::get_sp() - stack_size_limit
        }
    }

    /// Checks whether the previously stored stack limit has now been exceeded.
    #[inline(never)]
    pub fn limit_exceeded(&self) -> bool {
        let curr_sp = StackLimitChecker::get_sp();

        // Do some sanity-checking to ensure that our invariants hold, even in
        // the case where we've exceeded the soft limit.
        //
        // The correctness of depends on the assumption that no stack wraps
        // around the end of the address space.
        if cfg!(debug_assertions) {
            // Compute the actual bottom of the stack by subtracting our safety
            // margin from our soft limit. Note that this will be slightly below
            // the actual bottom of the stack, because there are a few initial
            // frames on the stack before we do the measurement that computes
            // the limit.
            let stack_bottom = self.lower_limit - STACK_SAFETY_MARGIN_KB * 1024;

            // The bottom of the stack should be below the current sp. If it
            // isn't, that means we've either waited too long to check the limit
            // and burned through our safety margin (in which case we probably
            // would have segfaulted by now), or we're using a limit computed for
            // a different thread.
            debug_assert!(stack_bottom < curr_sp);

            // Compute the distance between the current sp and the bottom of
            // the stack, and compare it against the current stack. It should be
            // no further from us than the total stack size. We allow some slop
            // to handle the fact that stack_bottom is a bit further than the
            // bottom of the stack, as discussed above.
            let distance_to_stack_bottom = curr_sp - stack_bottom;
            let max_allowable_distance = (STYLE_THREAD_STACK_SIZE_KB + 10) * 1024;
            debug_assert!(distance_to_stack_bottom <= max_allowable_distance);
        }

        // The actual bounds check.
        curr_sp <= self.lower_limit
    }

    // Technically, rustc can optimize this away, but shouldn't for now.
    // We should fix this once black_box is stable.
    #[inline(always)]
    fn get_sp() -> usize {
        let mut foo: usize = 42;
        (&mut foo as *mut usize) as usize
    }
}


/// A thread-local style context.
///
/// This context contains data that needs to be used during restyling, but is
/// not required to be unique among worker threads, so we create one per worker
/// thread in order to be able to mutate it without locking.
pub struct ThreadLocalStyleContext<E: TElement> {
    /// A cache to share style among siblings.
    pub sharing_cache: StyleSharingCache<E>,
    /// A cache from matched properties to elements that match those.
    pub rule_cache: RuleCache,
    /// The bloom filter used to fast-reject selector-matching.
    pub bloom_filter: StyleBloom<E>,
    /// A channel on which new animations that have been triggered by style
    /// recalculation can be sent.
    #[cfg(feature = "servo")]
    pub new_animations_sender: Sender<Animation>,
    /// A set of tasks to be run (on the parent thread) in sequential mode after
    /// the rest of the styling is complete. This is useful for
    /// infrequently-needed non-threadsafe operations.
    ///
    /// It's important that goes after the style sharing cache and the bloom
    /// filter, to ensure they're dropped before we execute the tasks, which
    /// could create another ThreadLocalStyleContext for style computation.
    pub tasks: SequentialTaskList<E>,
    /// ElementSelectorFlags that need to be applied after the traversal is
    /// complete. This map is used in cases where the matching algorithm needs
    /// to set flags on elements it doesn't have exclusive access to (i.e. other
    /// than the current element).
    pub selector_flags: SelectorFlagsMap<E>,
    /// Statistics about the traversal.
    pub statistics: TraversalStatistics,
    /// The struct used to compute and cache font metrics from style
    /// for evaluation of the font-relative em/ch units and font-size
    pub font_metrics_provider: E::FontMetricsProvider,
    /// A checker used to ensure that parallel.rs does not recurse indefinitely
    /// even on arbitrarily deep trees.  See Gecko bug 1376883.
    pub stack_limit_checker: StackLimitChecker,
    /// A cache for nth-index-like selectors.
    pub nth_index_cache: NthIndexCache,
}

impl<E: TElement> ThreadLocalStyleContext<E> {
    /// Creates a new `ThreadLocalStyleContext` from a shared one.
    #[cfg(feature = "servo")]
    pub fn new(shared: &SharedStyleContext) -> Self {
        ThreadLocalStyleContext {
            sharing_cache: StyleSharingCache::new(),
            rule_cache: RuleCache::new(),
            bloom_filter: StyleBloom::new(),
            new_animations_sender: shared.local_context_creation_data.lock().unwrap().new_animations_sender.clone(),
            tasks: SequentialTaskList(Vec::new()),
            selector_flags: SelectorFlagsMap::new(),
            statistics: TraversalStatistics::default(),
            font_metrics_provider: E::FontMetricsProvider::create_from(shared),
            stack_limit_checker: StackLimitChecker::new(
                (STYLE_THREAD_STACK_SIZE_KB - STACK_SAFETY_MARGIN_KB) * 1024),
            nth_index_cache: NthIndexCache::default(),
        }
    }

    #[cfg(feature = "gecko")]
    /// Creates a new `ThreadLocalStyleContext` from a shared one.
    pub fn new(shared: &SharedStyleContext) -> Self {
        ThreadLocalStyleContext {
            sharing_cache: StyleSharingCache::new(),
            rule_cache: RuleCache::new(),
            bloom_filter: StyleBloom::new(),
            tasks: SequentialTaskList(Vec::new()),
            selector_flags: SelectorFlagsMap::new(),
            statistics: TraversalStatistics::default(),
            font_metrics_provider: E::FontMetricsProvider::create_from(shared),
            stack_limit_checker: StackLimitChecker::new(
                (STYLE_THREAD_STACK_SIZE_KB - STACK_SAFETY_MARGIN_KB) * 1024),
            nth_index_cache: NthIndexCache::default(),
        }
    }
}

impl<E: TElement> Drop for ThreadLocalStyleContext<E> {
    fn drop(&mut self) {
        debug_assert!(thread_state::get() == ThreadState::LAYOUT);

        // Apply any slow selector flags that need to be set on parents.
        self.selector_flags.apply_flags();
    }
}

/// A `StyleContext` is just a simple container for a immutable reference to a
/// shared style context, and a mutable reference to a local one.
pub struct StyleContext<'a, E: TElement + 'a> {
    /// The shared style context reference.
    pub shared: &'a SharedStyleContext<'a>,
    /// The thread-local style context (mutable) reference.
    pub thread_local: &'a mut ThreadLocalStyleContext<E>,
}

/// A registered painter
#[cfg(feature = "servo")]
pub trait RegisteredSpeculativePainter: SpeculativePainter {
    /// The name it was registered with
    fn name(&self) -> Atom;
    /// The properties it was registered with
    fn properties(&self) -> &FnvHashMap<Atom, PropertyId>;
}

/// A set of registered painters
#[cfg(feature = "servo")]
pub trait RegisteredSpeculativePainters: Sync {
    /// Look up a speculative painter
    fn get(&self, name: &Atom) -> Option<&RegisteredSpeculativePainter>;
}
