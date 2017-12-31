/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{ParseErrorKind, Parser, ParserInput};
use cssparser::ToCss as ParserToCss;
use env_logger::LogBuilder;
use malloc_size_of::MallocSizeOfOps;
use selectors::{Element, NthIndexCache};
use selectors::matching::{MatchingContext, MatchingMode, matches_selector};
use servo_arc::{Arc, ArcBorrow, RawOffsetArc};
use std::cell::RefCell;
use std::env;
use std::fmt::Write;
use std::iter;
use std::mem;
use std::ptr;
use style::applicable_declarations::ApplicableDeclarationBlock;
use style::context::{CascadeInputs, QuirksMode, SharedStyleContext, StyleContext};
use style::context::ThreadLocalStyleContext;
use style::counter_style;
use style::data::{ElementStyles, self};
use style::dom::{ShowSubtreeData, TDocument, TElement, TNode};
use style::driver;
use style::element_state::{DocumentState, ElementState};
use style::error_reporting::{ContextualParseError, NullReporter, ParseErrorReporter};
use style::font_metrics::{FontMetricsProvider, get_metrics_provider_for_product};
use style::gecko::data::{GeckoStyleSheet, PerDocumentStyleData, PerDocumentStyleDataImpl};
use style::gecko::global_style_data::{GLOBAL_STYLE_DATA, GlobalStyleData, STYLE_THREAD_POOL};
use style::gecko::restyle_damage::GeckoRestyleDamage;
use style::gecko::selector_parser::PseudoElement;
use style::gecko::traversal::RecalcStyleOnly;
use style::gecko::wrapper::{GeckoElement, GeckoNode};
use style::gecko_bindings::bindings;
use style::gecko_bindings::bindings::{RawGeckoElementBorrowed, RawGeckoElementBorrowedOrNull, RawGeckoNodeBorrowed};
use style::gecko_bindings::bindings::{RawGeckoKeyframeListBorrowed, RawGeckoKeyframeListBorrowedMut};
use style::gecko_bindings::bindings::{RawServoDeclarationBlockBorrowed, RawServoDeclarationBlockStrong};
use style::gecko_bindings::bindings::{RawServoDocumentRule, RawServoDocumentRuleBorrowed};
use style::gecko_bindings::bindings::{RawServoFontFeatureValuesRule, RawServoFontFeatureValuesRuleBorrowed};
use style::gecko_bindings::bindings::{RawServoImportRule, RawServoImportRuleBorrowed};
use style::gecko_bindings::bindings::{RawServoKeyframe, RawServoKeyframeBorrowed, RawServoKeyframeStrong};
use style::gecko_bindings::bindings::{RawServoKeyframesRule, RawServoKeyframesRuleBorrowed};
use style::gecko_bindings::bindings::{RawServoMediaListBorrowed, RawServoMediaListStrong};
use style::gecko_bindings::bindings::{RawServoMediaRule, RawServoMediaRuleBorrowed};
use style::gecko_bindings::bindings::{RawServoNamespaceRule, RawServoNamespaceRuleBorrowed};
use style::gecko_bindings::bindings::{RawServoPageRule, RawServoPageRuleBorrowed};
use style::gecko_bindings::bindings::{RawServoSelectorListBorrowed, RawServoSelectorListOwned};
use style::gecko_bindings::bindings::{RawServoSourceSizeListBorrowedOrNull, RawServoSourceSizeListOwned};
use style::gecko_bindings::bindings::{RawServoStyleSetBorrowed, RawServoStyleSetBorrowedOrNull, RawServoStyleSetOwned};
use style::gecko_bindings::bindings::{RawServoStyleSheetContentsBorrowed, ServoComputedDataBorrowed};
use style::gecko_bindings::bindings::{RawServoStyleSheetContentsStrong, ServoStyleContextBorrowed};
use style::gecko_bindings::bindings::{RawServoSupportsRule, RawServoSupportsRuleBorrowed};
use style::gecko_bindings::bindings::{ServoCssRulesBorrowed, ServoCssRulesStrong};
use style::gecko_bindings::bindings::{nsACString, nsAString, nsCSSPropertyIDSetBorrowedMut};
use style::gecko_bindings::bindings::Gecko_AddPropertyToSet;
use style::gecko_bindings::bindings::Gecko_AppendPropertyValuePair;
use style::gecko_bindings::bindings::Gecko_ConstructFontFeatureValueSet;
use style::gecko_bindings::bindings::Gecko_GetOrCreateFinalKeyframe;
use style::gecko_bindings::bindings::Gecko_GetOrCreateInitialKeyframe;
use style::gecko_bindings::bindings::Gecko_GetOrCreateKeyframeAtStart;
use style::gecko_bindings::bindings::Gecko_HaveSeenPtr;
use style::gecko_bindings::bindings::Gecko_NewNoneTransform;
use style::gecko_bindings::bindings::RawGeckoAnimationPropertySegmentBorrowed;
use style::gecko_bindings::bindings::RawGeckoCSSPropertyIDListBorrowed;
use style::gecko_bindings::bindings::RawGeckoComputedKeyframeValuesListBorrowedMut;
use style::gecko_bindings::bindings::RawGeckoComputedTimingBorrowed;
use style::gecko_bindings::bindings::RawGeckoFontFaceRuleListBorrowedMut;
use style::gecko_bindings::bindings::RawGeckoServoAnimationValueListBorrowed;
use style::gecko_bindings::bindings::RawGeckoServoAnimationValueListBorrowedMut;
use style::gecko_bindings::bindings::RawGeckoServoStyleRuleListBorrowedMut;
use style::gecko_bindings::bindings::RawServoAnimationValueBorrowed;
use style::gecko_bindings::bindings::RawServoAnimationValueBorrowedOrNull;
use style::gecko_bindings::bindings::RawServoAnimationValueMapBorrowedMut;
use style::gecko_bindings::bindings::RawServoAnimationValueStrong;
use style::gecko_bindings::bindings::RawServoAnimationValueTableBorrowed;
use style::gecko_bindings::bindings::RawServoDeclarationBlockBorrowedOrNull;
use style::gecko_bindings::bindings::RawServoStyleRuleBorrowed;
use style::gecko_bindings::bindings::RawServoStyleSet;
use style::gecko_bindings::bindings::ServoStyleContextBorrowedOrNull;
use style::gecko_bindings::bindings::nsCSSValueBorrowedMut;
use style::gecko_bindings::bindings::nsTArrayBorrowed_uintptr_t;
use style::gecko_bindings::bindings::nsTimingFunctionBorrowed;
use style::gecko_bindings::bindings::nsTimingFunctionBorrowedMut;
use style::gecko_bindings::structs;
use style::gecko_bindings::structs::{CallerType, CSSPseudoElementType, CompositeOperation};
use style::gecko_bindings::structs::{Loader, LoaderReusableStyleSheets};
use style::gecko_bindings::structs::{RawServoStyleRule, ServoStyleContextStrong, RustString};
use style::gecko_bindings::structs::{ServoStyleSheet, SheetParsingMode, nsAtom, nsCSSPropertyID};
use style::gecko_bindings::structs::{nsCSSFontDesc, nsCSSFontFaceRule, nsCSSCounterStyleRule};
use style::gecko_bindings::structs::{nsRestyleHint, nsChangeHint, PropertyValuePair};
use style::gecko_bindings::structs::AtomArray;
use style::gecko_bindings::structs::IterationCompositeOperation;
use style::gecko_bindings::structs::MallocSizeOf as GeckoMallocSizeOf;
use style::gecko_bindings::structs::OriginFlags;
use style::gecko_bindings::structs::OriginFlags_Author;
use style::gecko_bindings::structs::OriginFlags_User;
use style::gecko_bindings::structs::OriginFlags_UserAgent;
use style::gecko_bindings::structs::RawGeckoGfxMatrix4x4;
use style::gecko_bindings::structs::RawGeckoPresContextOwned;
use style::gecko_bindings::structs::RawServoSelectorList;
use style::gecko_bindings::structs::RawServoSourceSizeList;
use style::gecko_bindings::structs::SeenPtrs;
use style::gecko_bindings::structs::ServoElementSnapshotTable;
use style::gecko_bindings::structs::ServoStyleSetSizes;
use style::gecko_bindings::structs::ServoTraversalFlags;
use style::gecko_bindings::structs::StyleRuleInclusion;
use style::gecko_bindings::structs::URLExtraData;
use style::gecko_bindings::structs::gfxFontFeatureValueSet;
use style::gecko_bindings::structs::nsCSSCounterDesc;
use style::gecko_bindings::structs::nsCSSValue;
use style::gecko_bindings::structs::nsCSSValueSharedList;
use style::gecko_bindings::structs::nsCompatibility;
use style::gecko_bindings::structs::nsIDocument;
use style::gecko_bindings::structs::nsStyleTransformMatrix::MatrixTransformOperator;
use style::gecko_bindings::structs::nsTArray;
use style::gecko_bindings::structs::nsresult;
use style::gecko_bindings::sugar::ownership::{FFIArcHelpers, HasFFI, HasArcFFI};
use style::gecko_bindings::sugar::ownership::{HasSimpleFFI, Strong};
use style::gecko_bindings::sugar::refptr::RefPtr;
use style::gecko_properties;
use style::invalidation::element::restyle_hints;
use style::media_queries::{Device, MediaList, parse_media_query_list};
use style::parser::{Parse, ParserContext, self};
use style::properties::{CascadeFlags, ComputedValues, DeclarationSource, Importance};
use style::properties::{LonghandId, LonghandIdSet, PropertyDeclaration, PropertyDeclarationBlock, PropertyId};
use style::properties::{PropertyDeclarationId, ShorthandId};
use style::properties::{SourcePropertyDeclaration, StyleBuilder};
use style::properties::{parse_one_declaration_into, parse_style_attribute};
use style::properties::animated_properties::AnimationValue;
use style::properties::animated_properties::compare_property_priority;
use style::rule_cache::RuleCacheConditions;
use style::rule_tree::{CascadeLevel, StrongRuleNode, StyleSource};
use style::selector_parser::{PseudoElementCascadeType, SelectorImpl};
use style::shared_lock::{SharedRwLockReadGuard, StylesheetGuards, ToCssWithGuard, Locked};
use style::string_cache::{Atom, WeakAtom};
use style::style_adjuster::StyleAdjuster;
use style::stylesheets::{CssRule, CssRules, CssRuleType, CssRulesHelpers, DocumentRule};
use style::stylesheets::{FontFeatureValuesRule, ImportRule, KeyframesRule, MediaRule};
use style::stylesheets::{NamespaceRule, Origin, OriginSet, PageRule, StyleRule};
use style::stylesheets::{StylesheetContents, SupportsRule};
use style::stylesheets::StylesheetLoader as StyleStylesheetLoader;
use style::stylesheets::keyframes_rule::{Keyframe, KeyframeSelector, KeyframesStepValue};
use style::stylesheets::supports_rule::parse_condition_or_declaration;
use style::stylist::{add_size_of_ua_cache, RuleInclusion, Stylist};
use style::thread_state;
use style::timer::Timer;
use style::traversal::DomTraversal;
use style::traversal::resolve_style;
use style::traversal_flags::{self, TraversalFlags};
use style::values::{CustomIdent, KeyframesName};
use style::values::animated::{Animate, Procedure, ToAnimatedZero};
use style::values::computed::{Context, ToComputedValue};
use style::values::distance::ComputeSquaredDistance;
use style::values::specified;
use style::values::specified::gecko::IntersectionObserverRootMargin;
use style::values::specified::source_size_list::SourceSizeList;
use style_traits::{ParsingMode, StyleParseErrorKind, ToCss};
use super::error_reporter::ErrorReporter;
use super::stylesheet_loader::StylesheetLoader;

/*
 * For Gecko->Servo function calls, we need to redeclare the same signature that was declared in
 * the C header in Gecko. In order to catch accidental mismatches, we run rust-bindgen against
 * those signatures as well, giving us a second declaration of all the Servo_* functions in this
 * crate. If there's a mismatch, LLVM will assert and abort, which is a rather awful thing to
 * depend on but good enough for our purposes.
 */

// A dummy url data for where we don't pass url data in.
// We need to get rid of this sooner than later.
static mut DUMMY_URL_DATA: *mut URLExtraData = 0 as *mut URLExtraData;

#[no_mangle]
pub extern "C" fn Servo_Initialize(dummy_url_data: *mut URLExtraData) {
    use style::gecko_bindings::sugar::origin_flags;

    // Initialize logging.
    let mut builder = LogBuilder::new();
    let default_level = if cfg!(debug_assertions) { "warn" } else { "error" };
    match env::var("RUST_LOG") {
      Ok(v) => builder.parse(&v).init().unwrap(),
      _ => builder.parse(default_level).init().unwrap(),
    };

    // Pretend that we're a Servo Layout thread, to make some assertions happy.
    thread_state::initialize(thread_state::ThreadState::LAYOUT);

    // Perform some debug-only runtime assertions.
    restyle_hints::assert_restyle_hints_match();
    origin_flags::assert_flags_match();
    parser::assert_parsing_mode_match();
    traversal_flags::assert_traversal_flags_match();
    specified::font::assert_variant_east_asian_matches();
    specified::font::assert_variant_ligatures_matches();
    specified::box_::assert_touch_action_matches();

    // Initialize the dummy url data
    unsafe { DUMMY_URL_DATA = dummy_url_data; }
}

#[no_mangle]
pub extern "C" fn Servo_InitializeCooperativeThread() {
    // Pretend that we're a Servo Layout thread to make some assertions happy.
    thread_state::initialize(thread_state::ThreadState::LAYOUT);
}

#[no_mangle]
pub extern "C" fn Servo_Shutdown() {
    // The dummy url will be released after shutdown, so clear the
    // reference to avoid use-after-free.
    unsafe { DUMMY_URL_DATA = ptr::null_mut(); }
    Stylist::shutdown();
}

unsafe fn dummy_url_data() -> &'static RefPtr<URLExtraData> {
    RefPtr::from_ptr_ref(&DUMMY_URL_DATA)
}

fn create_shared_context<'a>(
    global_style_data: &GlobalStyleData,
    guard: &'a SharedRwLockReadGuard,
    per_doc_data: &'a PerDocumentStyleDataImpl,
    traversal_flags: TraversalFlags,
    snapshot_map: &'a ServoElementSnapshotTable,
) -> SharedStyleContext<'a> {
    SharedStyleContext {
        stylist: &per_doc_data.stylist,
        visited_styles_enabled: per_doc_data.visited_styles_enabled(),
        options: global_style_data.options.clone(),
        guards: StylesheetGuards::same(guard),
        timer: Timer::new(),
        traversal_flags,
        snapshot_map,
    }
}

fn traverse_subtree(
    element: GeckoElement,
    global_style_data: &GlobalStyleData,
    per_doc_data: &PerDocumentStyleDataImpl,
    guard: &SharedRwLockReadGuard,
    traversal_flags: TraversalFlags,
    snapshots: &ServoElementSnapshotTable,
) {
    let shared_style_context = create_shared_context(
        &global_style_data,
        &guard,
        &per_doc_data,
        traversal_flags,
        snapshots,
    );

    let token = RecalcStyleOnly::pre_traverse(
        element,
        &shared_style_context,
    );

    if !token.should_traverse() {
        return;
    }

    debug!("Traversing subtree from {:?}", element);

    let thread_pool_holder = &*STYLE_THREAD_POOL;
    let thread_pool = if traversal_flags.contains(TraversalFlags::ParallelTraversal) {
        thread_pool_holder.style_thread_pool.as_ref()
    } else {
        None
    };

    let is_restyle = element.get_data().is_some();

    let traversal = RecalcStyleOnly::new(shared_style_context);
    let (used_parallel, stats) = driver::traverse_dom(&traversal, token, thread_pool);

    if traversal_flags.contains(TraversalFlags::ParallelTraversal) &&
       !traversal_flags.contains(TraversalFlags::AnimationOnly) &&
       is_restyle && !element.is_native_anonymous() {
       // We turn off parallel traversal for background tabs; this
       // shouldn't count in telemetry. We're also focusing on restyles so
       // we ensure that it's a restyle.
       per_doc_data.record_traversal(used_parallel, stats);
    }
}

/// Traverses the subtree rooted at `root` for restyling.
///
/// Returns whether the root was restyled. Whether anything else was restyled or
/// not can be inferred from the dirty bits in the rest of the tree.
#[no_mangle]
pub extern "C" fn Servo_TraverseSubtree(
    root: RawGeckoElementBorrowed,
    raw_data: RawServoStyleSetBorrowed,
    snapshots: *const ServoElementSnapshotTable,
    raw_flags: ServoTraversalFlags
) -> bool {
    let traversal_flags = TraversalFlags::from_bits_truncate(raw_flags);
    debug_assert!(!snapshots.is_null());

    let element = GeckoElement(root);

    debug!("Servo_TraverseSubtree (flags={:?})", traversal_flags);
    debug!("{:?}", ShowSubtreeData(element.as_node()));

    if cfg!(debug_assertions) {
        if let Some(parent) = element.traversal_parent() {
            let data =
                parent.borrow_data().expect("Styling element with unstyled parent");
            assert!(
                !data.styles.is_display_none(),
                "Styling element with display: none parent"
            );
        }
    }

    let needs_animation_only_restyle =
        element.has_animation_only_dirty_descendants() ||
        element.has_animation_restyle_hints();

    let per_doc_data = PerDocumentStyleData::from_ffi(raw_data).borrow();
    debug_assert!(!per_doc_data.stylist.stylesheets_have_changed());

    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();

    let was_initial_style = element.get_data().is_none();

    if needs_animation_only_restyle {
        debug!("Servo_TraverseSubtree doing animation-only restyle (aodd={})",
               element.has_animation_only_dirty_descendants());
        traverse_subtree(
            element,
            &global_style_data,
            &per_doc_data,
            &guard,
            traversal_flags | TraversalFlags::AnimationOnly,
            unsafe { &*snapshots },
        );
    }

    traverse_subtree(
        element,
        &global_style_data,
        &per_doc_data,
        &guard,
        traversal_flags,
        unsafe { &*snapshots },
    );

    debug!("Servo_TraverseSubtree complete (dd={}, aodd={}, lfcd={}, lfc={}, data={:?})",
           element.has_dirty_descendants(),
           element.has_animation_only_dirty_descendants(),
           element.descendants_need_frames(),
           element.needs_frame(),
           element.borrow_data().unwrap());

    if was_initial_style {
        debug_assert!(!element.borrow_data().unwrap().contains_restyle_data());
        false
    } else {
        let element_was_restyled =
            element.borrow_data().unwrap().contains_restyle_data();
        element_was_restyled
    }
}

/// Checks whether the rule tree has crossed its threshold for unused nodes, and
/// if so, frees them.
#[no_mangle]
pub extern "C" fn Servo_MaybeGCRuleTree(raw_data: RawServoStyleSetBorrowed) {
    let per_doc_data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    unsafe {
        per_doc_data.stylist.rule_tree().maybe_gc();
    }
}

#[no_mangle]
pub extern "C" fn Servo_AnimationValues_Interpolate(
    from: RawServoAnimationValueBorrowed,
    to: RawServoAnimationValueBorrowed,
    progress: f64,
) -> RawServoAnimationValueStrong {
    let from_value = AnimationValue::as_arc(&from);
    let to_value = AnimationValue::as_arc(&to);
    if let Ok(value) = from_value.animate(to_value, Procedure::Interpolate { progress }) {
        Arc::new(value).into_strong()
    } else {
        RawServoAnimationValueStrong::null()
    }
}

#[no_mangle]
pub extern "C" fn Servo_AnimationValues_IsInterpolable(from: RawServoAnimationValueBorrowed,
                                                       to: RawServoAnimationValueBorrowed)
                                                       -> bool {
    let from_value = AnimationValue::as_arc(&from);
    let to_value = AnimationValue::as_arc(&to);
    from_value.animate(to_value, Procedure::Interpolate { progress: 0.5 }).is_ok()
}

#[no_mangle]
pub extern "C" fn Servo_AnimationValues_Add(
    a: RawServoAnimationValueBorrowed,
    b: RawServoAnimationValueBorrowed,
) -> RawServoAnimationValueStrong {
    let a_value = AnimationValue::as_arc(&a);
    let b_value = AnimationValue::as_arc(&b);
    if let Ok(value) = a_value.animate(b_value, Procedure::Add) {
        Arc::new(value).into_strong()
    } else {
        RawServoAnimationValueStrong::null()
    }
}

#[no_mangle]
pub extern "C" fn Servo_AnimationValues_Accumulate(
    a: RawServoAnimationValueBorrowed,
    b: RawServoAnimationValueBorrowed,
    count: u64,
) -> RawServoAnimationValueStrong {
    let a_value = AnimationValue::as_arc(&a);
    let b_value = AnimationValue::as_arc(&b);
    if let Ok(value) = a_value.animate(b_value, Procedure::Accumulate { count }) {
        Arc::new(value).into_strong()
    } else {
        RawServoAnimationValueStrong::null()
    }
}

#[no_mangle]
pub extern "C" fn Servo_AnimationValues_GetZeroValue(
    value_to_match: RawServoAnimationValueBorrowed)
    -> RawServoAnimationValueStrong
{
    let value_to_match = AnimationValue::as_arc(&value_to_match);
    if let Ok(zero_value) = value_to_match.to_animated_zero() {
        Arc::new(zero_value).into_strong()
    } else {
        RawServoAnimationValueStrong::null()
    }
}

#[no_mangle]
pub extern "C" fn Servo_AnimationValues_ComputeDistance(from: RawServoAnimationValueBorrowed,
                                                        to: RawServoAnimationValueBorrowed)
                                                        -> f64 {
    let from_value = AnimationValue::as_arc(&from);
    let to_value = AnimationValue::as_arc(&to);
    // If compute_squared_distance() failed, this function will return negative value
    // in order to check whether we support the specified paced animation values.
    from_value.compute_squared_distance(to_value).map(|d| d.sqrt()).unwrap_or(-1.0)
}

