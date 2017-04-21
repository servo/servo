/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use atomic_refcell::AtomicRefMut;
use cssparser::Parser;
use cssparser::ToCss as ParserToCss;
use env_logger::LogBuilder;
use parking_lot::RwLock;
use selectors::Element;
use std::borrow::Cow;
use std::env;
use std::fmt::Write;
use std::ptr;
use std::sync::{Arc, Mutex};
use style::context::{QuirksMode, SharedStyleContext, StyleContext};
use style::context::{ThreadLocalStyleContext, ThreadLocalStyleContextCreationInfo};
use style::data::{ElementData, ElementStyles, RestyleData};
use style::dom::{AnimationOnlyDirtyDescendants, DirtyDescendants};
use style::dom::{ShowSubtreeData, TElement, TNode};
use style::error_reporting::StdoutErrorReporter;
use style::font_metrics::get_metrics_provider_for_product;
use style::gecko::data::{PerDocumentStyleData, PerDocumentStyleDataImpl};
use style::gecko::global_style_data::{GLOBAL_STYLE_DATA, GlobalStyleData};
use style::gecko::restyle_damage::GeckoRestyleDamage;
use style::gecko::selector_parser::{SelectorImpl, PseudoElement};
use style::gecko::traversal::RecalcStyleOnly;
use style::gecko::wrapper::GeckoElement;
use style::gecko_bindings::bindings;
use style::gecko_bindings::bindings::{RawGeckoKeyframeListBorrowed, RawGeckoKeyframeListBorrowedMut};
use style::gecko_bindings::bindings::{RawServoDeclarationBlockBorrowed, RawServoDeclarationBlockStrong};
use style::gecko_bindings::bindings::{RawServoMediaList, RawServoMediaListBorrowed, RawServoMediaListStrong};
use style::gecko_bindings::bindings::{RawServoMediaRule, RawServoMediaRuleBorrowed};
use style::gecko_bindings::bindings::{RawServoNamespaceRule, RawServoNamespaceRuleBorrowed};
use style::gecko_bindings::bindings::{RawServoPageRule, RawServoPageRuleBorrowed};
use style::gecko_bindings::bindings::{RawServoStyleSetBorrowed, RawServoStyleSetOwned};
use style::gecko_bindings::bindings::{RawServoStyleSheetBorrowed, ServoComputedValuesBorrowed};
use style::gecko_bindings::bindings::{RawServoStyleSheetStrong, ServoComputedValuesStrong};
use style::gecko_bindings::bindings::{ServoCssRulesBorrowed, ServoCssRulesStrong};
use style::gecko_bindings::bindings::{nsACString, nsAString};
use style::gecko_bindings::bindings::Gecko_AnimationAppendKeyframe;
use style::gecko_bindings::bindings::RawGeckoAnimationPropertySegmentBorrowed;
use style::gecko_bindings::bindings::RawGeckoComputedKeyframeValuesListBorrowedMut;
use style::gecko_bindings::bindings::RawGeckoComputedTimingBorrowed;
use style::gecko_bindings::bindings::RawGeckoElementBorrowed;
use style::gecko_bindings::bindings::RawGeckoFontFaceRuleListBorrowedMut;
use style::gecko_bindings::bindings::RawGeckoServoStyleRuleListBorrowedMut;
use style::gecko_bindings::bindings::RawServoAnimationValueBorrowed;
use style::gecko_bindings::bindings::RawServoAnimationValueMapBorrowed;
use style::gecko_bindings::bindings::RawServoAnimationValueStrong;
use style::gecko_bindings::bindings::RawServoImportRuleBorrowed;
use style::gecko_bindings::bindings::RawServoStyleRuleBorrowed;
use style::gecko_bindings::bindings::ServoComputedValuesBorrowedOrNull;
use style::gecko_bindings::bindings::nsTArrayBorrowed_uintptr_t;
use style::gecko_bindings::bindings::nsTimingFunctionBorrowed;
use style::gecko_bindings::bindings::nsTimingFunctionBorrowedMut;
use style::gecko_bindings::structs;
use style::gecko_bindings::structs::{RawServoStyleRule, ServoStyleSheet};
use style::gecko_bindings::structs::{SheetParsingMode, nsIAtom, nsCSSPropertyID};
use style::gecko_bindings::structs::{nsRestyleHint, nsChangeHint, nsCSSFontFaceRule};
use style::gecko_bindings::structs::Loader;
use style::gecko_bindings::structs::RawGeckoPresContextOwned;
use style::gecko_bindings::structs::URLExtraData;
use style::gecko_bindings::structs::nsCSSValueSharedList;
use style::gecko_bindings::structs::nsresult;
use style::gecko_bindings::sugar::ownership::{FFIArcHelpers, HasFFI, HasArcFFI, HasBoxFFI};
use style::gecko_bindings::sugar::ownership::{HasSimpleFFI, Strong};
use style::gecko_bindings::sugar::refptr::RefPtr;
use style::gecko_properties::{self, style_structs};
use style::keyframes::KeyframesStepValue;
use style::media_queries::{MediaList, parse_media_query_list};
use style::parallel;
use style::parser::{LengthParsingMode, ParserContext};
use style::properties::{CascadeFlags, ComputedValues, Importance, ParsedDeclaration};
use style::properties::{PropertyDeclarationBlock, PropertyId};
use style::properties::SKIP_ROOT_AND_ITEM_BASED_DISPLAY_FIXUP;
use style::properties::animated_properties::{AnimationValue, ComputeDistance, Interpolate, TransitionProperty};
use style::properties::parse_one_declaration;
use style::restyle_hints::{self, RestyleHint};
use style::rule_tree::StyleSource;
use style::selector_parser::PseudoElementCascadeType;
use style::sequential;
use style::shared_lock::{SharedRwLock, SharedRwLockReadGuard, StylesheetGuards, ToCssWithGuard, Locked};
use style::string_cache::Atom;
use style::stylesheets::{CssRule, CssRules, CssRuleType, CssRulesHelpers};
use style::stylesheets::{ImportRule, MediaRule, NamespaceRule, Origin};
use style::stylesheets::{PageRule, Stylesheet, StyleRule};
use style::stylesheets::StylesheetLoader as StyleStylesheetLoader;
use style::supports::parse_condition_or_declaration;
use style::thread_state;
use style::timer::Timer;
use style::traversal::{ANIMATION_ONLY, FOR_RECONSTRUCT, UNSTYLED_CHILDREN_ONLY};
use style::traversal::{resolve_style, DomTraversal, TraversalDriver, TraversalFlags};
use style_traits::ToCss;
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
    // Initialize logging.
    let mut builder = LogBuilder::new();
    let default_level = if cfg!(debug_assertions) { "warn" } else { "error" };
    match env::var("RUST_LOG") {
      Ok(v) => builder.parse(&v).init().unwrap(),
      _ => builder.parse(default_level).init().unwrap(),
    };

    // Pretend that we're a Servo Layout thread, to make some assertions happy.
    thread_state::initialize(thread_state::LAYOUT);

    // Perform some debug-only runtime assertions.
    restyle_hints::assert_restyle_hints_match();

    // Initialize some static data.
    gecko_properties::initialize();

    // Initialize the dummy url data
    unsafe { DUMMY_URL_DATA = dummy_url_data; }
}

#[no_mangle]
pub extern "C" fn Servo_Shutdown() {
    // Clear some static data to avoid shutdown leaks.
    gecko_properties::shutdown();

    // The dummy url will be released after shutdown, so clear the
    // reference to avoid use-after-free.
    unsafe { DUMMY_URL_DATA = ptr::null_mut(); }
}

unsafe fn dummy_url_data() -> &'static RefPtr<URLExtraData> {
    RefPtr::from_ptr_ref(&DUMMY_URL_DATA)
}

fn create_shared_context<'a>(global_style_data: &GlobalStyleData,
                             guard: &'a SharedRwLockReadGuard,
                             per_doc_data: &PerDocumentStyleDataImpl,
                             traversal_flags: TraversalFlags) -> SharedStyleContext<'a> {
    let local_context_data =
        ThreadLocalStyleContextCreationInfo::new(per_doc_data.new_animations_sender.clone());

    SharedStyleContext {
        stylist: per_doc_data.stylist.clone(),
        options: global_style_data.options.clone(),
        guards: StylesheetGuards::same(guard),
        running_animations: per_doc_data.running_animations.clone(),
        expired_animations: per_doc_data.expired_animations.clone(),
        // FIXME(emilio): Stop boxing here.
        error_reporter: Box::new(StdoutErrorReporter),
        local_context_creation_data: Mutex::new(local_context_data),
        timer: Timer::new(),
        // FIXME Find the real QuirksMode information for this document
        quirks_mode: QuirksMode::NoQuirks,
        traversal_flags: traversal_flags,
    }
}

