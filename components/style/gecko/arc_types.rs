/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This file lists all arc FFI types and defines corresponding addref
//! and release functions. This list corresponds to ServoArcTypeList.h
//! file in Gecko.

#![allow(non_snake_case, missing_docs)]

use gecko_bindings::bindings::{RawServoImportRule, RawServoSupportsRule};
use gecko_bindings::bindings::{RawServoKeyframe, RawServoKeyframesRule};
use gecko_bindings::bindings::{RawServoMediaRule, RawServoNamespaceRule, RawServoPageRule};
use gecko_bindings::bindings::{RawServoRuleNode, RawServoRuleNodeStrong, RawServoDocumentRule};
use gecko_bindings::bindings::ServoCssRules;
use gecko_bindings::structs::{RawServoAnimationValue, RawServoDeclarationBlock, RawServoStyleRule, RawServoMediaList};
use gecko_bindings::structs::{RawServoStyleSheetContents, ServoStyleContext};
use gecko_bindings::sugar::ownership::{HasArcFFI, HasFFI};
use media_queries::MediaList;
use properties::{ComputedValues, PropertyDeclarationBlock};
use properties::animated_properties::AnimationValue;
use rule_tree::StrongRuleNode;
use shared_lock::Locked;
use std::{mem, ptr};
use stylesheets::{CssRules, StylesheetContents, StyleRule, ImportRule, KeyframesRule, MediaRule};
use stylesheets::{NamespaceRule, PageRule, SupportsRule, DocumentRule};
use stylesheets::keyframes_rule::Keyframe;

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

impl_arc_ffi!(StylesheetContents => RawServoStyleSheetContents
              [Servo_StyleSheetContents_AddRef, Servo_StyleSheetContents_Release]);

impl_arc_ffi!(ComputedValues => ServoStyleContext
              [Servo_StyleContext_AddRef, Servo_StyleContext_Release]);

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

impl_arc_ffi!(Locked<DocumentRule> => RawServoDocumentRule
              [Servo_DocumentRule_AddRef, Servo_DocumentRule_Release]);

// RuleNode is a Arc-like type but it does not use Arc.

impl StrongRuleNode {
    pub fn into_strong(self) -> RawServoRuleNodeStrong {
        let ptr = self.ptr();
        mem::forget(self);
        unsafe { mem::transmute(ptr) }
    }

    pub fn from_ffi<'a>(ffi: &'a &RawServoRuleNode) -> &'a Self {
        unsafe { &*(ffi as *const &RawServoRuleNode as *const StrongRuleNode) }
    }
}

#[no_mangle]
pub unsafe extern "C" fn Servo_RuleNode_AddRef(obj: &RawServoRuleNode) {
    mem::forget(StrongRuleNode::from_ffi(&obj).clone());
}

#[no_mangle]
pub unsafe extern "C" fn Servo_RuleNode_Release(obj: &RawServoRuleNode) {
    let ptr = StrongRuleNode::from_ffi(&obj);
    ptr::read(ptr as *const StrongRuleNode);
}
