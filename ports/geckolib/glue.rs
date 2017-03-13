/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use atomic_refcell::AtomicRefMut;
use cssparser::Parser;
use cssparser::ToCss as ParserToCss;
use env_logger::LogBuilder;
use num_cpus;
use parking_lot::RwLock;
use rayon;
use selectors::Element;
use servo_url::ServoUrl;
use std::borrow::Cow;
use std::cmp;
use std::env;
use std::fmt::Write;
use std::ptr;
use std::sync::{Arc, Mutex};
use style::arc_ptr_eq;
use style::context::{QuirksMode, SharedStyleContext, StyleContext};
use style::context::{ThreadLocalStyleContext, ThreadLocalStyleContextCreationInfo};
use style::data::{ElementData, ElementStyles, RestyleData};
use style::dom::{ShowSubtreeData, TElement, TNode};
use style::error_reporting::StdoutErrorReporter;
use style::gecko::data::{PerDocumentStyleData, PerDocumentStyleDataImpl};
use style::gecko::restyle_damage::GeckoRestyleDamage;
use style::gecko::selector_parser::{SelectorImpl, PseudoElement};
use style::gecko::traversal::RecalcStyleOnly;
use style::gecko::wrapper::DUMMY_BASE_URL;
use style::gecko::wrapper::GeckoElement;
use style::gecko_bindings::bindings;
use style::gecko_bindings::bindings::{RawGeckoKeyframeListBorrowed, RawGeckoKeyframeListBorrowedMut};
use style::gecko_bindings::bindings::{RawServoDeclarationBlockBorrowed, RawServoDeclarationBlockStrong};
use style::gecko_bindings::bindings::{RawServoMediaListBorrowed, RawServoMediaListStrong};
use style::gecko_bindings::bindings::{RawServoMediaRuleBorrowed, RawServoMediaRuleStrong};
use style::gecko_bindings::bindings::{RawServoStyleRuleBorrowed, RawServoStyleRuleStrong};
use style::gecko_bindings::bindings::{RawServoStyleSetBorrowed, RawServoStyleSetOwned};
use style::gecko_bindings::bindings::{RawServoStyleSheetBorrowed, ServoComputedValuesBorrowed};
use style::gecko_bindings::bindings::{RawServoStyleSheetStrong, ServoComputedValuesStrong};
use style::gecko_bindings::bindings::{ServoCssRulesBorrowed, ServoCssRulesStrong};
use style::gecko_bindings::bindings::{nsACString, nsAString};
use style::gecko_bindings::bindings::Gecko_AnimationAppendKeyframe;
use style::gecko_bindings::bindings::RawGeckoComputedKeyframeValuesListBorrowedMut;
use style::gecko_bindings::bindings::RawGeckoElementBorrowed;
use style::gecko_bindings::bindings::RawServoAnimationValueBorrowed;
use style::gecko_bindings::bindings::RawServoAnimationValueStrong;
use style::gecko_bindings::bindings::RawServoImportRuleBorrowed;
use style::gecko_bindings::bindings::ServoComputedValuesBorrowedOrNull;
use style::gecko_bindings::bindings::nsTArrayBorrowed_uintptr_t;
use style::gecko_bindings::structs;
use style::gecko_bindings::structs::{SheetParsingMode, nsIAtom, nsCSSPropertyID};
use style::gecko_bindings::structs::{ThreadSafePrincipalHolder, ThreadSafeURIHolder};
use style::gecko_bindings::structs::{nsRestyleHint, nsChangeHint};
use style::gecko_bindings::structs::Loader;
use style::gecko_bindings::structs::RawGeckoPresContextOwned;
use style::gecko_bindings::structs::RawServoAnimationValueBorrowedListBorrowed;
use style::gecko_bindings::structs::ServoStyleSheet;
use style::gecko_bindings::structs::nsCSSValueSharedList;
use style::gecko_bindings::structs::nsTimingFunction;
use style::gecko_bindings::structs::nsresult;
use style::gecko_bindings::sugar::ownership::{FFIArcHelpers, HasArcFFI, HasBoxFFI};
use style::gecko_bindings::sugar::ownership::{HasSimpleFFI, Strong};
use style::gecko_bindings::sugar::refptr::{GeckoArcPrincipal, GeckoArcURI};
use style::gecko_properties::{self, style_structs};
use style::keyframes::KeyframesStepValue;
use style::media_queries::{MediaList, parse_media_query_list};
use style::parallel;
use style::parser::{ParserContext, ParserContextExtraData};
use style::properties::{ComputedValues, Importance, ParsedDeclaration};
use style::properties::{PropertyDeclarationBlock, PropertyId};
use style::properties::animated_properties::{AnimationValue, Interpolate, TransitionProperty};
use style::properties::parse_one_declaration;
use style::restyle_hints::{self, RestyleHint};
use style::selector_parser::PseudoElementCascadeType;
use style::sequential;
use style::string_cache::Atom;
use style::stylesheets::{CssRule, CssRules, ImportRule, MediaRule, Origin, Stylesheet, StyleRule};
use style::stylesheets::StylesheetLoader as StyleStylesheetLoader;
use style::supports::parse_condition_or_declaration;
use style::thread_state;
use style::timer::Timer;
use style::traversal::{resolve_style, DomTraversal, TraversalDriver};
use style_traits::ToCss;
use stylesheet_loader::StylesheetLoader;

/*
 * For Gecko->Servo function calls, we need to redeclare the same signature that was declared in
 * the C header in Gecko. In order to catch accidental mismatches, we run rust-bindgen against
 * those signatures as well, giving us a second declaration of all the Servo_* functions in this
 * crate. If there's a mismatch, LLVM will assert and abort, which is a rather awful thing to
 * depend on but good enough for our purposes.
 */

