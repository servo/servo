/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This file lists all arc FFI types and defines corresponding addref
//! and release functions. This list corresponds to ServoArcTypeList.h
//! file in Gecko.

#![allow(non_snake_case, missing_docs)]

use gecko_bindings::bindings::{RawServoMediaList, RawServoMediaRule, RawServoNamespaceRule, RawServoPageRule};
use gecko_bindings::bindings::{RawServoStyleSheet, RawServoImportRule, RawServoSupportsRule};
use gecko_bindings::bindings::{ServoComputedValues, ServoCssRules};
use gecko_bindings::structs::{RawServoDeclarationBlock, RawServoStyleRule};
use gecko_bindings::structs::RawServoAnimationValue;
use gecko_bindings::sugar::ownership::{HasArcFFI, HasFFI};
use media_queries::MediaList;
use properties::{ComputedValues, PropertyDeclarationBlock};
use properties::animated_properties::AnimationValue;
use shared_lock::Locked;
use stylesheets::{CssRules, Stylesheet, StyleRule, ImportRule, MediaRule};
use stylesheets::{NamespaceRule, PageRule, SupportsRule};

macro_rules! impl_arc_ffi {
    ($servo_type:ty => $gecko_type:ty [$addref:ident, $release:ident]) => {
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
    }
}

impl_arc_ffi!(Locked<CssRules> => ServoCssRules
              [Servo_CssRules_AddRef, Servo_CssRules_Release]);

impl_arc_ffi!(Stylesheet => RawServoStyleSheet
              [Servo_StyleSheet_AddRef, Servo_StyleSheet_Release]);

impl_arc_ffi!(ComputedValues => ServoComputedValues
              [Servo_ComputedValues_AddRef, Servo_ComputedValues_Release]);

impl_arc_ffi!(Locked<PropertyDeclarationBlock> => RawServoDeclarationBlock
              [Servo_DeclarationBlock_AddRef, Servo_DeclarationBlock_Release]);

impl_arc_ffi!(Locked<StyleRule> => RawServoStyleRule
              [Servo_StyleRule_AddRef, Servo_StyleRule_Release]);

impl_arc_ffi!(Locked<ImportRule> => RawServoImportRule
              [Servo_ImportRule_AddRef, Servo_ImportRule_Release]);

impl_arc_ffi!(AnimationValue => RawServoAnimationValue
              [Servo_AnimationValue_AddRef, Servo_AnimationValue_Release]);

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
