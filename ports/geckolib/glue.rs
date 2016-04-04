/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use app_units::Au;
use bindings::{RawGeckoDocument, RawGeckoElement};
use bindings::{RawServoStyleSet, RawServoStyleSheet, ServoComputedValues, ServoNodeData};
use bindings::{nsIAtom};
use data::PerDocumentStyleData;
use euclid::Size2D;
use properties::GeckoComputedValues;
use selector_impl::{SharedStyleContext, Stylesheet};
use std::marker::PhantomData;
use std::mem::{forget, transmute};
use std::ptr;
use std::slice;
use std::str::from_utf8_unchecked;
use std::sync::{Arc, Mutex};
use style::context::{ReflowGoal, StylistWrapper};
use style::dom::{TDocument, TElement, TNode};
use style::error_reporting::StdoutErrorReporter;
use style::parallel;
use style::properties::ComputedValues;
use style::stylesheets::Origin;
use traversal::RecalcStyleOnly;
use url::Url;
use util::arc_ptr_eq;
use wrapper::{GeckoDocument, GeckoElement, GeckoNode, NonOpaqueStyleData};

/*
 * For Gecko->Servo function calls, we need to redeclare the same signature that was declared in
 * the C header in Gecko. In order to catch accidental mismatches, we run rust-bindgen against
 * those signatures as well, giving us a second declaration of all the Servo_* functions in this
 * crate. If there's a mismatch, LLVM will assert and abort, which is a rather awful thing to
 * depend on but good enough for our purposes.
 */

#[no_mangle]
pub extern "C" fn Servo_RestyleDocument(doc: *mut RawGeckoDocument, raw_data: *mut RawServoStyleSet) -> () {
    let document = unsafe { GeckoDocument::from_raw(doc) };
    let node = match document.root_node() {
        Some(x) => x,
        None => return,
    };
    let data = unsafe { &mut *(raw_data as *mut PerDocumentStyleData) };

    // Force the creation of our lazily-constructed initial computed values on
    // the main thread, since it's not safe to call elsewhere. This should move
    // into a runtime-wide init hook at some point.
    GeckoComputedValues::initial_values();

    let _needs_dirtying = data.stylist.update(&data.stylesheets, data.stylesheets_changed);
    data.stylesheets_changed = false;

    let shared_style_context = SharedStyleContext {
        viewport_size: Size2D::new(Au(0), Au(0)),
        screen_size_changed: false,
        generation: 0,
        goal: ReflowGoal::ForScriptQuery,
        stylist: StylistWrapper(&data.stylist),
        new_animations_sender: Mutex::new(data.new_animations_sender.clone()),
        running_animations: data.running_animations.clone(),
        expired_animations: data.expired_animations.clone(),
        error_reporter: Box::new(StdoutErrorReporter),
    };

    if node.is_dirty() || node.has_dirty_descendants() {
        parallel::traverse_dom::<GeckoNode, RecalcStyleOnly>(node, &shared_style_context, &mut data.work_queue);
    }
}

#[no_mangle]
pub extern "C" fn Servo_DropNodeData(data: *mut ServoNodeData) -> () {
    unsafe {
        let _ = Box::<NonOpaqueStyleData>::from_raw(data as *mut NonOpaqueStyleData);
    }
}

#[no_mangle]
pub extern "C" fn Servo_StylesheetFromUTF8Bytes(bytes: *const u8,
                                                length: u32) -> *mut RawServoStyleSheet {

    let input = unsafe { from_utf8_unchecked(slice::from_raw_parts(bytes, length as usize)) };

    // FIXME(heycam): Pass in the real base URL and sheet origin to use.
    let url = Url::parse("about:none").unwrap();
    let sheet = Arc::new(Stylesheet::from_str(input, url, Origin::Author, Box::new(StdoutErrorReporter)));
    unsafe {
        transmute(sheet)
    }
}

struct ArcHelpers<GeckoType, ServoType> {
    phantom1: PhantomData<GeckoType>,
    phantom2: PhantomData<ServoType>,
}

impl<GeckoType, ServoType> ArcHelpers<GeckoType, ServoType> {
    fn with<F, Output>(raw: *mut GeckoType, cb: F) -> Output
                       where F: FnOnce(&Arc<ServoType>) -> Output {
        let owned = unsafe { Self::into(raw) };
        let result = cb(&owned);
        forget(owned);
        result
    }

    unsafe fn into(ptr: *mut GeckoType) -> Arc<ServoType> {
        transmute(ptr)
    }

    unsafe fn addref(ptr: *mut GeckoType) {
        Self::with(ptr, |arc| forget(arc.clone()));
    }

    unsafe fn release(ptr: *mut GeckoType) {
        let _ = Self::into(ptr);
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
pub extern "C" fn Servo_GetComputedValues(element: *mut RawGeckoElement)
     -> *mut ServoComputedValues {
    let node = unsafe { GeckoElement::from_raw(element).as_node() };
    let arc_cv = node.borrow_data().map(|data| data.style.clone());
    arc_cv.map_or(ptr::null_mut(), |arc| unsafe { transmute(arc) })
}

#[no_mangle]
pub extern "C" fn Servo_GetComputedValuesForAnonymousBox(_parentStyleOrNull: *mut ServoComputedValues,
                                                         _pseudoTag: *mut nsIAtom)
     -> *mut ServoComputedValues {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn Servo_AddRefComputedValues(ptr: *mut ServoComputedValues) -> () {
    type Helpers = ArcHelpers<ServoComputedValues, GeckoComputedValues>;
    unsafe { Helpers::addref(ptr) };
}

#[no_mangle]
pub extern "C" fn Servo_ReleaseComputedValues(ptr: *mut ServoComputedValues) -> () {
    type Helpers = ArcHelpers<ServoComputedValues, GeckoComputedValues>;
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
