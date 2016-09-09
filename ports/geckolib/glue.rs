/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use app_units::Au;
use data::{NUM_THREADS, PerDocumentStyleData};
use env_logger;
use euclid::Size2D;
use gecko_bindings::bindings::RawGeckoDocumentBorrowed;
use gecko_bindings::bindings::{RawGeckoDocument, RawGeckoElement, RawGeckoNode};
use gecko_bindings::bindings::{RawGeckoElementBorrowed, RawGeckoNodeBorrowed};
use gecko_bindings::bindings::{RawServoStyleSet, RawServoStyleSetBorrowedMut};
use gecko_bindings::bindings::{RawServoStyleSetBorrowed, RawServoStyleSetOwned, ServoNodeDataOwned};
use gecko_bindings::bindings::{RawServoStyleSheetBorrowed, ServoComputedValuesBorrowed};
use gecko_bindings::bindings::{RawServoStyleSheetStrong, ServoComputedValuesStrong};
use gecko_bindings::bindings::{ServoComputedValuesBorrowedOrNull, ServoDeclarationBlock};
use gecko_bindings::bindings::{ServoDeclarationBlockBorrowed, ServoDeclarationBlockStrong};
use gecko_bindings::bindings::{ThreadSafePrincipalHolder, ThreadSafeURIHolder, nsHTMLCSSStyleSheet};
use gecko_bindings::ptr::{GeckoArcPrincipal, GeckoArcURI};
use gecko_bindings::structs::ServoElementSnapshot;
use gecko_bindings::structs::nsRestyleHint;
use gecko_bindings::structs::{SheetParsingMode, nsIAtom};
use gecko_bindings::sugar::ownership::{FFIArcHelpers, HasArcFFI, HasBoxFFI};
use gecko_bindings::sugar::ownership::{HasSimpleFFI, HasFFI, Strong};
use gecko_string_cache::Atom;
use snapshot::GeckoElementSnapshot;
use std::mem::transmute;
use std::ptr;
use std::slice;
use std::str::from_utf8_unchecked;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::{Arc, Mutex};
use style::arc_ptr_eq;
use style::context::{LocalStyleContextCreationInfo, ReflowGoal, SharedStyleContext};
use style::dom::{TDocument, TElement, TNode};
use style::domrefcell::DOMRefCell;
use style::error_reporting::StdoutErrorReporter;
use style::gecko_selector_impl::{GeckoSelectorImpl, PseudoElement};
use style::parallel;
use style::parser::ParserContextExtraData;
use style::properties::{ComputedValues, PropertyDeclarationBlock, parse_one_declaration};
use style::selector_impl::PseudoElementCascadeType;
use style::sequential;
use style::stylesheets::{Stylesheet, Origin};
use style::timer::Timer;
use traversal::RecalcStyleOnly;
use url::Url;
use wrapper::{DUMMY_BASE_URL, GeckoDocument, GeckoElement, GeckoNode, NonOpaqueStyleData};

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

