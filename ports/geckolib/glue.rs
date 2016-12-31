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
use std::borrow::Cow;
use std::fmt::Write;
use std::mem::transmute;
use std::ptr;
use std::sync::{Arc, Mutex};
use style::arc_ptr_eq;
use style::atomic_refcell::AtomicRefMut;
use style::context::{QuirksMode, ReflowGoal, SharedStyleContext, StyleContext};
use style::context::{ThreadLocalStyleContext, ThreadLocalStyleContextCreationInfo};
use style::data::{ElementData, ElementStyles, RestyleData};
use style::dom::{ShowSubtreeData, TElement, TNode};
use style::error_reporting::StdoutErrorReporter;
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
use style::gecko_bindings::bindings::RawServoImportRuleBorrowed;
use style::gecko_bindings::bindings::ServoComputedValuesBorrowedOrNull;
use style::gecko_bindings::bindings::nsTArrayBorrowed_uintptr_t;
use style::gecko_bindings::structs;
use style::gecko_bindings::structs::{SheetParsingMode, nsIAtom, nsCSSPropertyID};
use style::gecko_bindings::structs::{ThreadSafePrincipalHolder, ThreadSafeURIHolder};
use style::gecko_bindings::structs::{nsRestyleHint, nsChangeHint};
use style::gecko_bindings::structs::Loader;
use style::gecko_bindings::structs::ServoStyleSheet;
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
use style::stylesheets::{CssRule, CssRules, Origin, Stylesheet, StyleRule, ImportRule};
use style::stylesheets::StylesheetLoader as StyleStylesheetLoader;
use style::thread_state;
use style::timer::Timer;
use style::traversal::{resolve_style, DomTraversal};
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
}

