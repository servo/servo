/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use cssparser::Parser;
use cssparser::ToCss as ParserToCss;
use env_logger;
use euclid::Size2D;
use parking_lot::RwLock;
use selectors::Element;
use servo_url::ServoUrl;
use std::fmt::Write;
use std::mem::transmute;
use std::ptr;
use std::sync::{Arc, Mutex};
use style::arc_ptr_eq;
use style::atomic_refcell::AtomicRefMut;
use style::context::{LocalStyleContextCreationInfo, ReflowGoal, SharedStyleContext};
use style::data::{ElementData, RestyleData};
use style::dom::{ShowSubtreeData, TElement, TNode};
use style::error_reporting::StdoutErrorReporter;
use style::gecko::context::StandaloneStyleContext;
use style::gecko::context::clear_local_context;
use style::gecko::data::{NUM_THREADS, PerDocumentStyleData, PerDocumentStyleDataImpl};
use style::gecko::restyle_damage::GeckoRestyleDamage;
use style::gecko::selector_parser::{SelectorImpl, PseudoElement};
use style::gecko::traversal::RecalcStyleOnly;
use style::gecko::wrapper::DUMMY_BASE_URL;
use style::gecko::wrapper::GeckoElement;
use style::gecko_bindings::bindings::{RawServoDeclarationBlockBorrowed, RawServoDeclarationBlockStrong};
use style::gecko_bindings::bindings::{RawServoStyleRuleBorrowed, RawServoStyleRuleStrong};
use style::gecko_bindings::bindings::{RawServoStyleSetBorrowed, RawServoStyleSetOwned};
use style::gecko_bindings::bindings::{RawServoStyleSheetBorrowed, ServoComputedValuesBorrowed};
use style::gecko_bindings::bindings::{RawServoStyleSheetStrong, ServoComputedValuesStrong};
use style::gecko_bindings::bindings::{ServoCssRulesBorrowed, ServoCssRulesStrong};
use style::gecko_bindings::bindings::{nsACString, nsAString};
use style::gecko_bindings::bindings::RawGeckoElementBorrowed;
use style::gecko_bindings::bindings::ServoComputedValuesBorrowedOrNull;
use style::gecko_bindings::bindings::nsTArrayBorrowed_uintptr_t;
use style::gecko_bindings::structs;
use style::gecko_bindings::structs::{SheetParsingMode, nsIAtom};
use style::gecko_bindings::structs::{ThreadSafePrincipalHolder, ThreadSafeURIHolder};
use style::gecko_bindings::structs::{nsRestyleHint, nsChangeHint};
use style::gecko_bindings::structs::nsresult;
use style::gecko_bindings::sugar::ownership::{FFIArcHelpers, HasArcFFI, HasBoxFFI};
use style::gecko_bindings::sugar::ownership::{HasSimpleFFI, Strong};
use style::gecko_bindings::sugar::refptr::{GeckoArcPrincipal, GeckoArcURI};
use style::parallel;
use style::parser::{ParserContext, ParserContextExtraData};
use style::properties::{CascadeFlags, ComputedValues, Importance, PropertyDeclaration};
use style::properties::{PropertyDeclarationParseResult, PropertyDeclarationBlock, PropertyId};
use style::properties::{apply_declarations, parse_one_declaration};
use style::restyle_hints::RestyleHint;
use style::selector_parser::PseudoElementCascadeType;
use style::sequential;
use style::string_cache::Atom;
use style::stylesheets::{CssRule, CssRules, Origin, Stylesheet, StyleRule};
use style::thread_state;
use style::timer::Timer;
use style::traversal::{recalc_style_at, DomTraversalContext, PerLevelTraversalData};
use style_traits::ToCss;
use stylesheet_loader::StylesheetLoader;

/*
 * For Gecko->Servo function calls, we need to redeclare the same signature that was declared in
 * the C header in Gecko. In order to catch accidental mismatches, we run rust-bindgen against
 * those signatures as well, giving us a second declaration of all the Servo_* functions in this
 * crate. If there's a mismatch, LLVM will assert and abort, which is a rather awful thing to
 * depend on but good enough for our purposes.
 */