fn traverse_subtree(element: GeckoElement, raw_data: RawServoStyleSetBorrowed,
                    traversal_flags: TraversalFlags) {
    // When new content is inserted in a display:none subtree, we will call into
    // servo to try to style it. Detect that here and bail out.
    if let Some(parent) = element.parent_element() {
        if parent.borrow_data().map_or(true, |d| d.styles().is_display_none()) {
            debug!("{:?} has unstyled parent {:?} - ignoring call to traverse_subtree", element, parent);
            return;
        }
    }

    let per_doc_data = PerDocumentStyleData::from_ffi(raw_data).borrow();

    let token = RecalcStyleOnly::pre_traverse(element, &per_doc_data.stylist, traversal_flags);
    if !token.should_traverse() {
        return;
    }

    debug!("Traversing subtree:");
    debug!("{:?}", ShowSubtreeData(element.as_node()));

    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let shared_style_context = create_shared_context(&global_style_data,
                                                     &guard,
                                                     &per_doc_data,
                                                     traversal_flags);

    let traversal_driver = if global_style_data.style_thread_pool.is_none() {
        TraversalDriver::Sequential
    } else {
        TraversalDriver::Parallel
    };

    let traversal = RecalcStyleOnly::new(shared_style_context, traversal_driver);
    if traversal_driver.is_parallel() {
        parallel::traverse_dom(&traversal, element, token,
                               global_style_data.style_thread_pool.as_ref().unwrap());
    } else {
        sequential::traverse_dom(&traversal, element, token);
    }
}

/// Traverses the subtree rooted at `root` for restyling.  Returns whether a
/// Gecko post-traversal (to perform lazy frame construction, or consume any
/// RestyleData, or drop any ElementData) is required.
#[no_mangle]
pub extern "C" fn Servo_TraverseSubtree(root: RawGeckoElementBorrowed,
                                        raw_data: RawServoStyleSetBorrowed,
                                        root_behavior: structs::TraversalRootBehavior,
                                        restyle_behavior: structs::TraversalRestyleBehavior)
                                        -> bool {
    use self::structs::TraversalRestyleBehavior as Restyle;
    use self::structs::TraversalRootBehavior as Root;

    let element = GeckoElement(root);
    debug!("Servo_TraverseSubtree: {:?}", element);

    let traversal_flags = match (root_behavior, restyle_behavior) {
        (Root::Normal, Restyle::Normal) => TraversalFlags::empty(),
        (Root::UnstyledChildrenOnly, Restyle::Normal) => UNSTYLED_CHILDREN_ONLY,
        (Root::Normal, Restyle::ForReconstruct) => FOR_RECONSTRUCT,
        _ => panic!("invalid combination of TraversalRootBehavior and TraversalRestyleBehavior"),
    };

    if element.has_animation_only_dirty_descendants() ||
       element.has_animation_restyle_hints() {
        traverse_subtree(element, raw_data, traversal_flags | ANIMATION_ONLY);
    }

    traverse_subtree(element, raw_data, traversal_flags);

    element.has_dirty_descendants() || element.mutate_data().unwrap().has_restyle()
}

