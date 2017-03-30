/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The context within which style is calculated.
#![deny(missing_docs)]

use animation::Animation;
use app_units::Au;
use bloom::StyleBloom;
use data::ElementData;
use dom::{OpaqueNode, TNode, TElement, SendElement};
use error_reporting::ParseErrorReporter;
use euclid::Size2D;
#[cfg(feature = "gecko")] use gecko_bindings::structs;
use matching::StyleSharingCandidateCache;
use parking_lot::RwLock;
#[cfg(feature = "gecko")] use selector_parser::PseudoElement;
use selectors::matching::ElementSelectorFlags;
use servo_config::opts;
use shared_lock::StylesheetGuards;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::ops::Add;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use stylist::Stylist;
use thread_state;
use time;
use timer::Timer;
use traversal::DomTraversal;

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

/// A shared style context.
///
/// There's exactly one of these during a given restyle traversal, and it's
/// shared among the worker threads.
pub struct SharedStyleContext<'a> {
    /// The CSS selector stylist.
    pub stylist: Arc<Stylist>,

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

    /// True if the traversal is processing only animation restyles.
    pub animation_only_restyle: bool,
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
struct CurrentElementInfo {
    /// The element being processed. Currently we use an OpaqueNode since we only
    /// use this for identity checks, but we could use SendElement if there were
    /// a good reason to.
    element: OpaqueNode,
    /// Whether the element is being styled for the first time.
    is_initial_style: bool,
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
        TraversalStatistics {
            elements_traversed: self.elements_traversed + other.elements_traversed,
            elements_styled: self.elements_styled + other.elements_styled,
            elements_matched: self.elements_matched + other.elements_matched,
            styles_shared: self.styles_shared + other.styles_shared,
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
        try!(writeln!(f, "[PERF],traversal_time_ms,{}", self.traversal_time_ms));
        writeln!(f, "[PERF] perf block end")
    }
}

lazy_static! {
    /// Whether to dump style statistics, computed statically. We use an environmental
    /// variable so that this is easy to set for Gecko builds, and matches the
    /// mechanism we use to dump statistics on the Gecko style system.
    static ref DUMP_STYLE_STATISTICS: bool = {
        match env::var("DUMP_STYLE_STATISTICS") {
            Ok(s) => !s.is_empty(),
            Err(_) => false,
        }
    };
}

impl TraversalStatistics {
    /// Returns whether statistics dumping is enabled.
    pub fn should_dump() -> bool {
        *DUMP_STYLE_STATISTICS || opts::get().style_sharing_stats
    }

    /// Computes the traversal time given the start time in seconds.
    pub fn finish<E, D>(&mut self, traversal: &D, start: f64)
        where E: TElement,
              D: DomTraversal<E>,
    {
        self.is_parallel = Some(traversal.is_parallel());
        self.traversal_time_ms = (time::precise_time_s() - start) * 1000.0;
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
    /// Sets selector flags. This is used when we need to set flags on an
    /// element that we don't have exclusive access to (i.e. the parent).
    SetSelectorFlags(SendElement<E>, ElementSelectorFlags),

    #[cfg(feature = "gecko")]
    /// Marks that we need to update CSS animations, update effect properties of
    /// any type of animations after the normal traversal.
    UpdateAnimations(SendElement<E>, Option<PseudoElement>, UpdateAnimationsTasks),
}

impl<E: TElement> SequentialTask<E> {
    /// Executes this task.
    pub fn execute(self) {
        use self::SequentialTask::*;
        debug_assert!(thread_state::get() == thread_state::LAYOUT);
        match self {
            SetSelectorFlags(el, flags) => {
                unsafe { el.set_selector_flags(flags) };
            }
            #[cfg(feature = "gecko")]
            UpdateAnimations(el, pseudo, tasks) => {
                unsafe { el.update_animations(pseudo.as_ref(), tasks) };
            }
        }
    }

    /// Creates a task to set the selector flags on an element.
    pub fn set_selector_flags(el: E, flags: ElementSelectorFlags) -> Self {
        use self::SequentialTask::*;
        SetSelectorFlags(unsafe { SendElement::new(el) }, flags)
    }

    #[cfg(feature = "gecko")]
    /// Creates a task to update various animation state on a given (pseudo-)element.
    pub fn update_animations(el: E, pseudo: Option<PseudoElement>,
                             tasks: UpdateAnimationsTasks) -> Self {
        use self::SequentialTask::*;
        UpdateAnimations(unsafe { SendElement::new(el) }, pseudo, tasks)
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
    /// Statistics about the traversal.
    pub statistics: TraversalStatistics,
    /// Information related to the current element, non-None during processing.
    current_element_info: Option<CurrentElementInfo>,
}

impl<E: TElement> ThreadLocalStyleContext<E> {
    /// Creates a new `ThreadLocalStyleContext` from a shared one.
    pub fn new(shared: &SharedStyleContext) -> Self {
        ThreadLocalStyleContext {
            style_sharing_candidate_cache: StyleSharingCandidateCache::new(),
            bloom_filter: StyleBloom::new(),
            new_animations_sender: shared.local_context_creation_data.lock().unwrap().new_animations_sender.clone(),
            tasks: Vec::new(),
            statistics: TraversalStatistics::default(),
            current_element_info: None,
        }
    }

    /// Notes when the style system starts traversing an element.
    pub fn begin_element(&mut self, element: E, data: &ElementData) {
        debug_assert!(self.current_element_info.is_none());
        self.current_element_info = Some(CurrentElementInfo {
            element: element.as_node().opaque(),
            is_initial_style: !data.has_styles(),
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

        // Execute any enqueued sequential tasks.
        debug_assert!(thread_state::get() == thread_state::LAYOUT);
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