#[no_mangle]
pub extern "C" fn Servo_Initialize() -> () {
    // Enable standard Rust logging.
    //
    // See https://doc.rust-lang.org/log/env_logger/index.html for instructions.
    env_logger::init().unwrap();

    // Allocate our default computed values.
    unsafe { ComputedValues::initialize(); }

    // Pretend that we're a Servo Layout thread, to make some assertions happy.
    thread_state::initialize(thread_state::LAYOUT);
}

#[no_mangle]
pub extern "C" fn Servo_Shutdown() -> () {
    // Destroy our default computed values.
    unsafe { ComputedValues::shutdown(); }

    // In general, LocalStyleContexts will get destroyed when the worker thread
    // is joined and the TLS is dropped. However, under some configurations we
    // may do sequential style computation on the main thread, so we need to be
    // sure to clear the main thread TLS entry as well.
    clear_local_context();
}

fn create_shared_context(mut per_doc_data: &mut AtomicRefMut<PerDocumentStyleDataImpl>) -> SharedStyleContext {
    // The stylist consumes stylesheets lazily.
    per_doc_data.flush_stylesheets();

    let local_context_data =
        LocalStyleContextCreationInfo::new(per_doc_data.new_animations_sender.clone());

    SharedStyleContext {
        // FIXME (bug 1303229): Use the actual viewport size here
        viewport_size: Size2D::new(Au(0), Au(0)),
        screen_size_changed: false,
        generation: 0,
        goal: ReflowGoal::ForScriptQuery,
        stylist: per_doc_data.stylist.clone(),
        running_animations: per_doc_data.running_animations.clone(),
        expired_animations: per_doc_data.expired_animations.clone(),
        error_reporter: Box::new(StdoutErrorReporter),
        local_context_creation_data: Mutex::new(local_context_data),
        timer: Timer::new(),
    }
}

fn traverse_subtree(element: GeckoElement, raw_data: RawServoStyleSetBorrowed,
                    unstyled_children_only: bool) {
    // Force the creation of our lazily-constructed initial computed values on
    // the main thread, since it's not safe to call elsewhere.
    //
    // FIXME(bholley): this should move into Servo_Initialize as soon as we get
    // rid of the HackilyFindSomeDeviceContext stuff that happens during
    // initial_values computation, since that stuff needs to be called further
    // along in startup than the sensible place to call Servo_Initialize.
    ComputedValues::initial_values();

    // When new content is inserted in a display:none subtree, we will call into
    // servo to try to style it. Detect that here and bail out.
    if let Some(parent) = element.parent_element() {
        if parent.get_data().is_none() || parent.is_display_none() {
            debug!("{:?} has unstyled parent - ignoring call to traverse_subtree", parent);
            return;
        }
    }

    let mut per_doc_data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();

    let token = RecalcStyleOnly::pre_traverse(element, &per_doc_data.stylist, unstyled_children_only);
    if !token.should_traverse() {
        error!("Unnecessary call to traverse_subtree");
        return;
    }

    debug!("Traversing subtree:");
    debug!("{:?}", ShowSubtreeData(element.as_node()));

    let shared_style_context = create_shared_context(&mut per_doc_data);
    let known_depth = None;

    if per_doc_data.num_threads == 1 || per_doc_data.work_queue.is_none() {
        sequential::traverse_dom::<_, RecalcStyleOnly>(element, &shared_style_context, token);
    } else {
        parallel::traverse_dom::<_, RecalcStyleOnly>(element, known_depth,
                                                     &shared_style_context, token,
                                                     per_doc_data.work_queue.as_mut().unwrap());
    }
}

#[no_mangle]
pub extern "C" fn Servo_TraverseSubtree(root: RawGeckoElementBorrowed,
                                        raw_data: RawServoStyleSetBorrowed,
                                        behavior: structs::TraversalRootBehavior) -> () {
    let element = GeckoElement(root);
    debug!("Servo_TraverseSubtree: {:?}", element);
    traverse_subtree(element, raw_data,
                     behavior == structs::TraversalRootBehavior::UnstyledChildrenOnly);
}