/// Compute one of the endpoints for the interpolation interval, compositing it with the
/// underlying value if needed.
/// An None returned value means, "Just use endpoint_value as-is."
/// It is the responsibility of the caller to ensure that |underlying_value| is provided
/// when it will be used.
fn composite_endpoint(
    endpoint_value: Option<&RawOffsetArc<AnimationValue>>,
    composite: CompositeOperation,
    underlying_value: Option<&AnimationValue>,
) -> Option<AnimationValue> {
    match endpoint_value {
        Some(endpoint_value) => {
            match composite {
                CompositeOperation::Add => {
                    underlying_value
                        .expect("We should have an underlying_value")
                        .animate(endpoint_value, Procedure::Add).ok()
                },
                CompositeOperation::Accumulate => {
                    underlying_value
                        .expect("We should have an underlying value")
                        .animate(endpoint_value, Procedure::Accumulate { count: 1 })
                        .ok()
                },
                _ => None,
            }
        },
        None => underlying_value.map(|v| v.clone()),
    }
}

/// Accumulate one of the endpoints of the animation interval.
/// A returned value of None means, "Just use endpoint_value as-is."
fn accumulate_endpoint(
    endpoint_value: Option<&RawOffsetArc<AnimationValue>>,
    composited_value: Option<AnimationValue>,
    last_value: &AnimationValue,
    current_iteration: u64
) -> Option<AnimationValue> {
    debug_assert!(endpoint_value.is_some() || composited_value.is_some(),
                  "Should have a suitable value to use");

    let count = current_iteration;
    match composited_value {
        Some(endpoint) => {
            last_value
                .animate(&endpoint, Procedure::Accumulate { count })
                .ok()
                .or(Some(endpoint))
        },
        None => {
            last_value
                .animate(endpoint_value.unwrap(), Procedure::Accumulate { count })
                .ok()
        },
    }
}

/// Compose the animation segment. We composite it with the underlying_value and last_value if
/// needed.
/// The caller is responsible for providing an underlying value and last value
/// in all situations where there are needed.
fn compose_animation_segment(
    segment: RawGeckoAnimationPropertySegmentBorrowed,
    underlying_value: Option<&AnimationValue>,
    last_value: Option<&AnimationValue>,
    iteration_composite: IterationCompositeOperation,
    current_iteration: u64,
    total_progress: f64,
    segment_progress: f64,
) -> AnimationValue {
    // Extract keyframe values.
    let raw_from_value;
    let keyframe_from_value = if !segment.mFromValue.mServo.mRawPtr.is_null() {
        raw_from_value = unsafe { &*segment.mFromValue.mServo.mRawPtr };
        Some(AnimationValue::as_arc(&raw_from_value))
    } else {
        None
    };

    let raw_to_value;
    let keyframe_to_value = if !segment.mToValue.mServo.mRawPtr.is_null() {
        raw_to_value = unsafe { &*segment.mToValue.mServo.mRawPtr };
        Some(AnimationValue::as_arc(&raw_to_value))
    } else {
        None
    };

    let mut composited_from_value = composite_endpoint(keyframe_from_value,
                                                       segment.mFromComposite,
                                                       underlying_value);
    let mut composited_to_value = composite_endpoint(keyframe_to_value,
                                                     segment.mToComposite,
                                                     underlying_value);

    debug_assert!(keyframe_from_value.is_some() || composited_from_value.is_some(),
                  "Should have a suitable from value to use");
    debug_assert!(keyframe_to_value.is_some() || composited_to_value.is_some(),
                  "Should have a suitable to value to use");

    // Apply iteration composite behavior.
    if iteration_composite == IterationCompositeOperation::Accumulate && current_iteration > 0 {
        let last_value = last_value.unwrap_or_else(|| {
            underlying_value.expect("Should have a valid underlying value")
        });

        composited_from_value = accumulate_endpoint(keyframe_from_value,
                                                    composited_from_value,
                                                    last_value,
                                                    current_iteration);
        composited_to_value = accumulate_endpoint(keyframe_to_value,
                                                  composited_to_value,
                                                  last_value,
                                                  current_iteration);
    }

    // Use the composited value if there is one, otherwise, use the original keyframe value.
    let from = composited_from_value.as_ref().unwrap_or_else(|| keyframe_from_value.unwrap());
    let to   = composited_to_value.as_ref().unwrap_or_else(|| keyframe_to_value.unwrap());

    if segment.mToKey == segment.mFromKey {
        return if total_progress < 0. { from.clone() } else { to.clone() };
    }

    match from.animate(to, Procedure::Interpolate { progress: segment_progress }) {
        Ok(value) => value,
        _ => if segment_progress < 0.5 { from.clone() } else { to.clone() },
    }
}