struct GlobalStyleData {
    // How many threads parallel styling can use.
    pub num_threads: usize,

    // The parallel styling thread pool.
    pub style_thread_pool: Option<rayon::ThreadPool>,
}

impl GlobalStyleData {
    pub fn new() -> Self {
        let stylo_threads = env::var("STYLO_THREADS")
            .map(|s| s.parse::<usize>().expect("invalid STYLO_THREADS value"));
        let num_threads = match stylo_threads {
            Ok(num) => num,
            _ => cmp::max(num_cpus::get() * 3 / 4, 1),
        };

        let pool = if num_threads <= 1 {
            None
        } else {
            let configuration =
                rayon::Configuration::new().set_num_threads(num_threads);
            let pool = rayon::ThreadPool::new(configuration).ok();
            pool
        };

        GlobalStyleData {
            num_threads: num_threads,
            style_thread_pool: pool,
        }
    }
}

lazy_static! {
    static ref GLOBAL_STYLE_DATA: GlobalStyleData = {
        GlobalStyleData::new()
    };
}

#[no_mangle]
pub extern "C" fn Servo_Initialize() -> () {
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
}

#[no_mangle]
pub extern "C" fn Servo_Shutdown() -> () {
    // Clear some static data to avoid shutdown leaks.
    gecko_properties::shutdown();
}

fn create_shared_context(per_doc_data: &PerDocumentStyleDataImpl) -> SharedStyleContext {
    let local_context_data =
        ThreadLocalStyleContextCreationInfo::new(per_doc_data.new_animations_sender.clone());

    SharedStyleContext {
        stylist: per_doc_data.stylist.clone(),
        running_animations: per_doc_data.running_animations.clone(),
        expired_animations: per_doc_data.expired_animations.clone(),
        error_reporter: Box::new(StdoutErrorReporter),
        local_context_creation_data: Mutex::new(local_context_data),
        timer: Timer::new(),
        // FIXME Find the real QuirksMode information for this document
        quirks_mode: QuirksMode::NoQuirks,
        default_computed_values: per_doc_data.default_computed_values().clone(),
    }
}

