/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use cssparser::Parser;
use env_logger;
use euclid::Size2D;
use parking_lot::RwLock;
use std::fmt::Write;
use std::mem::transmute;
use std::sync::{Arc, Mutex};
use style::arc_ptr_eq;
use style::context::{LocalStyleContextCreationInfo, ReflowGoal, SharedStyleContext};
use style::dom::{NodeInfo, StylingMode, TElement, TNode};
use style::error_reporting::StdoutErrorReporter;
use style::gecko::data::{NUM_THREADS, PerDocumentStyleData};
use style::gecko::selector_impl::{GeckoSelectorImpl, PseudoElement};
use style::gecko::snapshot::GeckoElementSnapshot;
use style::gecko::traversal::RecalcStyleOnly;
use style::gecko::wrapper::{GeckoElement, GeckoNode};
use style::gecko::wrapper::DUMMY_BASE_URL;
use style::gecko_bindings::bindings::{RawGeckoElementBorrowed, RawGeckoNodeBorrowed};
use style::gecko_bindings::bindings::{RawServoDeclarationBlockBorrowed, RawServoDeclarationBlockStrong};
use style::gecko_bindings::bindings::{RawServoStyleSetBorrowed, RawServoStyleSetOwned};
use style::gecko_bindings::bindings::{RawServoStyleSheetBorrowed, ServoComputedValuesBorrowed};
use style::gecko_bindings::bindings::{RawServoStyleSheetStrong, ServoComputedValuesStrong};
use style::gecko_bindings::bindings::{ThreadSafePrincipalHolder, ThreadSafeURIHolder};
use style::gecko_bindings::bindings::{nsACString, nsAString};
use style::gecko_bindings::bindings::Gecko_Utf8SliceToString;
use style::gecko_bindings::bindings::ServoComputedValuesBorrowedOrNull;
use style::gecko_bindings::structs::{SheetParsingMode, nsIAtom};
use style::gecko_bindings::structs::ServoElementSnapshot;
use style::gecko_bindings::structs::nsRestyleHint;
use style::gecko_bindings::structs::nsString;
use style::gecko_bindings::sugar::ownership::{FFIArcHelpers, HasArcFFI, HasBoxFFI};
use style::gecko_bindings::sugar::ownership::{HasSimpleFFI, Strong};
use style::gecko_bindings::sugar::refptr::{GeckoArcPrincipal, GeckoArcURI};
use style::parallel;
use style::parser::{ParserContext, ParserContextExtraData};
use style::properties::{CascadeFlags, ComputedValues, Importance, PropertyDeclaration};
use style::properties::{PropertyDeclarationParseResult, PropertyDeclarationBlock};
use style::properties::{apply_declarations, parse_one_declaration};
use style::selector_impl::PseudoElementCascadeType;
use style::sequential;
use style::string_cache::Atom;
use style::stylesheets::{Origin, Stylesheet};
use style::timer::Timer;
use style_traits::ToCss;
use url::Url;

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
}

#[no_mangle]
pub extern "C" fn Servo_Shutdown() -> () {
    // Destroy our default computed values.
    unsafe { ComputedValues::shutdown(); }
}

fn restyle_subtree(element: GeckoElement, raw_data: RawServoStyleSetBorrowed) {
    // Force the creation of our lazily-constructed initial computed values on
    // the main thread, since it's not safe to call elsewhere.
    //
    // FIXME(bholley): this should move into Servo_Initialize as soon as we get
    // rid of the HackilyFindSomeDeviceContext stuff that happens during
    // initial_values computation, since that stuff needs to be called further
    // along in startup than the sensible place to call Servo_Initialize.
    ComputedValues::initial_values();

    // The stylist consumes stylesheets lazily.
    let mut per_doc_data = PerDocumentStyleData::from_ffi(raw_data).borrow_mut();
    per_doc_data.flush_stylesheets();

    let local_context_data =
        LocalStyleContextCreationInfo::new(per_doc_data.new_animations_sender.clone());

    let shared_style_context = SharedStyleContext {
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
    };

    if element.styling_mode() == StylingMode::Stop {
        error!("Unnecessary call to restyle_subtree");
        return;
    }

    if per_doc_data.num_threads == 1 || per_doc_data.work_queue.is_none() {
        sequential::traverse_dom::<_, RecalcStyleOnly>(element.as_node(), &shared_style_context);
    } else {
        parallel::traverse_dom::<_, RecalcStyleOnly>(element.as_node(), &shared_style_context,
                                                     per_doc_data.work_queue.as_mut().unwrap());
    }
}