#[no_mangle]
pub extern "C" fn Servo_AnimationValues_Interpolate(from: RawServoAnimationValueBorrowed,
                                                    to: RawServoAnimationValueBorrowed,
                                                    progress: f64)
     -> RawServoAnimationValueStrong
{
    let from_value = AnimationValue::as_arc(&from);
    let to_value = AnimationValue::as_arc(&to);
    if let Ok(value) = from_value.interpolate(to_value, progress) {
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
    from_value.interpolate(to_value, 0.5).is_ok()
}

#[no_mangle]
pub extern "C" fn Servo_AnimationValues_ComputeDistance(from: RawServoAnimationValueBorrowed,
                                                        to: RawServoAnimationValueBorrowed)
                                                        -> f64 {
    let from_value = AnimationValue::as_arc(&from);
    let to_value = AnimationValue::as_arc(&to);
    from_value.compute_distance(to_value).unwrap_or(0.0)
}

#[no_mangle]
pub extern "C" fn Servo_AnimationValueMap_Push(value_map: RawServoAnimationValueMapBorrowed,
                                               property: nsCSSPropertyID,
                                               value: RawServoAnimationValueBorrowed)
{
    use style::properties::animated_properties::AnimationValueMap;

    let value_map = RwLock::<AnimationValueMap>::as_arc(&value_map);
    let value = AnimationValue::as_arc(&value).as_ref();
    value_map.write().insert(property.into(), value.clone());
}

#[no_mangle]
pub extern "C" fn Servo_AnimationCompose(raw_value_map: RawServoAnimationValueMapBorrowed,
                                         base_values: *mut ::std::os::raw::c_void,
                                         css_property: nsCSSPropertyID,
                                         segment: RawGeckoAnimationPropertySegmentBorrowed,
                                         computed_timing: RawGeckoComputedTimingBorrowed)
{
    use style::gecko_bindings::bindings::Gecko_AnimationGetBaseStyle;
    use style::gecko_bindings::bindings::Gecko_GetPositionInSegment;
    use style::gecko_bindings::bindings::Gecko_GetProgressFromComputedTiming;
    use style::properties::animated_properties::AnimationValueMap;

    let property: TransitionProperty = css_property.into();
    let value_map = RwLock::<AnimationValueMap>::as_arc(&raw_value_map);

    // If either of the segment endpoints are null, get the underlying value to
    // use from the current value in the values map (set by a lower-priority
    // effect), or, if there is no current value, look up the cached base value
    // for this property.
    let underlying_value = if segment.mFromValue.mServo.mRawPtr.is_null() ||
                              segment.mToValue.mServo.mRawPtr.is_null() {
        let previous_composed_value = value_map.read().get(&property).cloned();
        previous_composed_value.or_else(|| {
            let raw_base_style = unsafe { Gecko_AnimationGetBaseStyle(base_values, css_property) };
            AnimationValue::arc_from_borrowed(&raw_base_style).map(|v| v.as_ref()).cloned()
        })
    } else {
        None
    };

    if (segment.mFromValue.mServo.mRawPtr.is_null() ||
        segment.mToValue.mServo.mRawPtr.is_null()) &&
        underlying_value.is_none() {
        warn!("Underlying value should be valid in the case where either 'from' value or 'to' value is null");
        return;
    }

    // Declare for making derefenced raw pointer alive outside the if block.
    let raw_from_value;
    let from_value = if !segment.mFromValue.mServo.mRawPtr.is_null() {
        raw_from_value = unsafe { &*segment.mFromValue.mServo.mRawPtr };
        AnimationValue::as_arc(&raw_from_value).as_ref()
    } else {
        underlying_value.as_ref().unwrap()
    };

    let raw_to_value;
    let to_value = if !segment.mToValue.mServo.mRawPtr.is_null() {
        raw_to_value = unsafe { &*segment.mToValue.mServo.mRawPtr };
        AnimationValue::as_arc(&raw_to_value).as_ref()
    } else {
        underlying_value.as_ref().unwrap()
    };

    let progress = unsafe { Gecko_GetProgressFromComputedTiming(computed_timing) };
    if segment.mToKey == segment.mFromKey {
        if progress < 0. {
            value_map.write().insert(property, from_value.clone());
        } else {
            value_map.write().insert(property, to_value.clone());
        }
        return;
    }

    let position = unsafe {
        Gecko_GetPositionInSegment(segment, progress, computed_timing.mBeforeFlag)
    };
    if let Ok(value) = from_value.interpolate(to_value, position) {
        value_map.write().insert(property, value);
    } else if progress < 0.5 {
        value_map.write().insert(property, from_value.clone());
    } else {
        value_map.write().insert(property, to_value.clone());
    }
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
        .single_value_to_css(&get_property_id_from_nscsspropertyid!(property, ()), &mut string);
    debug_assert!(rv.is_ok());

    write!(unsafe { &mut *buffer }, "{}", string).expect("Failed to copy string");
}

#[no_mangle]
pub extern "C" fn Servo_AnimationValue_GetOpacity(value: RawServoAnimationValueBorrowed)
     -> f32
{
    let value = AnimationValue::as_arc(&value);
    if let AnimationValue::Opacity(opacity) = **value {
        opacity
    } else {
        panic!("The AnimationValue should be Opacity");
    }
}

#[no_mangle]
pub extern "C" fn Servo_AnimationValue_GetTransform(value: RawServoAnimationValueBorrowed,
                                                    list: *mut structs::RefPtr<nsCSSValueSharedList>)
{
    let value = AnimationValue::as_arc(&value);
    if let AnimationValue::Transform(ref servo_list) = **value {
        style_structs::Box::convert_transform(servo_list.0.clone().unwrap(), unsafe { &mut *list });
    } else {
        panic!("The AnimationValue should be transform");
    }
}

#[no_mangle]
pub extern "C" fn Servo_AnimationValue_DeepEqual(this: RawServoAnimationValueBorrowed,
                                                 other: RawServoAnimationValueBorrowed)
     -> bool
{
    let this_value = AnimationValue::as_arc(&this);
    let other_value = AnimationValue::as_arc(&other);
    this_value == other_value
}

#[no_mangle]
pub extern "C" fn Servo_AnimationValue_Uncompute(value: RawServoAnimationValueBorrowed)
                                                 -> RawServoDeclarationBlockStrong {
    let value = AnimationValue::as_arc(&value);
    let global_style_data = &*GLOBAL_STYLE_DATA;
    Arc::new(global_style_data.shared_lock.wrap(
        PropertyDeclarationBlock::with_one(value.uncompute(), Importance::Normal))).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_GetBaseComputedValuesForElement(raw_data: RawServoStyleSetBorrowed,
                                                                 element: RawGeckoElementBorrowed,
                                                                 pseudo_tag: *mut nsIAtom)
                                                                 -> ServoComputedValuesStrong
{
    use style::matching::MatchMethods;

    let doc_data = PerDocumentStyleData::from_ffi(raw_data).borrow();
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let shared_context = &create_shared_context(&global_style_data,
                                                &guard,
                                                &doc_data,
                                                TraversalFlags::empty());
    let element = GeckoElement(element);
    let element_data = element.borrow_data().unwrap();
    let styles = element_data.styles();

    let pseudo = if pseudo_tag.is_null() {
        None
    } else {
        let atom = Atom::from(pseudo_tag);
        Some(PseudoElement::from_atom_unchecked(atom, /* anon_box = */ false))
    };
    let pseudos = &styles.pseudos;
    let pseudo_style = pseudo.as_ref().map(|p| (p, pseudos.get(p).unwrap()));

    let provider = get_metrics_provider_for_product();
    element.get_base_style(shared_context, &provider, &styles.primary, &pseudo_style)
           .into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_ComputedValues_ExtractAnimationValue(computed_values: ServoComputedValuesBorrowed,
                                                             property_id: nsCSSPropertyID)
                                                             -> RawServoAnimationValueStrong
{
    let computed_values = ComputedValues::as_arc(&computed_values);
    Arc::new(AnimationValue::from_computed_values(&property_id.into(), computed_values)).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_Property_IsAnimatable(property: nsCSSPropertyID) -> bool {
    use style::properties::animated_properties;
    animated_properties::nscsspropertyid_is_animatable(property)
}

#[no_mangle]
pub extern "C" fn Servo_Property_IsDiscreteAnimatable(property: nsCSSPropertyID) -> bool {
    let property: TransitionProperty = property.into();
    property.is_discrete()
}

#[no_mangle]
pub extern "C" fn Servo_StyleWorkerThreadCount() -> u32 {
    GLOBAL_STYLE_DATA.num_threads as u32
}

#[no_mangle]
pub extern "C" fn Servo_Element_ClearData(element: RawGeckoElementBorrowed) {
    GeckoElement(element).clear_data();
}

#[no_mangle]
pub extern "C" fn Servo_StyleSheet_Empty(mode: SheetParsingMode) -> RawServoStyleSheetStrong {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let origin = match mode {
        SheetParsingMode::eAuthorSheetFeatures => Origin::Author,
        SheetParsingMode::eUserSheetFeatures => Origin::User,
        SheetParsingMode::eAgentSheetFeatures => Origin::UserAgent,
    };
    let shared_lock = global_style_data.shared_lock.clone();
    Arc::new(Stylesheet::from_str(
        "", unsafe { dummy_url_data() }.clone(), origin,
        Arc::new(shared_lock.wrap(MediaList::empty())),
        shared_lock, None, &StdoutErrorReporter, 0u64)
    ).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_StyleSheet_FromUTF8Bytes(loader: *mut Loader,
                                                 stylesheet: *mut ServoStyleSheet,
                                                 data: *const nsACString,
                                                 mode: SheetParsingMode,
                                                 media_list: *const RawServoMediaList,
                                                 extra_data: *mut URLExtraData)
                                                 -> RawServoStyleSheetStrong {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let input = unsafe { data.as_ref().unwrap().as_str_unchecked() };

    let origin = match mode {
        SheetParsingMode::eAuthorSheetFeatures => Origin::Author,
        SheetParsingMode::eUserSheetFeatures => Origin::User,
        SheetParsingMode::eAgentSheetFeatures => Origin::UserAgent,
    };

    let url_data = unsafe { RefPtr::from_ptr_ref(&extra_data) };
    let loader = if loader.is_null() {
        None
    } else {
        Some(StylesheetLoader::new(loader, stylesheet))
    };

    // FIXME(emilio): loader.as_ref() doesn't typecheck for some reason?
    let loader: Option<&StyleStylesheetLoader> = match loader {
        None => None,
        Some(ref s) => Some(s),
    };

    let shared_lock = global_style_data.shared_lock.clone();
    let media = if media_list.is_null() {
        Arc::new(shared_lock.wrap(MediaList::empty()))
    } else {
        Locked::<MediaList>::as_arc(unsafe { &&*media_list }).clone()
    };

    Arc::new(Stylesheet::from_str(
        input, url_data.clone(), origin, media,
        shared_lock, loader, &StdoutErrorReporter, 0u64)
    ).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_StyleSheet_ClearAndUpdate(stylesheet: RawServoStyleSheetBorrowed,
                                                  loader: *mut Loader,
                                                  gecko_stylesheet: *mut ServoStyleSheet,
                                                  data: *const nsACString,
                                                  extra_data: *mut URLExtraData)
{
    let input = unsafe { data.as_ref().unwrap().as_str_unchecked() };
    let url_data = unsafe { RefPtr::from_ptr_ref(&extra_data) };

    let loader = if loader.is_null() {
        None
    } else {
        Some(StylesheetLoader::new(loader, gecko_stylesheet))
    };

    // FIXME(emilio): loader.as_ref() doesn't typecheck for some reason?
    let loader: Option<&StyleStylesheetLoader> = match loader {
        None => None,
        Some(ref s) => Some(s),
    };

    let sheet = Stylesheet::as_arc(&stylesheet);
    Stylesheet::update_from_str(&sheet, input, url_data,
                                loader, &StdoutErrorReporter);
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_AppendStyleSheet(raw_data: RawServoStyleSetBorrowed,
                                                  raw_sheet: RawServoStyleSheetBorrowed,
                                                  flush: bool) {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    let sheet = HasArcFFI::as_arc(&raw_sheet);
    data.stylesheets.append_stylesheet(sheet);
    if flush {
        data.flush_stylesheets(&guard);
    }
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_PrependStyleSheet(raw_data: RawServoStyleSetBorrowed,
                                                   raw_sheet: RawServoStyleSheetBorrowed,
                                                   flush: bool) {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    let sheet = HasArcFFI::as_arc(&raw_sheet);
    data.stylesheets.prepend_stylesheet(sheet);
    if flush {
        data.flush_stylesheets(&guard);
    }
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_InsertStyleSheetBefore(raw_data: RawServoStyleSetBorrowed,
                                                        raw_sheet: RawServoStyleSheetBorrowed,
                                                        raw_reference: RawServoStyleSheetBorrowed,
                                                        flush: bool) {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    let sheet = HasArcFFI::as_arc(&raw_sheet);
    let reference = HasArcFFI::as_arc(&raw_reference);
    data.stylesheets.insert_stylesheet_before(sheet, reference);
    if flush {
        data.flush_stylesheets(&guard);
    }
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_RemoveStyleSheet(raw_data: RawServoStyleSetBorrowed,
                                                  raw_sheet: RawServoStyleSheetBorrowed,
                                                  flush: bool) {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    let sheet = HasArcFFI::as_arc(&raw_sheet);
    data.stylesheets.remove_stylesheet(sheet);
    if flush {
        data.flush_stylesheets(&guard);
    }
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_FlushStyleSheets(raw_data: RawServoStyleSetBorrowed) {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    data.flush_stylesheets(&guard);
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_NoteStyleSheetsChanged(raw_data: RawServoStyleSetBorrowed,
                                                        author_style_disabled: bool) {
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    data.stylesheets.force_dirty();
    data.stylesheets.set_author_style_disabled(author_style_disabled);
}

#[no_mangle]
pub extern "C" fn Servo_StyleSheet_HasRules(raw_sheet: RawServoStyleSheetBorrowed) -> bool {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    !Stylesheet::as_arc(&raw_sheet).rules.read_with(&guard).0.is_empty()
}

#[no_mangle]
pub extern "C" fn Servo_StyleSheet_GetRules(sheet: RawServoStyleSheetBorrowed) -> ServoCssRulesStrong {
    Stylesheet::as_arc(&sheet).rules.clone().into_strong()
}

fn read_locked_arc<T, R, F>(raw: &<Locked<T> as HasFFI>::FFIType, func: F) -> R
    where Locked<T>: HasArcFFI, F: FnOnce(&T) -> R
{
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    func(Locked::<T>::as_arc(&raw).read_with(&guard))
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
pub extern "C" fn Servo_CssRules_InsertRule(rules: ServoCssRulesBorrowed,
                                            sheet: RawServoStyleSheetBorrowed,
                                            rule: *const nsACString,
                                            index: u32,
                                            nested: bool,
                                            loader: *mut Loader,
                                            gecko_stylesheet: *mut ServoStyleSheet,
                                            rule_type: *mut u16) -> nsresult {
    let sheet = Stylesheet::as_arc(&sheet);
    let loader = if loader.is_null() {
        None
    } else {
        Some(StylesheetLoader::new(loader, gecko_stylesheet))
    };
    let loader = loader.as_ref().map(|loader| loader as &StyleStylesheetLoader);
    let rule = unsafe { rule.as_ref().unwrap().as_str_unchecked() };

    let global_style_data = &*GLOBAL_STYLE_DATA;
    match Locked::<CssRules>::as_arc(&rules).insert_rule(&global_style_data.shared_lock,
                                                         rule,
                                                         sheet,
                                                         index as usize,
                                                         nested,
                                                         loader) {
        Ok(new_rule) => {
            *unsafe { rule_type.as_mut().unwrap() } = new_rule.rule_type() as u16;
            nsresult::NS_OK
        }
        Err(err) => err.into(),
    }
}

#[no_mangle]
pub extern "C" fn Servo_CssRules_DeleteRule(rules: ServoCssRulesBorrowed, index: u32) -> nsresult {
    write_locked_arc(rules, |rules: &mut CssRules| {
        match rules.remove_rule(index as usize) {
            Ok(_) => nsresult::NS_OK,
            Err(err) => err.into()
        }
    })
}

macro_rules! impl_basic_rule_funcs {
    { ($name:ident, $rule_type:ty, $raw_type:ty),
        getter: $getter:ident,
        debug: $debug:ident,
        to_css: $to_css:ident,
    } => {
        #[no_mangle]
        pub extern "C" fn $getter(rules: ServoCssRulesBorrowed, index: u32) -> Strong<$raw_type> {
            read_locked_arc(rules, |rules: &CssRules| {
                match rules.0[index as usize] {
                    CssRule::$name(ref rule) => rule.clone().into_strong(),
                    _ => {
                        unreachable!(concat!(stringify!($getter), "should only be called ",
                                             "on a ", stringify!($name), " rule"));
                    }
                }
            })
        }

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

impl_basic_rule_funcs! { (Style, StyleRule, RawServoStyleRule),
    getter: Servo_CssRules_GetStyleRuleAt,
    debug: Servo_StyleRule_Debug,
    to_css: Servo_StyleRule_GetCssText,
}

impl_basic_rule_funcs! { (Media, MediaRule, RawServoMediaRule),
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

#[no_mangle]
pub extern "C" fn Servo_CssRules_GetFontFaceRuleAt(rules: ServoCssRulesBorrowed, index: u32)
                                                   -> *mut nsCSSFontFaceRule
{
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let rules = Locked::<CssRules>::as_arc(&rules).read_with(&guard);
    match rules.0[index as usize] {
        CssRule::FontFace(ref rule) => rule.read_with(&guard).get(),
        _ => unreachable!("Servo_CssRules_GetFontFaceRuleAt should only be called on a FontFace rule"),
    }
}

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
        rule.block = declarations.clone();
    })
}

#[no_mangle]
pub extern "C" fn Servo_StyleRule_GetSelectorText(rule: RawServoStyleRuleBorrowed, result: *mut nsAString) {
    read_locked_arc(rule, |rule: &StyleRule| {
        rule.selectors.to_css(unsafe { result.as_mut().unwrap() }).unwrap();
    })
}

#[no_mangle]
pub extern "C" fn Servo_MediaRule_GetMedia(rule: RawServoMediaRuleBorrowed) -> RawServoMediaListStrong {
    read_locked_arc(rule, |rule: &MediaRule| {
        rule.media_queries.clone().into_strong()
    })
}

#[no_mangle]
pub extern "C" fn Servo_MediaRule_GetRules(rule: RawServoMediaRuleBorrowed) -> ServoCssRulesStrong {
    read_locked_arc(rule, |rule: &MediaRule| {
        rule.rules.clone().into_strong()
    })
}

#[no_mangle]
pub extern "C" fn Servo_NamespaceRule_GetPrefix(rule: RawServoNamespaceRuleBorrowed) -> *mut nsIAtom {
    read_locked_arc(rule, |rule: &NamespaceRule| {
        rule.prefix.as_ref().unwrap_or(&atom!("")).as_ptr()
    })
}

#[no_mangle]
pub extern "C" fn Servo_NamespaceRule_GetURI(rule: RawServoNamespaceRuleBorrowed) -> *mut nsIAtom {
    read_locked_arc(rule, |rule: &NamespaceRule| rule.url.0.as_ptr())
}

#[no_mangle]
pub extern "C" fn Servo_PageRule_GetStyle(rule: RawServoPageRuleBorrowed) -> RawServoDeclarationBlockStrong {
    read_locked_arc(rule, |rule: &PageRule| {
        rule.0.clone().into_strong()
    })
}

#[no_mangle]
pub extern "C" fn Servo_PageRule_SetStyle(rule: RawServoPageRuleBorrowed,
                                           declarations: RawServoDeclarationBlockBorrowed) {
    let declarations = Locked::<PropertyDeclarationBlock>::as_arc(&declarations);
    write_locked_arc(rule, |rule: &mut PageRule| {
        rule.0 = declarations.clone();
    })
}

#[no_mangle]
pub extern "C" fn Servo_ComputedValues_GetForAnonymousBox(parent_style_or_null: ServoComputedValuesBorrowedOrNull,
                                                          pseudo_tag: *mut nsIAtom,
                                                          skip_display_fixup: bool,
                                                          raw_data: RawServoStyleSetBorrowed)
     -> ServoComputedValuesStrong {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let guards = StylesheetGuards::same(&guard);
    let data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    let atom = Atom::from(pseudo_tag);
    let pseudo = PseudoElement::from_atom_unchecked(atom, /* anon_box = */ true);


    let maybe_parent = ComputedValues::arc_from_borrowed(&parent_style_or_null);
    let mut cascade_flags = CascadeFlags::empty();
    if skip_display_fixup {
        cascade_flags.insert(SKIP_ROOT_AND_ITEM_BASED_DISPLAY_FIXUP);
    }
    let metrics = get_metrics_provider_for_product();
    data.stylist.precomputed_values_for_pseudo(&guards, &pseudo, maybe_parent,
                                               cascade_flags, &metrics)
        .values.unwrap()
        .into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_ResolvePseudoStyle(element: RawGeckoElementBorrowed,
                                           pseudo_tag: *mut nsIAtom, is_probe: bool,
                                           raw_data: RawServoStyleSetBorrowed)
     -> ServoComputedValuesStrong
{
    let element = GeckoElement(element);
    let data = unsafe { element.ensure_data() }.borrow_mut();
    let doc_data = PerDocumentStyleData::from_ffi(raw_data);

    // FIXME(bholley): Assert against this.
    if data.get_styles().is_none() {
        warn!("Calling Servo_ResolvePseudoStyle on unstyled element");
        return if is_probe {
            Strong::null()
        } else {
            doc_data.borrow().default_computed_values().clone().into_strong()
        };
    }

    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    match get_pseudo_style(&guard, element, pseudo_tag, data.styles(), doc_data) {
        Some(values) => values.into_strong(),
        None if !is_probe => data.styles().primary.values().clone().into_strong(),
        None => Strong::null(),
    }
}

fn get_pseudo_style(guard: &SharedRwLockReadGuard, element: GeckoElement, pseudo_tag: *mut nsIAtom,
                    styles: &ElementStyles, doc_data: &PerDocumentStyleData)
                    -> Option<Arc<ComputedValues>>
{
    let pseudo = PseudoElement::from_atom_unchecked(Atom::from(pseudo_tag), false);
    match SelectorImpl::pseudo_element_cascade_type(&pseudo) {
        PseudoElementCascadeType::Eager => styles.pseudos.get(&pseudo).map(|s| s.values().clone()),
        PseudoElementCascadeType::Precomputed => unreachable!("No anonymous boxes"),
        PseudoElementCascadeType::Lazy => {
            let d = doc_data.borrow_mut();
            let base = styles.primary.values();
            let guards = StylesheetGuards::same(guard);
            let metrics = get_metrics_provider_for_product();
            d.stylist.lazily_compute_pseudo_element_style(&guards,
                                                          &element,
                                                          &pseudo,
                                                          base,
                                                          &metrics)
                     .map(|s| s.values().clone())
        },
    }
}

#[no_mangle]
pub extern "C" fn Servo_ComputedValues_Inherit(
  raw_data: RawServoStyleSetBorrowed,
  parent_style: ServoComputedValuesBorrowedOrNull)
     -> ServoComputedValuesStrong {
    let data = PerDocumentStyleData::from_ffi(raw_data).borrow();
    let maybe_arc = ComputedValues::arc_from_borrowed(&parent_style);
    let style = if let Some(reference) = maybe_arc.as_ref() {
        ComputedValues::inherit_from(reference, &data.default_computed_values())
    } else {
        data.default_computed_values().clone()
    };
    style.into_strong()
}

/// See the comment in `Device` to see why it's ok to pass an owned reference to
/// the pres context (hint: the context outlives the StyleSet, that holds the
/// device alive).
#[no_mangle]
pub extern "C" fn Servo_StyleSet_Init(pres_context: RawGeckoPresContextOwned)
  -> RawServoStyleSetOwned {
    let data = Box::new(PerDocumentStyleData::new(pres_context));
    data.into_ffi()
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_RebuildData(raw_data: RawServoStyleSetBorrowed) {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    data.reset_device(&guard);
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_Drop(data: RawServoStyleSetOwned) {
    let _ = data.into_box::<PerDocumentStyleData>();
}

#[no_mangle]
pub extern "C" fn Servo_ParseProperty(property: nsCSSPropertyID, value: *const nsACString,
                                      data: *mut URLExtraData)
                                      -> RawServoDeclarationBlockStrong {
    let id = get_property_id_from_nscsspropertyid!(property,
                                                   RawServoDeclarationBlockStrong::null());
    let value = unsafe { value.as_ref().unwrap().as_str_unchecked() };

    let url_data = unsafe { RefPtr::from_ptr_ref(&data) };
    let reporter = StdoutErrorReporter;
    let context = ParserContext::new(Origin::Author, url_data, &reporter,
                                     Some(CssRuleType::Style), LengthParsingMode::Default);

    match ParsedDeclaration::parse(id, &context, &mut Parser::new(value)) {
        Ok(parsed) => {
            let global_style_data = &*GLOBAL_STYLE_DATA;
            let mut block = PropertyDeclarationBlock::new();
            parsed.expand_push_into(&mut block, Importance::Normal);
            Arc::new(global_style_data.shared_lock.wrap(block)).into_strong()
        }
        Err(_) => RawServoDeclarationBlockStrong::null()
    }
}

#[no_mangle]
pub extern "C" fn Servo_ParseEasing(easing: *const nsAString,
                                    data: *mut URLExtraData,
                                    output: nsTimingFunctionBorrowedMut)
                                    -> bool {
    use style::properties::longhands::transition_timing_function;

    let url_data = unsafe { RefPtr::from_ptr_ref(&data) };
    let reporter = StdoutErrorReporter;
    let context = ParserContext::new(Origin::Author, url_data, &reporter,
                                     Some(CssRuleType::Style), LengthParsingMode::Default);
    let easing = unsafe { (*easing).to_string() };
    match transition_timing_function::single_value::parse(&context, &mut Parser::new(&easing)) {
        Ok(parsed_easing) => {
            *output = parsed_easing.into();
            true
        },
        Err(_) => false
    }
}

#[no_mangle]
pub extern "C" fn Servo_ParseStyleAttribute(data: *const nsACString,
                                            raw_extra_data: *mut URLExtraData)
                                            -> RawServoDeclarationBlockStrong {
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let value = unsafe { data.as_ref().unwrap().as_str_unchecked() };
    let url_data = unsafe { RefPtr::from_ptr_ref(&raw_extra_data) };
    Arc::new(global_style_data.shared_lock.wrap(
        GeckoElement::parse_style_attribute(value, url_data))).into_strong()
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
pub extern "C" fn Servo_DeclarationBlock_Equals(a: RawServoDeclarationBlockBorrowed,
                                                b: RawServoDeclarationBlockBorrowed)
                                                -> bool {
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
    property_id: nsCSSPropertyID, buffer: *mut nsAString)
{
    let property_id = get_property_id_from_nscsspropertyid!(property_id, ());
    read_locked_arc(declarations, |decls: &PropertyDeclarationBlock| {
        let mut string = String::new();
        let rv = decls.single_value_to_css(&property_id, &mut string);
        debug_assert!(rv.is_ok());

        write!(unsafe { &mut *buffer }, "{}", string).expect("Failed to copy string");
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_Count(declarations: RawServoDeclarationBlockBorrowed) -> u32 {
    read_locked_arc(declarations, |decls: &PropertyDeclarationBlock| {
        decls.declarations().len() as u32
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_GetNthProperty(declarations: RawServoDeclarationBlockBorrowed,
                                                        index: u32, result: *mut nsAString) -> bool {
    read_locked_arc(declarations, |decls: &PropertyDeclarationBlock| {
        if let Some(&(ref decl, _)) = decls.declarations().get(index as usize) {
            let result = unsafe { result.as_mut().unwrap() };
            decl.id().to_css(result).unwrap();
            true
        } else {
            false
        }
    })
}

macro_rules! get_property_id_from_property {
    ($property: ident, $ret: expr) => {{
        let property = unsafe { $property.as_ref().unwrap().as_str_unchecked() };
        match PropertyId::parse(Cow::Borrowed(property)) {
            Ok(property_id) => property_id,
            Err(()) => { return $ret; }
        }
    }}
}

fn get_property_value(declarations: RawServoDeclarationBlockBorrowed,
                      property_id: PropertyId, value: *mut nsAString) {
    read_locked_arc(declarations, |decls: &PropertyDeclarationBlock| {
        decls.property_value_to_css(&property_id, unsafe { value.as_mut().unwrap() }).unwrap();
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_GetPropertyValue(declarations: RawServoDeclarationBlockBorrowed,
                                                          property: *const nsACString, value: *mut nsAString) {
    get_property_value(declarations, get_property_id_from_property!(property, ()), value)
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_GetPropertyValueById(declarations: RawServoDeclarationBlockBorrowed,
                                                              property: nsCSSPropertyID, value: *mut nsAString) {
    get_property_value(declarations, get_property_id_from_nscsspropertyid!(property, ()), value)
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_GetPropertyIsImportant(declarations: RawServoDeclarationBlockBorrowed,
                                                                property: *const nsACString) -> bool {
    let property_id = get_property_id_from_property!(property, false);
    read_locked_arc(declarations, |decls: &PropertyDeclarationBlock| {
        decls.property_priority(&property_id).important()
    })
}

fn set_property(declarations: RawServoDeclarationBlockBorrowed, property_id: PropertyId,
                value: *const nsACString, is_important: bool, data: *mut URLExtraData,
                length_parsing_mode: structs::LengthParsingMode) -> bool {
    let value = unsafe { value.as_ref().unwrap().as_str_unchecked() };
    let url_data = unsafe { RefPtr::from_ptr_ref(&data) };
    let length_parsing_mode = match length_parsing_mode {
        structs::LengthParsingMode::Default => LengthParsingMode::Default,
        structs::LengthParsingMode::SVG => LengthParsingMode::SVG,
    };
    if let Ok(parsed) = parse_one_declaration(property_id, value, url_data, &StdoutErrorReporter,
                                              length_parsing_mode) {
        let importance = if is_important { Importance::Important } else { Importance::Normal };
        write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
            parsed.expand_set_into(decls, importance)
        })
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetProperty(declarations: RawServoDeclarationBlockBorrowed,
                                                     property: *const nsACString, value: *const nsACString,
                                                     is_important: bool, data: *mut URLExtraData,
                                                     length_parsing_mode: structs::LengthParsingMode) -> bool {
    set_property(declarations, get_property_id_from_property!(property, false),
                 value, is_important, data, length_parsing_mode)
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetPropertyById(declarations: RawServoDeclarationBlockBorrowed,
                                                         property: nsCSSPropertyID, value: *const nsACString,
                                                         is_important: bool, data: *mut URLExtraData,
                                                         length_parsing_mode: structs::LengthParsingMode) -> bool {
    set_property(declarations, get_property_id_from_nscsspropertyid!(property, false),
                 value, is_important, data, length_parsing_mode)
}

fn remove_property(declarations: RawServoDeclarationBlockBorrowed, property_id: PropertyId) {
    write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
        decls.remove_property(&property_id);
    });
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_RemoveProperty(declarations: RawServoDeclarationBlockBorrowed,
                                                        property: *const nsACString) {
    remove_property(declarations, get_property_id_from_property!(property, ()))
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_RemovePropertyById(declarations: RawServoDeclarationBlockBorrowed,
                                                            property: nsCSSPropertyID) {
    remove_property(declarations, get_property_id_from_nscsspropertyid!(property, ()))
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
pub extern "C" fn Servo_MediaList_Matches(list: RawServoMediaListBorrowed,
                                          raw_data: RawServoStyleSetBorrowed)
                                          -> bool {
    let per_doc_data = PerDocumentStyleData::from_ffi(raw_data).borrow();
    read_locked_arc(list, |list: &MediaList| {
        list.evaluate(&per_doc_data.stylist.device)
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_HasCSSWideKeyword(declarations: RawServoDeclarationBlockBorrowed,
                                                           property: nsCSSPropertyID) -> bool {
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
pub extern "C" fn Servo_MediaList_SetText(list: RawServoMediaListBorrowed, text: *const nsACString) {
    let text = unsafe { text.as_ref().unwrap().as_str_unchecked() };
    let mut parser = Parser::new(&text);
    let url_data = unsafe { dummy_url_data() };
    let reporter = StdoutErrorReporter;
    let context = ParserContext::new_for_cssom(url_data, &reporter, Some(CssRuleType::Media),
                                               LengthParsingMode::Default);
     write_locked_arc(list, |list: &mut MediaList| {
        *list = parse_media_query_list(&context, &mut parser);
    })
}

#[no_mangle]
pub extern "C" fn Servo_MediaList_GetLength(list: RawServoMediaListBorrowed) -> u32 {
    read_locked_arc(list, |list: &MediaList| list.media_queries.len() as u32)
}

#[no_mangle]
pub extern "C" fn Servo_MediaList_GetMediumAt(list: RawServoMediaListBorrowed, index: u32,
                                              result: *mut nsAString) -> bool {
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
pub extern "C" fn Servo_MediaList_AppendMedium(list: RawServoMediaListBorrowed,
                                               new_medium: *const nsACString) {
    let new_medium = unsafe { new_medium.as_ref().unwrap().as_str_unchecked() };
    let url_data = unsafe { dummy_url_data() };
    let reporter = StdoutErrorReporter;
    let context = ParserContext::new_for_cssom(url_data, &reporter, Some(CssRuleType::Media),
                                               LengthParsingMode::Default);
    write_locked_arc(list, |list: &mut MediaList| {
        list.append_medium(&context, new_medium);
    })
}

#[no_mangle]
pub extern "C" fn Servo_MediaList_DeleteMedium(list: RawServoMediaListBorrowed,
                                               old_medium: *const nsACString) -> bool {
    let old_medium = unsafe { old_medium.as_ref().unwrap().as_str_unchecked() };
    let url_data = unsafe { dummy_url_data() };
    let reporter = StdoutErrorReporter;
    let context = ParserContext::new_for_cssom(url_data, &reporter, Some(CssRuleType::Media),
                                               LengthParsingMode::Default);
    write_locked_arc(list, |list: &mut MediaList| list.delete_medium(&context, old_medium))
}

macro_rules! get_longhand_from_id {
    ($id:expr, $retval:expr) => {
        match PropertyId::from_nscsspropertyid($id) {
            Ok(PropertyId::Longhand(long)) => long,
            _ => {
                error!("stylo: unknown presentation property with id {:?}", $id);
                return $retval
            }
        }
    };
    ($id:expr) => {
        get_longhand_from_id!($id, ())
    }
}

macro_rules! match_wrap_declared {
    ($longhand:ident, $($property:ident => $inner:expr,)*) => (
        match $longhand {
            $(
                LonghandId::$property => PropertyDeclaration::$property($inner),
            )*
            _ => {
                error!("stylo: Don't know how to handle presentation property {:?}", $longhand);
                return
            }
        }
    )
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_PropertyIsSet(declarations:
                                                       RawServoDeclarationBlockBorrowed,
                                                       property: nsCSSPropertyID)
        -> bool {
    use style::properties::PropertyDeclarationId;
    let long = get_longhand_from_id!(property, false);
    read_locked_arc(declarations, |decls: &PropertyDeclarationBlock| {
        decls.get(PropertyDeclarationId::Longhand(long)).is_some()
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetIdentStringValue(declarations:
                                                             RawServoDeclarationBlockBorrowed,
                                                             property:
                                                             nsCSSPropertyID,
                                                             value:
                                                             *mut nsIAtom) {
    use style::properties::{PropertyDeclaration, LonghandId};
    use style::properties::longhands::_x_lang::computed_value::T as Lang;

    let long = get_longhand_from_id!(property);
    let prop = match_wrap_declared! { long,
        XLang => Lang(Atom::from(value)),
    };
    write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
        decls.push(prop, Importance::Normal);
    })
}

#[no_mangle]
#[allow(unreachable_code)]
pub extern "C" fn Servo_DeclarationBlock_SetKeywordValue(declarations:
                                                         RawServoDeclarationBlockBorrowed,
                                                         property: nsCSSPropertyID,
                                                         value: i32) {
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
        FontStyle => longhands::font_style::SpecifiedValue::from_gecko_keyword(value),
        FontWeight => longhands::font_weight::SpecifiedValue::from_gecko_keyword(value),
        ListStyleType => longhands::list_style_type::SpecifiedValue::from_gecko_keyword(value),
        MozMathVariant => longhands::_moz_math_variant::SpecifiedValue::from_gecko_keyword(value),
        WhiteSpace => longhands::white_space::SpecifiedValue::from_gecko_keyword(value),
        CaptionSide => longhands::caption_side::SpecifiedValue::from_gecko_keyword(value),
        BorderTopStyle => BorderStyle::from_gecko_keyword(value),
        BorderRightStyle => BorderStyle::from_gecko_keyword(value),
        BorderBottomStyle => BorderStyle::from_gecko_keyword(value),
        BorderLeftStyle => BorderStyle::from_gecko_keyword(value),
    };
    write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
        decls.push(prop, Importance::Normal);
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetIntValue(declarations: RawServoDeclarationBlockBorrowed,
                                                     property: nsCSSPropertyID,
                                                     value: i32) {
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
        decls.push(prop, Importance::Normal);
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetPixelValue(declarations:
                                                       RawServoDeclarationBlockBorrowed,
                                                       property: nsCSSPropertyID,
                                                       value: f32) {
    use style::properties::{PropertyDeclaration, LonghandId};
    use style::properties::longhands::border_spacing::SpecifiedValue as BorderSpacing;
    use style::values::specified::BorderWidth;
    use style::values::specified::length::NoCalcLength;

    let long = get_longhand_from_id!(property);
    let nocalc = NoCalcLength::from_px(value);

    let prop = match_wrap_declared! { long,
        Height => nocalc.into(),
        Width => nocalc.into(),
        BorderTopWidth => BorderWidth::Width(nocalc.into()),
        BorderRightWidth => BorderWidth::Width(nocalc.into()),
        BorderBottomWidth => BorderWidth::Width(nocalc.into()),
        BorderLeftWidth => BorderWidth::Width(nocalc.into()),
        MarginTop => nocalc.into(),
        MarginRight => nocalc.into(),
        MarginBottom => nocalc.into(),
        MarginLeft => nocalc.into(),
        PaddingTop => nocalc.into(),
        PaddingRight => nocalc.into(),
        PaddingBottom => nocalc.into(),
        PaddingLeft => nocalc.into(),
        BorderSpacing => Box::new(
            BorderSpacing {
                horizontal: nocalc.into(),
                vertical: None,
            }
        ),
    };
    write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
        decls.push(prop, Importance::Normal);
    })
}


#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetLengthValue(declarations:
                                                        RawServoDeclarationBlockBorrowed,
                                                        property: nsCSSPropertyID,
                                                        value: f32,
                                                        unit: structs::nsCSSUnit) {
    use style::properties::{PropertyDeclaration, LonghandId};
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
        Width => nocalc.into(),
        FontSize => LengthOrPercentage::from(nocalc).into(),
        MozScriptMinSize => nocalc.into(),
    };
    write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
        decls.push(prop, Importance::Normal);
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetNumberValue(declarations:
                                                       RawServoDeclarationBlockBorrowed,
                                                       property: nsCSSPropertyID,
                                                       value: f32) {
    use style::properties::{PropertyDeclaration, LonghandId};
    use style::properties::longhands::_moz_script_level::SpecifiedValue as MozScriptLevel;

    let long = get_longhand_from_id!(property);

    let prop = match_wrap_declared! { long,
        MozScriptSizeMultiplier => value,
        // Gecko uses Number values to signal that it is absolute
        MozScriptLevel => MozScriptLevel::Absolute(value as i32),
    };
    write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
        decls.push(prop, Importance::Normal);
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetPercentValue(declarations:
                                                         RawServoDeclarationBlockBorrowed,
                                                         property: nsCSSPropertyID,
                                                         value: f32) {
    use style::properties::{PropertyDeclaration, LonghandId};
    use style::values::specified::length::{LengthOrPercentage, Percentage};

    let long = get_longhand_from_id!(property);
    let pc = Percentage(value);

    let prop = match_wrap_declared! { long,
        Height => pc.into(),
        Width => pc.into(),
        MarginTop => pc.into(),
        MarginRight => pc.into(),
        MarginBottom => pc.into(),
        MarginLeft => pc.into(),
        FontSize => LengthOrPercentage::from(pc).into(),
    };
    write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
        decls.push(prop, Importance::Normal);
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetAutoValue(declarations:
                                                      RawServoDeclarationBlockBorrowed,
                                                      property: nsCSSPropertyID) {
    use style::properties::{PropertyDeclaration, LonghandId};
    use style::values::specified::LengthOrPercentageOrAuto;

    let long = get_longhand_from_id!(property);
    let auto = LengthOrPercentageOrAuto::Auto;

    let prop = match_wrap_declared! { long,
        Height => auto,
        Width => auto,
        MarginTop => auto,
        MarginRight => auto,
        MarginBottom => auto,
        MarginLeft => auto,
    };
    write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
        decls.push(prop, Importance::Normal);
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetCurrentColor(declarations:
                                                         RawServoDeclarationBlockBorrowed,
                                                         property: nsCSSPropertyID) {
    use style::properties::{PropertyDeclaration, LonghandId};
    use style::values::specified::{Color, CSSColor};

    let long = get_longhand_from_id!(property);
    let cc = CSSColor { parsed: Color::CurrentColor, authored: None };

    let prop = match_wrap_declared! { long,
        BorderTopColor => cc,
        BorderRightColor => cc,
        BorderBottomColor => cc,
        BorderLeftColor => cc,
    };
    write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
        decls.push(prop, Importance::Normal);
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetColorValue(declarations:
                                                       RawServoDeclarationBlockBorrowed,
                                                       property: nsCSSPropertyID,
                                                       value: structs::nscolor) {
    use style::gecko::values::convert_nscolor_to_rgba;
    use style::properties::{PropertyDeclaration, LonghandId};
    use style::properties::longhands;
    use style::values::specified::{Color, CSSColor};

    let long = get_longhand_from_id!(property);
    let rgba = convert_nscolor_to_rgba(value);
    let color = CSSColor { parsed: Color::RGBA(rgba), authored: None };

    let prop = match_wrap_declared! { long,
        BorderTopColor => color,
        BorderRightColor => color,
        BorderBottomColor => color,
        BorderLeftColor => color,
        Color => longhands::color::SpecifiedValue(color),
        BackgroundColor => color,
    };
    write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
        decls.push(prop, Importance::Normal);
    })
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetFontFamily(declarations:
                                                       RawServoDeclarationBlockBorrowed,
                                                       value: *const nsAString) {
    use cssparser::Parser;
    use style::properties::PropertyDeclaration;
    use style::properties::longhands::font_family::SpecifiedValue as FontFamily;

    let string = unsafe { (*value).to_string() };
    let mut parser = Parser::new(&string);
    if let Ok(family) = FontFamily::parse(&mut parser) {
        if parser.is_exhausted() {
            let decl = PropertyDeclaration::FontFamily(family);
            write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
                decls.push(decl, Importance::Normal);
            })
        }
    }
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetBackgroundImage(declarations:
                                                            RawServoDeclarationBlockBorrowed,
                                                            value: *const nsAString,
                                                            raw_extra_data: *mut URLExtraData) {
    use style::properties::PropertyDeclaration;
    use style::properties::longhands::background_image::SpecifiedValue as BackgroundImage;
    use style::properties::longhands::background_image::single_value::SpecifiedValue as SingleBackgroundImage;
    use style::values::specified::image::Image;
    use style::values::specified::url::SpecifiedUrl;

    let url_data = unsafe { RefPtr::from_ptr_ref(&raw_extra_data) };
    let string = unsafe { (*value).to_string() };
    let error_reporter = StdoutErrorReporter;
    let context = ParserContext::new(Origin::Author, url_data, &error_reporter,
                                     Some(CssRuleType::Style), LengthParsingMode::Default);
    if let Ok(url) = SpecifiedUrl::parse_from_string(string.into(), &context) {
        let decl = PropertyDeclaration::BackgroundImage(BackgroundImage(
            vec![SingleBackgroundImage(
                Some(Image::Url(url))
            )]
        ));
        write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
            decls.push(decl, Importance::Normal);
        })
    }
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetTextDecorationColorOverride(declarations:
                                                                RawServoDeclarationBlockBorrowed) {
    use style::properties::PropertyDeclaration;
    use style::properties::longhands::text_decoration_line;

    let mut decoration = text_decoration_line::computed_value::none;
    decoration |= text_decoration_line::COLOR_OVERRIDE;
    let decl = PropertyDeclaration::TextDecorationLine(decoration);
    write_locked_arc(declarations, |decls: &mut PropertyDeclarationBlock| {
        decls.push(decl, Importance::Normal);
    })
}

#[no_mangle]
pub extern "C" fn Servo_CSSSupports2(property: *const nsACString, value: *const nsACString) -> bool {
    let property = unsafe { property.as_ref().unwrap().as_str_unchecked() };
    let id =  if let Ok(id) = PropertyId::parse(property.into()) {
        id
    } else {
        return false
    };
    let value = unsafe { value.as_ref().unwrap().as_str_unchecked() };

    let url_data = unsafe { dummy_url_data() };
    parse_one_declaration(id, &value, url_data, &StdoutErrorReporter, LengthParsingMode::Default).is_ok()
}

#[no_mangle]
pub extern "C" fn Servo_CSSSupports(cond: *const nsACString) -> bool {
    let condition = unsafe { cond.as_ref().unwrap().as_str_unchecked() };
    let mut input = Parser::new(&condition);
    let cond = parse_condition_or_declaration(&mut input);
    if let Ok(cond) = cond {
        let url_data = unsafe { dummy_url_data() };
        let reporter = StdoutErrorReporter;
        let context = ParserContext::new_for_cssom(url_data, &reporter, Some(CssRuleType::Style),
                                                   LengthParsingMode::Default);
        cond.eval(&context)
    } else {
        false
    }
}

/// Only safe to call on the main thread, with exclusive access to the element and
/// its ancestors.
unsafe fn maybe_restyle<'a>(data: &'a mut AtomicRefMut<ElementData>,
                            element: GeckoElement,
                            animation_only: bool)
    -> Option<&'a mut RestyleData>
{
    // Don't generate a useless RestyleData if the element hasn't been styled.
    if !data.has_styles() {
        return None;
    }

    // Propagate the bit up the chain.
    if animation_only {
        element.parent_element().map(|p| p.note_descendants::<AnimationOnlyDirtyDescendants>());
    } else  {
        element.parent_element().map(|p| p.note_descendants::<DirtyDescendants>());
    };

    bindings::Gecko_SetOwnerDocumentNeedsStyleFlush(element.0);

    // Ensure and return the RestyleData.
    Some(data.ensure_restyle())
}

#[no_mangle]
pub extern "C" fn Servo_Element_GetSnapshot(element: RawGeckoElementBorrowed) -> *mut structs::ServoElementSnapshot
{
    let element = GeckoElement(element);
    let snapshot = match element.mutate_data() {
        None => ptr::null_mut(),
        Some(mut data) => {
            if let Some(restyle_data) = unsafe { maybe_restyle(&mut data, element, false) } {
                restyle_data.snapshot.ensure(|| element.create_snapshot()).borrow_mut_raw()
            } else {
                ptr::null_mut()
            }
        },
    };

    debug!("Servo_Element_GetSnapshot: {:?}: {:?}", element, snapshot);
    snapshot
}

#[no_mangle]
pub extern "C" fn Servo_Element_GetStyleRuleList(element: RawGeckoElementBorrowed,
                                                 rules: RawGeckoServoStyleRuleListBorrowedMut) {
    let element = GeckoElement(element);
    let data = match element.borrow_data() {
        Some(element_data) => element_data,
        None => return,
    };
    let computed = match data.get_styles() {
        Some(styles) => &styles.primary,
        None => return,
    };
    let mut result = vec![];
    for rule_node in computed.rules.self_and_ancestors() {
        if let Some(&StyleSource::Style(ref rule)) = rule_node.style_source() {
            result.push(Locked::<StyleRule>::arc_as_borrowed(&rule));
        }
    }
    unsafe { rules.set_len(result.len() as u32) };
    for (&src, dest) in result.into_iter().zip(rules.iter_mut()) {
        *dest = src;
    }
}

#[no_mangle]
pub extern "C" fn Servo_NoteExplicitHints(element: RawGeckoElementBorrowed,
                                          restyle_hint: nsRestyleHint,
                                          change_hint: nsChangeHint) {
    let element = GeckoElement(element);
    let damage = GeckoRestyleDamage::new(change_hint);
    debug!("Servo_NoteExplicitHints: {:?}, restyle_hint={:?}, change_hint={:?}",
           element, restyle_hint, change_hint);
    debug_assert!(restyle_hint == structs::nsRestyleHint_eRestyle_CSSAnimations ||
                  restyle_hint == structs::nsRestyleHint_eRestyle_CSSTransitions ||
                  (restyle_hint.0 & (structs::nsRestyleHint_eRestyle_CSSAnimations.0 |
                                     structs::nsRestyleHint_eRestyle_CSSTransitions.0)) == 0,
                  "eRestyle_CSSAnimations or eRestyle_CSSTransitions should only appear by itself");

    let mut maybe_data = element.mutate_data();
    let maybe_restyle_data = maybe_data.as_mut().and_then(|d| unsafe {
        maybe_restyle(d, element, restyle_hint == structs::nsRestyleHint_eRestyle_CSSAnimations ||
                                  restyle_hint == structs::nsRestyleHint_eRestyle_CSSTransitions)
    });
    if let Some(restyle_data) = maybe_restyle_data {
        let restyle_hint: RestyleHint = restyle_hint.into();
        restyle_data.hint.insert(&restyle_hint.into());
        restyle_data.damage |= damage;
    } else {
        debug!("(Element not styled, discarding hints)");
    }
}

#[no_mangle]
pub extern "C" fn Servo_ImportRule_GetSheet(import_rule:
                                            RawServoImportRuleBorrowed)
                                            -> RawServoStyleSheetStrong {
    read_locked_arc(import_rule, |rule: &ImportRule| {
        rule.stylesheet.clone().into_strong()
    })
}

#[no_mangle]
pub extern "C" fn Servo_TakeChangeHint(element: RawGeckoElementBorrowed) -> nsChangeHint
{
    let element = GeckoElement(element);
    let damage = if let Some(mut data) = element.mutate_data() {
        let d = data.get_restyle().map_or(GeckoRestyleDamage::empty(), |r| r.damage);
        data.clear_restyle();
        d
    } else {
        warn!("Trying to get change hint from unstyled element");
        GeckoRestyleDamage::empty()
    };

    debug!("Servo_TakeChangeHint: {:?}, damage={:?}", element, damage);
    damage.as_change_hint()
}

#[no_mangle]
pub extern "C" fn Servo_ResolveStyle(element: RawGeckoElementBorrowed,
                                     raw_data: RawServoStyleSetBorrowed,
                                     allow_stale: bool)
                                     -> ServoComputedValuesStrong
{
    let element = GeckoElement(element);
    debug!("Servo_ResolveStyle: {:?}", element);
    let data = unsafe { element.ensure_data() }.borrow_mut();

    let valid_styles = if allow_stale {
      data.has_styles()
    } else {
      data.has_current_styles()
    };

    if !valid_styles {
        debug_assert!(false, "Resolving style on element without current styles with lazy \
                              computation forbidden.");
        let per_doc_data = PerDocumentStyleData::from_ffi(raw_data).borrow();
        return per_doc_data.default_computed_values().clone().into_strong();
    }

    data.styles().primary.values().clone().into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_ResolveStyleLazily(element: RawGeckoElementBorrowed,
                                           pseudo_tag: *mut nsIAtom,
                                           raw_data: RawServoStyleSetBorrowed)
     -> ServoComputedValuesStrong
{
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let element = GeckoElement(element);
    let doc_data = PerDocumentStyleData::from_ffi(raw_data);
    let finish = |styles: &ElementStyles| -> Arc<ComputedValues> {
        let maybe_pseudo = if !pseudo_tag.is_null() {
            get_pseudo_style(&guard, element, pseudo_tag, styles, doc_data)
        } else {
            None
        };
        maybe_pseudo.unwrap_or_else(|| styles.primary.values().clone())
    };

    // In the common case we already have the style. Check that before setting
    // up all the computation machinery.
    let mut result = element.mutate_data()
                            .and_then(|d| d.get_styles().map(&finish));
    if result.is_some() {
        return result.unwrap().into_strong();
    }

    // We don't have the style ready. Go ahead and compute it as necessary.
    let shared = create_shared_context(&global_style_data,
                                       &guard,
                                       &mut doc_data.borrow_mut(),
                                       TraversalFlags::empty());
    let mut tlc = ThreadLocalStyleContext::new(&shared);
    let mut context = StyleContext {
        shared: &shared,
        thread_local: &mut tlc,
    };
    let ensure = |el: GeckoElement| { unsafe { el.ensure_data(); } };
    let clear = |el: GeckoElement| el.clear_data();
    resolve_style(&mut context, element, &ensure, &clear,
                  |styles| result = Some(finish(styles)));

    result.unwrap().into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_GetComputedKeyframeValues(keyframes: RawGeckoKeyframeListBorrowed,
                                                  style: ServoComputedValuesBorrowed,
                                                  parent_style: ServoComputedValuesBorrowedOrNull,
                                                  raw_data: RawServoStyleSetBorrowed,
                                                  computed_keyframes: RawGeckoComputedKeyframeValuesListBorrowedMut)
{
    use std::mem;
    use style::properties::LonghandIdSet;
    use style::properties::declaration_block::Importance;
    use style::values::computed::Context;

    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();


    let data = PerDocumentStyleData::from_ffi(raw_data).borrow();

    let style = ComputedValues::as_arc(&style);
    let parent_style = parent_style.as_ref().map(|r| &**ComputedValues::as_arc(&r));

    let default_values = data.default_computed_values();
    let metrics = get_metrics_provider_for_product();

    let context = Context {
        is_root_element: false,
        device: &data.stylist.device,
        inherited_style: parent_style.unwrap_or(default_values),
        layout_parent_style: parent_style.unwrap_or(default_values),
        style: (**style).clone(),
        font_metrics_provider: &metrics,
        in_media_query: false,
    };

    for (index, keyframe) in keyframes.iter().enumerate() {
        let ref mut animation_values = computed_keyframes[index];

        let mut seen = LonghandIdSet::new();

        // mServoDeclarationBlock is null in the case where we have an invalid css property.
        let iter = keyframe.mPropertyValues.iter()
                                           .filter(|&property| !property.mServoDeclarationBlock.mRawPtr.is_null());
        for property in iter {
            let declarations = unsafe { &*property.mServoDeclarationBlock.mRawPtr.clone() };
            let declarations = Locked::<PropertyDeclarationBlock>::as_arc(&declarations);
            let guard = declarations.read_with(&guard);

            let anim_iter = guard.declarations()
                            .iter()
                            .filter_map(|&(ref decl, imp)| {
                                if imp == Importance::Normal {
                                    let property = TransitionProperty::from_declaration(decl);
                                    let animation = AnimationValue::from_declaration(decl, &context, default_values);
                                    debug_assert!(property.is_none() == animation.is_none(),
                                                  "The failure condition of TransitionProperty::from_declaration \
                                                   and AnimationValue::from_declaration should be the same");
                                    // Skip the property if either ::from_declaration fails.
                                    if property.is_none() || animation.is_none() {
                                        None
                                    } else {
                                        Some((property.unwrap(), animation.unwrap()))
                                    }
                                } else {
                                    None
                                }
                            });

            for (i, anim) in anim_iter.enumerate() {
                if !seen.has_transition_property_bit(&anim.0) {
                    // This is safe since we immediately write to the uninitialized values.
                    unsafe { animation_values.set_len((i + 1) as u32) };
                    seen.set_transition_property_bit(&anim.0);
                    animation_values[i].mProperty = anim.0.into();
                    // We only make sure we have enough space for this variable,
                    // but didn't construct a default value for StyleAnimationValue,
                    // so we should zero it to avoid getting undefined behaviors.
                    animation_values[i].mValue.mGecko = unsafe { mem::zeroed() };
                    animation_values[i].mValue.mServo.set_arc_leaky(Arc::new(anim.1));
                }
            }
        }
    }
}


#[no_mangle]
pub extern "C" fn Servo_AssertTreeIsClean(root: RawGeckoElementBorrowed) {
    if !cfg!(debug_assertions) {
        panic!("Calling Servo_AssertTreeIsClean in release build");
    }

    let root = GeckoElement(root);
    fn assert_subtree_is_clean<'le>(el: GeckoElement<'le>) {
        debug_assert!(!el.has_dirty_descendants() && !el.has_animation_only_dirty_descendants());
        for child in el.as_node().children() {
            if let Some(child) = child.as_element() {
                assert_subtree_is_clean(child);
            }
        }
    }

    assert_subtree_is_clean(root);
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_FillKeyframesForName(raw_data: RawServoStyleSetBorrowed,
                                                      name: *const nsACString,
                                                      timing_function: nsTimingFunctionBorrowed,
                                                      style: ServoComputedValuesBorrowed,
                                                      keyframes: RawGeckoKeyframeListBorrowedMut) -> bool {
    use style::gecko_bindings::structs::Keyframe;
    use style::properties::LonghandIdSet;


    let data = PerDocumentStyleData::from_ffi(raw_data).borrow();
    let name = unsafe { Atom::from(name.as_ref().unwrap().as_str_unchecked()) };
    let style = ComputedValues::as_arc(&style);

    if let Some(ref animation) = data.stylist.animations().get(&name) {
        let global_style_data = &*GLOBAL_STYLE_DATA;
        let guard = global_style_data.shared_lock.read();
        for step in &animation.steps {
            // Override timing_function if the keyframe has animation-timing-function.
            let timing_function = if let Some(val) = step.get_animation_timing_function(&guard) {
                val.into()
            } else {
                *timing_function
            };

            let keyframe = unsafe {
                Gecko_AnimationAppendKeyframe(keyframes,
                                              step.start_percentage.0 as f32,
                                              &timing_function)
            };

            fn add_computed_property_value(keyframe: *mut Keyframe,
                                           index: usize,
                                           style: &ComputedValues,
                                           property: &TransitionProperty,
                                           shared_lock: &SharedRwLock) {
                let block = style.to_declaration_block(property.clone().into());
                unsafe {
                    (*keyframe).mPropertyValues.set_len((index + 1) as u32);
                    (*keyframe).mPropertyValues[index].mProperty = property.clone().into();
                    // FIXME. Do not set computed values once we handles missing keyframes
                    // with additive composition.
                    (*keyframe).mPropertyValues[index].mServoDeclarationBlock.set_arc_leaky(
                        Arc::new(shared_lock.wrap(block)));
                }
            }

            match step.value {
                KeyframesStepValue::ComputedValues => {
                    for (index, property) in animation.properties_changed.iter().enumerate() {
                        add_computed_property_value(
                            keyframe, index, style, property, &global_style_data.shared_lock);
                    }
                },
                KeyframesStepValue::Declarations { ref block } => {
                    let guard = block.read_with(&guard);
                    // Filter out non-animatable properties.
                    let animatable =
                        guard.declarations()
                             .iter()
                             .filter(|&&(ref declaration, _)| {
                                 declaration.is_animatable()
                             });

                    let mut seen = LonghandIdSet::new();

                    for (index, &(ref declaration, _)) in animatable.enumerate() {
                        unsafe {
                            let property = TransitionProperty::from_declaration(declaration).unwrap();
                            (*keyframe).mPropertyValues.set_len((index + 1) as u32);
                            (*keyframe).mPropertyValues[index].mProperty = property.into();
                            (*keyframe).mPropertyValues[index].mServoDeclarationBlock.set_arc_leaky(
                                Arc::new(global_style_data.shared_lock.wrap(
                                  PropertyDeclarationBlock::with_one(
                                      declaration.clone(), Importance::Normal
                                ))));
                            if step.start_percentage.0 == 0. ||
                               step.start_percentage.0 == 1. {
                                seen.set_transition_property_bit(&property);
                            }
                        }
                    }

                    // Append missing property values in the initial or the finial keyframes.
                    if step.start_percentage.0 == 0. ||
                       step.start_percentage.0 == 1. {
                        let mut index = unsafe { (*keyframe).mPropertyValues.len() };
                        for property in animation.properties_changed.iter() {
                            if !seen.has_transition_property_bit(&property) {
                                add_computed_property_value(
                                    keyframe, index, style, property, &global_style_data.shared_lock);
                                index += 1;
                            }
                        }
                    }
                },
            }
        }
        return true
    }
    false
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_GetFontFaceRules(raw_data: RawServoStyleSetBorrowed,
                                                  rules: RawGeckoFontFaceRuleListBorrowedMut) {
    let data = PerDocumentStyleData::from_ffi(raw_data).borrow();
    debug_assert!(rules.len() == 0);

    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();

    unsafe { rules.set_len(data.font_faces.len() as u32) };
    for (src, dest) in data.font_faces.iter().zip(rules.iter_mut()) {
        dest.mRule = src.0.read_with(&guard).clone().forget();
        dest.mSheetType = src.1.into();
    }
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_ResolveForDeclarations(raw_data: RawServoStyleSetBorrowed,
                                                        parent_style_or_null: ServoComputedValuesBorrowedOrNull,
                                                        declarations: RawServoDeclarationBlockBorrowed)
                                                        -> ServoComputedValuesStrong
{
    let doc_data = PerDocumentStyleData::from_ffi(raw_data).borrow();
    let global_style_data = &*GLOBAL_STYLE_DATA;
    let guard = global_style_data.shared_lock.read();
    let guards = StylesheetGuards::same(&guard);

    let parent_style = match ComputedValues::arc_from_borrowed(&parent_style_or_null) {
        Some(parent) => &parent,
        None => doc_data.default_computed_values(),
    };

    let declarations = Locked::<PropertyDeclarationBlock>::as_arc(&declarations);

    doc_data.stylist.compute_for_declarations(&guards,
                                              parent_style,
                                              declarations.clone()).into_strong()
}

