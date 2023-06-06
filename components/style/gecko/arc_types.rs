/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This file lists all arc FFI types and defines corresponding addref and release functions. This
//! list loosely corresponds to ServoLockedArcTypeList.h file in Gecko.

#![allow(non_snake_case, missing_docs)]

use crate::gecko::url::CssUrlData;
use crate::media_queries::MediaList;
use crate::properties::animated_properties::AnimationValue;
use crate::properties::{ComputedValues, PropertyDeclarationBlock};
use crate::shared_lock::Locked;
use crate::stylesheets::keyframes_rule::Keyframe;
use crate::stylesheets::{
    ContainerRule, CounterStyleRule, CssRules, DocumentRule, FontFaceRule, FontFeatureValuesRule,
    FontPaletteValuesRule, ImportRule, KeyframesRule, LayerBlockRule, LayerStatementRule,
    MediaRule, NamespaceRule, PageRule, PropertyRule, StyleRule, StylesheetContents, SupportsRule,
};
use servo_arc::Arc;

macro_rules! impl_simple_arc_ffi {
    ($ty:ty, $addref:ident, $release:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $addref(obj: *const $ty) {
            std::mem::forget(Arc::from_raw_addrefed(obj));
        }

        #[no_mangle]
        pub unsafe extern "C" fn $release(obj: *const $ty) {
            let _ = Arc::from_raw(obj);
        }
    };
}

macro_rules! impl_locked_arc_ffi {
    ($servo_type:ty, $alias:ident, $addref:ident, $release:ident) => {
        /// A simple alias for a locked type.
        pub type $alias = Locked<$servo_type>;
        impl_simple_arc_ffi!($alias, $addref, $release);
    };
}

impl_locked_arc_ffi!(
    CssRules,
    LockedCssRules,
    Servo_CssRules_AddRef,
    Servo_CssRules_Release
);
impl_locked_arc_ffi!(
    PropertyDeclarationBlock,
    LockedDeclarationBlock,
    Servo_DeclarationBlock_AddRef,
    Servo_DeclarationBlock_Release
);
impl_locked_arc_ffi!(
    StyleRule,
    LockedStyleRule,
    Servo_StyleRule_AddRef,
    Servo_StyleRule_Release
);
impl_locked_arc_ffi!(
    ImportRule,
    LockedImportRule,
    Servo_ImportRule_AddRef,
    Servo_ImportRule_Release
);
impl_locked_arc_ffi!(
    Keyframe,
    LockedKeyframe,
    Servo_Keyframe_AddRef,
    Servo_Keyframe_Release
);
impl_locked_arc_ffi!(
    KeyframesRule,
    LockedKeyframesRule,
    Servo_KeyframesRule_AddRef,
    Servo_KeyframesRule_Release
);
impl_simple_arc_ffi!(
    LayerBlockRule,
    Servo_LayerBlockRule_AddRef,
    Servo_LayerBlockRule_Release
);
impl_simple_arc_ffi!(
    LayerStatementRule,
    Servo_LayerStatementRule_AddRef,
    Servo_LayerStatementRule_Release
);
impl_locked_arc_ffi!(
    MediaList,
    LockedMediaList,
    Servo_MediaList_AddRef,
    Servo_MediaList_Release
);
impl_simple_arc_ffi!(MediaRule, Servo_MediaRule_AddRef, Servo_MediaRule_Release);
impl_simple_arc_ffi!(
    NamespaceRule,
    Servo_NamespaceRule_AddRef,
    Servo_NamespaceRule_Release
);
impl_locked_arc_ffi!(
    PageRule,
    LockedPageRule,
    Servo_PageRule_AddRef,
    Servo_PageRule_Release
);
impl_simple_arc_ffi!(
    PropertyRule,
    Servo_PropertyRule_AddRef,
    Servo_PropertyRule_Release
);
impl_simple_arc_ffi!(
    SupportsRule,
    Servo_SupportsRule_AddRef,
    Servo_SupportsRule_Release
);
impl_simple_arc_ffi!(
    ContainerRule,
    Servo_ContainerRule_AddRef,
    Servo_ContainerRule_Release
);
impl_simple_arc_ffi!(
    DocumentRule,
    Servo_DocumentRule_AddRef,
    Servo_DocumentRule_Release
);
impl_simple_arc_ffi!(
    FontFeatureValuesRule,
    Servo_FontFeatureValuesRule_AddRef,
    Servo_FontFeatureValuesRule_Release
);
impl_simple_arc_ffi!(
    FontPaletteValuesRule,
    Servo_FontPaletteValuesRule_AddRef,
    Servo_FontPaletteValuesRule_Release
);
impl_locked_arc_ffi!(
    FontFaceRule,
    LockedFontFaceRule,
    Servo_FontFaceRule_AddRef,
    Servo_FontFaceRule_Release
);
impl_locked_arc_ffi!(
    CounterStyleRule,
    LockedCounterStyleRule,
    Servo_CounterStyleRule_AddRef,
    Servo_CounterStyleRule_Release
);

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
impl_simple_arc_ffi!(
    AnimationValue,
    Servo_AnimationValue_AddRef,
    Servo_AnimationValue_Release
);
