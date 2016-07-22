/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use app_units::Au;
use data::{NUM_THREADS, PerDocumentStyleData};
use env_logger;
use euclid::Size2D;
use gecko_bindings::bindings::{RawGeckoDocument, RawGeckoElement, RawGeckoNode};
use gecko_bindings::bindings::{RawServoStyleSet, RawServoStyleSheet, ServoComputedValues};
use gecko_bindings::bindings::{ServoDeclarationBlock, ServoNodeData, ThreadSafePrincipalHolder};
use gecko_bindings::bindings::{ThreadSafeURIHolder, nsHTMLCSSStyleSheet};
use gecko_bindings::ptr::{GeckoArcPrincipal, GeckoArcURI};
use gecko_bindings::structs::ServoElementSnapshot;
use gecko_bindings::structs::nsRestyleHint;
use gecko_bindings::structs::{SheetParsingMode, nsIAtom};
use snapshot::GeckoElementSnapshot;
use std::mem::transmute;
use std::ptr;
use std::slice;
use std::str::from_utf8_unchecked;
use std::sync::{Arc, Mutex};
use style::arc_ptr_eq;
use style::context::{LocalStyleContextCreationInfo, ReflowGoal, SharedStyleContext};
use style::dom::{TDocument, TElement, TNode};
use style::error_reporting::StdoutErrorReporter;
use style::gecko_glue::ArcHelpers;
use style::gecko_selector_impl::{GeckoSelectorImpl, PseudoElement};
use style::parallel;
use style::parser::ParserContextExtraData;
use style::properties::{ComputedValues, PropertyDeclarationBlock, parse_one_declaration};
use style::selector_impl::{SelectorImplExt, PseudoElementCascadeType};
use style::sequential;
use style::stylesheets::{Stylesheet, Origin};
use style::timer::Timer;
use traversal::RecalcStyleOnly;
use url::Url;
use wrapper::{DUMMY_BASE_URL, GeckoDocument, GeckoElement, GeckoNode, NonOpaqueStyleData};

// TODO: This is ugly and should go away once we get an atom back-end.
pub fn pseudo_element_from_atom(pseudo: *mut nsIAtom,
                                in_ua_stylesheet: bool) -> Result<PseudoElement, String> {
    use gecko_bindings::bindings::Gecko_GetAtomAsUTF16;
    use selectors::parser::{ParserContext, SelectorImpl};

    let pseudo_string = unsafe {
        let mut length = 0;
        let mut buff = Gecko_GetAtomAsUTF16(pseudo, &mut length);

        // Handle the annoying preceding colon in front of everything in nsCSSAnonBoxList.h.
        debug_assert!(length >= 2 && *buff == ':' as u16 && *buff.offset(1) != ':' as u16);
        buff = buff.offset(1);
        length -= 1;

        String::from_utf16(slice::from_raw_parts(buff, length as usize)).unwrap()
    };

    let mut context = ParserContext::new();
    context.in_user_agent_stylesheet = in_ua_stylesheet;
    GeckoSelectorImpl::parse_pseudo_element(&context, &pseudo_string).map_err(|_| pseudo_string)
}

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
}

fn restyle_subtree(node: GeckoNode, raw_data: *mut RawServoStyleSet) {
    debug_assert!(node.is_element() || node.is_text_node());

    let per_doc_data = unsafe { &mut *(raw_data as *mut PerDocumentStyleData) };

    // Force the creation of our lazily-constructed initial computed values on
    // the main thread, since it's not safe to call elsewhere.
    //
    // FIXME(bholley): this should move into Servo_Initialize as soon as we get
    // rid of the HackilyFindSomeDeviceContext stuff that happens during
    // initial_values computation, since that stuff needs to be called further
    // along in startup than the sensible place to call Servo_Initialize.
    ComputedValues::initial_values();

    let _needs_dirtying = Arc::get_mut(&mut per_doc_data.stylist).unwrap()
                              .update(&per_doc_data.stylesheets,
                                      per_doc_data.stylesheets_changed);
    per_doc_data.stylesheets_changed = false;

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

    if node.is_dirty() || node.has_dirty_descendants() {
        if per_doc_data.num_threads == 1 {
            sequential::traverse_dom::<GeckoNode, RecalcStyleOnly>(node, &shared_style_context);
        } else {
            parallel::traverse_dom::<GeckoNode, RecalcStyleOnly>(node, &shared_style_context,
                                                                 &mut per_doc_data.work_queue);
        }
    }
}

