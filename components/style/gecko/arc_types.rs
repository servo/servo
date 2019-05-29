/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This file lists all arc FFI types and defines corresponding addref
//! and release functions. This list corresponds to ServoArcTypeList.h
//! file in Gecko.

#![allow(non_snake_case, missing_docs)]

use crate::gecko::url::CssUrlData;
use crate::gecko_bindings::structs::RawServoAnimationValue;
use crate::gecko_bindings::structs::RawServoCounterStyleRule;
use crate::gecko_bindings::structs::RawServoCssUrlData;
use crate::gecko_bindings::structs::RawServoDeclarationBlock;
use crate::gecko_bindings::structs::RawServoFontFaceRule;
use crate::gecko_bindings::structs::RawServoFontFeatureValuesRule;
use crate::gecko_bindings::structs::RawServoImportRule;
use crate::gecko_bindings::structs::RawServoKeyframe;
use crate::gecko_bindings::structs::RawServoKeyframesRule;
use crate::gecko_bindings::structs::RawServoMediaList;
use crate::gecko_bindings::structs::RawServoMediaRule;
use crate::gecko_bindings::structs::RawServoMozDocumentRule;
use crate::gecko_bindings::structs::RawServoNamespaceRule;
use crate::gecko_bindings::structs::RawServoPageRule;
use crate::gecko_bindings::structs::RawServoStyleRule;
use crate::gecko_bindings::structs::RawServoStyleSheetContents;
use crate::gecko_bindings::structs::RawServoSupportsRule;
use crate::gecko_bindings::structs::ServoCssRules;
use crate::gecko_bindings::sugar::ownership::{HasArcFFI, HasFFI, Strong};
use crate::media_queries::MediaList;
use crate::properties::animated_properties::AnimationValue;
use crate::properties::{ComputedValues, PropertyDeclarationBlock};
use crate::shared_lock::Locked;
use crate::stylesheets::keyframes_rule::Keyframe;
use crate::stylesheets::{CounterStyleRule, CssRules, FontFaceRule, FontFeatureValuesRule};
use crate::stylesheets::{DocumentRule, ImportRule, KeyframesRule, MediaRule};
use crate::stylesheets::{NamespaceRule, PageRule};
use crate::stylesheets::{StyleRule, StylesheetContents, SupportsRule};
use servo_arc::{Arc, ArcBorrow};
use std::{mem, ptr};

macro_rules! impl_arc_ffi {
    ($servo_type:ty => $gecko_type:ty[$addref:ident, $release:ident]) => {
        unsafe impl HasFFI for $servo_type {
            type FFIType = $gecko_type;
        }
        unsafe impl HasArcFFI for $servo_type {}

        #[no_mangle]
        pub unsafe extern "C" fn $addref(obj: &$gecko_type) {
            <$servo_type>::addref(obj);
        }

        #[no_mangle]
        pub unsafe extern "C" fn $release(obj: &$gecko_type) {
            <$servo_type>::release(obj);
        }
    };
}

impl_arc_ffi!(Locked<CssRules> => ServoCssRules
              [Servo_CssRules_AddRef, Servo_CssRules_Release]);

impl_arc_ffi!(StylesheetContents => RawServoStyleSheetContents
              [Servo_StyleSheetContents_AddRef, Servo_StyleSheetContents_Release]);

impl_arc_ffi!(Locked<PropertyDeclarationBlock> => RawServoDeclarationBlock
              [Servo_DeclarationBlock_AddRef, Servo_DeclarationBlock_Release]);

impl_arc_ffi!(Locked<StyleRule> => RawServoStyleRule
              [Servo_StyleRule_AddRef, Servo_StyleRule_Release]);

impl_arc_ffi!(Locked<ImportRule> => RawServoImportRule
              [Servo_ImportRule_AddRef, Servo_ImportRule_Release]);

impl_arc_ffi!(AnimationValue => RawServoAnimationValue
              [Servo_AnimationValue_AddRef, Servo_AnimationValue_Release]);

impl_arc_ffi!(Locked<Keyframe> => RawServoKeyframe
              [Servo_Keyframe_AddRef, Servo_Keyframe_Release]);

impl_arc_ffi!(Locked<KeyframesRule> => RawServoKeyframesRule
              [Servo_KeyframesRule_AddRef, Servo_KeyframesRule_Release]);

impl_arc_ffi!(Locked<MediaList> => RawServoMediaList
              [Servo_MediaList_AddRef, Servo_MediaList_Release]);

impl_arc_ffi!(Locked<MediaRule> => RawServoMediaRule
              [Servo_MediaRule_AddRef, Servo_MediaRule_Release]);

impl_arc_ffi!(Locked<NamespaceRule> => RawServoNamespaceRule
              [Servo_NamespaceRule_AddRef, Servo_NamespaceRule_Release]);

impl_arc_ffi!(Locked<PageRule> => RawServoPageRule
              [Servo_PageRule_AddRef, Servo_PageRule_Release]);

impl_arc_ffi!(Locked<SupportsRule> => RawServoSupportsRule
              [Servo_SupportsRule_AddRef, Servo_SupportsRule_Release]);

impl_arc_ffi!(Locked<DocumentRule> => RawServoMozDocumentRule
              [Servo_DocumentRule_AddRef, Servo_DocumentRule_Release]);

impl_arc_ffi!(Locked<FontFeatureValuesRule> => RawServoFontFeatureValuesRule
              [Servo_FontFeatureValuesRule_AddRef, Servo_FontFeatureValuesRule_Release]);

impl_arc_ffi!(Locked<FontFaceRule> => RawServoFontFaceRule
              [Servo_FontFaceRule_AddRef, Servo_FontFaceRule_Release]);

impl_arc_ffi!(Locked<CounterStyleRule> => RawServoCounterStyleRule
              [Servo_CounterStyleRule_AddRef, Servo_CounterStyleRule_Release]);

impl_arc_ffi!(CssUrlData => RawServoCssUrlData
              [Servo_CssUrlData_AddRef, Servo_CssUrlData_Release]);

// ComputedStyle is not an opaque type on any side of FFI.
// This means that doing the HasArcFFI type trick is actually unsound,
// since it gives us a way to construct an Arc<ComputedStyle> from
// an &ComputedStyle, which in general is not allowed. So we
// implement the restricted set of arc type functionality we need.

#[no_mangle]
pub unsafe extern "C" fn Servo_ComputedStyle_AddRef(obj: &ComputedValues) {
    mem::forget(ArcBorrow::from_ref(obj).clone_arc());
}

#[no_mangle]
pub unsafe extern "C" fn Servo_ComputedStyle_Release(obj: &ComputedValues) {
    ArcBorrow::from_ref(obj).with_arc(|a: &Arc<ComputedValues>| {
        let _: Arc<ComputedValues> = ptr::read(a);
    });
}

impl From<Arc<ComputedValues>> for Strong<ComputedValues> {
    fn from(arc: Arc<ComputedValues>) -> Self {
        unsafe { mem::transmute(Arc::into_raw_offset(arc)) }
    }
}
