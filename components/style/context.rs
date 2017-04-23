/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The context within which style is calculated.
#![deny(missing_docs)]

use animation::{Animation, PropertyAnimation};
use app_units::Au;
use bit_vec::BitVec;
use bloom::StyleBloom;
use cache::LRUCache;
use data::ElementData;
use dom::{OpaqueNode, TNode, TElement, SendElement};
use error_reporting::ParseErrorReporter;
use euclid::Size2D;
use fnv::FnvHashMap;
use font_metrics::FontMetricsProvider;
#[cfg(feature = "gecko")] use gecko_bindings::structs;
use matching::StyleSharingCandidateCache;
use parking_lot::RwLock;
#[cfg(feature = "gecko")] use properties::ComputedValues;
#[cfg(feature = "gecko")] use selector_parser::PseudoElement;
use selectors::matching::ElementSelectorFlags;
#[cfg(feature = "servo")] use servo_config::opts;
use shared_lock::StylesheetGuards;
use std::collections::HashMap;
#[cfg(not(feature = "servo"))] use std::env;
use std::fmt;
use std::ops::Add;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use stylist::Stylist;
use thread_state;
use time;
use timer::Timer;
use traversal::{DomTraversal, TraversalFlags};

/// This structure is used to create a local style context from a shared one.
pub struct ThreadLocalStyleContextCreationInfo {
    new_animations_sender: Sender<Animation>,
}

impl ThreadLocalStyleContextCreationInfo {
    /// Trivially constructs a `ThreadLocalStyleContextCreationInfo`.
    pub fn new(animations_sender: Sender<Animation>) -> Self {
        ThreadLocalStyleContextCreationInfo {
            new_animations_sender: animations_sender,
        }
    }
}

/// Which quirks mode is this document in.
///
/// See: https://quirks.spec.whatwg.org/
#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum QuirksMode {
    /// Quirks mode.
    Quirks,
    /// Limited quirks mode.
    LimitedQuirks,
    /// No quirks mode.
    NoQuirks,
}

/// A global options structure for the style system. We use this instead of
/// opts to abstract across Gecko and Servo.
#[derive(Clone)]
pub struct StyleSystemOptions {
    /// Whether the style sharing cache is disabled.
    pub disable_style_sharing_cache: bool,
    /// Whether we should dump statistics about the style system.
    pub dump_style_statistics: bool,
}

#[cfg(feature = "gecko")]
fn get_env(name: &str) -> bool {
    match env::var(name) {
        Ok(s) => !s.is_empty(),
        Err(_) => false,
    }
}

impl Default for StyleSystemOptions {
    #[cfg(feature = "servo")]
    fn default() -> Self {
        StyleSystemOptions {
            disable_style_sharing_cache: opts::get().disable_share_style_cache,
            dump_style_statistics: opts::get().style_sharing_stats,
        }
    }

    #[cfg(feature = "gecko")]
    fn default() -> Self {
        StyleSystemOptions {
            disable_style_sharing_cache:
                // Disable the style sharing cache on opt builds until
                // bug 1358693 is fixed, but keep it on debug builds to make
                // sure we don't introduce correctness bugs.
                if cfg!(debug_assertions) { get_env("DISABLE_STYLE_SHARING_CACHE") } else { true },
            dump_style_statistics: get_env("DUMP_STYLE_STATISTICS"),
        }
    }
}

/// A shared style context.
///
/// There's exactly one of these during a given restyle traversal, and it's
/// shared among the worker threads.
pub struct SharedStyleContext<'a> {
    /// The CSS selector stylist.
    pub stylist: Arc<Stylist>,

    /// Configuration options.
    pub options: StyleSystemOptions,

    /// Guards for pre-acquired locks
    pub guards: StylesheetGuards<'a>,

    /// The animations that are currently running.
    pub running_animations: Arc<RwLock<HashMap<OpaqueNode, Vec<Animation>>>>,

    /// The list of animations that have expired since the last style recalculation.
    pub expired_animations: Arc<RwLock<HashMap<OpaqueNode, Vec<Animation>>>>,

    ///The CSS error reporter for all CSS loaded in this layout thread
    pub error_reporter: Box<ParseErrorReporter>,

    /// Data needed to create the thread-local style context from the shared one.
    pub local_context_creation_data: Mutex<ThreadLocalStyleContextCreationInfo>,

    /// The current timer for transitions and animations. This is needed to test
    /// them.
    pub timer: Timer,

    /// The QuirksMode state which the document needs to be rendered with
    pub quirks_mode: QuirksMode,

    /// Flags controlling how we traverse the tree.
    pub traversal_flags: TraversalFlags,
}

