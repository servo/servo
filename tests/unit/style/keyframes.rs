/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use parking_lot::RwLock;
use std::sync::Arc;
use style::keyframes::{Keyframe, KeyframesAnimation, KeyframePercentage,  KeyframeSelector};
use style::keyframes::{KeyframesStep, KeyframesStepValue};
use style::properties::{DeclaredValue, PropertyDeclaration, PropertyDeclarationBlock, Importance};
use style::properties::animated_properties::TransitionProperty;
use style::values::specified::{LengthOrPercentageOrAuto, NoCalcLength};

#[test]
fn test_empty_keyframe() {
    let keyframes = vec![];
    let animation = KeyframesAnimation::from_keyframes(&keyframes);
    let expected = KeyframesAnimation {
        steps: vec![],
        properties_changed: vec![],
    };

    assert_eq!(format!("{:#?}", animation), format!("{:#?}", expected));
}

#[test]
fn test_no_property_in_keyframe() {
    let keyframes = vec![
        Arc::new(RwLock::new(Keyframe {
            selector: KeyframeSelector::new_for_unit_testing(vec![KeyframePercentage::new(1.)]),
            block: Arc::new(RwLock::new(PropertyDeclarationBlock {
                declarations: vec![],
                important_count: 0,
            }))
        })),
    ];
    let animation = KeyframesAnimation::from_keyframes(&keyframes);
    let expected = KeyframesAnimation {
        steps: vec![],
        properties_changed: vec![],
    };

    assert_eq!(format!("{:#?}", animation), format!("{:#?}", expected));
}

#[test]
fn test_missing_property_in_initial_keyframe() {
    let declarations_on_initial_keyframe =
        Arc::new(RwLock::new(PropertyDeclarationBlock {
            declarations: vec![
                (PropertyDeclaration::Width(
                    DeclaredValue::Value(LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(20f32)))),
                 Importance::Normal),
            ],
            important_count: 0,
        }));

    let declarations_on_final_keyframe =
        Arc::new(RwLock::new(PropertyDeclarationBlock {
            declarations: vec![
                (PropertyDeclaration::Width(
                    DeclaredValue::Value(LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(20f32)))),
                 Importance::Normal),

                (PropertyDeclaration::Height(
                    DeclaredValue::Value(LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(20f32)))),
                 Importance::Normal),
            ],
            important_count: 0,
        }));

    let keyframes = vec![
        Arc::new(RwLock::new(Keyframe {
            selector: KeyframeSelector::new_for_unit_testing(vec![KeyframePercentage::new(0.)]),
            block: declarations_on_initial_keyframe.clone(),
        })),

        Arc::new(RwLock::new(Keyframe {
            selector: KeyframeSelector::new_for_unit_testing(vec![KeyframePercentage::new(1.)]),
            block: declarations_on_final_keyframe.clone(),
        })),
    ];
    let animation = KeyframesAnimation::from_keyframes(&keyframes);
    let expected = KeyframesAnimation {
        steps: vec![
            KeyframesStep {
                start_percentage: KeyframePercentage(0.),
                value: KeyframesStepValue::Declarations { block: declarations_on_initial_keyframe },
                declared_timing_function: false,
            },
            KeyframesStep {
                start_percentage: KeyframePercentage(1.),
                value: KeyframesStepValue::Declarations { block: declarations_on_final_keyframe },
                declared_timing_function: false,
            },
        ],
        properties_changed: vec![TransitionProperty::Width, TransitionProperty::Height],
    };

    assert_eq!(format!("{:#?}", animation), format!("{:#?}", expected));
}

#[test]
fn test_missing_property_in_final_keyframe() {
    let declarations_on_initial_keyframe =
        Arc::new(RwLock::new(PropertyDeclarationBlock {
            declarations: vec![
                (PropertyDeclaration::Width(
                    DeclaredValue::Value(LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(20f32)))),
                 Importance::Normal),

                (PropertyDeclaration::Height(
                    DeclaredValue::Value(LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(20f32)))),
                 Importance::Normal),
            ],
            important_count: 0,
        }));

    let declarations_on_final_keyframe =
        Arc::new(RwLock::new(PropertyDeclarationBlock {
            declarations: vec![
                (PropertyDeclaration::Height(
                    DeclaredValue::Value(LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(20f32)))),
                 Importance::Normal),
            ],
            important_count: 0,
        }));

    let keyframes = vec![
        Arc::new(RwLock::new(Keyframe {
            selector: KeyframeSelector::new_for_unit_testing(vec![KeyframePercentage::new(0.)]),
            block: declarations_on_initial_keyframe.clone(),
        })),

        Arc::new(RwLock::new(Keyframe {
            selector: KeyframeSelector::new_for_unit_testing(vec![KeyframePercentage::new(1.)]),
            block: declarations_on_final_keyframe.clone(),
        })),
    ];
    let animation = KeyframesAnimation::from_keyframes(&keyframes);
    let expected = KeyframesAnimation {
        steps: vec![
            KeyframesStep {
                start_percentage: KeyframePercentage(0.),
                value: KeyframesStepValue::Declarations { block: declarations_on_initial_keyframe },
                declared_timing_function: false,
            },
            KeyframesStep {
                start_percentage: KeyframePercentage(1.),
                value: KeyframesStepValue::Declarations { block: declarations_on_final_keyframe },
                declared_timing_function: false,
            },
        ],
        properties_changed: vec![TransitionProperty::Width, TransitionProperty::Height],
    };

    assert_eq!(format!("{:#?}", animation), format!("{:#?}", expected));
}

#[test]
fn test_missing_keyframe_in_both_of_initial_and_final_keyframe() {
    let declarations =
        Arc::new(RwLock::new(PropertyDeclarationBlock {
            declarations: vec![
                (PropertyDeclaration::Width(
                        DeclaredValue::Value(LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(20f32)))),
                 Importance::Normal),

                (PropertyDeclaration::Height(
                        DeclaredValue::Value(LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(20f32)))),
                Importance::Normal),
            ],
            important_count: 0,
        }));

    let keyframes = vec![
        Arc::new(RwLock::new(Keyframe {
            selector: KeyframeSelector::new_for_unit_testing(vec![KeyframePercentage::new(0.)]),
            block: Arc::new(RwLock::new(PropertyDeclarationBlock {
                declarations: vec![],
                important_count: 0,
            }))
        })),
        Arc::new(RwLock::new(Keyframe {
            selector: KeyframeSelector::new_for_unit_testing(vec![KeyframePercentage::new(0.5)]),
            block: declarations.clone(),
        })),
    ];
    let animation = KeyframesAnimation::from_keyframes(&keyframes);
    let expected = KeyframesAnimation {
        steps: vec![
            KeyframesStep {
                start_percentage: KeyframePercentage(0.),
                value: KeyframesStepValue::Declarations {
                    block: Arc::new(RwLock::new(PropertyDeclarationBlock {
                        // XXX: Should we use ComputedValues in this case?
                        declarations: vec![],
                        important_count: 0,
                    }))
                },
                declared_timing_function: false,
            },
            KeyframesStep {
                start_percentage: KeyframePercentage(0.5),
                value: KeyframesStepValue::Declarations { block: declarations },
                declared_timing_function: false,
            },
            KeyframesStep {
                start_percentage: KeyframePercentage(1.),
                value: KeyframesStepValue::ComputedValues,
                declared_timing_function: false,
            }
        ],
        properties_changed: vec![TransitionProperty::Width, TransitionProperty::Height],
    };

    assert_eq!(format!("{:#?}", animation), format!("{:#?}", expected));
}