fn restyle_subtree(node: GeckoNode, raw_data: RawServoStyleSetBorrowedMut) {
    debug_assert!(node.is_element() || node.is_text_node());

    // Force the creation of our lazily-constructed initial computed values on
    // the main thread, since it's not safe to call elsewhere.
    //
    // FIXME(bholley): this should move into Servo_Initialize as soon as we get
    // rid of the HackilyFindSomeDeviceContext stuff that happens during
    // initial_values computation, since that stuff needs to be called further
    // along in startup than the sensible place to call Servo_Initialize.
    ComputedValues::initial_values();

    // The stylist consumes stylesheets lazily.
    let per_doc_data = PerDocumentStyleData::from_ffi_mut(raw_data);
    per_doc_data.flush_stylesheets();

    let local_context_data =
        LocalStyleContextCreationInfo::new(per_doc_data.new_animations_sender.clone());

    let shared_style_context = SharedStyleContext {
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

    // We ensure this is true before calling Servo_RestyleSubtree()
    debug_assert!(node.is_dirty() || node.has_dirty_descendants());
    if per_doc_data.num_threads == 1 || per_doc_data.work_queue.is_none() {
        sequential::traverse_dom::<GeckoNode, RecalcStyleOnly>(node, &shared_style_context);
    } else {
        parallel::traverse_dom::<GeckoNode, RecalcStyleOnly>(node, &shared_style_context,
                                                             per_doc_data.work_queue.as_mut().unwrap());
    }
}

#[no_mangle]
pub extern "C" fn Servo_RestyleSubtree(node: RawGeckoNodeBorrowed,
                                       raw_data: RawServoStyleSetBorrowedMut) -> () {
    let node = GeckoNode(node);
    restyle_subtree(node, raw_data);
}

#[no_mangle]
pub extern "C" fn Servo_RestyleDocument(doc: RawGeckoDocumentBorrowed, raw_data: RawServoStyleSetBorrowedMut) -> () {
    let document = GeckoDocument(doc);
    let node = match document.root_node() {
        Some(x) => x,
        None => return,
    };
    restyle_subtree(node, raw_data);
}

#[no_mangle]
pub extern "C" fn Servo_StyleWorkerThreadCount() -> u32 {
    *NUM_THREADS as u32
}

#[no_mangle]
pub extern "C" fn Servo_NodeData_Drop(data: ServoNodeDataOwned) -> () {
    let _ = data.into_box::<NonOpaqueStyleData>();
}

#[no_mangle]
pub extern "C" fn Servo_StyleSheet_FromUTF8Bytes(bytes: *const u8,
                                                length: u32,
                                                mode: SheetParsingMode,
                                                base_bytes: *const u8,
                                                base_length: u32,
                                                base: *mut ThreadSafeURIHolder,
                                                referrer: *mut ThreadSafeURIHolder,
                                                principal: *mut ThreadSafePrincipalHolder)
                                                -> RawServoStyleSheetStrong {
    let input = unsafe { from_utf8_unchecked(slice::from_raw_parts(bytes, length as usize)) };

    let origin = match mode {
        SheetParsingMode::eAuthorSheetFeatures => Origin::Author,
        SheetParsingMode::eUserSheetFeatures => Origin::User,
        SheetParsingMode::eAgentSheetFeatures => Origin::UserAgent,
    };

    let base_str = unsafe { from_utf8_unchecked(slice::from_raw_parts(base_bytes, base_length as usize)) };
    let url = Url::parse(base_str).unwrap();
    let extra_data = ParserContextExtraData {
        base: Some(GeckoArcURI::new(base)),
        referrer: Some(GeckoArcURI::new(referrer)),
        principal: Some(GeckoArcPrincipal::new(principal)),
    };
    let sheet = Arc::new(Stylesheet::from_str(input, url, origin, Box::new(StdoutErrorReporter),
                                              extra_data));
    unsafe {
        transmute(sheet)
    }
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_AppendStyleSheet(raw_data: RawServoStyleSetBorrowedMut,
                                                  raw_sheet: RawServoStyleSheetBorrowed) {
    let data = PerDocumentStyleData::from_ffi_mut(raw_data);
    let sheet = HasArcFFI::as_arc(&raw_sheet);
    data.stylesheets.retain(|x| !arc_ptr_eq(x, sheet));
    data.stylesheets.push(sheet.clone());
    data.stylesheets_changed = true;
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_PrependStyleSheet(raw_data: RawServoStyleSetBorrowedMut,
                                                   raw_sheet: RawServoStyleSheetBorrowed) {
    let data = PerDocumentStyleData::from_ffi_mut(raw_data);
    let sheet = HasArcFFI::as_arc(&raw_sheet);
    data.stylesheets.retain(|x| !arc_ptr_eq(x, sheet));
    data.stylesheets.insert(0, sheet.clone());
    data.stylesheets_changed = true;
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_InsertStyleSheetBefore(raw_data: RawServoStyleSetBorrowedMut,
                                                        raw_sheet: RawServoStyleSheetBorrowed,
                                                        raw_reference: RawServoStyleSheetBorrowed) {
    let data = PerDocumentStyleData::from_ffi_mut(raw_data);
    let sheet = HasArcFFI::as_arc(&raw_sheet);
    let reference = HasArcFFI::as_arc(&raw_reference);
    data.stylesheets.retain(|x| !arc_ptr_eq(x, sheet));
    let index = data.stylesheets.iter().position(|x| arc_ptr_eq(x, reference)).unwrap();
    data.stylesheets.insert(index, sheet.clone());
    data.stylesheets_changed = true;
}

#[no_mangle]
pub extern "C" fn Servo_StyleSet_RemoveStyleSheet(raw_data: RawServoStyleSetBorrowedMut,
                                                  raw_sheet: RawServoStyleSheetBorrowed) {
    let data = PerDocumentStyleData::from_ffi_mut(raw_data);
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
    let node = unsafe { GeckoNode(node) };
    let arc_cv = match node.borrow_data().map_or(None, |data| data.style.clone()) {
        Some(style) => style,
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
                                                         raw_data: RawServoStyleSetBorrowedMut)
     -> ServoComputedValuesStrong {
    // The stylist consumes stylesheets lazily.
    let data = PerDocumentStyleData::from_ffi_mut(raw_data);
    data.flush_stylesheets();

    let atom = Atom::from(pseudo_tag);
    let pseudo = PseudoElement::from_atom_unchecked(atom, /* anon_box = */ true);


    let maybe_parent = parent_style_or_null.as_arc_opt();
    let new_computed = data.stylist.precomputed_values_for_pseudo(&pseudo, maybe_parent);
    new_computed.map_or(Strong::null(), |c| c.into_strong())
}

#[no_mangle]
pub extern "C" fn Servo_ComputedValues_GetForPseudoElement(parent_style: ServoComputedValuesBorrowed,
                                                           match_element: RawGeckoElementBorrowed,
                                                           pseudo_tag: *mut nsIAtom,
                                                           raw_data: RawServoStyleSetBorrowedMut,
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
    let data = PerDocumentStyleData::from_ffi_mut(raw_data);
    data.flush_stylesheets();

    let element = unsafe { GeckoElement(match_element) };


    match GeckoSelectorImpl::pseudo_element_cascade_type(&pseudo) {
        PseudoElementCascadeType::Eager => {
            let node = element.as_node();
            let maybe_computed = node.borrow_data()
                                     .and_then(|data| {
                                         data.per_pseudo.get(&pseudo).map(|c| c.clone())
                                     });
            maybe_computed.map_or_else(parent_or_null, FFIArcHelpers::into_strong)
        }
        PseudoElementCascadeType::Lazy => {
            let parent = ComputedValues::as_arc(&parent_style);
            data.stylist
                .lazily_compute_pseudo_element_style(&element, &pseudo, parent)
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
    let style = if parent_style.is_null() {
        Arc::new(ComputedValues::initial_values().clone())
    } else {
        ComputedValues::inherit_from(parent_style.as_arc())
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

pub struct GeckoDeclarationBlock {
    pub declarations: Option<Arc<DOMRefCell<PropertyDeclarationBlock>>>,
    // XXX The following two fields are made atomic to work around the
    // ownership system so that they can be changed inside a shared
    // instance. It wouldn't provide safety as Rust usually promises,
    // but it is fine as far as we only access them in a single thread.
    // If we need to access them in different threads, we would need
    // to redesign how it works with MiscContainer in Gecko side.
    pub cache: AtomicPtr<nsHTMLCSSStyleSheet>,
    pub immutable: AtomicBool,
}

unsafe impl HasFFI for GeckoDeclarationBlock {
    type FFIType = ServoDeclarationBlock;
}
unsafe impl HasArcFFI for GeckoDeclarationBlock {}

#[no_mangle]
pub extern "C" fn Servo_ParseStyleAttribute(bytes: *const u8, length: u32,
                                            cache: *mut nsHTMLCSSStyleSheet)
                                            -> ServoDeclarationBlockStrong {
    let value = unsafe { from_utf8_unchecked(slice::from_raw_parts(bytes, length as usize)) };
    Arc::new(GeckoDeclarationBlock {
        declarations: GeckoElement::parse_style_attribute(value).map(|block| {
            Arc::new(DOMRefCell::new(block))
        }),
        cache: AtomicPtr::new(cache),
        immutable: AtomicBool::new(false),
    }).into_strong()
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_AddRef(declarations: ServoDeclarationBlockBorrowed) {
    unsafe { GeckoDeclarationBlock::addref(declarations) };
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_Release(declarations: ServoDeclarationBlockBorrowed) {
    unsafe { GeckoDeclarationBlock::release(declarations) };
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_GetCache(declarations: ServoDeclarationBlockBorrowed)
                                                 -> *mut nsHTMLCSSStyleSheet {
    GeckoDeclarationBlock::as_arc(&declarations).cache.load(Ordering::Relaxed)
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_SetImmutable(declarations: ServoDeclarationBlockBorrowed) {
    GeckoDeclarationBlock::as_arc(&declarations).immutable.store(true, Ordering::Relaxed)
}

#[no_mangle]
pub extern "C" fn Servo_DeclarationBlock_ClearCachePointer(declarations: ServoDeclarationBlockBorrowed) {
    GeckoDeclarationBlock::as_arc(&declarations).cache.store(ptr::null_mut(), Ordering::Relaxed)
}

#[no_mangle]
pub extern "C" fn Servo_CSSSupports(property: *const u8, property_length: u32,
                                    value: *const u8, value_length: u32) -> bool {
    let property = unsafe { from_utf8_unchecked(slice::from_raw_parts(property, property_length as usize)) };
    let value    = unsafe { from_utf8_unchecked(slice::from_raw_parts(value, value_length as usize)) };

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
    let per_doc_data = PerDocumentStyleData::from_ffi(raw_data);
    let snapshot = unsafe { GeckoElementSnapshot::from_raw(snapshot) };
    let element = unsafe { GeckoElement(element) };

    // NB: This involves an FFI call, we can get rid of it easily if needed.
    let current_state = element.get_state();

    let hint = per_doc_data.stylist
                           .compute_restyle_hint(&element, &snapshot,
                                                 current_state);

    // NB: Binary representations match.
    unsafe { transmute(hint.bits() as u32) }
}