impl<'a> SharedStyleContext<'a> {
    /// Return a suitable viewport size in order to be used for viewport units.
    pub fn viewport_size(&self) -> Size2D<Au> {
        self.stylist.device.au_viewport_size()
    }
}

/// Information about the current element being processed. We group this together
/// into a single struct within ThreadLocalStyleContext so that we can instantiate
/// and destroy it easily at the beginning and end of element processing.
pub struct CurrentElementInfo {
    /// The element being processed. Currently we use an OpaqueNode since we only
    /// use this for identity checks, but we could use SendElement if there were
    /// a good reason to.
    element: OpaqueNode,
    /// Whether the element is being styled for the first time.
    is_initial_style: bool,
    /// Lazy cache of the result of matching the current element against the
    /// revalidation selectors.
    pub revalidation_match_results: Option<BitVec>,
    /// A Vec of possibly expired animations. Used only by Servo.
    #[allow(dead_code)]
    pub possibly_expired_animations: Vec<PropertyAnimation>,
}

/// Statistics gathered during the traversal. We gather statistics on each thread
/// and then combine them after the threads join via the Add implementation below.
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
}

/// Implementation of Add to aggregate statistics across different threads.
impl<'a> Add for &'a TraversalStatistics {
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
            selectors: 0,
            revalidation_selectors: 0,
            dependency_selectors: 0,
            declarations: 0,
            stylist_rebuilds: 0,
            traversal_time_ms: 0.0,
            is_parallel: None,
        }
    }
}

/// Format the statistics in a way that the performance test harness understands.
/// See https://bugzilla.mozilla.org/show_bug.cgi?id=1331856#c2
impl fmt::Display for TraversalStatistics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        debug_assert!(self.traversal_time_ms != 0.0, "should have set traversal time");
        try!(writeln!(f, "[PERF] perf block start"));
        try!(writeln!(f, "[PERF],traversal,{}", if self.is_parallel.unwrap() {
            "parallel"
        } else {
            "sequential"
        }));
        try!(writeln!(f, "[PERF],elements_traversed,{}", self.elements_traversed));
        try!(writeln!(f, "[PERF],elements_styled,{}", self.elements_styled));
        try!(writeln!(f, "[PERF],elements_matched,{}", self.elements_matched));
        try!(writeln!(f, "[PERF],styles_shared,{}", self.styles_shared));
        try!(writeln!(f, "[PERF],selectors,{}", self.selectors));
        try!(writeln!(f, "[PERF],revalidation_selectors,{}", self.revalidation_selectors));
        try!(writeln!(f, "[PERF],dependency_selectors,{}", self.dependency_selectors));
        try!(writeln!(f, "[PERF],declarations,{}", self.declarations));
        try!(writeln!(f, "[PERF],stylist_rebuilds,{}", self.stylist_rebuilds));
        try!(writeln!(f, "[PERF],traversal_time_ms,{}", self.traversal_time_ms));
        writeln!(f, "[PERF] perf block end")
    }
}

impl TraversalStatistics {
    /// Computes the traversal time given the start time in seconds.
    pub fn finish<E, D>(&mut self, traversal: &D, start: f64)
        where E: TElement,
              D: DomTraversal<E>,
    {
        self.is_parallel = Some(traversal.is_parallel());
        self.traversal_time_ms = (time::precise_time_s() - start) * 1000.0;
        self.selectors = traversal.shared_context().stylist.num_selectors() as u32;
        self.revalidation_selectors = traversal.shared_context().stylist.num_revalidation_selectors() as u32;
        self.dependency_selectors = traversal.shared_context().stylist.num_dependencies() as u32;
        self.declarations = traversal.shared_context().stylist.num_declarations() as u32;
        self.stylist_rebuilds = traversal.shared_context().stylist.num_rebuilds() as u32;
    }