fn create_shared_context(per_doc_data: &PerDocumentStyleDataImpl) -> SharedStyleContext {
    let local_context_data =
        ThreadLocalStyleContextCreationInfo::new(per_doc_data.new_animations_sender.clone());

    SharedStyleContext {
        // FIXME (bug 1303229): Use the actual viewport size here
        viewport_size: Size2D::new(Au(0), Au(0)),
        screen_size_changed: false,
        goal: ReflowGoal::ForScriptQuery,
        stylist: per_doc_data.stylist.clone(),
        running_animations: per_doc_data.running_animations.clone(),
        expired_animations: per_doc_data.expired_animations.clone(),
        error_reporter: Box::new(StdoutErrorReporter),
        local_context_creation_data: Mutex::new(local_context_data),
        timer: Timer::new(),
        // FIXME Find the real QuirksMode information for this document
        quirks_mode: QuirksMode::NoQuirks,
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
        if parent.borrow_data().map_or(true, |d| d.styles().is_display_none()) {
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

    let shared_style_context = create_shared_context(&per_doc_data);
    let traversal = RecalcStyleOnly::new(shared_style_context);
    let known_depth = None;

    if per_doc_data.num_threads == 1 || per_doc_data.work_queue.is_none() {
        sequential::traverse_dom(&traversal, element, token);
    } else {
        parallel::traverse_dom(&traversal, element, known_depth, token,
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
    let sheet = Arc::new(Stylesheet::from_str(
        "", url, origin, Default::default(), None,
        Box::new(StdoutErrorReporter), extra_data));
    unsafe {
        transmute(sheet)
    }
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

    let sheet = Arc::new(Stylesheet::from_str(
        input, url, origin, Default::default(), loader,
        Box::new(StdoutErrorReporter), extra_data));
    unsafe {
        transmute(sheet)
    }
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
pub extern "C" fn Servo_ImportRule_AddRef(rule: RawServoImportRuleBorrowed) -> () {
    unsafe { RwLock::<ImportRule>::addref(rule) };
}

#[no_mangle]
pub extern "C" fn Servo_ImportRule_Release(rule: RawServoImportRuleBorrowed) -> () {
    unsafe { RwLock::<ImportRule>::release(rule) };
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
    let new_computed = data.stylist.precomputed_values_for_pseudo(&pseudo, maybe_parent, false)
                           .map(|styles| styles.values);
    new_computed.map_or(Strong::null(), |c| c.into_strong())
}

#[no_mangle]
pub extern "C" fn Servo_ResolvePseudoStyle(element: RawGeckoElementBorrowed,
                                           pseudo_tag: *mut nsIAtom, is_probe: bool,
                                           raw_data: RawServoStyleSetBorrowed)
     -> ServoComputedValuesStrong
{
    let element = GeckoElement(element);
    let data = unsafe { element.ensure_data() }.borrow_mut();

    // FIXME(bholley): Assert against this.
    if data.get_styles().is_none() {
        error!("Calling Servo_ResolvePseudoStyle on unstyled element");
        return if is_probe {
            Strong::null()
        } else {
            Arc::new(ComputedValues::initial_values().clone()).into_strong()
        };
    }

    let doc_data = PerDocumentStyleData::from_ffi(raw_data);
    match get_pseudo_style(element, pseudo_tag, data.styles(), doc_data) {
        Some(values) => values.into_strong(),
        None if !is_probe => data.styles().primary.values.clone().into_strong(),
        None => Strong::null(),
    }
}

fn get_pseudo_style(element: GeckoElement, pseudo_tag: *mut nsIAtom,
                    styles: &ElementStyles, doc_data: &PerDocumentStyleData)
                    -> Option<Arc<ComputedValues>>
{
    let pseudo = PseudoElement::from_atom_unchecked(Atom::from(pseudo_tag), false);
    match SelectorImpl::pseudo_element_cascade_type(&pseudo) {
        PseudoElementCascadeType::Eager => styles.pseudos.get(&pseudo).map(|s| s.values.clone()),
        PseudoElementCascadeType::Precomputed => unreachable!("No anonymous boxes"),
        PseudoElementCascadeType::Lazy => {
            let d = doc_data.borrow_mut();
            let base = &styles.primary.values;
            d.stylist.lazily_compute_pseudo_element_style(&element, &pseudo, base)
                     .map(|s| s.values.clone())
        },
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

macro_rules! get_property_id_from_nscsspropertyid {
    ($property_id: ident, $ret: expr) => {{
        match PropertyId::from_nscsspropertyid($property_id) {
            Ok(property_id) => property_id,
            Err(()) => { return $ret; }
        }
    }}
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
                value: *mut nsACString, is_important: bool) -> bool {
    let value = unsafe { value.as_ref().unwrap().as_str_unchecked() };
    // FIXME Needs real URL and ParserContextExtraData.
    let base_url = &*DUMMY_BASE_URL;
    let extra_data = ParserContextExtraData::default();
    if let Ok(decls) = parse_one_declaration(property_id, value, &base_url,
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
pub extern "C" fn Servo_DeclarationBlock_SetProperty(declarations: RawServoDeclarationBlockBorrowed,
                                                     property: *const nsACString, value: *mut nsACString,
                                                     is_important: bool) -> bool {
    set_property(declarations, get_property_id_from_property!(property, false), value, is_important)
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetPropertyById(declarations: RawServoDeclarationBlockBorrowed,
                                                         property: nsCSSPropertyID, value: *mut nsACString,
                                                         is_important: bool) -> bool {
    set_property(declarations, get_property_id_from_nscsspropertyid!(property, false), value, is_important)
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
pub extern "C" fn Servo_ImportRule_GetSheet(import_rule:
                                            RawServoImportRuleBorrowed)
                                            -> RawServoStyleSheetStrong {
    let import_rule = RwLock::<ImportRule>::as_arc(&import_rule);
    unsafe { transmute(import_rule.read().stylesheet.clone()) }
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
                                     consume: structs::ConsumeStyleBehavior)
                                     -> ServoComputedValuesStrong
{
    let element = GeckoElement(element);
    debug!("Servo_ResolveStyle: {:?}, consume={:?}", element, consume);

    let mut data = unsafe { element.ensure_data() }.borrow_mut();

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
pub extern "C" fn Servo_ResolveStyleLazily(element: RawGeckoElementBorrowed,
                                           pseudo_tag: *mut nsIAtom,
                                           consume: structs::ConsumeStyleBehavior,
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
        maybe_pseudo.unwrap_or_else(|| styles.primary.values.clone())
    };

    // In the common case we already have the style. Check that before setting
    // up all the computation machinery.
    let mut result = element.mutate_data()
                            .and_then(|d| d.get_styles().map(&finish));
    if result.is_some() {
        if consume == structs::ConsumeStyleBehavior::Consume {
            let mut d = element.mutate_data().unwrap();
            if !d.is_persistent() {
                // XXXheycam is it right to persist an ElementData::Restyle?
                // Couldn't we lose restyle hints that would cause us to
                // restyle descendants?
                d.persist();
            }
        }
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

    // Consume the style if requested, though it may not exist anymore if the
    // element is in a display:none subtree.
    if consume == structs::ConsumeStyleBehavior::Consume {
        element.mutate_data().map(|mut d| d.persist());
    }

    result.unwrap().into_strong()
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
