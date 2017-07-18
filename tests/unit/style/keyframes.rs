/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo_arc::Arc;
use style::properties::{PropertyDeclaration, PropertyDeclarationBlock, Importance};
use style::properties::animated_properties::AnimatableLonghand;
use style::shared_lock::SharedRwLock;
use style::stylesheets::keyframes_rule::{Keyframe, KeyframesAnimation, KeyframePercentage,  KeyframeSelector};
use style::stylesheets::keyframes_rule::{KeyframesStep, KeyframesStepValue};
use style::values::specified::{LengthOrPercentageOrAuto, NoCalcLength};

#[test]
fn test_empty_keyframe() {
    let shared_lock = SharedRwLock::new();
    let keyframes = vec![];
    let animation = KeyframesAnimation::from_keyframes(&keyframes,
                                                       /* vendor_prefix = */ None,
                                                       &shared_lock.read());
    let expected = KeyframesAnimation {
        steps: vec![],
        properties_changed: vec![],
        vendor_prefix: None,
    };

    assert_eq!(format!("{:#?}", animation), format!("{:#?}", expected));
}

#[test]
fn test_no_property_in_keyframe() {
    let shared_lock = SharedRwLock::new();
    let keyframes = vec![
        Arc::new(shared_lock.wrap(Keyframe {
            selector: KeyframeSelector::new_for_unit_testing(vec![KeyframePercentage::new(1.)]),
            block: Arc::new(shared_lock.wrap(PropertyDeclarationBlock::new()))
        })),
    ];
    let animation = KeyframesAnimation::from_keyframes(&keyframes,
                                                       /* vendor_prefix = */ None,
                                                       &shared_lock.read());
    let expected = KeyframesAnimation {
        steps: vec![],
        properties_changed: vec![],
        vendor_prefix: None,
    };

    assert_eq!(format!("{:#?}", animation), format!("{:#?}", expected));
}

#[test]
fn test_missing_property_in_initial_keyframe() {
    let shared_lock = SharedRwLock::new();
    let declarations_on_initial_keyframe =
        Arc::new(shared_lock.wrap(PropertyDeclarationBlock::with_one(
            PropertyDeclaration::Width(
                LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(20f32))),
            Importance::Normal
        )));

    let declarations_on_final_keyframe =
        Arc::new(shared_lock.wrap({
            let mut block = PropertyDeclarationBlock::new();
            block.push(
                PropertyDeclaration::Width(
                    LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(20f32))),
                Importance::Normal
            );
            block.push(
                PropertyDeclaration::Height(
                    LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(20f32))),
                Importance::Normal
            );
            block
        }));

    let keyframes = vec![
        Arc::new(shared_lock.wrap(Keyframe {
            selector: KeyframeSelector::new_for_unit_testing(vec![KeyframePercentage::new(0.)]),
            block: declarations_on_initial_keyframe.clone(),
        })),

        Arc::new(shared_lock.wrap(Keyframe {
            selector: KeyframeSelector::new_for_unit_testing(vec![KeyframePercentage::new(1.)]),
            block: declarations_on_final_keyframe.clone(),
        })),
    ];
    let animation = KeyframesAnimation::from_keyframes(&keyframes,
                                                       /* vendor_prefix = */ None,
                                                       &shared_lock.read());
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
        properties_changed: vec![AnimatableLonghand::Width, AnimatableLonghand::Height],
        vendor_prefix: None,
    };

    assert_eq!(format!("{:#?}", animation), format!("{:#?}", expected));
}

#[test]
fn test_missing_property_in_final_keyframe() {
    let shared_lock = SharedRwLock::new();
    let declarations_on_initial_keyframe =
        Arc::new(shared_lock.wrap({
            let mut block = PropertyDeclarationBlock::new();
            block.push(
                PropertyDeclaration::Width(
                    LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(20f32))),
                Importance::Normal
            );
            block.push(
                PropertyDeclaration::Height(
                    LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(20f32))),
                Importance::Normal
            );
            block
        }));

    let declarations_on_final_keyframe =
        Arc::new(shared_lock.wrap(PropertyDeclarationBlock::with_one(
            PropertyDeclaration::Height(
                LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(20f32))),
            Importance::Normal,
        )));

    let keyframes = vec![
        Arc::new(shared_lock.wrap(Keyframe {
            selector: KeyframeSelector::new_for_unit_testing(vec![KeyframePercentage::new(0.)]),
            block: declarations_on_initial_keyframe.clone(),
        })),

        Arc::new(shared_lock.wrap(Keyframe {
            selector: KeyframeSelector::new_for_unit_testing(vec![KeyframePercentage::new(1.)]),
            block: declarations_on_final_keyframe.clone(),
        })),
    ];
    let animation = KeyframesAnimation::from_keyframes(&keyframes,
                                                       /* vendor_prefix = */ None,
                                                       &shared_lock.read());
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
        properties_changed: vec![AnimatableLonghand::Width, AnimatableLonghand::Height],
        vendor_prefix: None,
    };

    assert_eq!(format!("{:#?}", animation), format!("{:#?}", expected));
}

#[test]
fn test_missing_keyframe_in_both_of_initial_and_final_keyframe() {
    let shared_lock = SharedRwLock::new();
    let declarations =
        Arc::new(shared_lock.wrap({
            let mut block = PropertyDeclarationBlock::new();
            block.push(
                PropertyDeclaration::Width(
                    LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(20f32))),
                Importance::Normal
            );
            block.push(
                PropertyDeclaration::Height(
                    LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(20f32))),
                Importance::Normal
            );
            block
        }));

    let keyframes = vec![
        Arc::new(shared_lock.wrap(Keyframe {
            selector: KeyframeSelector::new_for_unit_testing(vec![KeyframePercentage::new(0.)]),
            block: Arc::new(shared_lock.wrap(PropertyDeclarationBlock::new()))
        })),
        Arc::new(shared_lock.wrap(Keyframe {
            selector: KeyframeSelector::new_for_unit_testing(vec![KeyframePercentage::new(0.5)]),
            block: declarations.clone(),
        })),
    ];
    let animation = KeyframesAnimation::from_keyframes(&keyframes,
                                                       /* vendor_prefix = */ None,
                                                       &shared_lock.read());
    let expected = KeyframesAnimation {
        steps: vec![
            KeyframesStep {
                start_percentage: KeyframePercentage(0.),
                value: KeyframesStepValue::Declarations {
                    block: Arc::new(shared_lock.wrap(
                        // XXX: Should we use ComputedValues in this case?
                        PropertyDeclarationBlock::new()
                    ))
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
        properties_changed: vec![AnimatableLonghand::Width, AnimatableLonghand::Height],
        vendor_prefix: None,
    };

    assert_eq!(format!("{:#?}", animation), format!("{:#?}", expected));
}