    /// Returns whether this traversal is 'large' in order to avoid console spam
    /// from lots of tiny traversals.
    pub fn is_large_traversal(&self) -> bool {
        self.elements_traversed >= 50
    }
}

#[cfg(feature = "gecko")]
bitflags! {
    /// Represents which tasks are performed in a SequentialTask of UpdateAnimations.
    pub flags UpdateAnimationsTasks: u8 {
        /// Update CSS Animations.
        const CSS_ANIMATIONS = structs::UpdateAnimationsTasks_CSSAnimations,
        /// Update CSS Transitions.
        const CSS_TRANSITIONS = structs::UpdateAnimationsTasks_CSSTransitions,
        /// Update effect properties.
        const EFFECT_PROPERTIES = structs::UpdateAnimationsTasks_EffectProperties,
        /// Update animation cacade results for animations running on the compositor.
        const CASCADE_RESULTS = structs::UpdateAnimationsTasks_CascadeResults,
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
        /// The target element.
        el: SendElement<E>,
        /// The target pseudo element.
        pseudo: Option<PseudoElement>,
        /// The before-change style for transitions. We use before-change style as the initial
        /// value of its Keyframe. Required if |tasks| includes CSSTransitions.
        before_change_style: Option<Arc<ComputedValues>>,
        /// The tasks which are performed in this SequentialTask.
        tasks: UpdateAnimationsTasks
    },
}

impl<E: TElement> SequentialTask<E> {
    /// Executes this task.
    pub fn execute(self) {
        use self::SequentialTask::*;
        debug_assert!(thread_state::get() == thread_state::LAYOUT);
        match self {
            Unused(_) => unreachable!(),
            #[cfg(feature = "gecko")]
            UpdateAnimations { el, pseudo, before_change_style, tasks } => {
                unsafe { el.update_animations(pseudo.as_ref(), before_change_style, tasks) };
            }
        }
    }

    /// Creates a task to update various animation-related state on
    /// a given (pseudo-)element.
    #[cfg(feature = "gecko")]
    pub fn update_animations(el: E,
                             pseudo: Option<PseudoElement>,
                             before_change_style: Option<Arc<ComputedValues>>,
                             tasks: UpdateAnimationsTasks) -> Self {
        use self::SequentialTask::*;
        UpdateAnimations { el: unsafe { SendElement::new(el) },
                           pseudo: pseudo,
                           before_change_style: before_change_style,
                           tasks: tasks }
    }
}

/// Map from Elements to ElementSelectorFlags. Used to defer applying selector
/// flags until after the traversal.
pub struct SelectorFlagsMap<E: TElement> {
    /// The hashmap storing the flags to apply.
    map: FnvHashMap<SendElement<E>, ElementSelectorFlags>,
    /// An LRU cache to avoid hashmap lookups, which can be slow if the map
    /// gets big.
    cache: LRUCache<(SendElement<E>, ElementSelectorFlags)>,
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
            cache: LRUCache::new(4),
        }
    }

    /// Inserts some flags into the map for a given element.
    pub fn insert_flags(&mut self, element: E, flags: ElementSelectorFlags) {
        let el = unsafe { SendElement::new(element) };
        // Check the cache. If the flags have already been noted, we're done.
        if self.cache.iter().find(|x| x.0 == el)
               .map_or(ElementSelectorFlags::empty(), |x| x.1)
               .contains(flags) {
            return;
        }

        let f = self.map.entry(el).or_insert(ElementSelectorFlags::empty());
        *f |= flags;

        // Insert into the cache. We don't worry about duplicate entries,
        // which lets us avoid reshuffling.
        self.cache.insert((unsafe { SendElement::new(element) }, *f))
    }

    /// Applies the flags. Must be called on the main thread.
    pub fn apply_flags(&mut self) {
        debug_assert!(thread_state::get() == thread_state::LAYOUT);
        for (el, flags) in self.map.drain() {
            unsafe { el.set_selector_flags(flags); }
        }
    }
}