fn traverse_subtree(element: GeckoElement, raw_data: RawServoStyleSetBorrowed,
                    unstyled_children_only: bool) {
    // When new content is inserted in a display:none subtree, we will call into
    // servo to try to style it. Detect that here and bail out.
    if let Some(parent) = element.parent_element() {
        if parent.borrow_data().map_or(true, |d| d.styles().is_display_none()) {
            debug!("{:?} has unstyled parent - ignoring call to traverse_subtree", parent);
            return;
        }
    }

    let per_doc_data = PerDocumentStyleData::from_ffi(raw_data).borrow();

    let token = RecalcStyleOnly::pre_traverse(element, &per_doc_data.stylist, unstyled_children_only);
    if !token.should_traverse() {
        return;
    }

    debug!("Traversing subtree:");
    debug!("{:?}", ShowSubtreeData(element.as_node()));

    let shared_style_context = create_shared_context(&per_doc_data);
    let ref global_style_data = *GLOBAL_STYLE_DATA;

    let traversal_driver = if global_style_data.style_thread_pool.is_none() {
        TraversalDriver::Sequential
    } else {
        TraversalDriver::Parallel
    };

    let traversal = RecalcStyleOnly::new(shared_style_context, traversal_driver);
    let known_depth = None;
    if traversal_driver.is_parallel() {
        parallel::traverse_dom(&traversal, element, known_depth, token,
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
                                        behavior: structs::TraversalRootBehavior) -> bool {
    let element = GeckoElement(root);
    debug!("Servo_TraverseSubtree: {:?}", element);
    traverse_subtree(element, raw_data,
                     behavior == structs::TraversalRootBehavior::UnstyledChildrenOnly);

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
pub extern "C" fn Servo_AnimationValues_Uncompute(value: RawServoAnimationValueBorrowedListBorrowed)
     -> RawServoDeclarationBlockStrong
{
    let value = unsafe { value.as_ref().unwrap() };
    let mut block = PropertyDeclarationBlock::new();
    for v in value.iter() {
        let raw_anim = unsafe { v.as_ref().unwrap() };
        let anim = AnimationValue::as_arc(&raw_anim);
        block.push(anim.uncompute(), Importance::Normal);
    }
    Arc::new(RwLock::new(block)).into_strong()
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
pub extern "C" fn Servo_StyleWorkerThreadCount() -> u32 {
    GLOBAL_STYLE_DATA.num_threads as u32
}

#[no_mangle]
pub extern "C" fn Servo_Element_ClearData(element: RawGeckoElementBorrowed) -> () {
    GeckoElement(element).clear_data();
}

#[no_mangle]
pub extern "C" fn Servo_StyleSheet_Empty(mode: SheetParsingMode) -> RawServoStyleSheetStrong {
    let url = ServoUrl::parse("about:blank").unwrap();
    let extra_data = ParserContextExtraData::default();
    let origin = match mode {
        SheetParsingMode::eAuthorSheetFeatures => Origin::Author,
        SheetParsingMode::eUserSheetFeatures => Origin::User,
        SheetParsingMode::eAgentSheetFeatures => Origin::UserAgent,
    };
    Arc::new(Stylesheet::from_str(
        "", url, origin, Default::default(), None,
        Box::new(StdoutErrorReporter), extra_data)
    ).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_StyleSheet_FromUTF8Bytes(loader: *mut Loader,
                                                 stylesheet: *mut ServoStyleSheet,
                                                 data: *const nsACString,
                                                 mode: SheetParsingMode,
                                                 base_url: *const nsACString,
                                                 base: *mut ThreadSafeURIHolder,
                                                 referrer: *mut ThreadSafeURIHolder,
                                                 principal: *mut ThreadSafePrincipalHolder)
                                                 -> RawServoStyleSheetStrong {
    let input = unsafe { data.as_ref().unwrap().as_str_unchecked() };

    let origin = match mode {
        SheetParsingMode::eAuthorSheetFeatures => Origin::Author,
        SheetParsingMode::eUserSheetFeatures => Origin::User,
        SheetParsingMode::eAgentSheetFeatures => Origin::UserAgent,
    };

    let base_str = unsafe { base_url.as_ref().unwrap().as_str_unchecked() };
    let url = ServoUrl::parse(base_str).unwrap();
    let extra_data = unsafe { ParserContextExtraData {
        base: Some(GeckoArcURI::new(base)),
        referrer: Some(GeckoArcURI::new(referrer)),
        principal: Some(GeckoArcPrincipal::new(principal)),
    }};
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

    Arc::new(Stylesheet::from_str(
        input, url, origin, Default::default(), loader,
        Box::new(StdoutErrorReporter), extra_data)
    ).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_StyleSheet_ClearAndUpdate(stylesheet: RawServoStyleSheetBorrowed,
                                                  loader: *mut Loader,
                                                  gecko_stylesheet: *mut ServoStyleSheet,
                                                  data: *const nsACString,
                                                  base: *mut ThreadSafeURIHolder,
                                                  referrer: *mut ThreadSafeURIHolder,
                                                  principal: *mut ThreadSafePrincipalHolder)
{
    let input = unsafe { data.as_ref().unwrap().as_str_unchecked() };
    let extra_data = unsafe { ParserContextExtraData {
        base: Some(GeckoArcURI::new(base)),
        referrer: Some(GeckoArcURI::new(referrer)),
        principal: Some(GeckoArcPrincipal::new(principal)),
    }};

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
    sheet.rules.write().0.clear();

    Stylesheet::update_from_str(&sheet, input, loader,
                                Box::new(StdoutErrorReporter), extra_data);
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_AppendStyleSheet(raw_data: RawServoStyleSetBorrowed,
                                                  raw_sheet: RawServoStyleSheetBorrowed,
                                                  flush: bool) {
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    let sheet = HasArcFFI::as_arc(&raw_sheet);
    data.stylesheets.retain(|x| !arc_ptr_eq(x, sheet));
    data.stylesheets.push(sheet.clone());
    data.stylesheets_changed = true;
    if flush {
        data.flush_stylesheets();
    }
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_PrependStyleSheet(raw_data: RawServoStyleSetBorrowed,
                                                   raw_sheet: RawServoStyleSheetBorrowed,
                                                   flush: bool) {
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    let sheet = HasArcFFI::as_arc(&raw_sheet);
    data.stylesheets.retain(|x| !arc_ptr_eq(x, sheet));
    data.stylesheets.insert(0, sheet.clone());
    data.stylesheets_changed = true;
    if flush {
        data.flush_stylesheets();
    }
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_InsertStyleSheetBefore(raw_data: RawServoStyleSetBorrowed,
                                                        raw_sheet: RawServoStyleSheetBorrowed,
                                                        raw_reference: RawServoStyleSheetBorrowed,
                                                        flush: bool) {
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    let sheet = HasArcFFI::as_arc(&raw_sheet);
    let reference = HasArcFFI::as_arc(&raw_reference);
    data.stylesheets.retain(|x| !arc_ptr_eq(x, sheet));
    let index = data.stylesheets.iter().position(|x| arc_ptr_eq(x, reference)).unwrap();
    data.stylesheets.insert(index, sheet.clone());
    data.stylesheets_changed = true;
    if flush {
        data.flush_stylesheets();
    }
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_RemoveStyleSheet(raw_data: RawServoStyleSetBorrowed,
                                                  raw_sheet: RawServoStyleSheetBorrowed,
                                                  flush: bool) {
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    let sheet = HasArcFFI::as_arc(&raw_sheet);
    data.stylesheets.retain(|x| !arc_ptr_eq(x, sheet));
    data.stylesheets_changed = true;
    if flush {
        data.flush_stylesheets();
    }
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_FlushStyleSheets(raw_data: RawServoStyleSetBorrowed) {
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    data.flush_stylesheets();
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_NoteStyleSheetsChanged(raw_data: RawServoStyleSetBorrowed) {
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    data.stylesheets_changed = true;
}

#[no_mangle]
pub extern "C" fn Servo_StyleSheet_HasRules(raw_sheet: RawServoStyleSheetBorrowed) -> bool {
    !Stylesheet::as_arc(&raw_sheet).rules.read().0.is_empty()
}

#[no_mangle]
pub extern "C" fn Servo_StyleSheet_GetRules(sheet: RawServoStyleSheetBorrowed) -> ServoCssRulesStrong {
    Stylesheet::as_arc(&sheet).rules.clone().into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_CssRules_ListTypes(rules: ServoCssRulesBorrowed,
                                           result: nsTArrayBorrowed_uintptr_t) -> () {
    let rules = RwLock::<CssRules>::as_arc(&rules).read();
    let iter = rules.0.iter().map(|rule| rule.rule_type() as usize);
    let (size, upper) = iter.size_hint();
    debug_assert_eq!(size, upper.unwrap());
    unsafe { result.set_len(size as u32) };
    result.iter_mut().zip(iter).fold((), |_, (r, v)| *r = v);
}

#[no_mangle]
pub extern "C" fn Servo_CssRules_GetStyleRuleAt(rules: ServoCssRulesBorrowed, index: u32)
                                                -> RawServoStyleRuleStrong {
    let rules = RwLock::<CssRules>::as_arc(&rules).read();
    match rules.0[index as usize] {
        CssRule::Style(ref rule) => rule.clone().into_strong(),
        _ => {
            unreachable!("GetStyleRuleAt should only be called on a style rule");
        }
    }
}

#[no_mangle]
pub extern "C" fn Servo_CssRules_GetMediaRuleAt(rules: ServoCssRulesBorrowed, index: u32)
                                                -> RawServoMediaRuleStrong {
    let rules = RwLock::<CssRules>::as_arc(&rules).read();
    match rules.0[index as usize] {
        CssRule::Media(ref rule) => rule.clone().into_strong(),
        _ => {
            unreachable!("GetMediaRuleAt should only be called on a media rule");
        }
    }
}

#[no_mangle]
pub extern "C" fn Servo_CssRules_InsertRule(rules: ServoCssRulesBorrowed, sheet: RawServoStyleSheetBorrowed,
                                            rule: *const nsACString, index: u32, nested: bool,
                                            rule_type: *mut u16) -> nsresult {
    let rules = RwLock::<CssRules>::as_arc(&rules);
    let sheet = Stylesheet::as_arc(&sheet);
    let rule = unsafe { rule.as_ref().unwrap().as_str_unchecked() };
    match rules.write().insert_rule(rule, sheet, index as usize, nested) {
        Ok(new_rule) => {
            *unsafe { rule_type.as_mut().unwrap() } = new_rule.rule_type() as u16;
            nsresult::NS_OK
        }
        Err(err) => err.into()
    }
}

#[no_mangle]
pub extern "C" fn Servo_CssRules_DeleteRule(rules: ServoCssRulesBorrowed, index: u32) -> nsresult {
    let rules = RwLock::<CssRules>::as_arc(&rules);
    match rules.write().remove_rule(index as usize) {
        Ok(_) => nsresult::NS_OK,
        Err(err) => err.into()
    }
}

#[no_mangle]
pub extern "C" fn Servo_StyleRule_Debug(rule: RawServoStyleRuleBorrowed, result: *mut nsACString) -> () {
    let rule = RwLock::<StyleRule>::as_arc(&rule);
    let result = unsafe { result.as_mut().unwrap() };
    write!(result, "{:?}", *rule.read()).unwrap();
}

#[no_mangle]
pub extern "C" fn Servo_StyleRule_GetStyle(rule: RawServoStyleRuleBorrowed) -> RawServoDeclarationBlockStrong {
    let rule = RwLock::<StyleRule>::as_arc(&rule);
    rule.read().block.clone().into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_StyleRule_SetStyle(rule: RawServoStyleRuleBorrowed,
                                           declarations: RawServoDeclarationBlockBorrowed) -> () {
    let rule = RwLock::<StyleRule>::as_arc(&rule);
    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    rule.write().block = declarations.clone();
}

#[no_mangle]
pub extern "C" fn Servo_StyleRule_GetCssText(rule: RawServoStyleRuleBorrowed, result: *mut nsAString) -> () {
    let rule = RwLock::<StyleRule>::as_arc(&rule);
    rule.read().to_css(unsafe { result.as_mut().unwrap() }).unwrap();
}

#[no_mangle]
pub extern "C" fn Servo_StyleRule_GetSelectorText(rule: RawServoStyleRuleBorrowed, result: *mut nsAString) -> () {
    let rule = RwLock::<StyleRule>::as_arc(&rule);
    rule.read().selectors.to_css(unsafe { result.as_mut().unwrap() }).unwrap();
}

#[no_mangle]
pub extern "C" fn Servo_MediaRule_Debug(rule: RawServoMediaRuleBorrowed, result: *mut nsACString) -> () {
    let rule = RwLock::<MediaRule>::as_arc(&rule);
    let result = unsafe { result.as_mut().unwrap() };
    write!(result, "{:?}", *rule.read()).unwrap();
}

#[no_mangle]
pub extern "C" fn Servo_MediaRule_GetMedia(rule: RawServoMediaRuleBorrowed) -> RawServoMediaListStrong {
    let rule = RwLock::<MediaRule>::as_arc(&rule);
    rule.read().media_queries.clone().into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_MediaRule_GetRules(rule: RawServoMediaRuleBorrowed) -> ServoCssRulesStrong {
    let rule = RwLock::<MediaRule>::as_arc(&rule);
    rule.read().rules.clone().into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_MediaRule_GetCssText(rule: RawServoMediaRuleBorrowed, result: *mut nsAString) -> () {
    let rule = RwLock::<MediaRule>::as_arc(&rule);
    rule.read().to_css(unsafe { result.as_mut().unwrap() }).unwrap();
}

#[no_mangle]
pub extern "C" fn Servo_ComputedValues_GetForAnonymousBox(parent_style_or_null: ServoComputedValuesBorrowedOrNull,
                                                         pseudo_tag: *mut nsIAtom,
                                                         raw_data: RawServoStyleSetBorrowed)
     -> ServoComputedValuesStrong {
    let data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    let atom = Atom::from(pseudo_tag);
    let pseudo = PseudoElement::from_atom_unchecked(atom, /* anon_box = */ true);


    let maybe_parent = ComputedValues::arc_from_borrowed(&parent_style_or_null);
    data.stylist.precomputed_values_for_pseudo(&pseudo, maybe_parent,
                                               data.default_computed_values(), false)
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

    match get_pseudo_style(element, pseudo_tag, data.styles(), doc_data) {
        Some(values) => values.into_strong(),
        None if !is_probe => data.styles().primary.values().clone().into_strong(),
        None => Strong::null(),
    }
}

fn get_pseudo_style(element: GeckoElement, pseudo_tag: *mut nsIAtom,
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
            d.stylist.lazily_compute_pseudo_element_style(&element,
                                                          &pseudo,
                                                          base,
                                                          &d.default_computed_values())
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
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    data.reset_device();
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_Drop(data: RawServoStyleSetOwned) -> () {
    let _ = data.into_box::<PerDocumentStyleData>();
}

// Must be a macro since we need to store the base_url on the stack somewhere
/// Initializes the data needed for constructing a ParserContext from
/// Gecko-side values
macro_rules! make_context {
    (($base:ident, $data:ident) => ($base_url:ident, $extra_data:ident)) => {
        let base_str = unsafe { $base.as_ref().unwrap().as_str_unchecked() };
        let $base_url = ServoUrl::parse(base_str).unwrap();
        let $extra_data = unsafe { ParserContextExtraData::new($data) };
    }
}

#[no_mangle]
pub extern "C" fn Servo_ParseProperty(property: *const nsACString, value: *const nsACString,
                                      base: *const nsACString,
                                      data: *const structs::GeckoParserExtraData)
                                      -> RawServoDeclarationBlockStrong {
    let name = unsafe { property.as_ref().unwrap().as_str_unchecked() };
    let id = if let Ok(id) = PropertyId::parse(name.into()) {
        id
    } else {
        return RawServoDeclarationBlockStrong::null()
    };
    let value = unsafe { value.as_ref().unwrap().as_str_unchecked() };

    make_context!((base, data) => (base_url, extra_data));

    let context = ParserContext::new_with_extra_data(Origin::Author,
                                                     &base_url,
                                                     Box::new(StdoutErrorReporter),
                                                     extra_data);

    match ParsedDeclaration::parse(id, &context, &mut Parser::new(value), false) {
        Ok(parsed) => {
            let mut block = PropertyDeclarationBlock::new();
            parsed.expand(|d| block.push(d, Importance::Normal));
            Arc::new(RwLock::new(block)).into_strong()
        }
        Err(_) => RawServoDeclarationBlockStrong::null()
    }
}

#[no_mangle]
pub extern "C" fn Servo_ParseStyleAttribute(data: *const nsACString,
                                            base: *const nsACString,
                                            raw_extra_data: *const structs::GeckoParserExtraData)
                                            -> RawServoDeclarationBlockStrong {
    let value = unsafe { data.as_ref().unwrap().as_str_unchecked() };
    make_context!((base, raw_extra_data) => (base_url, extra_data));
    Arc::new(RwLock::new(GeckoElement::parse_style_attribute(value, &base_url, extra_data))).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_CreateEmpty() -> RawServoDeclarationBlockStrong {
    Arc::new(RwLock::new(PropertyDeclarationBlock::new())).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_Clone(declarations: RawServoDeclarationBlockBorrowed)
                                               -> RawServoDeclarationBlockStrong {
    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    Arc::new(RwLock::new(declarations.read().clone())).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_Equals(a: RawServoDeclarationBlockBorrowed,
                                                b: RawServoDeclarationBlockBorrowed)
                                                -> bool {
    *RwLock::<PropertyDeclarationBlock>::as_arc(&a).read().declarations() ==
    *RwLock::<PropertyDeclarationBlock>::as_arc(&b).read().declarations()
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_GetCssText(declarations: RawServoDeclarationBlockBorrowed,
                                                    result: *mut nsAString) {
    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    declarations.read().to_css(unsafe { result.as_mut().unwrap() }).unwrap();
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SerializeOneValue(
    declarations: RawServoDeclarationBlockBorrowed,
    property_id: nsCSSPropertyID, buffer: *mut nsAString)
{
    let property_id = get_property_id_from_nscsspropertyid!(property_id, ());
    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    let mut string = String::new();
    let rv = declarations.read().single_value_to_css(&property_id, &mut string);
    debug_assert!(rv.is_ok());

    write!(unsafe { &mut *buffer }, "{}", string).expect("Failed to copy string");
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_Count(declarations: RawServoDeclarationBlockBorrowed) -> u32 {
     let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
     declarations.read().declarations().len() as u32
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_GetNthProperty(declarations: RawServoDeclarationBlockBorrowed,
                                                        index: u32, result: *mut nsAString) -> bool {
    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    if let Some(&(ref decl, _)) = declarations.read().declarations().get(index as usize) {
        let result = unsafe { result.as_mut().unwrap() };
        decl.id().to_css(result).unwrap();
        true
    } else {
        false
    }
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
    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    declarations.read().property_value_to_css(&property_id, unsafe { value.as_mut().unwrap() }).unwrap();
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
    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    declarations.read().property_priority(&property_id).important()
}

fn set_property(declarations: RawServoDeclarationBlockBorrowed, property_id: PropertyId,
                value: *const nsACString, is_important: bool,
                base: *const nsACString, data: *const structs::GeckoParserExtraData) -> bool {
    let value = unsafe { value.as_ref().unwrap().as_str_unchecked() };

    make_context!((base, data) => (base_url, extra_data));
    if let Ok(parsed) = parse_one_declaration(property_id, value, &base_url,
                                              Box::new(StdoutErrorReporter), extra_data) {
        let mut declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations).write();
        let importance = if is_important { Importance::Important } else { Importance::Normal };
        let mut changed = false;
        parsed.expand(|decl| {
            changed |= declarations.set_parsed_declaration(decl, importance);
        });
        changed
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetProperty(declarations: RawServoDeclarationBlockBorrowed,
                                                     property: *const nsACString, value: *const nsACString,
                                                     is_important: bool,
                                                     base: *const nsACString,
                                                     data: *const structs::GeckoParserExtraData) -> bool {
    set_property(declarations, get_property_id_from_property!(property, false),
                 value, is_important, base, data)
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetPropertyById(declarations: RawServoDeclarationBlockBorrowed,
                                                         property: nsCSSPropertyID, value: *const nsACString,
                                                         is_important: bool,
                                                         base: *const nsACString,
                                                         data: *const structs::GeckoParserExtraData) -> bool {
    set_property(declarations, get_property_id_from_nscsspropertyid!(property, false),
                 value, is_important, base, data)
}

fn remove_property(declarations: RawServoDeclarationBlockBorrowed, property_id: PropertyId) {
    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    declarations.write().remove_property(&property_id);
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
pub extern "C" fn Servo_MediaList_GetText(list: RawServoMediaListBorrowed, result: *mut nsAString) {
    let list = RwLock::<MediaList>::as_arc(&list);
    list.read().to_css(unsafe { result.as_mut().unwrap() }).unwrap();
}

#[no_mangle]
pub extern "C" fn Servo_MediaList_SetText(list: RawServoMediaListBorrowed, text: *const nsACString) {
    let list = RwLock::<MediaList>::as_arc(&list);
    let text = unsafe { text.as_ref().unwrap().as_str_unchecked() };
    let mut parser = Parser::new(&text);
    *list.write() = parse_media_query_list(&mut parser);
}

#[no_mangle]
pub extern "C" fn Servo_MediaList_GetLength(list: RawServoMediaListBorrowed) -> u32 {
    let list = RwLock::<MediaList>::as_arc(&list);
    list.read().media_queries.len() as u32
}

#[no_mangle]
pub extern "C" fn Servo_MediaList_GetMediumAt(list: RawServoMediaListBorrowed, index: u32,
                                              result: *mut nsAString) -> bool {
    let list = RwLock::<MediaList>::as_arc(&list);
    if let Some(media_query) = list.read().media_queries.get(index as usize) {
        media_query.to_css(unsafe { result.as_mut().unwrap() }).unwrap();
        true
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn Servo_MediaList_AppendMedium(list: RawServoMediaListBorrowed,
                                               new_medium: *const nsACString) {
    let list = RwLock::<MediaList>::as_arc(&list);
    let new_medium = unsafe { new_medium.as_ref().unwrap().as_str_unchecked() };
    list.write().append_medium(new_medium);
}

#[no_mangle]
pub extern "C" fn Servo_MediaList_DeleteMedium(list: RawServoMediaListBorrowed,
                                               old_medium: *const nsACString) -> bool {
    let list = RwLock::<MediaList>::as_arc(&list);
    let old_medium = unsafe { old_medium.as_ref().unwrap().as_str_unchecked() };
    list.write().delete_medium(old_medium)
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
                LonghandId::$property => PropertyDeclaration::$property(DeclaredValue::Value($inner)),
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
    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    let long = get_longhand_from_id!(property, false);
    declarations.read().get(PropertyDeclarationId::Longhand(long)).is_some()
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetIdentStringValue(declarations:
                                                             RawServoDeclarationBlockBorrowed,
                                                             property:
                                                             nsCSSPropertyID,
                                                             value:
                                                             *mut nsIAtom) {
    use style::properties::{DeclaredValue, PropertyDeclaration, LonghandId};
    use style::properties::longhands::_x_lang::computed_value::T as Lang;

    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    let long = get_longhand_from_id!(property);
    let prop = match_wrap_declared! { long,
        XLang => Lang(Atom::from(value)),
    };
    declarations.write().push(prop, Importance::Normal);
}

#[no_mangle]
#[allow(unreachable_code)]
pub extern "C" fn Servo_DeclarationBlock_SetKeywordValue(declarations:
                                                         RawServoDeclarationBlockBorrowed,
                                                         property: nsCSSPropertyID,
                                                         value: i32) {
    use style::properties::{DeclaredValue, PropertyDeclaration, LonghandId};
    use style::properties::longhands;
    use style::values::specified::{BorderStyle, NoCalcLength};

    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    let long = get_longhand_from_id!(property);
    let value = value as u32;

    let prop = match_wrap_declared! { long,
        MozUserModify => longhands::_moz_user_modify::SpecifiedValue::from_gecko_keyword(value),
        // TextEmphasisPosition => FIXME implement text-emphasis-position
        Display => longhands::display::SpecifiedValue::from_gecko_keyword(value),
        Float => longhands::float::SpecifiedValue::from_gecko_keyword(value),
        VerticalAlign => longhands::vertical_align::SpecifiedValue::from_gecko_keyword(value),
        TextAlign => longhands::text_align::SpecifiedValue::from_gecko_keyword(value),
        Clear => longhands::clear::SpecifiedValue::from_gecko_keyword(value),
        FontSize => {
            // We rely on Gecko passing in font-size values (0...7) here.
            longhands::font_size::SpecifiedValue(NoCalcLength::from_font_size_int(value as u8).into())
        },
        ListStyleType => longhands::list_style_type::SpecifiedValue::from_gecko_keyword(value),
        WhiteSpace => longhands::white_space::SpecifiedValue::from_gecko_keyword(value),
        CaptionSide => longhands::caption_side::SpecifiedValue::from_gecko_keyword(value),
        BorderTopStyle => BorderStyle::from_gecko_keyword(value),
        BorderRightStyle => BorderStyle::from_gecko_keyword(value),
        BorderBottomStyle => BorderStyle::from_gecko_keyword(value),
        BorderLeftStyle => BorderStyle::from_gecko_keyword(value),
    };
    declarations.write().push(prop, Importance::Normal);
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetIntValue(declarations: RawServoDeclarationBlockBorrowed,
                                                     property: nsCSSPropertyID,
                                                     value: i32) {
    use style::properties::{DeclaredValue, PropertyDeclaration, LonghandId};
    use style::properties::longhands::_x_span::computed_value::T as Span;

    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    let long = get_longhand_from_id!(property);
    let prop = match_wrap_declared! { long,
        XSpan => Span(value),
    };
    declarations.write().push(prop, Importance::Normal);
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetPixelValue(declarations:
                                                       RawServoDeclarationBlockBorrowed,
                                                       property: nsCSSPropertyID,
                                                       value: f32) {
    use style::properties::{DeclaredValue, PropertyDeclaration, LonghandId};
    use style::properties::longhands::border_spacing::SpecifiedValue as BorderSpacing;
    use style::values::specified::BorderWidth;
    use style::values::specified::length::NoCalcLength;

    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
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
                vertical: nocalc.into(),
            }
        ),
    };
    declarations.write().push(prop, Importance::Normal);
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetPercentValue(declarations:
                                                         RawServoDeclarationBlockBorrowed,
                                                         property: nsCSSPropertyID,
                                                         value: f32) {
    use style::properties::{DeclaredValue, PropertyDeclaration, LonghandId};
    use style::values::specified::length::Percentage;

    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    let long = get_longhand_from_id!(property);
    let pc = Percentage(value);

    let prop = match_wrap_declared! { long,
        Height => pc.into(),
        Width => pc.into(),
        MarginTop => pc.into(),
        MarginRight => pc.into(),
        MarginBottom => pc.into(),
        MarginLeft => pc.into(),
    };
    declarations.write().push(prop, Importance::Normal);
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetAutoValue(declarations:
                                                      RawServoDeclarationBlockBorrowed,
                                                      property: nsCSSPropertyID) {
    use style::properties::{DeclaredValue, PropertyDeclaration, LonghandId};
    use style::values::specified::LengthOrPercentageOrAuto;

    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
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
    declarations.write().push(prop, Importance::Normal);
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetCurrentColor(declarations:
                                                         RawServoDeclarationBlockBorrowed,
                                                         property: nsCSSPropertyID) {
    use cssparser::Color;
    use style::properties::{DeclaredValue, PropertyDeclaration, LonghandId};
    use style::values::specified::CSSColor;

    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    let long = get_longhand_from_id!(property);
    let cc = CSSColor { parsed: Color::CurrentColor, authored: None };

    let prop = match_wrap_declared! { long,
        BorderTopColor => cc,
        BorderRightColor => cc,
        BorderBottomColor => cc,
        BorderLeftColor => cc,
    };
    declarations.write().push(prop, Importance::Normal);
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetColorValue(declarations:
                                                       RawServoDeclarationBlockBorrowed,
                                                       property: nsCSSPropertyID,
                                                       value: structs::nscolor) {
    use cssparser::Color;
    use style::gecko::values::convert_nscolor_to_rgba;
    use style::properties::{DeclaredValue, PropertyDeclaration, LonghandId};
    use style::properties::longhands;
    use style::values::specified::CSSColor;

    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
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
    declarations.write().push(prop, Importance::Normal);
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetFontFamily(declarations:
                                                       RawServoDeclarationBlockBorrowed,
                                                       value: *const nsAString) {
    use cssparser::Parser;
    use style::properties::{DeclaredValue, PropertyDeclaration};
    use style::properties::longhands::font_family::SpecifiedValue as FontFamily;

    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    let string = unsafe { (*value).to_string() };
    let mut parser = Parser::new(&string);
    if let Ok(family) = FontFamily::parse(&mut parser) {
        if parser.is_exhausted() {
            let decl = PropertyDeclaration::FontFamily(DeclaredValue::Value(family));
            declarations.write().push(decl, Importance::Normal);
        }
    }
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetTextDecorationColorOverride(declarations:
                                                                RawServoDeclarationBlockBorrowed) {
    use style::properties::{DeclaredValue, PropertyDeclaration};
    use style::properties::longhands::text_decoration_line;

    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    let mut decoration = text_decoration_line::computed_value::none;
    decoration |= text_decoration_line::COLOR_OVERRIDE;
    let decl = PropertyDeclaration::TextDecorationLine(DeclaredValue::Value(decoration));
    declarations.write().push(decl, Importance::Normal);
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

    let base_url = &*DUMMY_BASE_URL;
    let extra_data = ParserContextExtraData::default();

    parse_one_declaration(id, &value, &base_url, Box::new(StdoutErrorReporter), extra_data).is_ok()
}

#[no_mangle]
pub extern "C" fn Servo_CSSSupports(cond: *const nsACString) -> bool {
    let condition = unsafe { cond.as_ref().unwrap().as_str_unchecked() };
    let mut input = Parser::new(&condition);
    let cond = parse_condition_or_declaration(&mut input);
    if let Ok(cond) = cond {
        let url = ServoUrl::parse("about:blank").unwrap();
        let context = ParserContext::new_for_cssom(&url);
        cond.eval(&context)
    } else {
        false
    }
}

/// Only safe to call on the main thread, with exclusive access to the element and
/// its ancestors.
unsafe fn maybe_restyle<'a>(data: &'a mut AtomicRefMut<ElementData>, element: GeckoElement)
    -> Option<&'a mut RestyleData>
{
    // Don't generate a useless RestyleData if the element hasn't been styled.
    if !data.has_styles() {
        return None;
    }

    // Propagate the bit up the chain.
    let mut curr = element;
    while let Some(parent) = curr.parent_element() {
        curr = parent;
        if curr.has_dirty_descendants() { break; }
        curr.set_dirty_descendants();
    }
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
            if let Some(restyle_data) = unsafe { maybe_restyle(&mut data, element) } {
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
pub extern "C" fn Servo_NoteExplicitHints(element: RawGeckoElementBorrowed,
                                          restyle_hint: nsRestyleHint,
                                          change_hint: nsChangeHint) {
    let element = GeckoElement(element);
    let damage = GeckoRestyleDamage::new(change_hint);
    debug!("Servo_NoteExplicitHints: {:?}, restyle_hint={:?}, change_hint={:?}",
           element, restyle_hint, change_hint);

    let mut maybe_data = element.mutate_data();
    let maybe_restyle_data =
        maybe_data.as_mut().and_then(|d| unsafe { maybe_restyle(d, element) });
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
    let import_rule = RwLock::<ImportRule>::as_arc(&import_rule);
    import_rule.read().stylesheet.clone().into_strong()
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
                                     raw_data: RawServoStyleSetBorrowed)
                                     -> ServoComputedValuesStrong
{
    let element = GeckoElement(element);
    debug!("Servo_ResolveStyle: {:?}", element);
    let data = unsafe { element.ensure_data() }.borrow_mut();

    if !data.has_current_styles() {
        warn!("Resolving style on unstyled element with lazy computation forbidden.");
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
    let element = GeckoElement(element);
    let doc_data = PerDocumentStyleData::from_ffi(raw_data);
    let finish = |styles: &ElementStyles| -> Arc<ComputedValues> {
        let maybe_pseudo = if !pseudo_tag.is_null() {
            get_pseudo_style(element, pseudo_tag, styles, doc_data)
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
    let shared = create_shared_context(&mut doc_data.borrow_mut());
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
    use style::properties::LonghandIdSet;
    use style::properties::declaration_block::Importance;
    use style::values::computed::Context;
    let data = PerDocumentStyleData::from_ffi(raw_data).borrow();

    let style = ComputedValues::as_arc(&style);
    let parent_style = parent_style.as_ref().map(|r| &**ComputedValues::as_arc(&r));

    let default_values = data.stylist.device.default_values();

    let context = Context {
        is_root_element: false,
        viewport_size: data.stylist.device.au_viewport_size(),
        inherited_style: parent_style.unwrap_or(default_values),
        layout_parent_style: parent_style.unwrap_or(default_values),
        style: (**style).clone(),
        font_metrics_provider: None,
    };

    for (index, keyframe) in keyframes.iter().enumerate() {
        let ref mut animation_values = computed_keyframes[index];

        let mut seen = LonghandIdSet::new();

        // mServoDeclarationBlock is null in the case where we have an invalid css property.
        let iter = keyframe.mPropertyValues.iter()
                                           .filter(|&property| !property.mServoDeclarationBlock.mRawPtr.is_null());
        for property in iter {
            let declarations = unsafe { &*property.mServoDeclarationBlock.mRawPtr.clone() };
            let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
            let guard = declarations.read();

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
        debug_assert!(!el.has_dirty_descendants());
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
                                                      timing_function: *const nsTimingFunction,
                                                      style: ServoComputedValuesBorrowed,
                                                      keyframes: RawGeckoKeyframeListBorrowedMut) -> bool {
    use style::gecko_bindings::structs::Keyframe;
    use style::properties::LonghandIdSet;

    let data = PerDocumentStyleData::from_ffi(raw_data).borrow();
    let name = unsafe { Atom::from(name.as_ref().unwrap().as_str_unchecked()) };
    let style_timing_function = unsafe { timing_function.as_ref().unwrap() };
    let style = ComputedValues::as_arc(&style);

    if let Some(ref animation) = data.stylist.animations().get(&name) {
       for step in &animation.steps {
          // Override timing_function if the keyframe has animation-timing-function.
          let timing_function = if let Some(val) = step.get_animation_timing_function() {
              val.into()
          } else {
              *style_timing_function
          };

          let keyframe = unsafe {
                Gecko_AnimationAppendKeyframe(keyframes,
                                              step.start_percentage.0 as f32,
                                              &timing_function)
          };

          fn add_computed_property_value(keyframe: *mut Keyframe,
                                         index: usize,
                                         style: &ComputedValues,
                                         property: &TransitionProperty) {
              let block = style.to_declaration_block(property.clone().into());
              unsafe {
                  (*keyframe).mPropertyValues.set_len((index + 1) as u32);
                  (*keyframe).mPropertyValues[index].mProperty = property.clone().into();
                  // FIXME. Do not set computed values once we handles missing keyframes
                  // with additive composition.
                  (*keyframe).mPropertyValues[index].mServoDeclarationBlock.set_arc_leaky(
                      Arc::new(RwLock::new(block)));
              }
          }

          match step.value {
              KeyframesStepValue::ComputedValues => {
                  for (index, property) in animation.properties_changed.iter().enumerate() {
                      add_computed_property_value(keyframe, index, style, property);
                  }
              },
              KeyframesStepValue::Declarations { ref block } => {
                  let guard = block.read();
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
                              Arc::new(RwLock::new(PropertyDeclarationBlock::with_one(
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
                              add_computed_property_value(keyframe, index, style, property);
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

