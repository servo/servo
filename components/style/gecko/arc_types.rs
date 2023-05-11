/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This file lists all arc FFI types and defines corresponding addref
//! and release functions. This list corresponds to ServoArcTypeList.h
//! file in Gecko.

#![allow(non_snake_case, missing_docs)]

use crate::gecko::url::CssUrlData;
use crate::gecko_bindings::structs::{
    RawServoAnimationValue, RawServoContainerRule, RawServoCounterStyleRule,
    RawServoDeclarationBlock, RawServoFontFaceRule, RawServoFontFeatureValuesRule,
    RawServoFontPaletteValuesRule, RawServoImportRule, RawServoKeyframe, RawServoKeyframesRule,
    RawServoLayerBlockRule, RawServoLayerStatementRule, RawServoMediaList, RawServoMediaRule,
    RawServoMozDocumentRule, RawServoNamespaceRule, RawServoPageRule, RawServoStyleRule,
    RawServoSupportsRule, ServoCssRules,
};
use crate::gecko_bindings::sugar::ownership::HasArcFFI;
use crate::media_queries::MediaList;
use crate::properties::animated_properties::AnimationValue;
use crate::properties::{ComputedValues, PropertyDeclarationBlock};
use crate::shared_lock::Locked;
use crate::stylesheets::keyframes_rule::Keyframe;
use crate::stylesheets::{
    ContainerRule, CounterStyleRule, CssRules, DocumentRule, FontFaceRule, FontFeatureValuesRule,
    FontPaletteValuesRule, ImportRule, KeyframesRule, LayerBlockRule, LayerStatementRule,
    MediaRule, NamespaceRule, PageRule, StyleRule, StylesheetContents, SupportsRule,
};
use servo_arc::Arc;

macro_rules! impl_arc_ffi {
    ($servo_type:ty => $gecko_type:ty[$addref:ident, $release:ident]) => {
        unsafe impl HasArcFFI for $servo_type {
            type FFIType = $gecko_type;
        }

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

impl_arc_ffi!(Locked<LayerBlockRule> => RawServoLayerBlockRule
              [Servo_LayerBlockRule_AddRef, Servo_LayerBlockRule_Release]);

impl_arc_ffi!(Locked<LayerStatementRule> => RawServoLayerStatementRule
              [Servo_LayerStatementRule_AddRef, Servo_LayerStatementRule_Release]);

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

impl_arc_ffi!(Locked<ContainerRule> => RawServoContainerRule
              [Servo_ContainerRule_AddRef, Servo_ContainerRule_Release]);

impl_arc_ffi!(Locked<DocumentRule> => RawServoMozDocumentRule
              [Servo_DocumentRule_AddRef, Servo_DocumentRule_Release]);

impl_arc_ffi!(Locked<FontFeatureValuesRule> => RawServoFontFeatureValuesRule
              [Servo_FontFeatureValuesRule_AddRef, Servo_FontFeatureValuesRule_Release]);

impl_arc_ffi!(Locked<FontPaletteValuesRule> => RawServoFontPaletteValuesRule
              [Servo_FontPaletteValuesRule_AddRef, Servo_FontPaletteValuesRule_Release]);

impl_arc_ffi!(Locked<FontFaceRule> => RawServoFontFaceRule
              [Servo_FontFaceRule_AddRef, Servo_FontFaceRule_Release]);

impl_arc_ffi!(Locked<CounterStyleRule> => RawServoCounterStyleRule
              [Servo_CounterStyleRule_AddRef, Servo_CounterStyleRule_Release]);

macro_rules! impl_simple_arc_ffi {
    ($ty:ty, $addref:ident, $release:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $addref(obj: &$ty) {
            std::mem::forget(Arc::from_raw_addrefed(obj));
        }

        #[no_mangle]
        pub unsafe extern "C" fn $release(obj: &$ty) {
            let _ = Arc::from_raw(obj);
        }
    };
}

impl_simple_arc_ffi!(
    StylesheetContents,
    Servo_StyleSheetContents_AddRef,
    Servo_StyleSheetContents_Release
);
impl_simple_arc_ffi!(
    CssUrlData,
    Servo_CssUrlData_AddRef,
    Servo_CssUrlData_Release
);
impl_simple_arc_ffi!(
    ComputedValues,
    Servo_ComputedStyle_AddRef,
    Servo_ComputedStyle_Release
);
