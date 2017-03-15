/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This file lists all arc FFI types and defines corresponding addref
//! and release functions. This list corresponds to ServoArcTypeList.h
//! file in Gecko.

#![allow(non_snake_case, missing_docs)]

use gecko_bindings::bindings::{RawServoMediaList, RawServoMediaRule, RawServoNamespaceRule};
use gecko_bindings::bindings::{RawServoStyleSheet, RawServoStyleRule, RawServoImportRule};
use gecko_bindings::bindings::{ServoComputedValues, ServoCssRules};
use gecko_bindings::structs::{RawServoAnimationValue, RawServoDeclarationBlock};
use gecko_bindings::sugar::ownership::{HasArcFFI, HasFFI};
use media_queries::MediaList;
use parking_lot::RwLock;
use properties::{ComputedValues, PropertyDeclarationBlock};
use properties::animated_properties::AnimationValue;
use stylesheets::{CssRules, Stylesheet, StyleRule, ImportRule, MediaRule, NamespaceRule};

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

impl_arc_ffi!(RwLock<CssRules> => ServoCssRules
              [Servo_CssRules_AddRef, Servo_CssRules_Release]);

impl_arc_ffi!(Stylesheet => RawServoStyleSheet
              [Servo_StyleSheet_AddRef, Servo_StyleSheet_Release]);

impl_arc_ffi!(ComputedValues => ServoComputedValues
              [Servo_ComputedValues_AddRef, Servo_ComputedValues_Release]);

impl_arc_ffi!(RwLock<PropertyDeclarationBlock> => RawServoDeclarationBlock
              [Servo_DeclarationBlock_AddRef, Servo_DeclarationBlock_Release]);

impl_arc_ffi!(RwLock<StyleRule> => RawServoStyleRule
              [Servo_StyleRule_AddRef, Servo_StyleRule_Release]);

impl_arc_ffi!(RwLock<ImportRule> => RawServoImportRule
              [Servo_ImportRule_AddRef, Servo_ImportRule_Release]);

impl_arc_ffi!(AnimationValue => RawServoAnimationValue
              [Servo_AnimationValue_AddRef, Servo_AnimationValue_Release]);

impl_arc_ffi!(RwLock<MediaList> => RawServoMediaList
              [Servo_MediaList_AddRef, Servo_MediaList_Release]);

impl_arc_ffi!(RwLock<MediaRule> => RawServoMediaRule
              [Servo_MediaRule_AddRef, Servo_MediaRule_Release]);

impl_arc_ffi!(RwLock<NamespaceRule> => RawServoNamespaceRule
              [Servo_NamespaceRule_AddRef, Servo_NamespaceRule_Release]);
