/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::jsapi::JSContext;
use js::rust::MutableHandleValue;
use webxr_api::{FingerJoint, Hand, Joint};

use crate::dom::bindings::codegen::Bindings::XRHandBinding::{XRHandJoint, XRHandMethods};
use crate::dom::bindings::conversions::ToJSValConvertible;
use crate::dom::bindings::iterable::Iterable;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrinputsource::XRInputSource;
use crate::dom::xrjointspace::XRJointSpace;
use crate::script_runtime::CanGc;

const JOINT_SPACE_MAP: [(XRHandJoint, Joint); 25] = [
    (XRHandJoint::Wrist, Joint::Wrist),
    (XRHandJoint::Thumb_metacarpal, Joint::ThumbMetacarpal),
    (
        XRHandJoint::Thumb_phalanx_proximal,
        Joint::ThumbPhalanxProximal,
    ),
    (XRHandJoint::Thumb_phalanx_distal, Joint::ThumbPhalanxDistal),
    (XRHandJoint::Thumb_tip, Joint::ThumbPhalanxTip),
    (
        XRHandJoint::Index_finger_metacarpal,
        Joint::Index(FingerJoint::Metacarpal),
    ),
    (
        XRHandJoint::Index_finger_phalanx_proximal,
        Joint::Index(FingerJoint::PhalanxProximal),
    ),
    (XRHandJoint::Index_finger_phalanx_intermediate, {
        Joint::Index(FingerJoint::PhalanxIntermediate)
    }),
    (
        XRHandJoint::Index_finger_phalanx_distal,
        Joint::Index(FingerJoint::PhalanxDistal),
    ),
    (
        XRHandJoint::Index_finger_tip,
        Joint::Index(FingerJoint::PhalanxTip),
    ),
    (
        XRHandJoint::Middle_finger_metacarpal,
        Joint::Middle(FingerJoint::Metacarpal),
    ),
    (
        XRHandJoint::Middle_finger_phalanx_proximal,
        Joint::Middle(FingerJoint::PhalanxProximal),
    ),
    (XRHandJoint::Middle_finger_phalanx_intermediate, {
        Joint::Middle(FingerJoint::PhalanxIntermediate)
    }),
    (
        XRHandJoint::Middle_finger_phalanx_distal,
        Joint::Middle(FingerJoint::PhalanxDistal),
    ),
    (
        XRHandJoint::Middle_finger_tip,
        Joint::Middle(FingerJoint::PhalanxTip),
    ),
    (
        XRHandJoint::Ring_finger_metacarpal,
        Joint::Ring(FingerJoint::Metacarpal),
    ),
    (
        XRHandJoint::Ring_finger_phalanx_proximal,
        Joint::Ring(FingerJoint::PhalanxProximal),
    ),
    (XRHandJoint::Ring_finger_phalanx_intermediate, {
        Joint::Ring(FingerJoint::PhalanxIntermediate)
    }),
    (
        XRHandJoint::Ring_finger_phalanx_distal,
        Joint::Ring(FingerJoint::PhalanxDistal),
    ),
    (
        XRHandJoint::Ring_finger_tip,
        Joint::Ring(FingerJoint::PhalanxTip),
    ),
    (
        XRHandJoint::Pinky_finger_metacarpal,
        Joint::Little(FingerJoint::Metacarpal),
    ),
    (
        XRHandJoint::Pinky_finger_phalanx_proximal,
        Joint::Little(FingerJoint::PhalanxProximal),
    ),
    (XRHandJoint::Pinky_finger_phalanx_intermediate, {
        Joint::Little(FingerJoint::PhalanxIntermediate)
    }),
    (
        XRHandJoint::Pinky_finger_phalanx_distal,
        Joint::Little(FingerJoint::PhalanxDistal),
    ),
    (
        XRHandJoint::Pinky_finger_tip,
        Joint::Little(FingerJoint::PhalanxTip),
    ),
];

#[dom_struct]
pub(crate) struct XRHand {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webxr"]
    source: Dom<XRInputSource>,
    #[ignore_malloc_size_of = "partially defind in webxr"]
    #[custom_trace]
    spaces: Hand<Dom<XRJointSpace>>,
}

impl XRHand {
    fn new_inherited(source: &XRInputSource, spaces: &Hand<DomRoot<XRJointSpace>>) -> XRHand {
        XRHand {
            reflector_: Reflector::new(),
            source: Dom::from_ref(source),
            spaces: spaces.map(|j, _| j.as_ref().map(|j| Dom::from_ref(&**j))),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        source: &XRInputSource,
        support: Hand<()>,
    ) -> DomRoot<XRHand> {
        let id = source.id();
        let session = source.session();
        let spaces = support.map(|field, joint| {
            let hand_joint = JOINT_SPACE_MAP
                .iter()
                .find(|&&(_, value)| value == joint)
                .map(|&(hand_joint, _)| hand_joint)
                .expect("Invalid joint name");
            field.map(|_| XRJointSpace::new(global, session, id, joint, hand_joint))
        });
        reflect_dom_object(
            Box::new(XRHand::new_inherited(source, &spaces)),
            global,
            CanGc::note(),
        )
    }
}

impl XRHandMethods<crate::DomTypeHolder> for XRHand {
    /// <https://github.com/immersive-web/webxr-hands-input/blob/master/explainer.md>
    fn Size(&self) -> u32 {
        XRHandJoint::Pinky_finger_tip as u32 + 1
    }

    /// <https://github.com/immersive-web/webxr-hands-input/blob/master/explainer.md>
    fn Get(&self, joint_name: XRHandJoint) -> DomRoot<XRJointSpace> {
        let joint = JOINT_SPACE_MAP
            .iter()
            .find(|&&(key, _)| key == joint_name)
            .map(|&(_, joint)| joint)
            .expect("Invalid joint name");
        self.spaces
            .get(joint)
            .map(|j| DomRoot::from_ref(&**j))
            .expect("Failed to get joint pose")
    }
}

/// A wrapper to work around a crown error—Root<T> has a crown annotation on it that is not present
/// on the Iterable::Value associated type. The absence is harmless in this case.
pub(crate) struct ValueWrapper(pub DomRoot<XRJointSpace>);

impl ToJSValConvertible for ValueWrapper {
    #[allow(unsafe_code)]
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        self.0.to_jsval(cx, rval)
    }
}

impl Iterable for XRHand {
    type Key = XRHandJoint;
    type Value = ValueWrapper;

    fn get_iterable_length(&self) -> u32 {
        JOINT_SPACE_MAP.len() as u32
    }

    fn get_value_at_index(&self, n: u32) -> ValueWrapper {
        let joint = JOINT_SPACE_MAP[n as usize].1;
        self.spaces
            .get(joint)
            .map(|j| ValueWrapper(DomRoot::from_ref(&**j)))
            .expect("Failed to get joint pose")
    }

    fn get_key_at_index(&self, n: u32) -> XRHandJoint {
        JOINT_SPACE_MAP[n as usize].0
    }
}