/// A thread-local style context.
///
/// This context contains data that needs to be used during restyling, but is
/// not required to be unique among worker threads, so we create one per worker
/// thread in order to be able to mutate it without locking.
pub struct ThreadLocalStyleContext<E: TElement> {
    /// A cache to share style among siblings.
    pub style_sharing_candidate_cache: StyleSharingCandidateCache<E>,
    /// The bloom filter used to fast-reject selector-matching.
    pub bloom_filter: StyleBloom<E>,
    /// A channel on which new animations that have been triggered by style
    /// recalculation can be sent.
    pub new_animations_sender: Sender<Animation>,
    /// A set of tasks to be run (on the parent thread) in sequential mode after
    /// the rest of the styling is complete. This is useful for infrequently-needed
    /// non-threadsafe operations.
    pub tasks: Vec<SequentialTask<E>>,
    /// ElementSelectorFlags that need to be applied after the traversal is
    /// complete. This map is used in cases where the matching algorithm needs
    /// to set flags on elements it doesn't have exclusive access to (i.e. other
    /// than the current element).
    pub selector_flags: SelectorFlagsMap<E>,
    /// Statistics about the traversal.
    pub statistics: TraversalStatistics,
    /// Information related to the current element, non-None during processing.
    pub current_element_info: Option<CurrentElementInfo>,
    /// The struct used to compute and cache font metrics from style
    /// for evaluation of the font-relative em/ch units and font-size
    pub font_metrics_provider: E::FontMetricsProvider,
}

impl<E: TElement> ThreadLocalStyleContext<E> {
    /// Creates a new `ThreadLocalStyleContext` from a shared one.
    pub fn new(shared: &SharedStyleContext) -> Self {
        ThreadLocalStyleContext {
            style_sharing_candidate_cache: StyleSharingCandidateCache::new(),
            bloom_filter: StyleBloom::new(),
            new_animations_sender: shared.local_context_creation_data.lock().unwrap().new_animations_sender.clone(),
            tasks: Vec::new(),
            selector_flags: SelectorFlagsMap::new(),
            statistics: TraversalStatistics::default(),
            current_element_info: None,
            font_metrics_provider: E::FontMetricsProvider::create_from(shared),
        }
    }

    /// Notes when the style system starts traversing an element.
    pub fn begin_element(&mut self, element: E, data: &ElementData) {
        debug_assert!(self.current_element_info.is_none());
        self.current_element_info = Some(CurrentElementInfo {
            element: element.as_node().opaque(),
            is_initial_style: !data.has_styles(),
            revalidation_match_results: None,
            possibly_expired_animations: Vec::new(),
        });
    }

    /// Notes when the style system finishes traversing an element.
    pub fn end_element(&mut self, element: E) {
        debug_assert!(self.current_element_info.is_some());
        debug_assert!(self.current_element_info.as_ref().unwrap().element ==
                      element.as_node().opaque());
        self.current_element_info = None;
    }

    /// Returns true if the current element being traversed is being styled for
    /// the first time.
    ///
    /// Panics if called while no element is being traversed.
    pub fn is_initial_style(&self) -> bool {
        self.current_element_info.as_ref().unwrap().is_initial_style
    }
}

impl<E: TElement> Drop for ThreadLocalStyleContext<E> {
    fn drop(&mut self) {
        debug_assert!(self.current_element_info.is_none());
        debug_assert!(thread_state::get() == thread_state::LAYOUT);

        // Apply any slow selector flags that need to be set on parents.
        self.selector_flags.apply_flags();

        // Execute any enqueued sequential tasks.
        for task in self.tasks.drain(..) {
            task.execute();
        }
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

/// Why we're doing reflow.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ReflowGoal {
    /// We're reflowing in order to send a display list to the screen.
    ForDisplay,
    /// We're reflowing in order to satisfy a script query. No display list will be created.
    ForScriptQuery,
}