#[no_mangle]
pub extern "C" fn Servo_RestyleWithAddedDeclaration(declarations: RawServoDeclarationBlockBorrowed,
                                                    previous_style: ServoComputedValuesBorrowed)
  -> ServoComputedValuesStrong
{
    let previous_style = ComputedValues::as_arc(&previous_style);
    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);

    let guard = declarations.read();

    let declarations = || {
        guard.declarations.iter().rev().map(|&(ref decl, _importance)| decl)
    };

    // FIXME (bug 1303229): Use the actual viewport size here
    let computed = apply_declarations(Size2D::new(Au(0), Au(0)),
                                      /* is_root_element = */ false,
                                      declarations,
                                      previous_style,
                                      None,
                                      Box::new(StdoutErrorReporter),
                                      None,
                                      CascadeFlags::empty());
    Arc::new(computed).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_StyleWorkerThreadCount() -> u32 {
    *NUM_THREADS as u32
}

#[no_mangle]
pub extern "C" fn Servo_Element_ClearData(element: RawGeckoElementBorrowed) -> () {
    GeckoElement(element).clear_data();
}

#[no_mangle]
pub extern "C" fn Servo_Element_ShouldTraverse(element: RawGeckoElementBorrowed) -> bool {
    let element = GeckoElement(element);
    if let Some(data) = element.get_data() {
        debug_assert!(!element.has_dirty_descendants(),
                      "only call Servo_Element_ShouldTraverse if you know the element \
                       does not have dirty descendants");
        match *data.borrow() {
            ElementData::Initial(None) |
            ElementData::Restyle(..) => true,
            _ => false,
        }
    } else {
        false
    }
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
    let loader = StylesheetLoader::new();
    let sheet = Arc::new(Stylesheet::from_str(
        "", url, origin, Default::default(), Some(&loader),
        Box::new(StdoutErrorReporter), extra_data));
    unsafe {
        transmute(sheet)
    }
}