#[no_mangle]
pub extern "C" fn Servo_RestyleSubtree(node: *mut RawGeckoNode,
                                       raw_data: *mut RawServoStyleSet) -> () {
    let node = unsafe { GeckoNode::from_raw(node) };
    restyle_subtree(node, raw_data);
}

#[no_mangle]
pub extern "C" fn Servo_RestyleDocument(doc: *mut RawGeckoDocument, raw_data: *mut RawServoStyleSet) -> () {
    let document = unsafe { GeckoDocument::from_raw(doc) };
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
pub extern "C" fn Servo_DropNodeData(data: *mut ServoNodeData) -> () {
    unsafe {
        let _ = Box::<NonOpaqueStyleData>::from_raw(data as *mut NonOpaqueStyleData);
    }
}

#[no_mangle]
pub extern "C" fn Servo_StylesheetFromUTF8Bytes(bytes: *const u8,
                                                length: u32,
                                                mode: SheetParsingMode,
                                                base: *mut ThreadSafeURIHolder,
                                                referrer: *mut ThreadSafeURIHolder,
                                                principal: *mut ThreadSafePrincipalHolder)
                                                -> *mut RawServoStyleSheet {
    let input = unsafe { from_utf8_unchecked(slice::from_raw_parts(bytes, length as usize)) };

    let origin = match mode {
        SheetParsingMode::eAuthorSheetFeatures => Origin::Author,
        SheetParsingMode::eUserSheetFeatures => Origin::User,
        SheetParsingMode::eAgentSheetFeatures => Origin::UserAgent,
    };

    // FIXME(heycam): Pass in the real base URL.
    let url = Url::parse("about:none").unwrap();
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
pub extern "C" fn Servo_AppendStyleSheet(raw_sheet: *mut RawServoStyleSheet,
                                         raw_data: *mut RawServoStyleSet) {
    type Helpers = ArcHelpers<RawServoStyleSheet, Stylesheet>;
    let data = PerDocumentStyleData::borrow_mut_from_raw(raw_data);
    Helpers::with(raw_sheet, |sheet| {
        data.stylesheets.retain(|x| !arc_ptr_eq(x, sheet));
        data.stylesheets.push(sheet.clone());
        data.stylesheets_changed = true;
    });
}

#[no_mangle]
pub extern "C" fn Servo_PrependStyleSheet(raw_sheet: *mut RawServoStyleSheet,
                                          raw_data: *mut RawServoStyleSet) {
    type Helpers = ArcHelpers<RawServoStyleSheet, Stylesheet>;
    let data = PerDocumentStyleData::borrow_mut_from_raw(raw_data);
    Helpers::with(raw_sheet, |sheet| {
        data.stylesheets.retain(|x| !arc_ptr_eq(x, sheet));
        data.stylesheets.insert(0, sheet.clone());
        data.stylesheets_changed = true;
    })
}

#[no_mangle]
pub extern "C" fn Servo_InsertStyleSheetBefore(raw_sheet: *mut RawServoStyleSheet,
                                               raw_reference: *mut RawServoStyleSheet,
                                               raw_data: *mut RawServoStyleSet) {
    type Helpers = ArcHelpers<RawServoStyleSheet, Stylesheet>;
    let data = PerDocumentStyleData::borrow_mut_from_raw(raw_data);
    Helpers::with(raw_sheet, |sheet| {
        Helpers::with(raw_reference, |reference| {
            data.stylesheets.retain(|x| !arc_ptr_eq(x, sheet));
            let index = data.stylesheets.iter().position(|x| arc_ptr_eq(x, reference)).unwrap();
            data.stylesheets.insert(index, sheet.clone());
            data.stylesheets_changed = true;
        })
    })
}

#[no_mangle]
pub extern "C" fn Servo_RemoveStyleSheet(raw_sheet: *mut RawServoStyleSheet,
                                         raw_data: *mut RawServoStyleSet) {
    type Helpers = ArcHelpers<RawServoStyleSheet, Stylesheet>;
    let data = PerDocumentStyleData::borrow_mut_from_raw(raw_data);
    Helpers::with(raw_sheet, |sheet| {
        data.stylesheets.retain(|x| !arc_ptr_eq(x, sheet));
        data.stylesheets_changed = true;
    });
}

#[no_mangle]
pub extern "C" fn Servo_StyleSheetHasRules(raw_sheet: *mut RawServoStyleSheet) -> bool {
    type Helpers = ArcHelpers<RawServoStyleSheet, Stylesheet>;
    Helpers::with(raw_sheet, |sheet| !sheet.rules.is_empty())
}

#[no_mangle]
pub extern "C" fn Servo_AddRefStyleSheet(sheet: *mut RawServoStyleSheet) -> () {
    type Helpers = ArcHelpers<RawServoStyleSheet, Stylesheet>;
    unsafe { Helpers::addref(sheet) };
}

#[no_mangle]
pub extern "C" fn Servo_ReleaseStyleSheet(sheet: *mut RawServoStyleSheet) -> () {
    type Helpers = ArcHelpers<RawServoStyleSheet, Stylesheet>;
    unsafe { Helpers::release(sheet) };
}

#[no_mangle]
pub extern "C" fn Servo_GetComputedValues(node: *mut RawGeckoNode)
     -> *mut ServoComputedValues {
    let node = unsafe { GeckoNode::from_raw(node) };
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
    unsafe { transmute(arc_cv) }
}

#[no_mangle]
pub extern "C" fn Servo_GetComputedValuesForAnonymousBox(parent_style_or_null: *mut ServoComputedValues,
                                                         pseudo_tag: *mut nsIAtom,
                                                         raw_data: *mut RawServoStyleSet)
     -> *mut ServoComputedValues {
    let data = PerDocumentStyleData::borrow_mut_from_raw(raw_data);

    let pseudo = match pseudo_element_from_atom(pseudo_tag, /* ua_stylesheet = */ true) {
        Ok(pseudo) => pseudo,
        Err(pseudo) => {
            warn!("stylo: Unable to parse anonymous-box pseudo-element: {}", pseudo);
            return ptr::null_mut();
        }
    };

    type Helpers = ArcHelpers<ServoComputedValues, ComputedValues>;

    Helpers::maybe_with(parent_style_or_null, |maybe_parent| {
        let new_computed = data.stylist.precomputed_values_for_pseudo(&pseudo, maybe_parent);
        new_computed.map_or(ptr::null_mut(), |c| Helpers::from(c))
    })
}

#[no_mangle]
pub extern "C" fn Servo_GetComputedValuesForPseudoElement(parent_style: *mut ServoComputedValues,
                                                          match_element: *mut RawGeckoElement,
                                                          pseudo_tag: *mut nsIAtom,
                                                          raw_data: *mut RawServoStyleSet,
                                                          is_probe: bool)
     -> *mut ServoComputedValues {
    debug_assert!(!match_element.is_null());

    let parent_or_null = || {
        if is_probe {
            ptr::null_mut()
        } else {
            Servo_AddRefComputedValues(parent_style);
            parent_style
        }
    };

    let pseudo = match pseudo_element_from_atom(pseudo_tag, /* ua_stylesheet = */ true) {
        Ok(pseudo) => pseudo,
        Err(pseudo) => {
            warn!("stylo: Unable to parse anonymous-box pseudo-element: {}", pseudo);
            return parent_or_null();
        }
    };


    let data = PerDocumentStyleData::borrow_mut_from_raw(raw_data);

    let element = unsafe { GeckoElement::from_raw(match_element) };

    type Helpers = ArcHelpers<ServoComputedValues, ComputedValues>;

    match GeckoSelectorImpl::pseudo_element_cascade_type(&pseudo) {
        PseudoElementCascadeType::Eager => {
            let node = element.as_node();
            let maybe_computed = node.borrow_data()
                                     .and_then(|data| {
                                         data.per_pseudo.get(&pseudo).map(|c| c.clone())
                                     });
            maybe_computed.map_or_else(parent_or_null, Helpers::from)
        }
        PseudoElementCascadeType::Lazy => {
            Helpers::with(parent_style, |parent| {
                data.stylist
                    .lazily_compute_pseudo_element_style(&element, &pseudo, parent)
                    .map_or_else(parent_or_null, Helpers::from)
            })
        }
        PseudoElementCascadeType::Precomputed => {
            unreachable!("Anonymous pseudo found in \
                         Servo_GetComputedValuesForPseudoElement");
        }
    }
}

#[no_mangle]
pub extern "C" fn Servo_InheritComputedValues(parent_style: *mut ServoComputedValues)
     -> *mut ServoComputedValues {
    type Helpers = ArcHelpers<ServoComputedValues, ComputedValues>;
    Helpers::with(parent_style, |parent| {
        let style = ComputedValues::inherit_from(parent);
        Helpers::from(style)
    })
}

#[no_mangle]
pub extern "C" fn Servo_AddRefComputedValues(ptr: *mut ServoComputedValues) -> () {
    type Helpers = ArcHelpers<ServoComputedValues, ComputedValues>;
    unsafe { Helpers::addref(ptr) };
}

#[no_mangle]
pub extern "C" fn Servo_ReleaseComputedValues(ptr: *mut ServoComputedValues) -> () {
    type Helpers = ArcHelpers<ServoComputedValues, ComputedValues>;
    unsafe { Helpers::release(ptr) };
}

#[no_mangle]
pub extern "C" fn Servo_InitStyleSet() -> *mut RawServoStyleSet {
    let data = Box::new(PerDocumentStyleData::new());
    Box::into_raw(data) as *mut RawServoStyleSet
}

#[no_mangle]
pub extern "C" fn Servo_DropStyleSet(data: *mut RawServoStyleSet) -> () {
    unsafe {
        let _ = Box::<PerDocumentStyleData>::from_raw(data as *mut PerDocumentStyleData);
    }
}

pub struct GeckoDeclarationBlock {
    pub declarations: Option<PropertyDeclarationBlock>,
    pub cache: *mut nsHTMLCSSStyleSheet,
    pub immutable: bool,
}

#[no_mangle]
pub extern "C" fn Servo_ParseStyleAttribute(bytes: *const u8, length: u32,
                                            cache: *mut nsHTMLCSSStyleSheet)
                                            -> *mut ServoDeclarationBlock {
    let value = unsafe { from_utf8_unchecked(slice::from_raw_parts(bytes, length as usize)) };
    let declarations = Box::new(GeckoDeclarationBlock {
        declarations: GeckoElement::parse_style_attribute(value),
        cache: cache,
        immutable: false,
    });
    Box::into_raw(declarations) as *mut ServoDeclarationBlock
}

#[no_mangle]
pub extern "C" fn Servo_DropDeclarationBlock(declarations: *mut ServoDeclarationBlock) {
    unsafe {
        let _ = Box::<GeckoDeclarationBlock>::from_raw(declarations as *mut GeckoDeclarationBlock);
    }
}

#[no_mangle]
pub extern "C" fn Servo_GetDeclarationBlockCache(declarations: *mut ServoDeclarationBlock)
                                                 -> *mut nsHTMLCSSStyleSheet {
    let declarations = unsafe { (declarations as *const GeckoDeclarationBlock).as_ref().unwrap() };
    declarations.cache
}

#[no_mangle]
pub extern "C" fn Servo_SetDeclarationBlockImmutable(declarations: *mut ServoDeclarationBlock) {
    let declarations = unsafe { (declarations as *mut GeckoDeclarationBlock).as_mut().unwrap() };
    declarations.immutable = true;
}

#[no_mangle]
pub extern "C" fn Servo_ClearDeclarationBlockCachePointer(declarations: *mut ServoDeclarationBlock) {
    let declarations = unsafe { (declarations as *mut GeckoDeclarationBlock).as_mut().unwrap() };
    declarations.cache = ptr::null_mut();
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
pub extern "C" fn Servo_ComputeRestyleHint(element: *mut RawGeckoElement,
                                           snapshot: *mut ServoElementSnapshot,
                                           raw_data: *mut RawServoStyleSet) -> nsRestyleHint {
    let per_doc_data = unsafe { &mut *(raw_data as *mut PerDocumentStyleData) };
    let snapshot = unsafe { GeckoElementSnapshot::from_raw(snapshot) };
    let element = unsafe { GeckoElement::from_raw(element) };

    // NB: This involves an FFI call, we can get rid of it easily if needed.
    let current_state = element.get_state();

    let hint = per_doc_data.stylist
                           .compute_restyle_hint(&element, &snapshot,
                                                 current_state);

    // NB: Binary representations match.
    unsafe { transmute(hint.bits() as u32) }
}