#[no_mangle]
pub extern "C" fn Servo_RestyleSubtree(node: RawGeckoNodeBorrowed,
                                       raw_data: RawServoStyleSetBorrowed) -> () {
    let node = GeckoNode(node);
    if let Some(element) = node.as_element() {
        restyle_subtree(element, raw_data);
    }
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
                                      CascadeFlags::empty());
    Arc::new(computed).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_StyleWorkerThreadCount() -> u32 {
    *NUM_THREADS as u32
}

#[no_mangle]
pub extern "C" fn Servo_Node_ClearNodeData(node: RawGeckoNodeBorrowed) -> () {
    if let Some(element) = GeckoNode(node).as_element() {
        element.clear_data();
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
    let url = Url::parse(base_str).unwrap();
    let extra_data = unsafe { ParserContextExtraData {
        base: Some(GeckoArcURI::new(base)),
        referrer: Some(GeckoArcURI::new(referrer)),
        principal: Some(GeckoArcPrincipal::new(principal)),
    }};
    let sheet = Arc::new(Stylesheet::from_str(input, url, origin, Box::new(StdoutErrorReporter),
                                              extra_data));
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
    !Stylesheet::as_arc(&raw_sheet).rules.is_empty()
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
pub extern "C" fn Servo_ComputedValues_Get(node: RawGeckoNodeBorrowed)
     -> ServoComputedValuesStrong {
    let node = GeckoNode(node);

    // Gecko erroneously calls this function from ServoRestyleManager::RecreateStyleContexts.
    // We plan to fix that, but just support it for now until that code gets rewritten.
    if node.is_text_node() {
        error!("Don't call Servo_ComputedValue_Get() for text nodes");
        let parent = node.parent_node().unwrap().as_element().unwrap();
        let parent_cv = parent.borrow_data().map_or_else(|| Arc::new(ComputedValues::initial_values().clone()),
                                                         |x| x.get_current_styles().unwrap()
                                                              .primary.clone());
        return ComputedValues::inherit_from(&parent_cv).into_strong();
    }

    let element = node.as_element().unwrap();
    let data = element.borrow_data();
    let arc_cv = match data.as_ref().and_then(|x| x.get_current_styles()) {
        Some(styles) => styles.primary.clone(),
        None => {
            // FIXME(bholley): This case subverts the intended semantics of this
            // function, and exists only to make stylo builds more robust corner-
            // cases where Gecko wants the style for a node that Servo never
            // traversed. We should remove this as soon as possible.
            error!("stylo: encountered unstyled node, substituting default values.");
            Arc::new(ComputedValues::initial_values().clone())
        },
    };
    arc_cv.into_strong()
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
                           .map(|(computed, _rule_node)| computed);
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


    match GeckoSelectorImpl::pseudo_element_cascade_type(&pseudo) {
        PseudoElementCascadeType::Eager => {
            let maybe_computed = element.get_pseudo_style(&pseudo);
            maybe_computed.map_or_else(parent_or_null, FFIArcHelpers::into_strong)
        }
        PseudoElementCascadeType::Lazy => {
            let parent = ComputedValues::as_arc(&parent_style);
            data.stylist
                .lazily_compute_pseudo_element_style(&element, &pseudo, parent)
                .map(|(c, _rule_node)| c)
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
    let value = unsafe { value.as_ref().unwrap().as_str_unchecked() };
    let base_str = unsafe { base_url.as_ref().unwrap().as_str_unchecked() };
    let base_url = Url::parse(base_str).unwrap();
    let extra_data = unsafe { ParserContextExtraData {
        base: Some(GeckoArcURI::new(base)),
        referrer: Some(GeckoArcURI::new(referrer)),
        principal: Some(GeckoArcPrincipal::new(principal)),
    }};

    let context = ParserContext::new_with_extra_data(Origin::Author, &base_url,
                                                     Box::new(StdoutErrorReporter),
                                                     extra_data);

    let mut results = vec![];
    match PropertyDeclaration::parse(name, &context, &mut Parser::new(value),
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
    buffer: *mut nsString)
{
    let mut string = String::new();

    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    declarations.read().to_css(&mut string).unwrap();
    // FIXME: We are expecting |declarations| to be a declaration block with either a single
    // longhand property-declaration or a series of longhand property-declarations that make
    // up a single shorthand property. As a result, it should be possible to serialize
    // |declarations| as a single declaration. However, we only want to return the *value* from
    // that single declaration. For now, we just manually strip the property name, colon,
    // leading spacing, and trailing space. In future we should find a more robust way to do
    // this.
    //
    // See https://github.com/servo/servo/issues/13423
    debug_assert!(string.find(':').is_some());
    let position = string.find(':').unwrap();
    // Get the value after the first colon and any following whitespace.
    let value = &string[(position + 1)..].trim_left();
    debug_assert!(value.ends_with(';'));
    let length = value.len() - 1; // Strip last semicolon.

    // FIXME: Once we have nsString bindings for Servo (bug 1294742), we should be able to drop
    // this and fill in |buffer| directly.
    unsafe {
        Gecko_Utf8SliceToString(buffer, value.as_ptr(), length);
    }
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
        write!(result, "{}", decl.name()).unwrap();
        true
    } else {
        false
    }
}

// FIXME Methods of PropertyDeclarationBlock should take atoms directly.
// This function is just a temporary workaround before that finishes.
fn get_property_name_from_atom(atom: *mut nsIAtom, is_custom: bool) -> String {
    let atom = Atom::from(atom);
    if !is_custom {
        atom.to_string()
    } else {
        let mut result = String::with_capacity(atom.len() as usize + 2);
        write!(result, "--{}", atom).unwrap();
        result
    }
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_GetPropertyValue(declarations: RawServoDeclarationBlockBorrowed,
                                                          property: *mut nsIAtom, is_custom: bool,
                                                          value: *mut nsAString) {
    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    let property = get_property_name_from_atom(property, is_custom);
    declarations.read().property_value_to_css(&property, unsafe { value.as_mut().unwrap() }).unwrap();
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_GetPropertyIsImportant(declarations: RawServoDeclarationBlockBorrowed,
                                                                property: *mut nsIAtom, is_custom: bool) -> bool {
    let declarations = RwLock::<PropertyDeclarationBlock>::as_arc(&declarations);
    let property = get_property_name_from_atom(property, is_custom);
    declarations.read().property_priority(&property).important()
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetProperty(declarations: RawServoDeclarationBlockBorrowed,
                                                     property: *mut nsIAtom, is_custom: bool,
                                                     value: *mut nsACString, is_important: bool) -> bool {
    let property = get_property_name_from_atom(property, is_custom);
    let value = unsafe { value.as_ref().unwrap().as_str_unchecked() };
    // FIXME Needs real URL and ParserContextExtraData.
    let base_url = &*DUMMY_BASE_URL;
    let extra_data = ParserContextExtraData::default();
    if let Ok(decls) = parse_one_declaration(&property, value, &base_url,
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
    let property = get_property_name_from_atom(property, is_custom);
    declarations.write().remove_property(&property);
}

#[no_mangle]
pub extern "C" fn Servo_CSSSupports(property: *const nsACString, value: *const nsACString) -> bool {
    let property = unsafe { property.as_ref().unwrap().as_str_unchecked() };
    let value = unsafe { value.as_ref().unwrap().as_str_unchecked() };

    let base_url = &*DUMMY_BASE_URL;
    let extra_data = ParserContextExtraData::default();

    match parse_one_declaration(&property, &value, &base_url, Box::new(StdoutErrorReporter), extra_data) {
        Ok(decls) => !decls.is_empty(),
        Err(()) => false,
    }
}

#[no_mangle]
pub extern "C" fn Servo_ComputeRestyleHint(element: RawGeckoElementBorrowed,
                                           snapshot: *mut ServoElementSnapshot,
                                           raw_data: RawServoStyleSetBorrowed) -> nsRestyleHint {
    let per_doc_data = PerDocumentStyleData::from_ffi(raw_data).borrow();
    let snapshot = unsafe { GeckoElementSnapshot::from_raw(snapshot) };
    let element = GeckoElement(element);

    // NB: This involves an FFI call, we can get rid of it easily if needed.
    let current_state = element.get_state();

    let hint = per_doc_data.stylist
                           .compute_restyle_hint(&element, &snapshot,
                                                 current_state);

    // NB: Binary representations match.
    unsafe { transmute(hint.bits() as u32) }
}
