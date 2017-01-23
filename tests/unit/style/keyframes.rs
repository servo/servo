/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use parking_lot::RwLock;
use std::sync::Arc;
use style::keyframes::{Keyframe, KeyframesAnimation, KeyframePercentage,  KeyframeSelector};
use style::properties::PropertyDeclarationBlock;

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