#[no_mangle]
pub extern "C" fn Servo_StyleSheet_FromUTF8Bytes(data: *const nsACString,
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
    let loader = StylesheetLoader::new();
    let sheet = Arc::new(Stylesheet::from_str(
        input, url, origin, Default::default(), Some(&loader),
        Box::new(StdoutErrorReporter), extra_data));
    unsafe {
        transmute(sheet)
    }
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_AppendStyleSheet(raw_data: RawServoStyleSetBorrowed,
                                                  raw_sheet: RawServoStyleSheetBorrowed) {
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    let sheet = HasArcFFI::as_arc(&raw_sheet);
    data.stylesheets.retain(|x| !arc_ptr_eq(x, sheet));
    data.stylesheets.push(sheet.clone());
    data.stylesheets_changed = true;
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_PrependStyleSheet(raw_data: RawServoStyleSetBorrowed,
                                                   raw_sheet: RawServoStyleSheetBorrowed) {
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    let sheet = HasArcFFI::as_arc(&raw_sheet);
    data.stylesheets.retain(|x| !arc_ptr_eq(x, sheet));
    data.stylesheets.insert(0, sheet.clone());
    data.stylesheets_changed = true;
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_InsertStyleSheetBefore(raw_data: RawServoStyleSetBorrowed,
                                                        raw_sheet: RawServoStyleSheetBorrowed,
                                                        raw_reference: RawServoStyleSheetBorrowed) {
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    let sheet = HasArcFFI::as_arc(&raw_sheet);
    let reference = HasArcFFI::as_arc(&raw_reference);
    data.stylesheets.retain(|x| !arc_ptr_eq(x, sheet));
    let index = data.stylesheets.iter().position(|x| arc_ptr_eq(x, reference)).unwrap();
    data.stylesheets.insert(index, sheet.clone());
    data.stylesheets_changed = true;
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_RemoveStyleSheet(raw_data: RawServoStyleSetBorrowed,
                                                  raw_sheet: RawServoStyleSheetBorrowed) {
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    let sheet = HasArcFFI::as_arc(&raw_sheet);
    data.stylesheets.retain(|x| !arc_ptr_eq(x, sheet));
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
pub extern "C" fn Servo_StyleSheet_AddRef(sheet: RawServoStyleSheetBorrowed) -> () {
    unsafe { Stylesheet::addref(sheet) };
}

#[no_mangle]
pub extern "C" fn Servo_StyleSheet_Release(sheet: RawServoStyleSheetBorrowed) -> () {
    unsafe { Stylesheet::release(sheet) };
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
pub extern "C" fn Servo_CssRules_AddRef(rules: ServoCssRulesBorrowed) -> () {
    unsafe { RwLock::<CssRules>::addref(rules) };
}

#[no_mangle]
pub extern "C" fn Servo_CssRules_Release(rules: ServoCssRulesBorrowed) -> () {
    unsafe { RwLock::<CssRules>::release(rules) };
}

#[no_mangle]
pub extern "C" fn Servo_StyleRule_AddRef(rule: RawServoStyleRuleBorrowed) -> () {
    unsafe { RwLock::<StyleRule>::addref(rule) };
}

#[no_mangle]
pub extern "C" fn Servo_StyleRule_Release(rule: RawServoStyleRuleBorrowed) -> () {
    unsafe { RwLock::<StyleRule>::release(rule) };
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
pub extern "C" fn Servo_ComputedValues_GetForAnonymousBox(parent_style_or_null: ServoComputedValuesBorrowedOrNull,
                                                         pseudo_tag: *mut nsIAtom,
                                                         raw_data: RawServoStyleSetBorrowed)
     -> ServoComputedValuesStrong {
    // The stylist consumes stylesheets lazily.
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    data.flush_stylesheets();

    let atom = Atom::from(pseudo_tag);
    let pseudo = PseudoElement::from_atom_unchecked(atom, /* anon_box = */ true);


    let maybe_parent = ComputedValues::arc_from_borrowed(&parent_style_or_null);
    let new_computed = data.stylist.precomputed_values_for_pseudo(&pseudo, maybe_parent, false)
                           .map(|styles| styles.values);
    new_computed.map_or(Strong::null(), |c| c.into_strong())
}

#[no_mangle]
pub extern "C" fn Servo_ComputedValues_GetForPseudoElement(parent_style: ServoComputedValuesBorrowed,
                                                           match_element: RawGeckoElementBorrowed,
                                                           pseudo_tag: *mut nsIAtom,
                                                           raw_data: RawServoStyleSetBorrowed,
                                                           is_probe: bool)
     -> ServoComputedValuesStrong {
    debug_assert!(!(match_element as *const _).is_null());

    let parent_or_null = || {
        if is_probe {
            Strong::null()
        } else {
            ComputedValues::as_arc(&parent_style).clone().into_strong()
        }
    };

    let atom = Atom::from(pseudo_tag);
    let pseudo = PseudoElement::from_atom_unchecked(atom, /* anon_box = */ false);

    // The stylist consumes stylesheets lazily.
    let mut data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    data.flush_stylesheets();

    let element = GeckoElement(match_element);


    match SelectorImpl::pseudo_element_cascade_type(&pseudo) {
        PseudoElementCascadeType::Eager => {
            let maybe_computed = element.get_pseudo_style(&pseudo);
            maybe_computed.map_or_else(parent_or_null, FFIArcHelpers::into_strong)
        }
        PseudoElementCascadeType::Lazy => {
            let parent = ComputedValues::as_arc(&parent_style);
            data.stylist
                .lazily_compute_pseudo_element_style(&element, &pseudo, parent)
                .map(|styles| styles.values)
                .map_or_else(parent_or_null, FFIArcHelpers::into_strong)
        }
        PseudoElementCascadeType::Precomputed => {
            unreachable!("Anonymous pseudo found in \
                         Servo_GetComputedValuesForPseudoElement");
        }
    }
}

#[no_mangle]
pub extern "C" fn Servo_ComputedValues_Inherit(parent_style: ServoComputedValuesBorrowedOrNull)
     -> ServoComputedValuesStrong {
    let maybe_arc = ComputedValues::arc_from_borrowed(&parent_style);
    let style = if let Some(reference) = maybe_arc.as_ref() {
        ComputedValues::inherit_from(reference)
    } else {
        Arc::new(ComputedValues::initial_values().clone())
    };
    style.into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_ComputedValues_AddRef(ptr: ServoComputedValuesBorrowed) -> () {
    unsafe { ComputedValues::addref(ptr) };
}

#[no_mangle]
pub extern "C" fn Servo_ComputedValues_Release(ptr: ServoComputedValuesBorrowed) -> () {
    unsafe { ComputedValues::release(ptr) };
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_Init() -> RawServoStyleSetOwned {
    let data = Box::new(PerDocumentStyleData::new());
    data.into_ffi()
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_Drop(data: RawServoStyleSetOwned) -> () {
    let _ = data.into_box::<PerDocumentStyleData>();
}


#[no_mangle]
pub extern "C" fn Servo_ParseProperty(property: *const nsACString, value: *const nsACString,
                                      base_url: *const nsACString, base: *mut ThreadSafeURIHolder,
                                      referrer: *mut ThreadSafeURIHolder,
                                      principal: *mut ThreadSafePrincipalHolder)
                                      -> RawServoDeclarationBlockStrong {
    let name = unsafe { property.as_ref().unwrap().as_str_unchecked() };
    let id = if let Ok(id) = PropertyId::parse(name.into()) {
        id
    } else {
        return RawServoDeclarationBlockStrong::null()
    };
    let value = unsafe { value.as_ref().unwrap().as_str_unchecked() };
    let base_str = unsafe { base_url.as_ref().unwrap().as_str_unchecked() };
    let base_url = ServoUrl::parse(base_str).unwrap();
    let extra_data = unsafe { ParserContextExtraData {
        base: Some(GeckoArcURI::new(base)),
        referrer: Some(GeckoArcURI::new(referrer)),
        principal: Some(GeckoArcPrincipal::new(principal)),
    }};

    let context = ParserContext::new_with_extra_data(Origin::Author, &base_url,
                                                     Box::new(StdoutErrorReporter),
                                                     extra_data);

    let mut results = vec![];
    match PropertyDeclaration::parse(id, &context, &mut Parser::new(value),
                                     &mut results, false) {
        PropertyDeclarationParseResult::ValidOrIgnoredDeclaration => {},
        _ => return RawServoDeclarationBlockStrong::null(),
    }

    let results = results.into_iter().map(|r| (r, Importance::Normal)).collect();

    Arc::new(RwLock::new(PropertyDeclarationBlock {
        declarations: results,
        important_count: 0,
    })).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_ParseStyleAttribute(data: *const nsACString) -> RawServoDeclarationBlockStrong {
    let value = unsafe { data.as_ref().unwrap().as_str_unchecked() };
    Arc::new(RwLock::new(GeckoElement::parse_style_attribute(value))).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_CreateEmpty() -> RawServoDeclarationBlockStrong {
    Arc::new(RwLock::new(PropertyDeclarationBlock { declarations: vec![], important_count: 0 })).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_Clone(declarations: RawServoDeclarationBlockBorrowed)
                                               -> RawServoDeclarationBlockStrong {
    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    Arc::new(RwLock::new(declarations.read().clone())).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_AddRef(declarations: RawServoDeclarationBlockBorrowed) {
    unsafe { RwLock::<PropertyDeclarationBlock>::addref(declarations) };
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_Release(declarations: RawServoDeclarationBlockBorrowed) {
    unsafe { RwLock::<PropertyDeclarationBlock>::release(declarations) };
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_Equals(a: RawServoDeclarationBlockBorrowed,
                                                b: RawServoDeclarationBlockBorrowed)
                                                -> bool {
    *RwLock::<PropertyDeclarationBlock>::as_arc(&a).read() == *RwLock::<PropertyDeclarationBlock>::as_arc(&b).read()
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
    property: *mut nsIAtom, is_custom: bool,
    buffer: *mut nsAString)
{
    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    let property = get_property_id_from_atom(property, is_custom);
    let mut string = String::new();
    let rv = declarations.read().single_value_to_css(&property, &mut string);
    debug_assert!(rv.is_ok());

    write!(unsafe { &mut *buffer }, "{}", string).expect("Failed to copy string");
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_Count(declarations: RawServoDeclarationBlockBorrowed) -> u32 {
     let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
     declarations.read().declarations.len() as u32
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_GetNthProperty(declarations: RawServoDeclarationBlockBorrowed,
                                                        index: u32, result: *mut nsAString) -> bool {
    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    if let Some(&(ref decl, _)) = declarations.read().declarations.get(index as usize) {
        let result = unsafe { result.as_mut().unwrap() };
        decl.id().to_css(result).unwrap();
        true
    } else {
        false
    }
}

fn get_property_id_from_atom(atom: *mut nsIAtom, is_custom: bool) -> PropertyId {
    let atom = Atom::from(atom);
    if !is_custom {
        // FIXME: can we do this mapping without going through a UTF-8 string?
        // Maybe even from nsCSSPropertyID directly?
        PropertyId::parse(atom.to_string().into()).expect("got unknown property name from Gecko")
    } else {
        PropertyId::Custom(atom)
    }
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_GetPropertyValue(declarations: RawServoDeclarationBlockBorrowed,
                                                          property: *mut nsIAtom, is_custom: bool,
                                                          value: *mut nsAString) {
    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    let property = get_property_id_from_atom(property, is_custom);
    declarations.read().property_value_to_css(&property, unsafe { value.as_mut().unwrap() }).unwrap();
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_GetPropertyIsImportant(declarations: RawServoDeclarationBlockBorrowed,
                                                                property: *mut nsIAtom, is_custom: bool) -> bool {
    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    let property = get_property_id_from_atom(property, is_custom);
    declarations.read().property_priority(&property).important()
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetProperty(declarations: RawServoDeclarationBlockBorrowed,
                                                     property: *mut nsIAtom, is_custom: bool,
                                                     value: *mut nsACString, is_important: bool) -> bool {
    let property = get_property_id_from_atom(property, is_custom);
    let value = unsafe { value.as_ref().unwrap().as_str_unchecked() };
    // FIXME Needs real URL and ParserContextExtraData.
    let base_url = &*DUMMY_BASE_URL;
    let extra_data = ParserContextExtraData::default();
    if let Ok(decls) = parse_one_declaration(property, value, &base_url,
                                             Box::new(StdoutErrorReporter), extra_data) {
        let mut declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations).write();
        let importance = if is_important { Importance::Important } else { Importance::Normal };
        for decl in decls.into_iter() {
            declarations.set_parsed_declaration(decl, importance);
        }
        true
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_RemoveProperty(declarations: RawServoDeclarationBlockBorrowed,
                                                        property: *mut nsIAtom, is_custom: bool) {
    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    let property = get_property_id_from_atom(property, is_custom);
    declarations.write().remove_property(&property);
}

#[no_mangle]
pub extern "C" fn Servo_CSSSupports(property: *const nsACString, value: *const nsACString) -> bool {
    let property = unsafe { property.as_ref().unwrap().as_str_unchecked() };
    let id =  if let Ok(id) = PropertyId::parse(property.into()) {
        id
    } else {
        return false
    };
    let value = unsafe { value.as_ref().unwrap().as_str_unchecked() };

    let base_url = &*DUMMY_BASE_URL;
    let extra_data = ParserContextExtraData::default();

    match parse_one_declaration(id, &value, &base_url, Box::new(StdoutErrorReporter), extra_data) {
        Ok(decls) => !decls.is_empty(),
        Err(()) => false,
    }
}

/// Only safe to call on the main thread, with exclusive access to the element and
/// its ancestors.
unsafe fn maybe_restyle<'a>(data: &'a mut AtomicRefMut<ElementData>, element: GeckoElement)
    -> Option<&'a mut RestyleData>
{
    let r = data.restyle();
    if r.is_some() {
        // Propagate the bit up the chain.
        let mut curr = element;
        while let Some(parent) = curr.parent_element() {
            curr = parent;
            if curr.has_dirty_descendants() { break; }
            curr.set_dirty_descendants();
        }
    }
    r
}

#[no_mangle]
pub extern "C" fn Servo_Element_GetSnapshot(element: RawGeckoElementBorrowed) -> *mut structs::ServoElementSnapshot
{
    let element = GeckoElement(element);
    let mut data = unsafe { element.ensure_data().borrow_mut() };
    let snapshot = if let Some(restyle_data) = unsafe { maybe_restyle(&mut data, element) } {
        restyle_data.snapshot.ensure(|| element.create_snapshot()).borrow_mut_raw()
    } else {
        ptr::null_mut()
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
    let mut data = unsafe { element.ensure_data().borrow_mut() };
    debug!("Servo_NoteExplicitHints: {:?}, restyle_hint={:?}, change_hint={:?}",
           element, restyle_hint, change_hint);

    if let Some(restyle_data) = unsafe { maybe_restyle(&mut data, element) } {
        let restyle_hint: RestyleHint = restyle_hint.into();
        restyle_data.hint.insert(&restyle_hint.into());
        restyle_data.damage |= damage;
    } else {
        debug!("(Element not styled, discarding hints)");
    }
}

#[no_mangle]
pub extern "C" fn Servo_CheckChangeHint(element: RawGeckoElementBorrowed) -> nsChangeHint
{
    let element = GeckoElement(element);
    if element.get_data().is_none() {
        error!("Trying to get change hint from unstyled element");
        return nsChangeHint(0);
    }

    let mut data = element.get_data().unwrap().borrow_mut();
    let damage = data.damage_sloppy();

    // If there's no change hint, the caller won't consume the new style. Do that
    // ourselves.
    //
    // FIXME(bholley): Once we start storing style data on frames, we'll want to
    // drop the data here instead.
    if damage.is_empty() {
        data.persist();
    }

    debug!("Servo_GetChangeHint: {:?}, damage={:?}", element, damage);
    damage.as_change_hint()
}

#[no_mangle]
pub extern "C" fn Servo_ResolveStyle(element: RawGeckoElementBorrowed,
                                     raw_data: RawServoStyleSetBorrowed,
                                     consume: structs::ConsumeStyleBehavior,
                                     compute: structs::LazyComputeBehavior) -> ServoComputedValuesStrong
{
    let element = GeckoElement(element);
    debug!("Servo_ResolveStyle: {:?}, consume={:?}, compute={:?}", element, consume, compute);

    let mut data = unsafe { element.ensure_data() }.borrow_mut();

    if compute == structs::LazyComputeBehavior::Allow {
        let should_compute = !data.has_current_styles();
        if should_compute {
            debug!("Performing manual style computation");
            if let Some(parent) = element.parent_element() {
                if parent.borrow_data().map_or(true, |d| !d.has_current_styles()) {
                    error!("Attempting manual style computation with unstyled parent");
                    return Arc::new(ComputedValues::initial_values().clone()).into_strong();
                }
            }

            let mut per_doc_data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
            let shared_style_context = create_shared_context(&mut per_doc_data);
            let context = StandaloneStyleContext::new(&shared_style_context);

            let mut traversal_data = PerLevelTraversalData {
                current_dom_depth: None,
            };

            recalc_style_at::<_, _, RecalcStyleOnly>(&context, &mut traversal_data, element, &mut data);

            // The element was either unstyled or needed restyle. If it was unstyled, it may have
            // additional unstyled children that subsequent traversals won't find now that the style
            // on this element is up-to-date. Mark dirty descendants in that case.
            if element.first_child_element().is_some() {
                unsafe { element.set_dirty_descendants() };
            }
        }
    }

    if !data.has_current_styles() {
        error!("Resolving style on unstyled element with lazy computation forbidden.");
        return Arc::new(ComputedValues::initial_values().clone()).into_strong();
    }

    let values = data.styles().primary.values.clone();

    if consume == structs::ConsumeStyleBehavior::Consume {
        // FIXME(bholley): Once we start storing style data on frames, we'll want to
        // drop the data here instead.
        data.persist();
    }

    values.into_strong()
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