#[no_mangle]
pub extern "C" fn Servo_ComposeAnimationSegment(
    segment: RawGeckoAnimationPropertySegmentBorrowed,
    underlying_value: RawServoAnimationValueBorrowedOrNull,
    last_value: RawServoAnimationValueBorrowedOrNull,
    iteration_composite: IterationCompositeOperation,
    progress: f64,
    current_iteration: u64
) -> RawServoAnimationValueStrong {
    let underlying_value = AnimationValue::arc_from_borrowed(&underlying_value).map(|v| &**v);
    let last_value = AnimationValue::arc_from_borrowed(&last_value).map(|v| &**v);
    let result = compose_animation_segment(segment,
                                           underlying_value,
                                           last_value,
                                           iteration_composite,
                                           current_iteration,
                                           progress,
                                           progress);
    Arc::new(result).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_AnimationCompose(raw_value_map: RawServoAnimationValueMapBorrowedMut,
                                         base_values: RawServoAnimationValueTableBorrowed,
                                         css_property: nsCSSPropertyID,
                                         segment: RawGeckoAnimationPropertySegmentBorrowed,
                                         last_segment: RawGeckoAnimationPropertySegmentBorrowed,
                                         computed_timing: RawGeckoComputedTimingBorrowed,
                                         iteration_composite: IterationCompositeOperation)
{
    use style::gecko_bindings::bindings::Gecko_AnimationGetBaseStyle;
    use style::gecko_bindings::bindings::Gecko_GetPositionInSegment;
    use style::gecko_bindings::bindings::Gecko_GetProgressFromComputedTiming;
    use style::properties::animated_properties::AnimationValueMap;

    let property = match LonghandId::from_nscsspropertyid(css_property) {
        Ok(longhand) if longhand.is_animatable() => longhand,
        _ => return,
    };
    let value_map = AnimationValueMap::from_ffi_mut(raw_value_map);

    // We will need an underlying value if either of the endpoints is null...
    let need_underlying_value = segment.mFromValue.mServo.mRawPtr.is_null() ||
                                segment.mToValue.mServo.mRawPtr.is_null() ||
                                // ... or if they have a non-replace composite mode ...
                                segment.mFromComposite != CompositeOperation::Replace ||
                                segment.mToComposite != CompositeOperation::Replace ||
                                // ... or if we accumulate onto the last value and it is null.
                                (iteration_composite == IterationCompositeOperation::Accumulate &&
                                 computed_timing.mCurrentIteration > 0 &&
                                 last_segment.mToValue.mServo.mRawPtr.is_null());

    // If either of the segment endpoints are null, get the underlying value to
    // use from the current value in the values map (set by a lower-priority
    // effect), or, if there is no current value, look up the cached base value
    // for this property.
    let underlying_value = if need_underlying_value {
        let previous_composed_value = value_map.get(&property).cloned();
        previous_composed_value.or_else(|| {
            let raw_base_style = unsafe { Gecko_AnimationGetBaseStyle(base_values, css_property) };
            AnimationValue::arc_from_borrowed(&raw_base_style).map(|v| &**v).cloned()
        })
    } else {
        None
    };

    if need_underlying_value && underlying_value.is_none() {
        warn!("Underlying value should be valid when we expect to use it");
        return;
    }

    let raw_last_value;
    let last_value = if !last_segment.mToValue.mServo.mRawPtr.is_null() {
        raw_last_value = unsafe { &*last_segment.mToValue.mServo.mRawPtr };
        Some(&**AnimationValue::as_arc(&raw_last_value))
    } else {
        None
    };

    let progress = unsafe { Gecko_GetProgressFromComputedTiming(computed_timing) };
    let position = if segment.mToKey == segment.mFromKey {
        // Note: compose_animation_segment doesn't use this value
        // if segment.mFromKey == segment.mToKey, so assigning |progress| directly is fine.
        progress
    } else {
        unsafe { Gecko_GetPositionInSegment(segment, progress, computed_timing.mBeforeFlag) }
    };

    let result = compose_animation_segment(segment,
                                           underlying_value.as_ref(),
                                           last_value,
                                           iteration_composite,
                                           computed_timing.mCurrentIteration,
                                           progress,
                                           position);
    value_map.insert(property, result);
}

macro_rules! get_property_id_from_nscsspropertyid {
    ($property_id: ident, $ret: expr) => {{
        match PropertyId::from_nscsspropertyid($property_id) {
            Ok(property_id) => property_id,
            Err(()) => { return $ret; }
        }
    }}
}

#[no_mangle]
pub extern "C" fn Servo_AnimationValue_Serialize(value: RawServoAnimationValueBorrowed,
                                                 property: nsCSSPropertyID,
                                                 buffer: *mut nsAString)
{
    let uncomputed_value = AnimationValue::as_arc(&value).uncompute();
    let mut string = String::new();
    let rv = PropertyDeclarationBlock::with_one(uncomputed_value, Importance::Normal)
        .single_value_to_css(&get_property_id_from_nscsspropertyid!(property, ()), &mut string,
                             None, None /* No extra custom properties */);
    debug_assert!(rv.is_ok());

    let buffer = unsafe { buffer.as_mut().unwrap() };
    buffer.assign_utf8(&string);
}

#[no_mangle]
pub extern "C" fn Servo_Shorthand_AnimationValues_Serialize(shorthand_property: nsCSSPropertyID,
                                                            values: RawGeckoServoAnimationValueListBorrowed,
                                                            buffer: *mut nsAString)
{
    let property_id = get_property_id_from_nscsspropertyid!(shorthand_property, ());
    let shorthand = match property_id.as_shorthand() {
        Ok(shorthand) => shorthand,
        _ => return,
    };

    // Convert RawServoAnimationValue(s) into a vector of PropertyDeclaration
    // so that we can use reference of the PropertyDeclaration without worrying
    // about its lifetime. (longhands_to_css() expects &PropertyDeclaration
    // iterator.)
    let declarations: Vec<PropertyDeclaration> =
        values.iter().map(|v| AnimationValue::as_arc(unsafe { &&*v.mRawPtr }).uncompute()).collect();

    let mut string = String::new();
    let rv = shorthand.longhands_to_css(declarations.iter(), &mut string);
    if rv.is_ok() {
        let buffer = unsafe { buffer.as_mut().unwrap() };
        buffer.assign_utf8(&string);
    }
}

#[no_mangle]
pub extern "C" fn Servo_AnimationValue_GetOpacity(
    value: RawServoAnimationValueBorrowed
) -> f32 {
    let value = AnimationValue::as_arc(&value);
    if let AnimationValue::Opacity(opacity) = **value {
        opacity
    } else {
        panic!("The AnimationValue should be Opacity");
    }
}

#[no_mangle]
pub extern "C" fn Servo_AnimationValue_Opacity(
    opacity: f32
) -> RawServoAnimationValueStrong {
    Arc::new(AnimationValue::Opacity(opacity)).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_AnimationValue_GetTransform(
    value: RawServoAnimationValueBorrowed,
    list: *mut structs::RefPtr<nsCSSValueSharedList>
) {
    let value = AnimationValue::as_arc(&value);
    if let AnimationValue::Transform(ref servo_list) = **value {
        let list = unsafe { &mut *list };
        if servo_list.0.is_empty() {
            unsafe {
                list.set_move(RefPtr::from_addrefed(Gecko_NewNoneTransform()));
            }
        } else {
            gecko_properties::convert_transform(&servo_list.0, list);
        }
    } else {
        panic!("The AnimationValue should be transform");
    }
}

#[no_mangle]
pub extern "C" fn Servo_AnimationValue_Transform(
    list: *const nsCSSValueSharedList
) -> RawServoAnimationValueStrong {
    let list = unsafe { (&*list).mHead.as_ref() };
    let transform = gecko_properties::clone_transform_from_list(list);
    Arc::new(AnimationValue::Transform(transform)).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_AnimationValue_DeepEqual(
    this: RawServoAnimationValueBorrowed,
    other: RawServoAnimationValueBorrowed,
) -> bool {
    let this_value = AnimationValue::as_arc(&this);
    let other_value = AnimationValue::as_arc(&other);
    this_value == other_value
}

#[no_mangle]
pub extern "C" fn Servo_AnimationValue_Uncompute(
    value: RawServoAnimationValueBorrowed,
) -> RawServoDeclarationBlockStrong {
    let value = AnimationValue::as_arc(&value);
    let global_style_data = &*GLOBAL_STYLE_DATA;
    Arc::new(global_style_data.shared_lock.wrap(
        PropertyDeclarationBlock::with_one(value.uncompute(), Importance::Normal))).into_strong()
}

// Return the ComputedValues by a base ComputedValues and the rules.
fn resolve_rules_for_element_with_context<'a>(
    element: GeckoElement<'a>,
    mut context: StyleContext<GeckoElement<'a>>,
    rules: StrongRuleNode
) -> Arc<ComputedValues> {
    use style::style_resolver::{PseudoElementResolution, StyleResolverForElement};

    // This currently ignores visited styles, which seems acceptable, as
    // existing browsers don't appear to animate visited styles.
    let inputs =
        CascadeInputs {
            rules: Some(rules),
            visited_rules: None,
        };

    // Actually `PseudoElementResolution` doesn't matter.
    StyleResolverForElement::new(element,
                                 &mut context,
                                 RuleInclusion::All,
                                 PseudoElementResolution::IfApplicable)
        .cascade_style_and_visited_with_default_parents(inputs).0
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_GetBaseComputedValuesForElement(
    raw_style_set: RawServoStyleSetBorrowed,
    element: RawGeckoElementBorrowed,
    computed_values: ServoStyleContextBorrowed,
    snapshots: *const ServoElementSnapshotTable,
) -> ServoStyleContextStrong {
    debug_assert!(!snapshots.is_null());
    let computed_values = unsafe { ArcBorrow::from_ref(computed_values) };

    let rules = match computed_values.rules {
        None => return computed_values.clone_arc().into(),
        Some(ref rules) => rules,
    };

    let doc_data = PerDocumentStyleData::from_ffi(raw_style_set).borrow();
    let without_animations_rules = doc_data.stylist.rule_tree().remove_animation_rules(rules);
    if without_animations_rules == *rules {
        return computed_values.clone_arc().into();
    }

    let element = GeckoElement(element);

    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let shared = create_shared_context(&global_style_data,
                                       &guard,
                                       &doc_data,
                                       TraversalFlags::empty(),
                                       unsafe { &*snapshots });
    let mut tlc = ThreadLocalStyleContext::new(&shared);
    let context = StyleContext {
        shared: &shared,
        thread_local: &mut tlc,
    };

    resolve_rules_for_element_with_context(element, context, without_animations_rules).into()
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_GetComputedValuesByAddingAnimation(
    raw_style_set: RawServoStyleSetBorrowed,
    element: RawGeckoElementBorrowed,
    computed_values: ServoStyleContextBorrowed,
    snapshots: *const ServoElementSnapshotTable,
    animation_value: RawServoAnimationValueBorrowed,
) -> ServoStyleContextStrong {
    debug_assert!(!snapshots.is_null());
    let computed_values = unsafe { ArcBorrow::from_ref(computed_values) };
    let rules = match computed_values.rules {
        None => return ServoStyleContextStrong::null(),
        Some(ref rules) => rules,
    };

    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let uncomputed_value = AnimationValue::as_arc(&animation_value).uncompute();
    let doc_data = PerDocumentStyleData::from_ffi(raw_style_set).borrow();

    let with_animations_rules = {
        let guards = StylesheetGuards::same(&guard);
        let declarations =
            Arc::new(global_style_data.shared_lock.wrap(
                PropertyDeclarationBlock::with_one(uncomputed_value, Importance::Normal)));
        doc_data.stylist
            .rule_tree()
            .add_animation_rules_at_transition_level(rules, declarations, &guards)
    };

    let element = GeckoElement(element);
    if element.borrow_data().is_none() {
        return ServoStyleContextStrong::null();
    }

    let shared = create_shared_context(&global_style_data,
                                       &guard,
                                       &doc_data,
                                       TraversalFlags::empty(),
                                       unsafe { &*snapshots });
    let mut tlc: ThreadLocalStyleContext<GeckoElement> = ThreadLocalStyleContext::new(&shared);
    let context = StyleContext {
        shared: &shared,
        thread_local: &mut tlc,
    };

    resolve_rules_for_element_with_context(element, context, with_animations_rules).into()
}

#[no_mangle]
pub extern "C" fn Servo_ComputedValues_ExtractAnimationValue(
    computed_values: ServoStyleContextBorrowed,
    property_id: nsCSSPropertyID,
) -> RawServoAnimationValueStrong {
    let property = match LonghandId::from_nscsspropertyid(property_id) {
        Ok(longhand) => longhand,
        Err(()) => return Strong::null(),
    };

    match AnimationValue::from_computed_values(&property, &computed_values) {
        Some(v) => Arc::new(v).into_strong(),
        None => Strong::null(),
    }
}

#[no_mangle]
pub extern "C" fn Servo_Property_IsAnimatable(property: nsCSSPropertyID) -> bool {
    use style::properties::animated_properties;
    animated_properties::nscsspropertyid_is_animatable(property)
}

#[no_mangle]
pub extern "C" fn Servo_Property_IsTransitionable(property: nsCSSPropertyID) -> bool {
    use style::properties::animated_properties;
    animated_properties::nscsspropertyid_is_transitionable(property)
}

#[no_mangle]
pub extern "C" fn Servo_Property_IsDiscreteAnimatable(property: nsCSSPropertyID) -> bool {
    match LonghandId::from_nscsspropertyid(property) {
        Ok(longhand) => longhand.is_discrete_animatable(),
        Err(()) => return false,
    }
}

#[no_mangle]
pub extern "C" fn Servo_StyleWorkerThreadCount() -> u32 {
    STYLE_THREAD_POOL.num_threads as u32
}

#[no_mangle]
pub extern "C" fn Servo_Element_ClearData(element: RawGeckoElementBorrowed) {
    unsafe { GeckoElement(element).clear_data() };
}

#[no_mangle]
pub extern "C" fn Servo_Element_SizeOfExcludingThisAndCVs(malloc_size_of: GeckoMallocSizeOf,
                                                          malloc_enclosing_size_of:
                                                              GeckoMallocSizeOf,
                                                          seen_ptrs: *mut SeenPtrs,
                                                          element: RawGeckoElementBorrowed) -> usize {
    let element = GeckoElement(element);
    let borrow = element.borrow_data();
    if let Some(data) = borrow {
        let have_seen_ptr = move |ptr| { unsafe { Gecko_HaveSeenPtr(seen_ptrs, ptr) } };
        let mut ops = MallocSizeOfOps::new(malloc_size_of.unwrap(),
                                           Some(malloc_enclosing_size_of.unwrap()),
                                           Some(Box::new(have_seen_ptr)));
        (*data).size_of_excluding_cvs(&mut ops)
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn Servo_Element_HasPrimaryComputedValues(element: RawGeckoElementBorrowed) -> bool
{
    let element = GeckoElement(element);
    let data = element.borrow_data().expect("Looking for CVs on unstyled element");
    data.has_styles()
}

#[no_mangle]
pub extern "C" fn Servo_Element_GetPrimaryComputedValues(element: RawGeckoElementBorrowed)
                                                         -> ServoStyleContextStrong
{
    let element = GeckoElement(element);
    let data = element.borrow_data().expect("Getting CVs on unstyled element");
    data.styles.primary().clone().into()
}

#[no_mangle]
pub extern "C" fn Servo_Element_HasPseudoComputedValues(element: RawGeckoElementBorrowed,
                                                        index: usize) -> bool
{
    let element = GeckoElement(element);
    let data = element.borrow_data().expect("Looking for CVs on unstyled element");
    data.styles.pseudos.as_array()[index].is_some()
}

#[no_mangle]
pub extern "C" fn Servo_Element_GetPseudoComputedValues(element: RawGeckoElementBorrowed,
                                                        index: usize) -> ServoStyleContextStrong
{
    let element = GeckoElement(element);
    let data = element.borrow_data().expect("Getting CVs that aren't present");
    data.styles.pseudos.as_array()[index].as_ref().expect("Getting CVs that aren't present")
        .clone().into()
}

#[no_mangle]
pub extern "C" fn Servo_Element_IsDisplayNone(element: RawGeckoElementBorrowed) -> bool {
    let element = GeckoElement(element);
    let data = element.borrow_data().expect("Invoking Servo_Element_IsDisplayNone on unstyled element");
    data.styles.is_display_none()
}

#[no_mangle]
pub extern "C" fn Servo_Element_IsPrimaryStyleReusedViaRuleNode(element: RawGeckoElementBorrowed) -> bool {
    let element = GeckoElement(element);
    let data = element.borrow_data()
                      .expect("Invoking Servo_Element_IsPrimaryStyleReusedViaRuleNode on unstyled element");
    data.flags.contains(data::ElementDataFlags::PRIMARY_STYLE_REUSED_VIA_RULE_NODE)
}

#[no_mangle]
pub extern "C" fn Servo_StyleSheet_Empty(mode: SheetParsingMode) -> RawServoStyleSheetContentsStrong {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let origin = match mode {
        SheetParsingMode::eAuthorSheetFeatures => Origin::Author,
        SheetParsingMode::eUserSheetFeatures => Origin::User,
        SheetParsingMode::eAgentSheetFeatures => Origin::UserAgent,
        SheetParsingMode::eSafeAgentSheetFeatures => Origin::UserAgent,
    };
    let shared_lock = &global_style_data.shared_lock;
    Arc::new(
        StylesheetContents::from_str(
            "",
            unsafe { dummy_url_data() }.clone(),
            origin,
            shared_lock,
            /* loader = */ None,
            &NullReporter,
            QuirksMode::NoQuirks,
            0
        )
    ).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_StyleSheet_FromUTF8Bytes(
    loader: *mut Loader,
    stylesheet: *mut ServoStyleSheet,
    data: *const u8,
    data_len: usize,
    mode: SheetParsingMode,
    extra_data: *mut URLExtraData,
    line_number_offset: u32,
    quirks_mode: nsCompatibility,
    reusable_sheets: *mut LoaderReusableStyleSheets
) -> RawServoStyleSheetContentsStrong {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let input = unsafe { ::std::str::from_utf8_unchecked(::std::slice::from_raw_parts(data, data_len)) };

    let origin = match mode {
        SheetParsingMode::eAuthorSheetFeatures => Origin::Author,
        SheetParsingMode::eUserSheetFeatures => Origin::User,
        SheetParsingMode::eAgentSheetFeatures => Origin::UserAgent,
        SheetParsingMode::eSafeAgentSheetFeatures => Origin::UserAgent,
    };

    let reporter = ErrorReporter::new(stylesheet, loader, extra_data);
    let url_data = unsafe { RefPtr::from_ptr_ref(&extra_data) };
    let loader = if loader.is_null() {
        None
    } else {
        Some(StylesheetLoader::new(loader, stylesheet, reusable_sheets))
    };

    // FIXME(emilio): loader.as_ref() doesn't typecheck for some reason?
    let loader: Option<&StyleStylesheetLoader> = match loader {
        None => None,
        Some(ref s) => Some(s),
    };


    Arc::new(StylesheetContents::from_str(
        input, url_data.clone(), origin,
        &global_style_data.shared_lock, loader, &reporter,
        quirks_mode.into(), line_number_offset)
    ).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_AppendStyleSheet(
    raw_data: RawServoStyleSetBorrowed,
    sheet: *const ServoStyleSheet,
) {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    let data = &mut *data;
    let guard = global_style_data.shared_lock.read();
    let sheet = unsafe { GeckoStyleSheet::new(sheet) };
    data.stylist.append_stylesheet(sheet, &guard);
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_MediumFeaturesChanged(
    raw_data: RawServoStyleSetBorrowed,
    viewport_units_used: *mut bool,
) -> u8 {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();

    // NOTE(emilio): We don't actually need to flush the stylist here and ensure
    // it's up to date.
    //
    // In case it isn't we would trigger a rebuild + restyle as needed too.
    //
    // We need to ensure the default computed values are up to date though,
    // because those can influence the result of media query evaluation.
    //
    // FIXME(emilio, bug 1369984): do the computation conditionally, to do it
    // less often.
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();

    unsafe {
        *viewport_units_used = data.stylist.device().used_viewport_size();
    }
    data.stylist.device_mut().reset_computed_values();
    let guards = StylesheetGuards::same(&guard);
    let origins_in_which_rules_changed =
        data.stylist.media_features_change_changed_style(&guards);

    // We'd like to return `OriginFlags` here, but bindgen bitfield enums don't
    // work as return values with the Linux 32-bit ABI at the moment because
    // they wrap the value in a struct, so for now just unwrap it.
    OriginFlags::from(origins_in_which_rules_changed).0
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_SetDevice(
    raw_data: RawServoStyleSetBorrowed,
    pres_context: RawGeckoPresContextOwned
) -> u8 {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();

    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    let device = Device::new(pres_context);
    let guards = StylesheetGuards::same(&guard);
    let origins_in_which_rules_changed =
        data.stylist.set_device(device, &guards);

    // We'd like to return `OriginFlags` here, but bindgen bitfield enums don't
    // work as return values with the Linux 32-bit ABI at the moment because
    // they wrap the value in a struct, so for now just unwrap it.
    OriginFlags::from(origins_in_which_rules_changed).0
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_PrependStyleSheet(
    raw_data: RawServoStyleSetBorrowed,
    sheet: *const ServoStyleSheet,
) {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    let data = &mut *data;
    let guard = global_style_data.shared_lock.read();
    let sheet = unsafe { GeckoStyleSheet::new(sheet) };
    data.stylist.prepend_stylesheet(sheet, &guard);
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_InsertStyleSheetBefore(
    raw_data: RawServoStyleSetBorrowed,
    sheet: *const ServoStyleSheet,
    before_sheet: *const ServoStyleSheet
) {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    let data = &mut *data;
    let guard = global_style_data.shared_lock.read();
    let sheet = unsafe { GeckoStyleSheet::new(sheet) };
    data.stylist.insert_stylesheet_before(
        sheet,
        unsafe { GeckoStyleSheet::new(before_sheet) },
        &guard,
    );
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_RemoveStyleSheet(
    raw_data: RawServoStyleSetBorrowed,
    sheet: *const ServoStyleSheet
) {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    let data = &mut *data;
    let guard = global_style_data.shared_lock.read();
    let sheet = unsafe { GeckoStyleSheet::new(sheet) };
    data.stylist.remove_stylesheet(sheet, &guard);
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_FlushStyleSheets(
    raw_data: RawServoStyleSetBorrowed,
    doc_element: RawGeckoElementBorrowedOrNull,
) {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    let doc_element = doc_element.map(GeckoElement);
    let have_invalidations = data.flush_stylesheets(&guard, doc_element);
    if have_invalidations && doc_element.is_some() {
        // The invalidation machinery propagates the bits up, but we still
        // need to tell the gecko restyle root machinery about it.
        unsafe {
            bindings::Gecko_NoteDirtySubtreeForInvalidation(doc_element.unwrap().0);
        }
    }
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_NoteStyleSheetsChanged(
    raw_data: RawServoStyleSetBorrowed,
    author_style_disabled: bool,
    changed_origins: OriginFlags,
) {
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    data.stylist.force_stylesheet_origins_dirty(OriginSet::from(changed_origins));
    data.stylist.set_author_style_disabled(author_style_disabled);
}

#[no_mangle]
pub extern "C" fn Servo_StyleSheet_HasRules(
    raw_contents: RawServoStyleSheetContentsBorrowed
) -> bool {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    !StylesheetContents::as_arc(&raw_contents)
        .rules.read_with(&guard).0.is_empty()
}

#[no_mangle]
pub extern "C" fn Servo_StyleSheet_GetRules(
    sheet: RawServoStyleSheetContentsBorrowed
) -> ServoCssRulesStrong {
    StylesheetContents::as_arc(&sheet).rules.clone().into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_StyleSheet_Clone(
    raw_sheet: RawServoStyleSheetContentsBorrowed,
    reference_sheet: *const ServoStyleSheet,
) -> RawServoStyleSheetContentsStrong {
    use style::shared_lock::{DeepCloneParams, DeepCloneWithLock};
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let contents = StylesheetContents::as_arc(&raw_sheet);
    let params = DeepCloneParams { reference_sheet };

    Arc::new(contents.deep_clone_with_lock(
        &global_style_data.shared_lock,
        &guard,
        &params,
    )).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_StyleSheet_SizeOfIncludingThis(
    malloc_size_of: GeckoMallocSizeOf,
    malloc_enclosing_size_of: GeckoMallocSizeOf,
    sheet: RawServoStyleSheetContentsBorrowed
) -> usize {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let mut ops = MallocSizeOfOps::new(malloc_size_of.unwrap(),
                                       Some(malloc_enclosing_size_of.unwrap()),
                                       None);
    StylesheetContents::as_arc(&sheet).size_of(&guard, &mut ops)
}

#[no_mangle]
pub extern "C" fn Servo_StyleSheet_GetOrigin(
    sheet: RawServoStyleSheetContentsBorrowed
) -> u8 {
    let origin = match StylesheetContents::as_arc(&sheet).origin {
        Origin::UserAgent => OriginFlags_UserAgent,
        Origin::User => OriginFlags_User,
        Origin::Author => OriginFlags_Author,
    };
    // We'd like to return `OriginFlags` here, but bindgen bitfield enums don't
    // work as return values with the Linux 32-bit ABI at the moment because
    // they wrap the value in a struct, so for now just unwrap it.
    origin.0
}

#[no_mangle]
pub extern "C" fn Servo_StyleSheet_GetSourceMapURL(
    sheet: RawServoStyleSheetContentsBorrowed,
    result: *mut nsAString
) {
    let contents = StylesheetContents::as_arc(&sheet);
    let url_opt = contents.source_map_url.read();
    if let Some(ref url) = *url_opt {
        write!(unsafe { &mut *result }, "{}", url).unwrap();
    }
}

#[no_mangle]
pub extern "C" fn Servo_StyleSheet_GetSourceURL(
    sheet: RawServoStyleSheetContentsBorrowed,
    result: *mut nsAString
) {
    let contents = StylesheetContents::as_arc(&sheet);
    let url_opt = contents.source_url.read();
    if let Some(ref url) = *url_opt {
        write!(unsafe { &mut *result }, "{}", url).unwrap();
    }
}

fn read_locked_arc<T, R, F>(raw: &<Locked<T> as HasFFI>::FFIType, func: F) -> R
    where Locked<T>: HasArcFFI, F: FnOnce(&T) -> R
{
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    func(Locked::<T>::as_arc(&raw).read_with(&guard))
}

#[cfg(debug_assertions)]
unsafe fn read_locked_arc_unchecked<T, R, F>(raw: &<Locked<T> as HasFFI>::FFIType, func: F) -> R
    where Locked<T>: HasArcFFI, F: FnOnce(&T) -> R
{
    read_locked_arc(raw, func)
}

#[cfg(not(debug_assertions))]
unsafe fn read_locked_arc_unchecked<T, R, F>(raw: &<Locked<T> as HasFFI>::FFIType, func: F) -> R
    where Locked<T>: HasArcFFI, F: FnOnce(&T) -> R
{
    func(Locked::<T>::as_arc(&raw).read_unchecked())
}

fn write_locked_arc<T, R, F>(raw: &<Locked<T> as HasFFI>::FFIType, func: F) -> R
    where Locked<T>: HasArcFFI, F: FnOnce(&mut T) -> R
{
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let mut guard = global_style_data.shared_lock.write();
    func(Locked::<T>::as_arc(&raw).write_with(&mut guard))
}

#[no_mangle]
pub extern "C" fn Servo_CssRules_ListTypes(rules: ServoCssRulesBorrowed,
                                           result: nsTArrayBorrowed_uintptr_t) {
    read_locked_arc(rules, |rules: &CssRules| {
        let iter = rules.0.iter().map(|rule| rule.rule_type() as usize);
        let (size, upper) = iter.size_hint();
        debug_assert_eq!(size, upper.unwrap());
        unsafe { result.set_len(size as u32) };
        result.iter_mut().zip(iter).fold((), |_, (r, v)| *r = v);
    })
}

#[no_mangle]
pub extern "C" fn Servo_CssRules_InsertRule(
    rules: ServoCssRulesBorrowed,
    contents: RawServoStyleSheetContentsBorrowed,
    rule: *const nsACString,
    index: u32,
    nested: bool,
    loader: *mut Loader,
    gecko_stylesheet: *mut ServoStyleSheet,
    rule_type: *mut u16,
) -> nsresult {
    let loader = if loader.is_null() {
        None
    } else {
        Some(StylesheetLoader::new(loader, gecko_stylesheet, ptr::null_mut()))
    };
    let loader = loader.as_ref().map(|loader| loader as &StyleStylesheetLoader);
    let rule = unsafe { rule.as_ref().unwrap().as_str_unchecked() };

    let global_style_data = &*GLOBAL_STYLE_DATA;
    let contents = StylesheetContents::as_arc(&contents);
    let result = Locked::<CssRules>::as_arc(&rules).insert_rule(
        &global_style_data.shared_lock,
        rule,
        contents,
        index as usize,
        nested,
        loader
    );

    match result {
        Ok(new_rule) => {
            *unsafe { rule_type.as_mut().unwrap() } = new_rule.rule_type() as u16;
            nsresult::NS_OK
        }
        Err(err) => err.into(),
    }
}

#[no_mangle]
pub extern "C" fn Servo_CssRules_DeleteRule(
    rules: ServoCssRulesBorrowed,
    index: u32
) -> nsresult {
    write_locked_arc(rules, |rules: &mut CssRules| {
        match rules.remove_rule(index as usize) {
            Ok(_) => nsresult::NS_OK,
            Err(err) => err.into()
        }
    })
}

macro_rules! impl_basic_rule_funcs_without_getter {
    { ($rule_type:ty, $raw_type:ty),
        debug: $debug:ident,
        to_css: $to_css:ident,
    } => {
        #[no_mangle]
        pub extern "C" fn $debug(rule: &$raw_type, result: *mut nsACString) {
            read_locked_arc(rule, |rule: &$rule_type| {
                write!(unsafe { result.as_mut().unwrap() }, "{:?}", *rule).unwrap();
            })
        }

        #[no_mangle]
        pub extern "C" fn $to_css(rule: &$raw_type, result: *mut nsAString) {
            let global_style_data = &*GLOBAL_STYLE_DATA;
            let guard = global_style_data.shared_lock.read();
            let rule = Locked::<$rule_type>::as_arc(&rule);
            rule.read_with(&guard).to_css(&guard, unsafe { result.as_mut().unwrap() }).unwrap();
        }
    }
}

macro_rules! impl_basic_rule_funcs {
    { ($name:ident, $rule_type:ty, $raw_type:ty),
        getter: $getter:ident,
        debug: $debug:ident,
        to_css: $to_css:ident,
    } => {
        #[no_mangle]
        pub extern "C" fn $getter(rules: ServoCssRulesBorrowed, index: u32,
                                  line: *mut u32, column: *mut u32)
            -> Strong<$raw_type> {
            let global_style_data = &*GLOBAL_STYLE_DATA;
            let guard = global_style_data.shared_lock.read();
            let rules = Locked::<CssRules>::as_arc(&rules).read_with(&guard);
            let index = index as usize;

            if index >= rules.0.len() {
                return Strong::null();
            }

            match rules.0[index] {
                CssRule::$name(ref rule) => {
                    let location = rule.read_with(&guard).source_location;
                    *unsafe { line.as_mut().unwrap() } = location.line as u32;
                    *unsafe { column.as_mut().unwrap() } = location.column as u32;
                    rule.clone().into_strong()
                },
                _ => {
                    Strong::null()
                }
            }
        }

        impl_basic_rule_funcs_without_getter! { ($rule_type, $raw_type),
            debug: $debug,
            to_css: $to_css,
        }
    }
}

macro_rules! impl_group_rule_funcs {
    { ($name:ident, $rule_type:ty, $raw_type:ty),
      get_rules: $get_rules:ident,
      $($basic:tt)+
    } => {
        impl_basic_rule_funcs! { ($name, $rule_type, $raw_type), $($basic)+ }

        #[no_mangle]
        pub extern "C" fn $get_rules(rule: &$raw_type) -> ServoCssRulesStrong {
            read_locked_arc(rule, |rule: &$rule_type| {
                rule.rules.clone().into_strong()
            })
        }
    }
}

impl_basic_rule_funcs! { (Style, StyleRule, RawServoStyleRule),
    getter: Servo_CssRules_GetStyleRuleAt,
    debug: Servo_StyleRule_Debug,
    to_css: Servo_StyleRule_GetCssText,
}

impl_basic_rule_funcs! { (Import, ImportRule, RawServoImportRule),
    getter: Servo_CssRules_GetImportRuleAt,
    debug: Servo_ImportRule_Debug,
    to_css: Servo_ImportRule_GetCssText,
}

impl_basic_rule_funcs_without_getter! { (Keyframe, RawServoKeyframe),
    debug: Servo_Keyframe_Debug,
    to_css: Servo_Keyframe_GetCssText,
}

impl_basic_rule_funcs! { (Keyframes, KeyframesRule, RawServoKeyframesRule),
    getter: Servo_CssRules_GetKeyframesRuleAt,
    debug: Servo_KeyframesRule_Debug,
    to_css: Servo_KeyframesRule_GetCssText,
}

impl_group_rule_funcs! { (Media, MediaRule, RawServoMediaRule),
    get_rules: Servo_MediaRule_GetRules,
    getter: Servo_CssRules_GetMediaRuleAt,
    debug: Servo_MediaRule_Debug,
    to_css: Servo_MediaRule_GetCssText,
}

impl_basic_rule_funcs! { (Namespace, NamespaceRule, RawServoNamespaceRule),
    getter: Servo_CssRules_GetNamespaceRuleAt,
    debug: Servo_NamespaceRule_Debug,
    to_css: Servo_NamespaceRule_GetCssText,
}

impl_basic_rule_funcs! { (Page, PageRule, RawServoPageRule),
    getter: Servo_CssRules_GetPageRuleAt,
    debug: Servo_PageRule_Debug,
    to_css: Servo_PageRule_GetCssText,
}

impl_group_rule_funcs! { (Supports, SupportsRule, RawServoSupportsRule),
    get_rules: Servo_SupportsRule_GetRules,
    getter: Servo_CssRules_GetSupportsRuleAt,
    debug: Servo_SupportsRule_Debug,
    to_css: Servo_SupportsRule_GetCssText,
}

impl_group_rule_funcs! { (Document, DocumentRule, RawServoDocumentRule),
    get_rules: Servo_DocumentRule_GetRules,
    getter: Servo_CssRules_GetDocumentRuleAt,
    debug: Servo_DocumentRule_Debug,
    to_css: Servo_DocumentRule_GetCssText,
}

impl_basic_rule_funcs! { (FontFeatureValues, FontFeatureValuesRule, RawServoFontFeatureValuesRule),
    getter: Servo_CssRules_GetFontFeatureValuesRuleAt,
    debug: Servo_FontFeatureValuesRule_Debug,
    to_css: Servo_FontFeatureValuesRule_GetCssText,
}

macro_rules! impl_getter_for_embedded_rule {
    ($getter:ident: $name:ident -> $ty:ty) => {
        #[no_mangle]
        pub extern "C" fn $getter(rules: ServoCssRulesBorrowed, index: u32) -> *mut $ty
        {
            let global_style_data = &*GLOBAL_STYLE_DATA;
            let guard = global_style_data.shared_lock.read();
            let rules = Locked::<CssRules>::as_arc(&rules).read_with(&guard);
            match rules.0[index as usize] {
                CssRule::$name(ref rule) => rule.read_with(&guard).get(),
                _ => unreachable!(concat!(stringify!($getter), " should only be called on a ",
                                          stringify!($name), " rule")),
            }
        }
    }
}

impl_getter_for_embedded_rule!(Servo_CssRules_GetFontFaceRuleAt:
                              FontFace -> nsCSSFontFaceRule);
impl_getter_for_embedded_rule!(Servo_CssRules_GetCounterStyleRuleAt:
                              CounterStyle -> nsCSSCounterStyleRule);

#[no_mangle]
pub extern "C" fn Servo_StyleRule_GetStyle(rule: RawServoStyleRuleBorrowed) -> RawServoDeclarationBlockStrong {
    read_locked_arc(rule, |rule: &StyleRule| {
        rule.block.clone().into_strong()
    })
}

#[no_mangle]
pub extern "C" fn Servo_StyleRule_SetStyle(rule: RawServoStyleRuleBorrowed,
                                           declarations: RawServoDeclarationBlockBorrowed) {
    let declarations = Locked::<PropertyDeclarationBlock>::as_arc(&declarations);
    write_locked_arc(rule, |rule: &mut StyleRule| {
        rule.block = declarations.clone_arc();
    })
}

#[no_mangle]
pub extern "C" fn Servo_StyleRule_GetSelectorText(rule: RawServoStyleRuleBorrowed, result: *mut nsAString) {
    read_locked_arc(rule, |rule: &StyleRule| {
        rule.selectors.to_css(unsafe { result.as_mut().unwrap() }).unwrap();
    })
}

#[no_mangle]
pub extern "C" fn Servo_StyleRule_GetSelectorTextAtIndex(rule: RawServoStyleRuleBorrowed,
                                                         index: u32,
                                                         result: *mut nsAString) {
    read_locked_arc(rule, |rule: &StyleRule| {
        let index = index as usize;
        if index >= rule.selectors.0.len() {
            return;
        }
        rule.selectors.0[index].to_css(unsafe { result.as_mut().unwrap() }).unwrap();
    })
}

#[no_mangle]
pub extern "C" fn Servo_StyleRule_GetSelectorCount(rule: RawServoStyleRuleBorrowed, count: *mut u32) {
    read_locked_arc(rule, |rule: &StyleRule| {
        *unsafe { count.as_mut().unwrap() } = rule.selectors.0.len() as u32;
    })
}

#[no_mangle]
pub extern "C" fn Servo_StyleRule_GetSpecificityAtIndex(
    rule: RawServoStyleRuleBorrowed,
    index: u32,
    specificity: *mut u64
) {
    read_locked_arc(rule, |rule: &StyleRule| {
        let specificity =  unsafe { specificity.as_mut().unwrap() };
        let index = index as usize;
        if index >= rule.selectors.0.len() {
            *specificity = 0;
            return;
        }
        *specificity = rule.selectors.0[index].specificity() as u64;
    })
}

#[no_mangle]
pub extern "C" fn Servo_StyleRule_SelectorMatchesElement(rule: RawServoStyleRuleBorrowed,
                                                         element: RawGeckoElementBorrowed,
                                                         index: u32,
                                                         pseudo_type: CSSPseudoElementType) -> bool {
    read_locked_arc(rule, |rule: &StyleRule| {
        let index = index as usize;
        if index >= rule.selectors.0.len() {
            return false;
        }

        let selector = &rule.selectors.0[index];
        let mut matching_mode = MatchingMode::Normal;

        match PseudoElement::from_pseudo_type(pseudo_type) {
            Some(pseudo) => {
                // We need to make sure that the requested pseudo element type
                // matches the selector pseudo element type before proceeding.
                match selector.pseudo_element() {
                    Some(selector_pseudo) if *selector_pseudo == pseudo => {
                        matching_mode = MatchingMode::ForStatelessPseudoElement
                    },
                    _ => return false,
                };
            },
            None => {
                // Do not attempt to match if a pseudo element is requested and
                // this is not a pseudo element selector, or vice versa.
                if selector.has_pseudo_element() {
                    return false;
                }
            },
        };

        let element = GeckoElement(element);
        let quirks_mode = element.as_node().owner_doc().quirks_mode();
        let mut ctx =
            MatchingContext::new(matching_mode, None, None, quirks_mode);
        matches_selector(selector, 0, None, &element, &mut ctx, &mut |_, _| {})
    })
}

#[no_mangle]
pub unsafe extern "C" fn Servo_SelectorList_Closest(
    element: RawGeckoElementBorrowed,
    selectors: RawServoSelectorListBorrowed,
) -> *const structs::RawGeckoElement {
    use std::borrow::Borrow;
    use style::dom_apis;

    let element = GeckoElement(element);
    let quirks_mode = element.as_node().owner_doc().quirks_mode();
    let selectors = ::selectors::SelectorList::from_ffi(selectors).borrow();

    dom_apis::element_closest(element, &selectors, quirks_mode)
        .map_or(ptr::null(), |e| e.0)
}

#[no_mangle]
pub unsafe extern "C" fn Servo_SelectorList_Matches(
    element: RawGeckoElementBorrowed,
    selectors: RawServoSelectorListBorrowed,
) -> bool {
    use std::borrow::Borrow;
    use style::dom_apis;

    let element = GeckoElement(element);
    let quirks_mode = element.as_node().owner_doc().quirks_mode();
    let selectors = ::selectors::SelectorList::from_ffi(selectors).borrow();
    dom_apis::element_matches(
        &element,
        &selectors,
        quirks_mode,
    )
}

#[no_mangle]
pub unsafe extern "C" fn Servo_SelectorList_QueryFirst(
    node: RawGeckoNodeBorrowed,
    selectors: RawServoSelectorListBorrowed,
    may_use_invalidation: bool,
) -> *const structs::RawGeckoElement {
    use std::borrow::Borrow;
    use style::dom_apis::{self, MayUseInvalidation, QueryFirst};

    let node = GeckoNode(node);
    let selectors = ::selectors::SelectorList::from_ffi(selectors).borrow();
    let mut result = None;

    let may_use_invalidation =
        if may_use_invalidation {
            MayUseInvalidation::Yes
        } else {
            MayUseInvalidation::No
        };

    dom_apis::query_selector::<GeckoElement, QueryFirst>(
        node,
        &selectors,
        &mut result,
        may_use_invalidation,
    );

    result.map_or(ptr::null(), |e| e.0)
}

#[no_mangle]
pub unsafe extern "C" fn Servo_SelectorList_QueryAll(
    node: RawGeckoNodeBorrowed,
    selectors: RawServoSelectorListBorrowed,
    content_list: *mut structs::nsSimpleContentList,
    may_use_invalidation: bool,
) {
    use smallvec::SmallVec;
    use std::borrow::Borrow;
    use style::dom_apis::{self, MayUseInvalidation, QueryAll};

    let node = GeckoNode(node);
    let selectors = ::selectors::SelectorList::from_ffi(selectors).borrow();
    let mut result = SmallVec::new();

    let may_use_invalidation =
        if may_use_invalidation {
            MayUseInvalidation::Yes
        } else {
            MayUseInvalidation::No
        };

    dom_apis::query_selector::<GeckoElement, QueryAll>(
        node,
        &selectors,
        &mut result,
        may_use_invalidation,
    );

    if !result.is_empty() {
        // NOTE(emilio): This relies on a slice of GeckoElement having the same
        // memory representation than a slice of element pointers.
        bindings::Gecko_ContentList_AppendAll(
            content_list,
            result.as_ptr() as *mut *const _,
            result.len(),
        )
    }
}

#[no_mangle]
pub extern "C" fn Servo_ImportRule_GetHref(rule: RawServoImportRuleBorrowed, result: *mut nsAString) {
    read_locked_arc(rule, |rule: &ImportRule| {
        write!(unsafe { &mut *result }, "{}", rule.url.as_str()).unwrap();
    })
}

#[no_mangle]
pub extern "C" fn Servo_ImportRule_GetSheet(
    rule: RawServoImportRuleBorrowed
) -> *const ServoStyleSheet {
    read_locked_arc(rule, |rule: &ImportRule| {
        rule.stylesheet.0.raw() as *const ServoStyleSheet
    })
}

#[no_mangle]
pub extern "C" fn Servo_Keyframe_GetKeyText(
    keyframe: RawServoKeyframeBorrowed,
    result: *mut nsAString
) {
    read_locked_arc(keyframe, |keyframe: &Keyframe| {
        keyframe.selector.to_css(unsafe { result.as_mut().unwrap() }).unwrap()
    })
}

#[no_mangle]
pub extern "C" fn Servo_Keyframe_SetKeyText(keyframe: RawServoKeyframeBorrowed, text: *const nsACString) -> bool {
    let text = unsafe { text.as_ref().unwrap().as_str_unchecked() };
    let mut input = ParserInput::new(&text);
    if let Ok(selector) = Parser::new(&mut input).parse_entirely(KeyframeSelector::parse) {
        write_locked_arc(keyframe, |keyframe: &mut Keyframe| {
            keyframe.selector = selector;
        });
        true
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn Servo_Keyframe_GetStyle(keyframe: RawServoKeyframeBorrowed) -> RawServoDeclarationBlockStrong {
    read_locked_arc(keyframe, |keyframe: &Keyframe| keyframe.block.clone().into_strong())
}

#[no_mangle]
pub extern "C" fn Servo_Keyframe_SetStyle(keyframe: RawServoKeyframeBorrowed,
                                          declarations: RawServoDeclarationBlockBorrowed) {
    let declarations = Locked::<PropertyDeclarationBlock>::as_arc(&declarations);
    write_locked_arc(keyframe, |keyframe: &mut Keyframe| {
        keyframe.block = declarations.clone_arc();
    })
}

#[no_mangle]
pub extern "C" fn Servo_KeyframesRule_GetName(rule: RawServoKeyframesRuleBorrowed) -> *mut nsAtom {
    read_locked_arc(rule, |rule: &KeyframesRule| rule.name.as_atom().as_ptr())
}

#[no_mangle]
pub extern "C" fn Servo_KeyframesRule_SetName(rule: RawServoKeyframesRuleBorrowed, name: *mut nsAtom) {
    write_locked_arc(rule, |rule: &mut KeyframesRule| {
        rule.name = KeyframesName::Ident(CustomIdent(unsafe { Atom::from_addrefed(name) }));
    })
}

#[no_mangle]
pub extern "C" fn Servo_KeyframesRule_GetCount(rule: RawServoKeyframesRuleBorrowed) -> u32 {
    read_locked_arc(rule, |rule: &KeyframesRule| rule.keyframes.len() as u32)
}

#[no_mangle]
pub extern "C" fn Servo_KeyframesRule_GetKeyframeAt(rule: RawServoKeyframesRuleBorrowed, index: u32,
                                                    line: *mut u32, column: *mut u32) -> RawServoKeyframeStrong {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let key = Locked::<KeyframesRule>::as_arc(&rule).read_with(&guard)
                  .keyframes[index as usize].clone();
    let location = key.read_with(&guard).source_location;
    *unsafe { line.as_mut().unwrap() } = location.line as u32;
    *unsafe { column.as_mut().unwrap() } = location.column as u32;
    key.into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_KeyframesRule_FindRule(rule: RawServoKeyframesRuleBorrowed,
                                               key: *const nsACString) -> u32 {
    let key = unsafe { key.as_ref().unwrap().as_str_unchecked() };
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    Locked::<KeyframesRule>::as_arc(&rule).read_with(&guard)
        .find_rule(&guard, key).map(|index| index as u32)
        .unwrap_or(u32::max_value())
}

#[no_mangle]
pub extern "C" fn Servo_KeyframesRule_AppendRule(
    rule: RawServoKeyframesRuleBorrowed,
    contents: RawServoStyleSheetContentsBorrowed,
    css: *const nsACString
) -> bool {
    let css = unsafe { css.as_ref().unwrap().as_str_unchecked() };
    let contents = StylesheetContents::as_arc(&contents);
    let global_style_data = &*GLOBAL_STYLE_DATA;

    match Keyframe::parse(css, &contents, &global_style_data.shared_lock) {
        Ok(keyframe) => {
            write_locked_arc(rule, |rule: &mut KeyframesRule| {
                rule.keyframes.push(keyframe);
            });
            true
        }
        Err(..) => false,
    }
}

#[no_mangle]
pub extern "C" fn Servo_KeyframesRule_DeleteRule(rule: RawServoKeyframesRuleBorrowed, index: u32) {
    write_locked_arc(rule, |rule: &mut KeyframesRule| {
        rule.keyframes.remove(index as usize);
    })
}

#[no_mangle]
pub extern "C" fn Servo_MediaRule_GetMedia(rule: RawServoMediaRuleBorrowed) -> RawServoMediaListStrong {
    read_locked_arc(rule, |rule: &MediaRule| {
        rule.media_queries.clone().into_strong()
    })
}

#[no_mangle]
pub extern "C" fn Servo_NamespaceRule_GetPrefix(rule: RawServoNamespaceRuleBorrowed) -> *mut nsAtom {
    read_locked_arc(rule, |rule: &NamespaceRule| {
        rule.prefix.as_ref().unwrap_or(&atom!("")).as_ptr()
    })
}

#[no_mangle]
pub extern "C" fn Servo_NamespaceRule_GetURI(rule: RawServoNamespaceRuleBorrowed) -> *mut nsAtom {
    read_locked_arc(rule, |rule: &NamespaceRule| rule.url.0.as_ptr())
}

#[no_mangle]
pub extern "C" fn Servo_PageRule_GetStyle(rule: RawServoPageRuleBorrowed) -> RawServoDeclarationBlockStrong {
    read_locked_arc(rule, |rule: &PageRule| {
        rule.block.clone().into_strong()
    })
}

#[no_mangle]
pub extern "C" fn Servo_PageRule_SetStyle(rule: RawServoPageRuleBorrowed,
                                           declarations: RawServoDeclarationBlockBorrowed) {
    let declarations = Locked::<PropertyDeclarationBlock>::as_arc(&declarations);
    write_locked_arc(rule, |rule: &mut PageRule| {
        rule.block = declarations.clone_arc();
    })
}

#[no_mangle]
pub extern "C" fn Servo_SupportsRule_GetConditionText(rule: RawServoSupportsRuleBorrowed,
                                                      result: *mut nsAString) {
    read_locked_arc(rule, |rule: &SupportsRule| {
        rule.condition.to_css(unsafe { result.as_mut().unwrap() }).unwrap();
    })
}

#[no_mangle]
pub extern "C" fn Servo_DocumentRule_GetConditionText(rule: RawServoDocumentRuleBorrowed,
                                                      result: *mut nsAString) {
    read_locked_arc(rule, |rule: &DocumentRule| {
        rule.condition.to_css(unsafe { result.as_mut().unwrap() }).unwrap();
    })
}

#[no_mangle]
pub extern "C" fn Servo_FontFeatureValuesRule_GetFontFamily(rule: RawServoFontFeatureValuesRuleBorrowed,
                                                            result: *mut nsAString) {
    read_locked_arc(rule, |rule: &FontFeatureValuesRule| {
        rule.font_family_to_css(unsafe { result.as_mut().unwrap() }).unwrap();
    })
}

#[no_mangle]
pub extern "C" fn Servo_FontFeatureValuesRule_GetValueText(rule: RawServoFontFeatureValuesRuleBorrowed,
                                                           result: *mut nsAString) {
    read_locked_arc(rule, |rule: &FontFeatureValuesRule| {
        rule.value_to_css(unsafe { result.as_mut().unwrap() }).unwrap();
    })
}

#[no_mangle]
pub extern "C" fn Servo_ComputedValues_GetForAnonymousBox(parent_style_or_null: ServoStyleContextBorrowedOrNull,
                                                          pseudo_tag: *mut nsAtom,
                                                          raw_data: RawServoStyleSetBorrowed)
     -> ServoStyleContextStrong {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let guards = StylesheetGuards::same(&guard);
    let data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    let atom = Atom::from(pseudo_tag);
    let pseudo = PseudoElement::from_anon_box_atom(&atom)
        .expect("Not an anon box pseudo?");

    let metrics = get_metrics_provider_for_product();

    // If the pseudo element is PageContent, we should append the precomputed
    // pseudo element declerations with specified page rules.
    let page_decls = match pseudo {
        PseudoElement::PageContent => {
            let mut declarations = vec![];
            let iter = data.stylist.iter_extra_data_origins_rev();
            for (data, origin) in iter {
                let level = match origin {
                    Origin::UserAgent => CascadeLevel::UANormal,
                    Origin::User => CascadeLevel::UserNormal,
                    Origin::Author => CascadeLevel::AuthorNormal,
                };
                for rule in data.pages.iter() {
                    declarations.push(ApplicableDeclarationBlock::from_declarations(
                        rule.read_with(level.guard(&guards)).block.clone(),
                        level
                    ));
                }
            }
            Some(declarations)
        },
        _ => None,
    };

    let rule_node = data.stylist.rule_node_for_precomputed_pseudo(
        &guards,
        &pseudo,
        page_decls,
    );

    data.stylist.precomputed_values_for_pseudo_with_rule_node(
        &guards,
        &pseudo,
        parent_style_or_null.map(|x| &*x),
        CascadeFlags::empty(),
        &metrics,
        rule_node
    ).into()
}

#[no_mangle]
pub extern "C" fn Servo_ResolvePseudoStyle(element: RawGeckoElementBorrowed,
                                           pseudo_type: CSSPseudoElementType,
                                           is_probe: bool,
                                           inherited_style: ServoStyleContextBorrowedOrNull,
                                           raw_data: RawServoStyleSetBorrowed)
     -> ServoStyleContextStrong
{
    let element = GeckoElement(element);
    let doc_data = PerDocumentStyleData::from_ffi(raw_data).borrow();

    debug!("Servo_ResolvePseudoStyle: {:?} {:?}, is_probe: {}",
           element, PseudoElement::from_pseudo_type(pseudo_type), is_probe);

    let data = element.borrow_data();

    let data = match data.as_ref() {
        Some(data) if data.has_styles() => data,
        _ => {
            // FIXME(bholley, emilio): Assert against this.
            //
            // Known offender is nsMathMLmoFrame::MarkIntrinsicISizesDirty,
            // which goes and does a bunch of work involving style resolution.
            //
            // Bug 1403865 tracks fixing it, and potentially adding an assert
            // here instead.
            warn!("Calling Servo_ResolvePseudoStyle on unstyled element");
            return if is_probe {
                Strong::null()
            } else {
                doc_data.default_computed_values().clone().into()
            };
        }
    };

    let pseudo = PseudoElement::from_pseudo_type(pseudo_type)
                    .expect("ResolvePseudoStyle with a non-pseudo?");

    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let style = get_pseudo_style(
        &guard,
        element,
        &pseudo,
        RuleInclusion::All,
        &data.styles,
        inherited_style,
        &*doc_data,
        is_probe,
        /* matching_func = */ None,
    );

    match style {
        Some(s) => s.into(),
        None => {
            debug_assert!(is_probe);
            Strong::null()
        }
    }
}

fn debug_atom_array(atoms: &AtomArray) -> String {
    let mut result = String::from("[");
    for atom in atoms.iter() {
        if atom.mRawPtr.is_null() {
            result += "(null), ";
        } else {
            let atom = unsafe { WeakAtom::new(atom.mRawPtr) };
            write!(result, "{}, ", atom).unwrap();
        }
    }
    result.push(']');
    result
}

#[no_mangle]
pub extern "C" fn Servo_ComputedValues_ResolveXULTreePseudoStyle(
    element: RawGeckoElementBorrowed,
    pseudo_tag: *mut nsAtom,
    inherited_style: ServoStyleContextBorrowed,
    input_word: *const AtomArray,
    raw_data: RawServoStyleSetBorrowed
) -> ServoStyleContextStrong {
    let element = GeckoElement(element);
    let data = element.borrow_data()
        .expect("Calling ResolveXULTreePseudoStyle on unstyled element?");

    let pseudo = unsafe {
        Atom::with(pseudo_tag, |atom| {
            PseudoElement::from_tree_pseudo_atom(atom, Box::new([]))
        }).expect("ResolveXULTreePseudoStyle with a non-tree pseudo?")
    };
    let input_word = unsafe { input_word.as_ref().unwrap() };

    let doc_data = PerDocumentStyleData::from_ffi(raw_data).borrow();

    debug!("ResolveXULTreePseudoStyle: {:?} {:?} {}",
           element, pseudo, debug_atom_array(input_word));

    let matching_fn = |pseudo: &PseudoElement| {
        let args = pseudo.tree_pseudo_args().expect("Not a tree pseudo-element?");
        args.iter().all(|atom| {
            input_word.iter().any(|item| atom.as_ptr() == item.mRawPtr)
        })
    };

    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    get_pseudo_style(
        &guard,
        element,
        &pseudo,
        RuleInclusion::All,
        &data.styles,
        Some(inherited_style),
        &*doc_data,
        /* is_probe = */ false,
        Some(&matching_fn),
    ).unwrap().into()
}

#[no_mangle]
pub extern "C" fn Servo_SetExplicitStyle(element: RawGeckoElementBorrowed,
                                         style: ServoStyleContextBorrowed)
{
    let element = GeckoElement(element);
    debug!("Servo_SetExplicitStyle: {:?}", element);
    // We only support this API for initial styling. There's no reason it couldn't
    // work for other things, we just haven't had a reason to do so.
    debug_assert!(element.get_data().is_none());
    let mut data = unsafe { element.ensure_data() };
    data.styles.primary = Some(unsafe { ArcBorrow::from_ref(style) }.clone_arc());
}

#[no_mangle]
pub extern "C" fn Servo_HasAuthorSpecifiedRules(style: ServoStyleContextBorrowed,
                                                element: RawGeckoElementBorrowed,
                                                pseudo_type: CSSPseudoElementType,
                                                rule_type_mask: u32,
                                                author_colors_allowed: bool)
    -> bool
{
    let element = GeckoElement(element);
    let pseudo = PseudoElement::from_pseudo_type(pseudo_type);

    let guard = (*GLOBAL_STYLE_DATA).shared_lock.read();
    let guards = StylesheetGuards::same(&guard);

    style.rules().has_author_specified_rules(element,
                                             pseudo,
                                             &guards,
                                             rule_type_mask,
                                             author_colors_allowed)
}

fn get_pseudo_style(
    guard: &SharedRwLockReadGuard,
    element: GeckoElement,
    pseudo: &PseudoElement,
    rule_inclusion: RuleInclusion,
    styles: &ElementStyles,
    inherited_styles: Option<&ComputedValues>,
    doc_data: &PerDocumentStyleDataImpl,
    is_probe: bool,
    matching_func: Option<&Fn(&PseudoElement) -> bool>,
) -> Option<Arc<ComputedValues>> {
    let style = match pseudo.cascade_type() {
        PseudoElementCascadeType::Eager => {
            match *pseudo {
                PseudoElement::FirstLetter => {
                    styles.pseudos.get(&pseudo).and_then(|pseudo_styles| {
                        // inherited_styles can be None when doing lazy resolution
                        // (e.g. for computed style) or when probing.  In that case
                        // we just inherit from our element, which is what Gecko
                        // does in that situation.  What should actually happen in
                        // the computed style case is a bit unclear.
                        let inherited_styles =
                            inherited_styles.unwrap_or(styles.primary());
                        let guards = StylesheetGuards::same(guard);
                        let metrics = get_metrics_provider_for_product();
                        let inputs = CascadeInputs::new_from_style(pseudo_styles);
                        doc_data.stylist
                            .compute_pseudo_element_style_with_inputs(
                                &inputs,
                                pseudo,
                                &guards,
                                Some(inherited_styles),
                                &metrics,
                                CascadeFlags::empty(),
                            )
                    })
                },
                _ => {
                    // Unfortunately, we can't assert that inherited_styles, if
                    // present, is pointer-equal to styles.primary(), or even
                    // equal in any meaningful way.  The way it can fail is as
                    // follows.  Say we append an element with a ::before,
                    // ::after, or ::first-line to a parent with a ::first-line,
                    // such that the element ends up on the first line of the
                    // parent (e.g. it's an inline-block in the case it has a
                    // ::first-line, or any container in the ::before/::after
                    // cases).  Then gecko will update its frame's style to
                    // inherit from the parent's ::first-line.  The next time we
                    // try to get the ::before/::after/::first-line style for
                    // the kid, we'll likely pass in the frame's style as
                    // inherited_styles, but that's not pointer-identical to
                    // styles.primary(), because it got reparented.
                    //
                    // Now in practice this turns out to be OK, because all the
                    // cases in which there's a mismatch go ahead and reparent
                    // styles again as needed to make sure the ::first-line
                    // affects all the things it should affect.  But it makes it
                    // impossible to assert anything about the two styles
                    // matching here, unfortunately.
                    styles.pseudos.get(&pseudo).cloned()
                },
            }
        }
        PseudoElementCascadeType::Precomputed => unreachable!("No anonymous boxes"),
        PseudoElementCascadeType::Lazy => {
            debug_assert!(inherited_styles.is_none() ||
                          ptr::eq(inherited_styles.unwrap(),
                                  &**styles.primary()));
            let base = if pseudo.inherits_from_default_values() {
                doc_data.default_computed_values()
            } else {
                styles.primary()
            };
            let guards = StylesheetGuards::same(guard);
            let metrics = get_metrics_provider_for_product();
            doc_data.stylist
                .lazily_compute_pseudo_element_style(
                    &guards,
                    &element,
                    &pseudo,
                    rule_inclusion,
                    base,
                    is_probe,
                    &metrics,
                    matching_func,
                )
        },
    };

    if is_probe {
        return style;
    }

    Some(style.unwrap_or_else(|| {
        StyleBuilder::for_inheritance(
            doc_data.stylist.device(),
            styles.primary(),
            Some(pseudo),
        ).build()
    }))
}

#[no_mangle]
pub extern "C" fn Servo_ComputedValues_Inherit(
    raw_data: RawServoStyleSetBorrowed,
    pseudo_tag: *mut nsAtom,
    parent_style_context: ServoStyleContextBorrowedOrNull,
    target: structs::InheritTarget
) -> ServoStyleContextStrong {
    let data = PerDocumentStyleData::from_ffi(raw_data).borrow();

    let for_text = target == structs::InheritTarget::Text;
    let atom = Atom::from(pseudo_tag);
    let pseudo = PseudoElement::from_anon_box_atom(&atom)
        .expect("Not an anon-box? Gah!");
    let style = if let Some(reference) = parent_style_context {
        let mut style = StyleBuilder::for_inheritance(
            data.stylist.device(),
            reference,
            Some(&pseudo)
        );

        if for_text {
            StyleAdjuster::new(&mut style)
                .adjust_for_text();
        }

        style.build()
    } else {
        debug_assert!(!for_text);
        StyleBuilder::for_derived_style(
            data.stylist.device(),
            data.default_computed_values(),
            /* parent_style = */ None,
            Some(&pseudo),
        ).build()
    };

    style.into()
}

#[no_mangle]
pub extern "C" fn Servo_ComputedValues_GetStyleBits(values: ServoStyleContextBorrowed) -> u64 {
    use style::properties::computed_value_flags::ComputedValueFlags;
    // FIXME(emilio): We could do this more efficiently I'm quite sure.
    let flags = values.flags;
    let mut result = 0;
    if flags.contains(ComputedValueFlags::IS_RELEVANT_LINK_VISITED) {
        result |= structs::NS_STYLE_RELEVANT_LINK_VISITED as u64;
    }
    if flags.contains(ComputedValueFlags::HAS_TEXT_DECORATION_LINES) {
        result |= structs::NS_STYLE_HAS_TEXT_DECORATION_LINES as u64;
    }
    if flags.contains(ComputedValueFlags::SHOULD_SUPPRESS_LINEBREAK) {
        result |= structs::NS_STYLE_SUPPRESS_LINEBREAK as u64;
    }
    if flags.contains(ComputedValueFlags::IS_TEXT_COMBINED) {
        result |= structs::NS_STYLE_IS_TEXT_COMBINED as u64;
    }
    if flags.contains(ComputedValueFlags::IS_IN_PSEUDO_ELEMENT_SUBTREE) {
        result |= structs::NS_STYLE_HAS_PSEUDO_ELEMENT_DATA as u64;
    }
    if flags.contains(ComputedValueFlags::IS_IN_DISPLAY_NONE_SUBTREE) {
        result |= structs::NS_STYLE_IN_DISPLAY_NONE_SUBTREE as u64;
    }
    result
}

#[no_mangle]
pub extern "C" fn Servo_ComputedValues_SpecifiesAnimationsOrTransitions(values: ServoStyleContextBorrowed)
                                                                        -> bool {
    let b = values.get_box();
    b.specifies_animations() || b.specifies_transitions()
}

#[no_mangle]
pub extern "C" fn Servo_ComputedValues_EqualCustomProperties(
    first: ServoComputedDataBorrowed,
    second: ServoComputedDataBorrowed
) -> bool {
    first.custom_properties == second.custom_properties
}

#[no_mangle]
pub extern "C" fn Servo_ComputedValues_GetStyleRuleList(values: ServoStyleContextBorrowed,
                                                        rules: RawGeckoServoStyleRuleListBorrowedMut) {
    use smallvec::SmallVec;

    let rule_node = match values.rules {
        Some(ref r) => r,
        None => return,
    };

    let mut result = SmallVec::<[_; 10]>::new();
    for node in rule_node.self_and_ancestors() {
        let style_rule = match *node.style_source() {
            StyleSource::Style(ref rule) => rule,
            _ => continue,
        };

        // For the rules with any important declaration, we insert them into
        // rule tree twice, one for normal level and another for important
        // level. So, we skip the important one to keep the specificity order of
        // rules.
        if node.importance().important() {
            continue;
        }

        result.push(style_rule);
    }

    unsafe { rules.set_len(result.len() as u32) };
    for (ref src, ref mut dest) in result.into_iter().zip(rules.iter_mut()) {
        src.with_raw_offset_arc(|arc| {
            **dest = *Locked::<StyleRule>::arc_as_borrowed(arc);
        })
    }
}

/// See the comment in `Device` to see why it's ok to pass an owned reference to
/// the pres context (hint: the context outlives the StyleSet, that holds the
/// device alive).
#[no_mangle]
pub extern "C" fn Servo_StyleSet_Init(pres_context: RawGeckoPresContextOwned)
  -> *mut RawServoStyleSet {
    let data = Box::new(PerDocumentStyleData::new(pres_context));
    Box::into_raw(data) as *mut RawServoStyleSet
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_RebuildCachedData(raw_data: RawServoStyleSetBorrowed) {
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    data.stylist.device_mut().rebuild_cached_data();
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_Drop(data: RawServoStyleSetOwned) {
    let _ = data.into_box::<PerDocumentStyleData>();
}


/// Updating the stylesheets and redoing selector matching is always happens
/// before the document element is inserted. Therefore we don't need to call
/// `force_dirty` here.
#[no_mangle]
pub extern "C" fn Servo_StyleSet_CompatModeChanged(raw_data: RawServoStyleSetBorrowed) {
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    let quirks_mode = unsafe {
        (*data.stylist.device().pres_context().mDocument.raw::<nsIDocument>())
            .mCompatMode
    };

    data.stylist.set_quirks_mode(quirks_mode.into());
}

fn parse_property_into<R>(
    declarations: &mut SourcePropertyDeclaration,
    property_id: PropertyId,
    value: *const nsACString,
    data: *mut URLExtraData,
    parsing_mode: structs::ParsingMode,
    quirks_mode: QuirksMode,
    reporter: &R
) -> Result<(), ()>
where
    R: ParseErrorReporter
{
    use style_traits::ParsingMode;
    let value = unsafe { value.as_ref().unwrap().as_str_unchecked() };
    let url_data = unsafe { RefPtr::from_ptr_ref(&data) };
    let parsing_mode = ParsingMode::from_bits_truncate(parsing_mode);

    parse_one_declaration_into(
        declarations,
        property_id,
        value,
        url_data,
        reporter,
        parsing_mode,
        quirks_mode,
    )
}

#[no_mangle]
pub extern "C" fn Servo_ParseProperty(
    property: nsCSSPropertyID, value: *const nsACString,
    data: *mut URLExtraData,
    parsing_mode: structs::ParsingMode,
    quirks_mode: nsCompatibility,
    loader: *mut Loader,
) -> RawServoDeclarationBlockStrong {
    let id = get_property_id_from_nscsspropertyid!(property,
                                                   RawServoDeclarationBlockStrong::null());
    let mut declarations = SourcePropertyDeclaration::new();
    let reporter = ErrorReporter::new(ptr::null_mut(), loader, data);
    match parse_property_into(&mut declarations, id, value, data,
                              parsing_mode, quirks_mode.into(), &reporter) {
        Ok(()) => {
            let global_style_data = &*GLOBAL_STYLE_DATA;
            let mut block = PropertyDeclarationBlock::new();
            block.extend(
                declarations.drain(),
                Importance::Normal,
                DeclarationSource::CssOm,
            );
            Arc::new(global_style_data.shared_lock.wrap(block)).into_strong()
        }
        Err(_) => RawServoDeclarationBlockStrong::null()
    }
}

#[no_mangle]
pub extern "C" fn Servo_ParseEasing(
    easing: *const nsAString,
    data: *mut URLExtraData,
    output: nsTimingFunctionBorrowedMut
) -> bool {
    use style::properties::longhands::transition_timing_function;

    // FIXME Dummy URL data would work fine here.
    let url_data = unsafe { RefPtr::from_ptr_ref(&data) };
    let context = ParserContext::new(
        Origin::Author,
        url_data,
        Some(CssRuleType::Style),
        ParsingMode::DEFAULT,
        QuirksMode::NoQuirks,
    );
    let easing = unsafe { (*easing).to_string() };
    let mut input = ParserInput::new(&easing);
    let mut parser = Parser::new(&mut input);
    let result =
        parser.parse_entirely(|p| transition_timing_function::single_value::parse(&context, p));
    match result {
        Ok(parsed_easing) => {
            *output = parsed_easing.into();
            true
        },
        Err(_) => false
    }
}

#[no_mangle]
pub extern "C" fn Servo_GetProperties_Overriding_Animation(element: RawGeckoElementBorrowed,
                                                           list: RawGeckoCSSPropertyIDListBorrowed,
                                                           set: nsCSSPropertyIDSetBorrowedMut) {
    let element = GeckoElement(element);
    let element_data = match element.borrow_data() {
        Some(data) => data,
        None => return
    };
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let guards = StylesheetGuards::same(&guard);
    let (overridden, custom) =
        element_data.styles.primary().rules().get_properties_overriding_animations(&guards);
    for p in list.iter() {
        match PropertyId::from_nscsspropertyid(*p) {
            Ok(property) => {
                if let PropertyId::Longhand(id) = property {
                    if overridden.contains(id) {
                        unsafe { Gecko_AddPropertyToSet(set, *p) };
                    }
                }
            },
            Err(_) => {
                if *p == nsCSSPropertyID::eCSSPropertyExtra_variable && custom {
                    unsafe { Gecko_AddPropertyToSet(set, *p) };
                }
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn Servo_MatrixTransform_Operate(matrix_operator: MatrixTransformOperator,
                                                from: *const RawGeckoGfxMatrix4x4,
                                                to: *const RawGeckoGfxMatrix4x4,
                                                progress: f64,
                                                output: *mut RawGeckoGfxMatrix4x4) {
    use self::MatrixTransformOperator::{Accumulate, Interpolate};
    use style::values::computed::transform::Matrix3D;

    let from = Matrix3D::from(unsafe { from.as_ref() }.expect("not a valid 'from' matrix"));
    let to = Matrix3D::from(unsafe { to.as_ref() }.expect("not a valid 'to' matrix"));
    let result = match matrix_operator {
        Interpolate => from.animate(&to, Procedure::Interpolate { progress }),
        Accumulate => from.animate(&to, Procedure::Accumulate { count: progress as u64 }),
    };

    let output = unsafe { output.as_mut() }.expect("not a valid 'output' matrix");
    if let Ok(result) = result {
        *output = result.into();
    } else if progress < 0.5 {
        *output = from.clone().into();
    } else {
        *output = to.clone().into();
    }
}

#[no_mangle]
pub extern "C" fn Servo_ParseStyleAttribute(
    data: *const nsACString,
    raw_extra_data: *mut URLExtraData,
    quirks_mode: nsCompatibility,
    loader: *mut Loader,
) -> RawServoDeclarationBlockStrong {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let value = unsafe { data.as_ref().unwrap().as_str_unchecked() };
    let reporter = ErrorReporter::new(ptr::null_mut(), loader, raw_extra_data);
    let url_data = unsafe { RefPtr::from_ptr_ref(&raw_extra_data) };
    Arc::new(global_style_data.shared_lock.wrap(
        parse_style_attribute(
            value,
            url_data,
            &reporter,
            quirks_mode.into(),
        )
    )).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_CreateEmpty() -> RawServoDeclarationBlockStrong {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    Arc::new(global_style_data.shared_lock.wrap(PropertyDeclarationBlock::new())).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_Clone(declarations: RawServoDeclarationBlockBorrowed)
                                               -> RawServoDeclarationBlockStrong {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let declarations = Locked::<PropertyDeclarationBlock>::as_arc(&declarations);
    Arc::new(global_style_data.shared_lock.wrap(
        declarations.read_with(&guard).clone()
    )).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_Equals(
    a: RawServoDeclarationBlockBorrowed,
    b: RawServoDeclarationBlockBorrowed,
) -> bool {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    *Locked::<PropertyDeclarationBlock>::as_arc(&a).read_with(&guard).declarations() ==
    *Locked::<PropertyDeclarationBlock>::as_arc(&b).read_with(&guard).declarations()
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_GetCssText(declarations: RawServoDeclarationBlockBorrowed,
                                                    result: *mut nsAString) {
    read_locked_arc(declarations, |decls: &PropertyDeclarationBlock| {
        decls.to_css(unsafe { result.as_mut().unwrap() }).unwrap();
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SerializeOneValue(
    declarations: RawServoDeclarationBlockBorrowed,
    property_id: nsCSSPropertyID, buffer: *mut nsAString,
    computed_values: ServoStyleContextBorrowedOrNull,
    custom_properties: RawServoDeclarationBlockBorrowedOrNull,
) {
    let property_id = get_property_id_from_nscsspropertyid!(property_id, ());

    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let decls = Locked::<PropertyDeclarationBlock>::as_arc(&declarations).read_with(&guard);

    let mut string = String::new();

    let custom_properties = Locked::<PropertyDeclarationBlock>::arc_from_borrowed(&custom_properties);
    let custom_properties = custom_properties.map(|block| block.read_with(&guard));
    let rv = decls.single_value_to_css(
        &property_id, &mut string, computed_values, custom_properties);
    debug_assert!(rv.is_ok());

    let buffer = unsafe { buffer.as_mut().unwrap() };
    buffer.assign_utf8(&string);
}

#[no_mangle]
pub extern "C" fn Servo_SerializeFontValueForCanvas(
    declarations: RawServoDeclarationBlockBorrowed,
    buffer: *mut nsAString,
) {
    use style::properties::shorthands::font;

    read_locked_arc(declarations, |decls: &PropertyDeclarationBlock| {
        let longhands = match font::LonghandsToSerialize::from_iter(decls.declarations().iter()) {
            Ok(l) => l,
            Err(()) => {
                warn!("Unexpected property!");
                return;
            }
        };

        let mut string = String::new();
        let rv = longhands.to_css_for_canvas(&mut string);
        debug_assert!(rv.is_ok());

        let buffer = unsafe { buffer.as_mut().unwrap() };
        buffer.assign_utf8(&string);
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_Count(declarations: RawServoDeclarationBlockBorrowed) -> u32 {
    read_locked_arc(declarations, |decls: &PropertyDeclarationBlock| {
        decls.declarations().len() as u32
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_GetNthProperty(
    declarations: RawServoDeclarationBlockBorrowed,
    index: u32,
    result: *mut nsAString,
) -> bool {
    read_locked_arc(declarations, |decls: &PropertyDeclarationBlock| {
        if let Some(decl) = decls.declarations().get(index as usize) {
            let result = unsafe { result.as_mut().unwrap() };
            result.assign_utf8(&decl.id().name());
            true
        } else {
            false
        }
    })
}

macro_rules! get_property_id_from_property {
    ($property: ident, $ret: expr) => {{
        let property = $property.as_ref().unwrap().as_str_unchecked();
        match PropertyId::parse(property.into()) {
            Ok(property_id) => property_id,
            Err(_) => { return $ret; }
        }
    }}
}

unsafe fn get_property_value(
    declarations: RawServoDeclarationBlockBorrowed,
    property_id: PropertyId,
    value: *mut nsAString,
) {
    // This callsite is hot enough that the lock acquisition shows up in profiles.
    // Using an unchecked read here improves our performance by ~10% on the
    // microbenchmark in bug 1355599.
    read_locked_arc_unchecked(declarations, |decls: &PropertyDeclarationBlock| {
        decls.property_value_to_css(&property_id, value.as_mut().unwrap()).unwrap();
    })
}

#[no_mangle]
pub unsafe extern "C" fn Servo_DeclarationBlock_GetPropertyValue(
    declarations: RawServoDeclarationBlockBorrowed,
    property: *const nsACString,
    value: *mut nsAString,
) {
    get_property_value(
        declarations,
        get_property_id_from_property!(property, ()),
        value,
    )
}

#[no_mangle]
pub unsafe extern "C" fn Servo_DeclarationBlock_GetPropertyValueById(
    declarations: RawServoDeclarationBlockBorrowed,
    property: nsCSSPropertyID,
    value: *mut nsAString,
) {
    get_property_value(declarations, get_property_id_from_nscsspropertyid!(property, ()), value)
}

#[no_mangle]
pub unsafe extern "C" fn Servo_DeclarationBlock_GetPropertyIsImportant(
    declarations: RawServoDeclarationBlockBorrowed,
    property: *const nsACString,
) -> bool {
    let property_id = get_property_id_from_property!(property, false);
    read_locked_arc(declarations, |decls: &PropertyDeclarationBlock| {
        decls.property_priority(&property_id).important()
    })
}

fn set_property(
    declarations: RawServoDeclarationBlockBorrowed,
    property_id: PropertyId,
    value: *const nsACString,
    is_important: bool,
    data: *mut URLExtraData,
    parsing_mode: structs::ParsingMode,
    quirks_mode: QuirksMode,
    loader: *mut Loader
) -> bool {
    let mut source_declarations = SourcePropertyDeclaration::new();
    let reporter = ErrorReporter::new(ptr::null_mut(), loader, data);
    match parse_property_into(&mut source_declarations, property_id, value, data,
                              parsing_mode, quirks_mode, &reporter) {
        Ok(()) => {
            let importance = if is_important { Importance::Important } else { Importance::Normal };
            write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
                decls.extend(
                    source_declarations.drain(),
                    importance,
                    DeclarationSource::CssOm
                )
            })
        },
        Err(_) => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn Servo_DeclarationBlock_SetProperty(
    declarations: RawServoDeclarationBlockBorrowed,
    property: *const nsACString,
    value: *const nsACString,
    is_important: bool,
    data: *mut URLExtraData,
    parsing_mode: structs::ParsingMode,
    quirks_mode: nsCompatibility,
    loader: *mut Loader,
) -> bool {
    set_property(
        declarations,
        get_property_id_from_property!(property, false),
        value,
        is_important,
        data,
        parsing_mode,
        quirks_mode.into(),
        loader,
    )
}

#[no_mangle]
pub unsafe extern "C" fn Servo_DeclarationBlock_SetPropertyById(
    declarations: RawServoDeclarationBlockBorrowed,
    property: nsCSSPropertyID,
    value: *const nsACString,
    is_important: bool,
    data: *mut URLExtraData,
    parsing_mode: structs::ParsingMode,
    quirks_mode: nsCompatibility,
    loader: *mut Loader,
) -> bool {
    set_property(
        declarations,
        get_property_id_from_nscsspropertyid!(property, false),
        value,
        is_important,
        data,
        parsing_mode,
        quirks_mode.into(),
        loader,
    )
}

fn remove_property(
    declarations: RawServoDeclarationBlockBorrowed,
    property_id: PropertyId
) -> bool {
    write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
        decls.remove_property(&property_id)
    })
}

#[no_mangle]
pub unsafe extern "C" fn Servo_DeclarationBlock_RemoveProperty(
    declarations: RawServoDeclarationBlockBorrowed,
    property: *const nsACString,
) {
    remove_property(declarations, get_property_id_from_property!(property, ()));
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_RemovePropertyById(
    declarations: RawServoDeclarationBlockBorrowed,
    property: nsCSSPropertyID
) -> bool {
    remove_property(declarations, get_property_id_from_nscsspropertyid!(property, false))
}

#[no_mangle]
pub extern "C" fn Servo_MediaList_Create() -> RawServoMediaListStrong {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    Arc::new(global_style_data.shared_lock.wrap(MediaList::empty())).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_MediaList_DeepClone(list: RawServoMediaListBorrowed) -> RawServoMediaListStrong {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    read_locked_arc(list, |list: &MediaList| {
        Arc::new(global_style_data.shared_lock.wrap(list.clone()))
            .into_strong()
    })
}

#[no_mangle]
pub extern "C" fn Servo_MediaList_Matches(
    list: RawServoMediaListBorrowed,
    raw_data: RawServoStyleSetBorrowed,
) -> bool {
    let per_doc_data = PerDocumentStyleData::from_ffi(raw_data).borrow();
    read_locked_arc(list, |list: &MediaList| {
        list.evaluate(per_doc_data.stylist.device(), per_doc_data.stylist.quirks_mode())
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_HasCSSWideKeyword(
    declarations: RawServoDeclarationBlockBorrowed,
    property: nsCSSPropertyID,
) -> bool {
    let property_id = get_property_id_from_nscsspropertyid!(property, false);
    read_locked_arc(declarations, |decls: &PropertyDeclarationBlock| {
        decls.has_css_wide_keyword(&property_id)
    })
}

#[no_mangle]
pub extern "C" fn Servo_MediaList_GetText(list: RawServoMediaListBorrowed, result: *mut nsAString) {
    read_locked_arc(list, |list: &MediaList| {
        list.to_css(unsafe { result.as_mut().unwrap() }).unwrap();
    })
}

#[no_mangle]
pub unsafe extern "C" fn Servo_MediaList_SetText(
    list: RawServoMediaListBorrowed,
    text: *const nsACString,
    caller_type: CallerType,
) {
    let text = (*text).as_str_unchecked();

    let mut input = ParserInput::new(&text);
    let mut parser = Parser::new(&mut input);
    let url_data = dummy_url_data();

    // TODO(emilio): If the need for `CallerType` appears in more places,
    // consider adding an explicit member in `ParserContext` instead of doing
    // this (or adding a dummy "chrome://" url data).
    //
    // For media query parsing it's effectively the same, so for now...
    let origin = match caller_type {
        CallerType::System => Origin::UserAgent,
        CallerType::NonSystem => Origin::Author,
    };

    let context = ParserContext::new(
        origin,
        url_data,
        Some(CssRuleType::Media),
        ParsingMode::DEFAULT,
        QuirksMode::NoQuirks,
    );

    write_locked_arc(list, |list: &mut MediaList| {
        *list = parse_media_query_list(
            &context,
            &mut parser,
            &NullReporter,
        );
    })
}

#[no_mangle]
pub extern "C" fn Servo_MediaList_GetLength(list: RawServoMediaListBorrowed) -> u32 {
    read_locked_arc(list, |list: &MediaList| list.media_queries.len() as u32)
}

#[no_mangle]
pub extern "C" fn Servo_MediaList_GetMediumAt(
    list: RawServoMediaListBorrowed,
    index: u32,
    result: *mut nsAString,
) -> bool {
    read_locked_arc(list, |list: &MediaList| {
        if let Some(media_query) = list.media_queries.get(index as usize) {
            media_query.to_css(unsafe { result.as_mut().unwrap() }).unwrap();
            true
        } else {
            false
        }
    })
}

#[no_mangle]
pub extern "C" fn Servo_MediaList_AppendMedium(
    list: RawServoMediaListBorrowed,
    new_medium: *const nsACString,
) {
    let new_medium = unsafe { new_medium.as_ref().unwrap().as_str_unchecked() };
    let url_data = unsafe { dummy_url_data() };
    let context = ParserContext::new_for_cssom(
        url_data,
        Some(CssRuleType::Media),
        ParsingMode::DEFAULT,
        QuirksMode::NoQuirks,
    );
    write_locked_arc(list, |list: &mut MediaList| {
        list.append_medium(&context, new_medium);
    })
}

#[no_mangle]
pub extern "C" fn Servo_MediaList_DeleteMedium(
    list: RawServoMediaListBorrowed,
    old_medium: *const nsACString,
) -> bool {
    let old_medium = unsafe { old_medium.as_ref().unwrap().as_str_unchecked() };
    let url_data = unsafe { dummy_url_data() };
    let context = ParserContext::new_for_cssom(
        url_data,
        Some(CssRuleType::Media),
        ParsingMode::DEFAULT,
        QuirksMode::NoQuirks,
    );
    write_locked_arc(list, |list: &mut MediaList| list.delete_medium(&context, old_medium))
}

macro_rules! get_longhand_from_id {
    ($id:expr) => {
        match PropertyId::from_nscsspropertyid($id) {
            Ok(PropertyId::Longhand(long)) => long,
            _ => {
                panic!("stylo: unknown presentation property with id {:?}", $id);
            }
        }
    };
}

macro_rules! match_wrap_declared {
    ($longhand:ident, $($property:ident => $inner:expr,)*) => (
        match $longhand {
            $(
                LonghandId::$property => PropertyDeclaration::$property($inner),
            )*
            _ => {
                panic!("stylo: Don't know how to handle presentation property {:?}", $longhand);
            }
        }
    )
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_PropertyIsSet(declarations:
                                                       RawServoDeclarationBlockBorrowed,
                                                       property: nsCSSPropertyID)
        -> bool {
    read_locked_arc(declarations, |decls: &PropertyDeclarationBlock| {
        decls.contains(get_longhand_from_id!(property))
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetIdentStringValue(
    declarations: RawServoDeclarationBlockBorrowed,
    property: nsCSSPropertyID,
    value: *mut nsAtom,
) {
    use style::properties::{PropertyDeclaration, LonghandId};
    use style::properties::longhands::_x_lang::computed_value::T as Lang;

    let long = get_longhand_from_id!(property);
    let prop = match_wrap_declared! { long,
        XLang => Lang(Atom::from(value)),
    };
    write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
        decls.push(prop, Importance::Normal, DeclarationSource::CssOm);
    })
}

#[no_mangle]
#[allow(unreachable_code)]
pub extern "C" fn Servo_DeclarationBlock_SetKeywordValue(
    declarations: RawServoDeclarationBlockBorrowed,
    property: nsCSSPropertyID,
    value: i32
) {
    use style::properties::{PropertyDeclaration, LonghandId};
    use style::properties::longhands;
    use style::values::specified::BorderStyle;

    let long = get_longhand_from_id!(property);
    let value = value as u32;

    let prop = match_wrap_declared! { long,
        MozUserModify => longhands::_moz_user_modify::SpecifiedValue::from_gecko_keyword(value),
        // TextEmphasisPosition => FIXME implement text-emphasis-position
        Direction => longhands::direction::SpecifiedValue::from_gecko_keyword(value),
        Display => longhands::display::SpecifiedValue::from_gecko_keyword(value),
        Float => longhands::float::SpecifiedValue::from_gecko_keyword(value),
        VerticalAlign => longhands::vertical_align::SpecifiedValue::from_gecko_keyword(value),
        TextAlign => longhands::text_align::SpecifiedValue::from_gecko_keyword(value),
        TextEmphasisPosition => longhands::text_emphasis_position::SpecifiedValue::from_gecko_keyword(value),
        Clear => longhands::clear::SpecifiedValue::from_gecko_keyword(value),
        FontSize => {
            // We rely on Gecko passing in font-size values (0...7) here.
            longhands::font_size::SpecifiedValue::from_html_size(value as u8)
        },
        FontStyle => {
            ToComputedValue::from_computed_value(&longhands::font_style::computed_value::T::from_gecko_keyword(value))
        },
        FontWeight => longhands::font_weight::SpecifiedValue::from_gecko_keyword(value),
        ListStyleType => Box::new(longhands::list_style_type::SpecifiedValue::from_gecko_keyword(value)),
        MozMathVariant => longhands::_moz_math_variant::SpecifiedValue::from_gecko_keyword(value),
        WhiteSpace => longhands::white_space::SpecifiedValue::from_gecko_keyword(value),
        CaptionSide => longhands::caption_side::SpecifiedValue::from_gecko_keyword(value),
        BorderTopStyle => BorderStyle::from_gecko_keyword(value),
        BorderRightStyle => BorderStyle::from_gecko_keyword(value),
        BorderBottomStyle => BorderStyle::from_gecko_keyword(value),
        BorderLeftStyle => BorderStyle::from_gecko_keyword(value),
    };
    write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
        decls.push(prop, Importance::Normal, DeclarationSource::CssOm);
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetIntValue(
    declarations: RawServoDeclarationBlockBorrowed,
    property: nsCSSPropertyID,
    value: i32
) {
    use style::properties::{PropertyDeclaration, LonghandId};
    use style::properties::longhands::_moz_script_level::SpecifiedValue as MozScriptLevel;
    use style::properties::longhands::_x_span::computed_value::T as Span;

    let long = get_longhand_from_id!(property);
    let prop = match_wrap_declared! { long,
        XSpan => Span(value),
        // Gecko uses Integer values to signal that it is relative
        MozScriptLevel => MozScriptLevel::Relative(value),
    };
    write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
        decls.push(prop, Importance::Normal, DeclarationSource::CssOm);
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetPixelValue(
    declarations: RawServoDeclarationBlockBorrowed,
    property: nsCSSPropertyID,
    value: f32
) {
    use style::properties::{PropertyDeclaration, LonghandId};
    use style::properties::longhands::border_spacing::SpecifiedValue as BorderSpacing;
    use style::properties::longhands::height::SpecifiedValue as Height;
    use style::properties::longhands::width::SpecifiedValue as Width;
    use style::values::specified::{BorderSideWidth, MozLength, BorderCornerRadius};
    use style::values::specified::length::{NoCalcLength, NonNegativeLength, LengthOrPercentage};

    let long = get_longhand_from_id!(property);
    let nocalc = NoCalcLength::from_px(value);

    let prop = match_wrap_declared! { long,
        Height => Height(MozLength::LengthOrPercentageOrAuto(nocalc.into())),
        Width => Width(MozLength::LengthOrPercentageOrAuto(nocalc.into())),
        BorderTopWidth => BorderSideWidth::Length(nocalc.into()),
        BorderRightWidth => BorderSideWidth::Length(nocalc.into()),
        BorderBottomWidth => BorderSideWidth::Length(nocalc.into()),
        BorderLeftWidth => BorderSideWidth::Length(nocalc.into()),
        MarginTop => nocalc.into(),
        MarginRight => nocalc.into(),
        MarginBottom => nocalc.into(),
        MarginLeft => nocalc.into(),
        PaddingTop => nocalc.into(),
        PaddingRight => nocalc.into(),
        PaddingBottom => nocalc.into(),
        PaddingLeft => nocalc.into(),
        BorderSpacing => {
            let v = NonNegativeLength::from(nocalc);
            Box::new(BorderSpacing::new(v.clone(), v))
        },
        BorderTopLeftRadius => {
            let length = LengthOrPercentage::from(nocalc);
            Box::new(BorderCornerRadius::new(length.clone(), length))
        },
        BorderTopRightRadius => {
            let length = LengthOrPercentage::from(nocalc);
            Box::new(BorderCornerRadius::new(length.clone(), length))
        },
        BorderBottomLeftRadius => {
            let length = LengthOrPercentage::from(nocalc);
            Box::new(BorderCornerRadius::new(length.clone(), length))
        },
        BorderBottomRightRadius => {
            let length = LengthOrPercentage::from(nocalc);
            Box::new(BorderCornerRadius::new(length.clone(), length))
        },
    };
    write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
        decls.push(prop, Importance::Normal, DeclarationSource::CssOm);
    })
}


#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetLengthValue(
    declarations: RawServoDeclarationBlockBorrowed,
    property: nsCSSPropertyID,
    value: f32,
    unit: structs::nsCSSUnit,
) {
    use style::properties::{PropertyDeclaration, LonghandId};
    use style::properties::longhands::_moz_script_min_size::SpecifiedValue as MozScriptMinSize;
    use style::properties::longhands::width::SpecifiedValue as Width;
    use style::values::specified::MozLength;
    use style::values::specified::length::{AbsoluteLength, FontRelativeLength};
    use style::values::specified::length::{LengthOrPercentage, NoCalcLength};

    let long = get_longhand_from_id!(property);
    let nocalc = match unit {
        structs::nsCSSUnit::eCSSUnit_EM => NoCalcLength::FontRelative(FontRelativeLength::Em(value)),
        structs::nsCSSUnit::eCSSUnit_XHeight => NoCalcLength::FontRelative(FontRelativeLength::Ex(value)),
        structs::nsCSSUnit::eCSSUnit_Pixel => NoCalcLength::Absolute(AbsoluteLength::Px(value)),
        structs::nsCSSUnit::eCSSUnit_Inch => NoCalcLength::Absolute(AbsoluteLength::In(value)),
        structs::nsCSSUnit::eCSSUnit_Centimeter => NoCalcLength::Absolute(AbsoluteLength::Cm(value)),
        structs::nsCSSUnit::eCSSUnit_Millimeter => NoCalcLength::Absolute(AbsoluteLength::Mm(value)),
        structs::nsCSSUnit::eCSSUnit_Point => NoCalcLength::Absolute(AbsoluteLength::Pt(value)),
        structs::nsCSSUnit::eCSSUnit_Pica => NoCalcLength::Absolute(AbsoluteLength::Pc(value)),
        structs::nsCSSUnit::eCSSUnit_Quarter => NoCalcLength::Absolute(AbsoluteLength::Q(value)),
        _ => unreachable!("Unknown unit {:?} passed to SetLengthValue", unit)
    };

    let prop = match_wrap_declared! { long,
        Width => Width(MozLength::LengthOrPercentageOrAuto(nocalc.into())),
        FontSize => LengthOrPercentage::from(nocalc).into(),
        MozScriptMinSize => MozScriptMinSize(nocalc),
    };
    write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
        decls.push(prop, Importance::Normal, DeclarationSource::CssOm);
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetNumberValue(
    declarations: RawServoDeclarationBlockBorrowed,
    property: nsCSSPropertyID,
    value: f32,
) {
    use style::properties::{PropertyDeclaration, LonghandId};
    use style::properties::longhands::_moz_script_level::SpecifiedValue as MozScriptLevel;
    use style::properties::longhands::_moz_script_size_multiplier::SpecifiedValue as MozScriptSizeMultiplier;

    let long = get_longhand_from_id!(property);

    let prop = match_wrap_declared! { long,
        MozScriptSizeMultiplier => MozScriptSizeMultiplier(value),
        // Gecko uses Number values to signal that it is absolute
        MozScriptLevel => MozScriptLevel::MozAbsolute(value as i32),
    };
    write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
        decls.push(prop, Importance::Normal, DeclarationSource::CssOm);
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetPercentValue(
    declarations: RawServoDeclarationBlockBorrowed,
    property: nsCSSPropertyID,
    value: f32,
) {
    use style::properties::{PropertyDeclaration, LonghandId};
    use style::properties::longhands::height::SpecifiedValue as Height;
    use style::properties::longhands::width::SpecifiedValue as Width;
    use style::values::computed::Percentage;
    use style::values::specified::MozLength;
    use style::values::specified::length::LengthOrPercentage;

    let long = get_longhand_from_id!(property);
    let pc = Percentage(value);

    let prop = match_wrap_declared! { long,
        Height => Height(MozLength::LengthOrPercentageOrAuto(pc.into())),
        Width => Width(MozLength::LengthOrPercentageOrAuto(pc.into())),
        MarginTop => pc.into(),
        MarginRight => pc.into(),
        MarginBottom => pc.into(),
        MarginLeft => pc.into(),
        FontSize => LengthOrPercentage::from(pc).into(),
    };
    write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
        decls.push(prop, Importance::Normal, DeclarationSource::CssOm);
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetAutoValue(
    declarations: RawServoDeclarationBlockBorrowed,
    property: nsCSSPropertyID,
) {
    use style::properties::{PropertyDeclaration, LonghandId};
    use style::properties::longhands::height::SpecifiedValue as Height;
    use style::properties::longhands::width::SpecifiedValue as Width;
    use style::values::specified::LengthOrPercentageOrAuto;

    let long = get_longhand_from_id!(property);
    let auto = LengthOrPercentageOrAuto::Auto;

    let prop = match_wrap_declared! { long,
        Height => Height::auto(),
        Width => Width::auto(),
        MarginTop => auto,
        MarginRight => auto,
        MarginBottom => auto,
        MarginLeft => auto,
    };
    write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
        decls.push(prop, Importance::Normal, DeclarationSource::CssOm);
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetCurrentColor(
    declarations: RawServoDeclarationBlockBorrowed,
    property: nsCSSPropertyID,
) {
    use style::properties::{PropertyDeclaration, LonghandId};
    use style::values::specified::Color;

    let long = get_longhand_from_id!(property);
    let cc = Color::currentcolor();

    let prop = match_wrap_declared! { long,
        BorderTopColor => cc,
        BorderRightColor => cc,
        BorderBottomColor => cc,
        BorderLeftColor => cc,
    };
    write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
        decls.push(prop, Importance::Normal, DeclarationSource::CssOm);
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetColorValue(
    declarations: RawServoDeclarationBlockBorrowed,
    property: nsCSSPropertyID,
    value: structs::nscolor,
) {
    use style::gecko::values::convert_nscolor_to_rgba;
    use style::properties::{PropertyDeclaration, LonghandId};
    use style::properties::longhands;
    use style::values::specified::Color;

    let long = get_longhand_from_id!(property);
    let rgba = convert_nscolor_to_rgba(value);
    let color = Color::rgba(rgba);

    let prop = match_wrap_declared! { long,
        BorderTopColor => color,
        BorderRightColor => color,
        BorderBottomColor => color,
        BorderLeftColor => color,
        Color => longhands::color::SpecifiedValue(color),
        BackgroundColor => color,
    };
    write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
        decls.push(prop, Importance::Normal, DeclarationSource::CssOm);
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetFontFamily(
    declarations: RawServoDeclarationBlockBorrowed,
    value: *const nsAString,
) {
    use cssparser::{Parser, ParserInput};
    use style::properties::PropertyDeclaration;
    use style::properties::longhands::font_family::SpecifiedValue as FontFamily;

    let string = unsafe { (*value).to_string() };
    let mut input = ParserInput::new(&string);
    let mut parser = Parser::new(&mut input);
    let result = FontFamily::parse_specified(&mut parser);
    if let Ok(family) = result {
        if parser.is_exhausted() {
            let decl = PropertyDeclaration::FontFamily(family);
            write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
                decls.push(decl, Importance::Normal, DeclarationSource::CssOm);
            })
        }
    }
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetBackgroundImage(
    declarations: RawServoDeclarationBlockBorrowed,
    value: *const nsAString,
    raw_extra_data: *mut URLExtraData,
) {
    use style::properties::PropertyDeclaration;
    use style::properties::longhands::background_image::SpecifiedValue as BackgroundImage;
    use style::values::Either;
    use style::values::generics::image::Image;
    use style::values::specified::url::SpecifiedUrl;

    let url_data = unsafe { RefPtr::from_ptr_ref(&raw_extra_data) };
    let string = unsafe { (*value).to_string() };
    let context = ParserContext::new(
        Origin::Author,
        url_data,
        Some(CssRuleType::Style),
        ParsingMode::DEFAULT,
        QuirksMode::NoQuirks,
    );
    if let Ok(mut url) = SpecifiedUrl::parse_from_string(string.into(), &context) {
        url.build_image_value();
        let decl = PropertyDeclaration::BackgroundImage(BackgroundImage(
            vec![Either::Second(Image::Url(url))]
        ));
        write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
            decls.push(decl, Importance::Normal, DeclarationSource::CssOm);
        })
    }
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetTextDecorationColorOverride(
    declarations: RawServoDeclarationBlockBorrowed,
) {
    use style::properties::PropertyDeclaration;
    use style::values::specified::text::TextDecorationLine;

    let mut decoration = TextDecorationLine::none();
    decoration |= TextDecorationLine::COLOR_OVERRIDE;
    let decl = PropertyDeclaration::TextDecorationLine(decoration);
    write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
        decls.push(decl, Importance::Normal, DeclarationSource::CssOm);
    })
}

#[no_mangle]
pub unsafe extern "C" fn Servo_CSSSupports2(
    property: *const nsACString,
    value: *const nsACString,
) -> bool {
    let id = get_property_id_from_property!(property, false);

    let mut declarations = SourcePropertyDeclaration::new();
    parse_property_into(
        &mut declarations,
        id,
        value,
        DUMMY_URL_DATA,
        structs::ParsingMode_Default,
        QuirksMode::NoQuirks,
        &NullReporter,
    ).is_ok()
}

#[no_mangle]
pub extern "C" fn Servo_CSSSupports(cond: *const nsACString) -> bool {
    let condition = unsafe { cond.as_ref().unwrap().as_str_unchecked() };
    let mut input = ParserInput::new(&condition);
    let mut input = Parser::new(&mut input);
    let cond = input.parse_entirely(|i| parse_condition_or_declaration(i));
    if let Ok(cond) = cond {
        let url_data = unsafe { dummy_url_data() };
        // NOTE(emilio): The supports API is not associated to any stylesheet,
        // so the fact that there are no namespace map here is fine.
        let context = ParserContext::new_for_cssom(
            url_data,
            Some(CssRuleType::Style),
            ParsingMode::DEFAULT,
            QuirksMode::NoQuirks,
        );

        cond.eval(&context)
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn Servo_NoteExplicitHints(
    element: RawGeckoElementBorrowed,
    restyle_hint: nsRestyleHint,
    change_hint: nsChangeHint,
) {
    GeckoElement(element).note_explicit_hints(restyle_hint, change_hint);
}

#[no_mangle]
pub extern "C" fn Servo_TakeChangeHint(
    element: RawGeckoElementBorrowed,
    was_restyled: *mut bool
) -> u32 {
    let was_restyled =  unsafe { was_restyled.as_mut().unwrap() };
    let element = GeckoElement(element);

    let damage = match element.mutate_data() {
        Some(mut data) => {
            *was_restyled = data.is_restyle();

            let damage = data.damage;
            data.clear_restyle_state();
            damage
        }
        None => {
            warn!("Trying to get change hint from unstyled element");
            *was_restyled = false;
            GeckoRestyleDamage::empty()
        }
    };

    debug!("Servo_TakeChangeHint: {:?}, damage={:?}", element, damage);
    // We'd like to return `nsChangeHint` here, but bindgen bitfield enums don't
    // work as return values with the Linux 32-bit ABI at the moment because
    // they wrap the value in a struct, so for now just unwrap it.
    damage.as_change_hint().0
}

#[no_mangle]
pub extern "C" fn Servo_ResolveStyle(
    element: RawGeckoElementBorrowed,
    _raw_data: RawServoStyleSetBorrowed,
) -> ServoStyleContextStrong {
    let element = GeckoElement(element);
    debug!("Servo_ResolveStyle: {:?}", element);
    let data =
        element.borrow_data().expect("Resolving style on unstyled element");

    debug_assert!(element.has_current_styles(&*data),
                  "Resolving style on {:?} without current styles: {:?}", element, data);
    data.styles.primary().clone().into()
}

#[no_mangle]
pub extern "C" fn Servo_ResolveStyleLazily(
    element: RawGeckoElementBorrowed,
    pseudo_type: CSSPseudoElementType,
    rule_inclusion: StyleRuleInclusion,
    snapshots: *const ServoElementSnapshotTable,
    raw_data: RawServoStyleSetBorrowed,
    ignore_existing_styles: bool
) -> ServoStyleContextStrong {
    debug_assert!(!snapshots.is_null());
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let element = GeckoElement(element);
    let doc_data = PerDocumentStyleData::from_ffi(raw_data);
    let data = doc_data.borrow();
    let rule_inclusion = RuleInclusion::from(rule_inclusion);
    let pseudo = PseudoElement::from_pseudo_type(pseudo_type);
    let finish = |styles: &ElementStyles, is_probe: bool| -> Option<Arc<ComputedValues>> {
        match pseudo {
            Some(ref pseudo) => {
                get_pseudo_style(
                    &guard,
                    element,
                    pseudo,
                    rule_inclusion,
                    styles,
                    /* inherited_styles = */ None,
                    &*data,
                    is_probe,
                    /* matching_func = */ None,
                )
            }
            None => Some(styles.primary().clone()),
        }
    };

    let is_before_or_after = pseudo.as_ref().map_or(false, |p| p.is_before_or_after());

    // In the common case we already have the style. Check that before setting
    // up all the computation machinery. (Don't use it when we're getting
    // default styles or in a bfcached document (as indicated by
    // ignore_existing_styles), though.)
    //
    // Also, only probe in the ::before or ::after case, since their styles may
    // not be in the `ElementData`, given they may exist but not be applicable
    // to generate an actual pseudo-element (like, having a `content: none`).
    if rule_inclusion == RuleInclusion::All && !ignore_existing_styles {
        let styles = element.mutate_data().and_then(|d| {
            if d.has_styles() {
                finish(&d.styles, is_before_or_after)
            } else {
                None
            }
        });
        if let Some(result) = styles {
            return result.into();
        }
    }

    // We don't have the style ready. Go ahead and compute it as necessary.
    let shared = create_shared_context(&global_style_data,
                                       &guard,
                                       &data,
                                       TraversalFlags::empty(),
                                       unsafe { &*snapshots });
    let mut tlc = ThreadLocalStyleContext::new(&shared);
    let mut context = StyleContext {
        shared: &shared,
        thread_local: &mut tlc,
    };

    let styles = resolve_style(
        &mut context,
        element,
        rule_inclusion,
        ignore_existing_styles,
        pseudo.as_ref()
    );

    finish(&styles, /* is_probe = */ false)
        .expect("We're not probing, so we should always get a style back")
        .into()
}

#[no_mangle]
pub extern "C" fn Servo_ReparentStyle(
    style_to_reparent: ServoStyleContextBorrowed,
    parent_style: ServoStyleContextBorrowed,
    parent_style_ignoring_first_line: ServoStyleContextBorrowed,
    layout_parent_style: ServoStyleContextBorrowed,
    element: RawGeckoElementBorrowedOrNull,
    raw_data: RawServoStyleSetBorrowed,
) -> ServoStyleContextStrong {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let doc_data = PerDocumentStyleData::from_ffi(raw_data).borrow();
    let inputs = CascadeInputs::new_from_style(style_to_reparent);
    let metrics = get_metrics_provider_for_product();
    let pseudo = style_to_reparent.pseudo();
    let element = element.map(GeckoElement);

    let mut cascade_flags = CascadeFlags::empty();
    if let Some(element) = element {
        // NOTE(emilio): This relies on element.is_some() => pseudo.is_none(),
        // which the caller guarantees, fortunately. But this doesn't handle the
        // IS_ROOT_ELEMENT flag correctly!
        //
        // I think it's not possible to wrap a root element in a first-line
        // frame (and reparenting only happens for ::first-line and its
        // descendants), so this may be fine...
        //
        // We should just get rid of all these flags which pass element / pseudo
        // state.
        if element.is_link() {
            cascade_flags.insert(CascadeFlags::IS_LINK);
            if element.is_visited_link() && doc_data.visited_styles_enabled() {
                cascade_flags.insert(CascadeFlags::IS_VISITED_LINK);
            }
        };
    }

    doc_data.stylist.compute_style_with_inputs(
        &inputs,
        pseudo.as_ref(),
        &StylesheetGuards::same(&guard),
        Some(parent_style),
        Some(parent_style_ignoring_first_line),
        Some(layout_parent_style),
        &metrics,
        cascade_flags,
    ).into()
}

#[cfg(feature = "gecko_debug")]
fn simulate_compute_values_failure(property: &PropertyValuePair) -> bool {
    let p = property.mProperty;
    let id = get_property_id_from_nscsspropertyid!(p, false);
    id.as_shorthand().is_ok() && property.mSimulateComputeValuesFailure
}

#[cfg(not(feature = "gecko_debug"))]
fn simulate_compute_values_failure(_: &PropertyValuePair) -> bool {
    false
}

fn create_context<'a>(
    per_doc_data: &'a PerDocumentStyleDataImpl,
    font_metrics_provider: &'a FontMetricsProvider,
    style: &'a ComputedValues,
    parent_style: Option<&'a ComputedValues>,
    pseudo: Option<&'a PseudoElement>,
    for_smil_animation: bool,
    rule_cache_conditions: &'a mut RuleCacheConditions,
) -> Context<'a> {
    Context {
        is_root_element: false,
        builder: StyleBuilder::for_derived_style(
            per_doc_data.stylist.device(),
            style,
            parent_style,
            pseudo,
        ),
        font_metrics_provider: font_metrics_provider,
        cached_system_font: None,
        in_media_query: false,
        quirks_mode: per_doc_data.stylist.quirks_mode(),
        for_smil_animation,
        for_non_inherited_property: None,
        rule_cache_conditions: RefCell::new(rule_cache_conditions),
    }
}

struct PropertyAndIndex {
    property: PropertyId,
    index: usize,
}

struct PrioritizedPropertyIter<'a> {
    properties: &'a nsTArray<PropertyValuePair>,
    sorted_property_indices: Vec<PropertyAndIndex>,
    curr: usize,
}

impl<'a> PrioritizedPropertyIter<'a> {
    pub fn new(properties: &'a nsTArray<PropertyValuePair>) -> PrioritizedPropertyIter {
        // If we fail to convert a nsCSSPropertyID into a PropertyId we shouldn't fail outright
        // but instead by treating that property as the 'all' property we make it sort last.
        let all = PropertyId::Shorthand(ShorthandId::All);

        let mut sorted_property_indices: Vec<PropertyAndIndex> =
            properties.iter().enumerate().map(|(index, pair)| {
                PropertyAndIndex {
                    property: PropertyId::from_nscsspropertyid(pair.mProperty)
                              .unwrap_or(all.clone()),
                    index,
                }
            }).collect();
        sorted_property_indices.sort_by(|a, b| compare_property_priority(&a.property, &b.property));

        PrioritizedPropertyIter {
            properties,
            sorted_property_indices,
            curr: 0,
        }
    }
}

impl<'a> Iterator for PrioritizedPropertyIter<'a> {
    type Item = &'a PropertyValuePair;

    fn next(&mut self) -> Option<&'a PropertyValuePair> {
        if self.curr >= self.sorted_property_indices.len() {
            return None
        }
        self.curr += 1;
        Some(&self.properties[self.sorted_property_indices[self.curr - 1].index])
    }
}

#[no_mangle]
pub extern "C" fn Servo_GetComputedKeyframeValues(
    keyframes: RawGeckoKeyframeListBorrowed,
    element: RawGeckoElementBorrowed,
    style: ServoStyleContextBorrowed,
    raw_data: RawServoStyleSetBorrowed,
    computed_keyframes: RawGeckoComputedKeyframeValuesListBorrowedMut
) {
    use std::mem;
    use style::properties::LonghandIdSet;

    let data = PerDocumentStyleData::from_ffi(raw_data).borrow();
    let metrics = get_metrics_provider_for_product();

    let element = GeckoElement(element);
    let parent_element = element.inheritance_parent();
    let parent_data = parent_element.as_ref().and_then(|e| e.borrow_data());
    let parent_style = parent_data.as_ref().map(|d| d.styles.primary()).map(|x| &**x);

    let pseudo = style.pseudo();
    let mut conditions = Default::default();
    let mut context = create_context(
        &data,
        &metrics,
        &style,
        parent_style,
        pseudo.as_ref(),
        /* for_smil_animation = */ false,
        &mut conditions,
    );

    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let default_values = data.default_computed_values();

    let mut raw_custom_properties_block; // To make the raw block alive in the scope.
    for (index, keyframe) in keyframes.iter().enumerate() {
        let mut custom_properties = None;
        for property in keyframe.mPropertyValues.iter() {
            // Find the block for custom properties first.
            if property.mProperty == nsCSSPropertyID::eCSSPropertyExtra_variable {
                raw_custom_properties_block = unsafe {
                    &*property.mServoDeclarationBlock.mRawPtr.clone()
                };
                let guard = Locked::<PropertyDeclarationBlock>::as_arc(
                    &raw_custom_properties_block).read_with(&guard);
                custom_properties = guard.cascade_custom_properties_with_context(&context);
                // There should be one PropertyDeclarationBlock for custom properties.
                break;
            }
        }

        let ref mut animation_values = computed_keyframes[index];

        let mut seen = LonghandIdSet::new();

        let mut property_index = 0;
        for property in PrioritizedPropertyIter::new(&keyframe.mPropertyValues) {
            if simulate_compute_values_failure(property) {
                continue;
            }

            let mut maybe_append_animation_value = |property: LonghandId, value: Option<AnimationValue>| {
                if seen.contains(property) {
                    return;
                }
                seen.insert(property);

                // This is safe since we immediately write to the uninitialized values.
                unsafe { animation_values.set_len((property_index + 1) as u32) };
                animation_values[property_index].mProperty = property.to_nscsspropertyid();
                // We only make sure we have enough space for this variable,
                // but didn't construct a default value for StyleAnimationValue,
                // so we should zero it to avoid getting undefined behaviors.
                animation_values[property_index].mValue.mGecko = unsafe { mem::zeroed() };
                match value {
                    Some(v) => {
                        animation_values[property_index].mValue.mServo.set_arc_leaky(Arc::new(v));
                    },
                    None => {
                        animation_values[property_index].mValue.mServo.mRawPtr = ptr::null_mut();
                    },
                }
                property_index += 1;
            };

            if property.mServoDeclarationBlock.mRawPtr.is_null() {
                let property =
                    LonghandId::from_nscsspropertyid(property.mProperty);
                if let Ok(prop) = property {
                    maybe_append_animation_value(prop, None);
                }
                continue;
            }

            let declarations = unsafe { &*property.mServoDeclarationBlock.mRawPtr.clone() };
            let declarations = Locked::<PropertyDeclarationBlock>::as_arc(&declarations);
            let guard = declarations.read_with(&guard);
            let iter = guard.to_animation_value_iter(
                &mut context,
                &default_values,
                custom_properties.as_ref(),
            );

            for value in iter {
                let id = value.id();
                maybe_append_animation_value(id, Some(value));
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn Servo_GetAnimationValues(
    declarations: RawServoDeclarationBlockBorrowed,
    element: RawGeckoElementBorrowed,
    style: ServoStyleContextBorrowed,
    raw_data: RawServoStyleSetBorrowed,
    animation_values: RawGeckoServoAnimationValueListBorrowedMut,
) {
    let data = PerDocumentStyleData::from_ffi(raw_data).borrow();
    let metrics = get_metrics_provider_for_product();

    let element = GeckoElement(element);
    let parent_element = element.inheritance_parent();
    let parent_data = parent_element.as_ref().and_then(|e| e.borrow_data());
    let parent_style = parent_data.as_ref().map(|d| d.styles.primary()).map(|x| &**x);

    let pseudo = style.pseudo();
    let mut conditions = Default::default();
    let mut context = create_context(
        &data,
        &metrics,
        &style,
        parent_style,
        pseudo.as_ref(),
        /* for_smil_animation = */ true,
        &mut conditions,
    );

    let default_values = data.default_computed_values();
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();

    let declarations = Locked::<PropertyDeclarationBlock>::as_arc(&declarations);
    let guard = declarations.read_with(&guard);
    let iter = guard.to_animation_value_iter(
        &mut context,
        &default_values,
        None, // SMIL has no extra custom properties.
    );
    for (index, anim) in iter.enumerate() {
        unsafe { animation_values.set_len((index + 1) as u32) };
        animation_values[index].set_arc_leaky(Arc::new(anim));
    }
}

#[no_mangle]
pub extern "C" fn Servo_AnimationValue_Compute(
    element: RawGeckoElementBorrowed,
    declarations: RawServoDeclarationBlockBorrowed,
    style: ServoStyleContextBorrowed,
    raw_data: RawServoStyleSetBorrowed,
) -> RawServoAnimationValueStrong {
    let data = PerDocumentStyleData::from_ffi(raw_data).borrow();
    let metrics = get_metrics_provider_for_product();

    let element = GeckoElement(element);
    let parent_element = element.inheritance_parent();
    let parent_data = parent_element.as_ref().and_then(|e| e.borrow_data());
    let parent_style = parent_data.as_ref().map(|d| d.styles.primary()).map(|x| &**x);

    let pseudo = style.pseudo();
    let mut conditions = Default::default();
    let mut context = create_context(
        &data,
        &metrics,
        style,
        parent_style,
        pseudo.as_ref(),
        /* for_smil_animation = */ false,
        &mut conditions,
    );

    let default_values = data.default_computed_values();
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let declarations = Locked::<PropertyDeclarationBlock>::as_arc(&declarations);
    // We only compute the first element in declarations.
    match declarations.read_with(&guard).declaration_importance_iter().next() {
        Some((decl, imp)) if imp == Importance::Normal => {
            let animation = AnimationValue::from_declaration(
                decl,
                &mut context,
                None, // No extra custom properties for devtools.
                default_values,
            );
            animation.map_or(RawServoAnimationValueStrong::null(), |value| {
                Arc::new(value).into_strong()
            })
        },
        _ => RawServoAnimationValueStrong::null()
    }
}

#[no_mangle]
pub extern "C" fn Servo_AssertTreeIsClean(root: RawGeckoElementBorrowed) {
    if !cfg!(feature = "gecko_debug") {
        panic!("Calling Servo_AssertTreeIsClean in release build");
    }

    let root = GeckoElement(root);
    debug!("Servo_AssertTreeIsClean: ");
    debug!("{:?}", ShowSubtreeData(root.as_node()));

    fn assert_subtree_is_clean<'le>(el: GeckoElement<'le>) {
        debug_assert!(!el.has_dirty_descendants() && !el.has_animation_only_dirty_descendants(),
                      "{:?} has still dirty bit {:?} or animation-only dirty bit {:?}",
                      el, el.has_dirty_descendants(), el.has_animation_only_dirty_descendants());
        for child in el.traversal_children() {
            if let Some(child) = child.as_element() {
                assert_subtree_is_clean(child);
            }
        }
    }

    assert_subtree_is_clean(root);
}

#[no_mangle]
pub extern "C" fn Servo_IsWorkerThread() -> bool {
    thread_state::get().is_worker()
}

enum Offset {
    Zero,
    One
}

fn fill_in_missing_keyframe_values(
    all_properties: &LonghandIdSet,
    timing_function: nsTimingFunctionBorrowed,
    longhands_at_offset: &LonghandIdSet,
    offset: Offset,
    keyframes: RawGeckoKeyframeListBorrowedMut,
) {
    // Return early if all animated properties are already set.
    if longhands_at_offset.contains_all(all_properties) {
        return;
    }

    let keyframe = match offset {
        Offset::Zero => unsafe {
            Gecko_GetOrCreateInitialKeyframe(keyframes, timing_function)
        },
        Offset::One => unsafe {
            Gecko_GetOrCreateFinalKeyframe(keyframes, timing_function)
        },
    };

    // Append properties that have not been set at this offset.
    for property in all_properties.iter() {
        if !longhands_at_offset.contains(property) {
            unsafe {
                Gecko_AppendPropertyValuePair(
                    &mut (*keyframe).mPropertyValues,
                    property.to_nscsspropertyid()
                );
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_GetKeyframesForName(
    raw_data: RawServoStyleSetBorrowed,
    name: *mut nsAtom,
    inherited_timing_function: nsTimingFunctionBorrowed,
    keyframes: RawGeckoKeyframeListBorrowedMut,
) -> bool {
    debug_assert!(keyframes.len() == 0,
                  "keyframes should be initially empty");

    let data = PerDocumentStyleData::from_ffi(raw_data).borrow();
    let name = Atom::from(name);

    let animation = match data.stylist.get_animation(&name) {
        Some(animation) => animation,
        None => return false,
    };

    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();

    let mut properties_set_at_current_offset = LonghandIdSet::new();
    let mut properties_set_at_start = LonghandIdSet::new();
    let mut properties_set_at_end = LonghandIdSet::new();
    let mut has_complete_initial_keyframe = false;
    let mut has_complete_final_keyframe = false;
    let mut current_offset = -1.;

    // Iterate over the keyframe rules backwards so we can drop overridden
    // properties (since declarations in later rules override those in earlier
    // ones).
    for step in animation.steps.iter().rev() {
        if step.start_percentage.0 != current_offset {
            properties_set_at_current_offset.clear();
            current_offset = step.start_percentage.0;
        }

        // Override timing_function if the keyframe has an animation-timing-function.
        let timing_function = match step.get_animation_timing_function(&guard) {
            Some(val) => val.into(),
            None => *inherited_timing_function,
        };

        // Look for an existing keyframe with the same offset and timing
        // function or else add a new keyframe at the beginning of the keyframe
        // array.
        let keyframe = unsafe {
            Gecko_GetOrCreateKeyframeAtStart(keyframes,
                                             step.start_percentage.0 as f32,
                                             &timing_function)
        };

        match step.value {
            KeyframesStepValue::ComputedValues => {
                // In KeyframesAnimation::from_keyframes if there is no 0% or
                // 100% keyframe at all, we will create a 'ComputedValues' step
                // to represent that all properties animated by the keyframes
                // animation should be set to the underlying computed value for
                // that keyframe.
                for property in animation.properties_changed.iter() {
                    unsafe {
                        Gecko_AppendPropertyValuePair(
                            &mut (*keyframe).mPropertyValues,
                            property.to_nscsspropertyid(),
                        );
                    }
                }
                if current_offset == 0.0 {
                    has_complete_initial_keyframe = true;
                } else if current_offset == 1.0 {
                    has_complete_final_keyframe = true;
                }
            },
            KeyframesStepValue::Declarations { ref block } => {
                let guard = block.read_with(&guard);

                let mut custom_properties = PropertyDeclarationBlock::new();

                // Filter out non-animatable properties and properties with
                // !important.
                for declaration in guard.normal_declaration_iter() {
                    let id = declaration.id();

                    let id = match id {
                        PropertyDeclarationId::Longhand(id) => {
                            // Skip the 'display' property because although it
                            // is animatable from SMIL, it should not be
                            // animatable from CSS Animations.
                            if id == LonghandId::Display {
                                continue;
                            }

                            if !id.is_animatable() {
                                continue;
                            }

                            id
                        }
                        PropertyDeclarationId::Custom(..) => {
                            custom_properties.push(
                                declaration.clone(),
                                Importance::Normal,
                                DeclarationSource::CssOm,
                            );
                            continue;
                        }
                    };

                    if properties_set_at_current_offset.contains(id) {
                        continue;
                    }

                    let pair = unsafe {
                        Gecko_AppendPropertyValuePair(
                            &mut (*keyframe).mPropertyValues,
                            id.to_nscsspropertyid(),
                        )
                    };

                    unsafe {
                        (*pair).mServoDeclarationBlock.set_arc_leaky(
                            Arc::new(global_style_data.shared_lock.wrap(
                                PropertyDeclarationBlock::with_one(
                                    declaration.clone(),
                                    Importance::Normal,
                                )
                            ))
                        );
                    }

                    if current_offset == 0.0 {
                        properties_set_at_start.insert(id);
                    } else if current_offset == 1.0 {
                        properties_set_at_end.insert(id);
                    }
                    properties_set_at_current_offset.insert(id);
                }

                if custom_properties.any_normal() {
                    let pair = unsafe {
                        Gecko_AppendPropertyValuePair(
                            &mut (*keyframe).mPropertyValues,
                            nsCSSPropertyID::eCSSPropertyExtra_variable,
                        )
                    };

                    unsafe {
                        (*pair).mServoDeclarationBlock.set_arc_leaky(Arc::new(
                            global_style_data.shared_lock.wrap(custom_properties)
                        ));
                    }

                }
            },
        }
    }

    // Append property values that are missing in the initial or the final keyframes.
    if !has_complete_initial_keyframe {
        fill_in_missing_keyframe_values(
            &animation.properties_changed,
            inherited_timing_function,
            &properties_set_at_start,
            Offset::Zero,
            keyframes,
        );
    }
    if !has_complete_final_keyframe {
        fill_in_missing_keyframe_values(
            &animation.properties_changed,
            inherited_timing_function,
            &properties_set_at_end,
            Offset::One,
            keyframes,
        );
    }
    true
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_GetFontFaceRules(
    raw_data: RawServoStyleSetBorrowed,
    rules: RawGeckoFontFaceRuleListBorrowedMut,
) {
    let data = PerDocumentStyleData::from_ffi(raw_data).borrow();
    debug_assert!(rules.len() == 0);

    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();

    let len: u32 = data
        .stylist
        .iter_extra_data_origins()
        .map(|(d, _)| d.font_faces.len() as u32)
        .sum();

    // Reversed iterator because Gecko expects rules to appear sorted
    // UserAgent first, Author last.
    let font_face_iter = data
        .stylist
        .iter_extra_data_origins_rev()
        .flat_map(|(d, o)| d.font_faces.iter().zip(iter::repeat(o)));

    unsafe { rules.set_len(len) };
    for (src, dest) in font_face_iter.zip(rules.iter_mut()) {
        dest.mRule = src.0.read_with(&guard).clone().forget();
        dest.mSheetType = src.1.into();
    }
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_GetCounterStyleRule(
    raw_data: RawServoStyleSetBorrowed,
    name: *mut nsAtom,
) -> *mut nsCSSCounterStyleRule {
    let data = PerDocumentStyleData::from_ffi(raw_data).borrow();

    unsafe {
        Atom::with(name, |name| {
            data.stylist
                .iter_extra_data_origins()
                .filter_map(|(d, _)| d.counter_styles.get(name))
                .next()
        })
    }.map(|rule| {
        let global_style_data = &*GLOBAL_STYLE_DATA;
        let guard = global_style_data.shared_lock.read();
        rule.read_with(&guard).get()
    }).unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_BuildFontFeatureValueSet(
    raw_data: RawServoStyleSetBorrowed,
) -> *mut gfxFontFeatureValueSet {
    let data = PerDocumentStyleData::from_ffi(raw_data).borrow();

    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();

    let has_rule =
        data.stylist
            .iter_extra_data_origins()
            .any(|(d, _)| !d.font_feature_values.is_empty());

    if !has_rule {
      return ptr::null_mut();
    }

    let font_feature_values_iter =
        data.stylist
            .iter_extra_data_origins_rev()
            .flat_map(|(d, _)| d.font_feature_values.iter());

    let set = unsafe { Gecko_ConstructFontFeatureValueSet() };
    for src in font_feature_values_iter {
        let rule = src.read_with(&guard);
        rule.set_at_rules(set);
    }
    set
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_ResolveForDeclarations(
    raw_data: RawServoStyleSetBorrowed,
    parent_style_context: ServoStyleContextBorrowedOrNull,
    declarations: RawServoDeclarationBlockBorrowed,
) -> ServoStyleContextStrong {
    let doc_data = PerDocumentStyleData::from_ffi(raw_data).borrow();
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let guards = StylesheetGuards::same(&guard);

    let parent_style = match parent_style_context {
        Some(parent) => &*parent,
        None => doc_data.default_computed_values(),
    };

    let declarations = Locked::<PropertyDeclarationBlock>::as_arc(&declarations);

    doc_data.stylist.compute_for_declarations(
        &guards,
        parent_style,
        declarations.clone_arc(),
    ).into()
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_AddSizeOfExcludingThis(
    malloc_size_of: GeckoMallocSizeOf,
    malloc_enclosing_size_of: GeckoMallocSizeOf,
    sizes: *mut ServoStyleSetSizes,
    raw_data: RawServoStyleSetBorrowed
) {
    let data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    let mut ops = MallocSizeOfOps::new(malloc_size_of.unwrap(),
                                       Some(malloc_enclosing_size_of.unwrap()),
                                       None);
    let sizes = unsafe { sizes.as_mut() }.unwrap();
    data.add_size_of(&mut ops, sizes);
}

#[no_mangle]
pub extern "C" fn Servo_UACache_AddSizeOf(
    malloc_size_of: GeckoMallocSizeOf,
    malloc_enclosing_size_of: GeckoMallocSizeOf,
    sizes: *mut ServoStyleSetSizes
) {
    let mut ops = MallocSizeOfOps::new(malloc_size_of.unwrap(),
                                       Some(malloc_enclosing_size_of.unwrap()),
                                       None);
    let sizes = unsafe { sizes.as_mut() }.unwrap();
    add_size_of_ua_cache(&mut ops, sizes);
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_MightHaveAttributeDependency(
    raw_data: RawServoStyleSetBorrowed,
    element: RawGeckoElementBorrowed,
    local_name: *mut nsAtom,
) -> bool {
    let data = PerDocumentStyleData::from_ffi(raw_data).borrow();
    let element = GeckoElement(element);
    let mut has_dep = false;

    unsafe {
        Atom::with(local_name, |atom| {
            has_dep = data.stylist.might_have_attribute_dependency(atom);

            if !has_dep {
                // TODO(emilio): Consider optimizing this storing attribute
                // dependencies from UA sheets separately, so we could optimize
                // the above lookup if cut_off_inheritance is true.
                element.each_xbl_stylist(|stylist| {
                    has_dep =
                        has_dep || stylist.might_have_attribute_dependency(atom);
                });
            }
        })
    }

    has_dep
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_HasStateDependency(
    raw_data: RawServoStyleSetBorrowed,
    element: RawGeckoElementBorrowed,
    state: u64,
) -> bool {
    let element = GeckoElement(element);

    let state = ElementState::from_bits_truncate(state);
    let data = PerDocumentStyleData::from_ffi(raw_data).borrow();

    let mut has_dep = data.stylist.has_state_dependency(state);
    if !has_dep {
        // TODO(emilio): Consider optimizing this storing attribute
        // dependencies from UA sheets separately, so we could optimize
        // the above lookup if cut_off_inheritance is true.
        element.each_xbl_stylist(|stylist| {
            has_dep = has_dep || stylist.has_state_dependency(state);
        });
    }

    has_dep
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_HasDocumentStateDependency(
    raw_data: RawServoStyleSetBorrowed,
    state: u64,
) -> bool {
    let state = DocumentState::from_bits_truncate(state);
    let data = PerDocumentStyleData::from_ffi(raw_data).borrow();

    data.stylist.has_document_state_dependency(state)
}

#[no_mangle]
pub extern "C" fn Servo_GetCustomPropertyValue(
    computed_values: ServoStyleContextBorrowed,
    name: *const nsAString,
    value: *mut nsAString,
) -> bool {
    let custom_properties = match computed_values.custom_properties() {
        Some(p) => p,
        None => return false,
    };

    let name = unsafe { Atom::from((&*name)) };
    let computed_value = match custom_properties.get(&name) {
        Some(v) => v,
        None => return false,
    };

    computed_value.to_css(unsafe { value.as_mut().unwrap() }).unwrap();
    true
}

#[no_mangle]
pub extern "C" fn Servo_GetCustomPropertiesCount(computed_values: ServoStyleContextBorrowed) -> u32 {
    match computed_values.custom_properties() {
        Some(p) => p.len() as u32,
        None => 0,
    }
}

#[no_mangle]
pub extern "C" fn Servo_GetCustomPropertyNameAt(
    computed_values: ServoStyleContextBorrowed,
    index: u32,
    name: *mut nsAString,
) -> bool {
    let custom_properties = match computed_values.custom_properties() {
        Some(p) => p,
        None => return false,
    };

    let property_name = match custom_properties.get_key_at(index) {
        Some(n) => n,
        None => return false,
    };

    let name = unsafe { name.as_mut().unwrap() };
    name.assign(&*property_name.as_slice());

    true
}

#[no_mangle]
pub unsafe extern "C" fn Servo_ReleaseArcStringData(string: *const RawOffsetArc<RustString>) {
    let string = string as *const RawOffsetArc<String>;
    // Cause RawOffsetArc::drop to run, releasing the strong reference to the string data.
    let _ = ptr::read(string);
}

#[no_mangle]
pub unsafe extern "C" fn Servo_CloneArcStringData(
    string: *const RawOffsetArc<RustString>,
) -> RawOffsetArc<RustString> {
    let string = string as *const RawOffsetArc<String>;
    let cloned = (*string).clone();
    mem::transmute::<_, RawOffsetArc<RustString>>(cloned)
}

#[no_mangle]
pub unsafe extern "C" fn Servo_GetArcStringData(
    string: *const RustString,
    utf8_chars: *mut *const u8,
    utf8_len: *mut u32,
) {
    let string = &*(string as *const String);
    *utf8_len = string.len() as u32;
    *utf8_chars = string.as_ptr();
}

#[no_mangle]
pub extern "C" fn Servo_ProcessInvalidations(
    set: RawServoStyleSetBorrowed,
    element: RawGeckoElementBorrowed,
    snapshots: *const ServoElementSnapshotTable,
) {
    debug_assert!(!snapshots.is_null());

    let element = GeckoElement(element);
    debug_assert!(element.has_snapshot());
    debug_assert!(!element.handled_snapshot());

    let mut data = element.mutate_data();
    debug_assert!(data.is_some());

    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let per_doc_data = PerDocumentStyleData::from_ffi(set).borrow();
    let shared_style_context = create_shared_context(&global_style_data,
                                                     &guard,
                                                     &per_doc_data,
                                                     TraversalFlags::empty(),
                                                     unsafe { &*snapshots });
    let mut data = data.as_mut().map(|d| &mut **d);

    if let Some(ref mut data) = data {
        // FIXME(emilio): Ideally we could share the nth-index-cache across all
        // the elements?
        let result = data.invalidate_style_if_needed(
            element,
            &shared_style_context,
            None,
            &mut NthIndexCache::default(),
        );

        if result.has_invalidated_siblings() {
            let parent = element.traversal_parent().expect("How could we invalidate siblings without a common parent?");
            unsafe {
                parent.set_dirty_descendants();
                bindings::Gecko_NoteDirtySubtreeForInvalidation(parent.0);
            }
        } else if result.has_invalidated_descendants() {
            unsafe { bindings::Gecko_NoteDirtySubtreeForInvalidation(element.0) };
        } else if result.has_invalidated_self() {
            unsafe { bindings::Gecko_NoteDirtyElement(element.0) };
        }
    }
}

#[no_mangle]
pub extern "C" fn Servo_HasPendingRestyleAncestor(element: RawGeckoElementBorrowed) -> bool {
    let mut element = Some(GeckoElement(element));
    while let Some(e) = element {
        if e.has_animations() {
            return true;
        }

        // If the element needs a frame, it means that we haven't styled it yet
        // after it got inserted in the document, and thus we may need to do
        // that for transitions and animations to trigger.
        if e.needs_frame() {
            return true;
        }

        if let Some(data) = e.borrow_data() {
            if !data.hint.is_empty() {
                return true;
            }
        }

        element = e.traversal_parent();
    }
    false
}

#[no_mangle]
pub unsafe extern "C" fn Servo_SelectorList_Parse(
    selector_list: *const nsACString,
) -> *mut RawServoSelectorList {
    use style::selector_parser::SelectorParser;

    debug_assert!(!selector_list.is_null());

    let input = (*selector_list).as_str_unchecked();
    let selector_list = match SelectorParser::parse_author_origin_no_namespace(&input) {
        Ok(selector_list) => selector_list,
        Err(..) => return ptr::null_mut(),
    };

    Box::into_raw(Box::new(selector_list)) as *mut RawServoSelectorList
}

#[no_mangle]
pub unsafe extern "C" fn Servo_SelectorList_Drop(list: RawServoSelectorListOwned) {
    let _ = list.into_box::<::selectors::SelectorList<SelectorImpl>>();
}

fn parse_color(
    value: &str,
    error_reporter: Option<&ErrorReporter>,
) -> Result<specified::Color, ()> {
    let mut input = ParserInput::new(value);
    let mut parser = Parser::new(&mut input);
    let url_data = unsafe { dummy_url_data() };
    let context = ParserContext::new(
        Origin::Author,
        url_data,
        Some(CssRuleType::Style),
        ParsingMode::DEFAULT,
        QuirksMode::NoQuirks,
    );

    let start_position = parser.position();
    parser.parse_entirely(|i| specified::Color::parse(&context, i)).map_err(|err| {
        if let Some(error_reporter) = error_reporter {
            match err.kind {
                ParseErrorKind::Custom(StyleParseErrorKind::ValueError(..)) => {
                    let location = err.location.clone();
                    let error = ContextualParseError::UnsupportedValue(
                        parser.slice_from(start_position),
                        err,
                    );
                    error_reporter.report(location, error);
                }
                // Ignore other kinds of errors that might be reported, such as
                // ParseErrorKind::Basic(BasicParseErrorKind::UnexpectedToken),
                // since Gecko doesn't report those to the error console.
                _ => {}
            }
        }
    })
}

#[no_mangle]
pub extern "C" fn Servo_IsValidCSSColor(
    value: *const nsAString,
) -> bool {
    let value = unsafe { (*value).to_string() };
    parse_color(&value, None).is_ok()
}

#[no_mangle]
pub extern "C" fn Servo_ComputeColor(
    raw_data: RawServoStyleSetBorrowedOrNull,
    current_color: structs::nscolor,
    value: *const nsAString,
    result_color: *mut structs::nscolor,
    was_current_color: *mut bool,
    loader: *mut Loader,
) -> bool {
    use style::gecko;

    let current_color = gecko::values::convert_nscolor_to_rgba(current_color);
    let value = unsafe { (*value).to_string() };
    let result_color = unsafe { result_color.as_mut().unwrap() };

    let reporter = unsafe { loader.as_mut() }.map(|loader| {
        // Make an ErrorReporter that will report errors as being "from DOM".
        ErrorReporter::new(ptr::null_mut(), loader, ptr::null_mut())
    });

    match parse_color(&value, reporter.as_ref()) {
        Ok(specified_color) => {
            let computed_color = match raw_data {
                Some(raw_data) => {
                    let data = PerDocumentStyleData::from_ffi(raw_data).borrow();
                    let metrics = get_metrics_provider_for_product();
                    let mut conditions = Default::default();
                    let context = create_context(
                        &data,
                        &metrics,
                        data.stylist.device().default_computed_values(),
                        /* parent_style = */ None,
                        /* pseudo = */ None,
                        /* for_smil_animation = */ false,
                        &mut conditions,
                    );
                    specified_color.to_computed_color(Some(&context))
                }
                None => {
                    specified_color.to_computed_color(None)
                }
            };

            match computed_color {
                Some(computed_color) => {
                    let rgba = computed_color.to_rgba(current_color);
                    *result_color = gecko::values::convert_rgba_to_nscolor(&rgba);
                    if !was_current_color.is_null() {
                        unsafe {
                            *was_current_color = computed_color.is_currentcolor();
                        }
                    }
                    true
                }
                None => false,
            }
        }
        Err(_) => false,
    }
}

#[no_mangle]
pub extern "C" fn Servo_ParseIntersectionObserverRootMargin(
    value: *const nsAString,
    result: *mut structs::nsCSSRect,
) -> bool {
    let value = unsafe { value.as_ref().unwrap().to_string() };
    let result = unsafe { result.as_mut().unwrap() };

    let mut input = ParserInput::new(&value);
    let mut parser = Parser::new(&mut input);

    let url_data = unsafe { dummy_url_data() };
    let context = ParserContext::new(
        Origin::Author,
        url_data,
        Some(CssRuleType::Style),
        ParsingMode::DEFAULT,
        QuirksMode::NoQuirks,
    );

    let margin = parser.parse_entirely(|p| {
        IntersectionObserverRootMargin::parse(&context, p)
    });
    match margin {
        Ok(margin) => {
            let rect = margin.0;
            result.mTop.set_from(rect.0);
            result.mRight.set_from(rect.1);
            result.mBottom.set_from(rect.2);
            result.mLeft.set_from(rect.3);
            true
        }
        Err(..) => false,
    }
}

#[no_mangle]
pub extern "C" fn Servo_ParseTransformIntoMatrix(
    value: *const nsAString,
    contain_3d: *mut bool,
    result: *mut RawGeckoGfxMatrix4x4
) -> bool {
    use style::properties::longhands::transform;

    let string = unsafe { (*value).to_string() };
    let mut input = ParserInput::new(&string);
    let mut parser = Parser::new(&mut input);
    let context = ParserContext::new(
        Origin::Author,
        unsafe { dummy_url_data() },
        Some(CssRuleType::Style),
        ParsingMode::DEFAULT,
        QuirksMode::NoQuirks
    );

    let transform = match parser.parse_entirely(|t| transform::parse(&context, t)) {
        Ok(t) => t,
        Err(..) => return false,
    };

    let (m, is_3d) = match transform.to_transform_3d_matrix(None) {
        Ok(result) => result,
        Err(..) => return false,
    };

    let result = unsafe { result.as_mut() }.expect("not a valid matrix");
    let contain_3d = unsafe { contain_3d.as_mut() }.expect("not a valid bool");
    *result = m.to_row_major_array();
    *contain_3d = is_3d;
    true
}

// https://drafts.csswg.org/css-font-loading/#dom-fontface-fontface
#[no_mangle]
pub extern "C" fn Servo_ParseFontDescriptor(
    desc_id: nsCSSFontDesc,
    value: *const nsAString,
    data: *mut URLExtraData,
    result: nsCSSValueBorrowedMut,
) -> bool {
    use cssparser::UnicodeRange;
    use self::nsCSSFontDesc::*;
    use style::computed_values::{font_feature_settings, font_stretch, font_style};
    use style::font_face::{FontDisplay, FontWeight, Source};
    use style::properties::longhands::font_language_override;
    use style::values::computed::font::FamilyName;

    let string = unsafe { (*value).to_string() };
    let mut input = ParserInput::new(&string);
    let mut parser = Parser::new(&mut input);
    let url_data = unsafe { RefPtr::from_ptr_ref(&data) };
    let context = ParserContext::new(
        Origin::Author,
        url_data,
        Some(CssRuleType::FontFace),
        ParsingMode::DEFAULT,
        QuirksMode::NoQuirks,
    );

    macro_rules! parse_font_desc {
        (
            valid = [ $( $v_enum_name: ident / $t: ty, )* ]
            invalid = [ $( $i_enum_name: ident, )* ]
        ) => {
            match desc_id {
                $(
                    $v_enum_name => {
                        let f = match parser.parse_entirely(|i| <$t as Parse>::parse(&context, i)) {
                            Ok(f) => f,
                            Err(..) => return false,
                        };
                        result.set_from(f);
                    },
                )*
                $(
                    $i_enum_name => {
                        debug_assert!(false, "$i_enum_name is not a valid font descriptor");
                        return false;
                    },
                )*
            }
        }
    }

    // We implement the parser of each arm according to the implementation of @font-face rule.
    // see component/style/font_face.rs for more detail.
    parse_font_desc!(
        valid = [
            eCSSFontDesc_Family / FamilyName,
            eCSSFontDesc_Style / font_style::T,
            eCSSFontDesc_Weight / FontWeight,
            eCSSFontDesc_Stretch / font_stretch::T,
            eCSSFontDesc_Src / Vec<Source>,
            eCSSFontDesc_UnicodeRange / Vec<UnicodeRange>,
            eCSSFontDesc_FontFeatureSettings / font_feature_settings::T,
            eCSSFontDesc_FontLanguageOverride / font_language_override::SpecifiedValue,
            eCSSFontDesc_Display / FontDisplay,
        ]
        invalid = [
            eCSSFontDesc_UNKNOWN,
            eCSSFontDesc_COUNT,
        ]
    );

    true
}

#[no_mangle]
pub extern "C" fn Servo_ParseFontShorthandForMatching(
    value: *const nsAString,
    data: *mut URLExtraData,
    family: *mut structs::RefPtr<structs::SharedFontList>,
    style: nsCSSValueBorrowedMut,
    stretch: nsCSSValueBorrowedMut,
    weight: nsCSSValueBorrowedMut
) -> bool {
    use style::properties::longhands::{font_stretch, font_style};
    use style::properties::shorthands::font;
    use style::values::specified::font::{FontFamily, FontWeight};

    let string = unsafe { (*value).to_string() };
    let mut input = ParserInput::new(&string);
    let mut parser = Parser::new(&mut input);
    let url_data = unsafe { RefPtr::from_ptr_ref(&data) };
    let context = ParserContext::new(
        Origin::Author,
        url_data,
        Some(CssRuleType::FontFace),
        ParsingMode::DEFAULT,
        QuirksMode::NoQuirks,
    );

    let font = match parser.parse_entirely(|f| font::parse_value(&context, f)) {
        Ok(f) => f,
        Err(..) => return false,
    };

    // The system font is not acceptable, so we return false.
    let family = unsafe { &mut *family };
    match font.font_family {
        FontFamily::Values(list) => family.set_move(list.0),
        FontFamily::System(_) => return false,
    }
    style.set_from(match font.font_style {
        font_style::SpecifiedValue::Keyword(kw) => kw,
        font_style::SpecifiedValue::System(_) => return false,
    });
    stretch.set_from(match font.font_stretch {
        font_stretch::SpecifiedValue::Keyword(kw) => kw,
        font_stretch::SpecifiedValue::System(_) => return false,
    });
    match font.font_weight {
        FontWeight::Weight(w) => weight.set_from(w),
        FontWeight::Normal => weight.set_enum(structs::NS_STYLE_FONT_WEIGHT_NORMAL as i32),
        FontWeight::Bold => weight.set_enum(structs::NS_STYLE_FONT_WEIGHT_BOLD as i32),
        // Resolve relative font weights against the initial of font-weight
        // (normal, which is equivalent to 400).
        FontWeight::Bolder => weight.set_enum(structs::NS_FONT_WEIGHT_BOLD as i32),
        FontWeight::Lighter => weight.set_enum(structs::NS_FONT_WEIGHT_THIN as i32),
        FontWeight::System(_) => return false,
    }

    true
}

#[no_mangle]
pub unsafe extern "C" fn Servo_SourceSizeList_Parse(
    value: *const nsACString,
) -> *mut RawServoSourceSizeList {
    let value = (*value).as_str_unchecked();
    let mut input = ParserInput::new(value);
    let mut parser = Parser::new(&mut input);

    let context = ParserContext::new(
        Origin::Author,
        dummy_url_data(),
        Some(CssRuleType::Style),
        ParsingMode::DEFAULT,
        QuirksMode::NoQuirks,
    );

    // NB: Intentionally not calling parse_entirely.
    let list = SourceSizeList::parse(&context, &mut parser);
    Box::into_raw(Box::new(list)) as *mut _
}

#[no_mangle]
pub unsafe extern "C" fn Servo_SourceSizeList_Evaluate(
    raw_data: RawServoStyleSetBorrowed,
    list: RawServoSourceSizeListBorrowedOrNull,
) -> i32 {
    let doc_data = PerDocumentStyleData::from_ffi(raw_data).borrow();
    let device = doc_data.stylist.device();
    let quirks_mode = doc_data.stylist.quirks_mode();

    let result = match list {
        Some(list) => {
            SourceSizeList::from_ffi(list).evaluate(device, quirks_mode)
        }
        None => {
            SourceSizeList::empty().evaluate(device, quirks_mode)
        }
    };

    result.0
}

#[no_mangle]
pub unsafe extern "C" fn Servo_SourceSizeList_Drop(list: RawServoSourceSizeListOwned) {
    let _ = list.into_box::<SourceSizeList>();
}

#[no_mangle]
pub extern "C" fn Servo_ParseCounterStyleName(
    value: *const nsACString,
) -> *mut nsAtom {
    let value = unsafe { value.as_ref().unwrap().as_str_unchecked() };
    let mut input = ParserInput::new(&value);
    let mut parser = Parser::new(&mut input);
    match parser.parse_entirely(counter_style::parse_counter_style_name_definition) {
        Ok(name) => name.0.into_addrefed(),
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn Servo_ParseCounterStyleDescriptor(
    descriptor: nsCSSCounterDesc,
    value: *const nsACString,
    raw_extra_data: *mut URLExtraData,
    result: *mut nsCSSValue,
) -> bool {
    let value = unsafe { value.as_ref().unwrap().as_str_unchecked() };
    let url_data = unsafe {
        if raw_extra_data.is_null() {
            dummy_url_data()
        } else {
            RefPtr::from_ptr_ref(&raw_extra_data)
        }
    };
    let result = unsafe { result.as_mut().unwrap() };
    let mut input = ParserInput::new(&value);
    let mut parser = Parser::new(&mut input);
    let context = ParserContext::new(
        Origin::Author,
        url_data,
        Some(CssRuleType::CounterStyle),
        ParsingMode::DEFAULT,
        QuirksMode::NoQuirks,
    );
    counter_style::parse_counter_style_descriptor(
        &context,
        &mut parser,
        descriptor,
        result,
    ).is_ok()
}
