/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use webxr_api::{FingerJoint, Hand, Joint};

use crate::dom::bindings::codegen::Bindings::XRHandBinding::{XRHandConstants, XRHandMethods};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrinputsource::XRInputSource;
use crate::dom::xrjointspace::XRJointSpace;

#[dom_struct]
pub struct XRHand {
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

    pub fn new(global: &GlobalScope, source: &XRInputSource, support: Hand<()>) -> DomRoot<XRHand> {
        let id = source.id();
        let session = source.session();
        let spaces = support
            .map(|field, joint| field.map(|_| XRJointSpace::new(global, session, id, joint)));
        reflect_dom_object(Box::new(XRHand::new_inherited(source, &spaces)), global)
    }
}

impl XRHandMethods for XRHand {
    /// <https://github.com/immersive-web/webxr-hands-input/blob/master/explainer.md>
    fn Length(&self) -> i32 {
        XRHandConstants::LITTLE_PHALANX_TIP as i32 + 1
    }

    /// <https://github.com/immersive-web/webxr-hands-input/blob/master/explainer.md>
    fn IndexedGetter(&self, joint_index: u32) -> Option<DomRoot<XRJointSpace>> {
        let joint = match joint_index {
            XRHandConstants::WRIST => Joint::Wrist,
            XRHandConstants::THUMB_METACARPAL => Joint::ThumbMetacarpal,
            XRHandConstants::THUMB_PHALANX_PROXIMAL => Joint::ThumbPhalanxProximal,
            XRHandConstants::THUMB_PHALANX_DISTAL => Joint::ThumbPhalanxDistal,
            XRHandConstants::THUMB_PHALANX_TIP => Joint::ThumbPhalanxTip,
            XRHandConstants::INDEX_METACARPAL => Joint::Index(FingerJoint::Metacarpal),
            XRHandConstants::INDEX_PHALANX_PROXIMAL => Joint::Index(FingerJoint::PhalanxProximal),
            XRHandConstants::INDEX_PHALANX_INTERMEDIATE => {
                Joint::Index(FingerJoint::PhalanxIntermediate)
            },
            XRHandConstants::INDEX_PHALANX_DISTAL => Joint::Index(FingerJoint::PhalanxDistal),
            XRHandConstants::INDEX_PHALANX_TIP => Joint::Index(FingerJoint::PhalanxTip),
            XRHandConstants::MIDDLE_METACARPAL => Joint::Middle(FingerJoint::Metacarpal),
            XRHandConstants::MIDDLE_PHALANX_PROXIMAL => Joint::Middle(FingerJoint::PhalanxProximal),
            XRHandConstants::MIDDLE_PHALANX_INTERMEDIATE => {
                Joint::Middle(FingerJoint::PhalanxIntermediate)
            },
            XRHandConstants::MIDDLE_PHALANX_DISTAL => Joint::Middle(FingerJoint::PhalanxDistal),
            XRHandConstants::MIDDLE_PHALANX_TIP => Joint::Middle(FingerJoint::PhalanxTip),
            XRHandConstants::RING_METACARPAL => Joint::Ring(FingerJoint::Metacarpal),
            XRHandConstants::RING_PHALANX_PROXIMAL => Joint::Ring(FingerJoint::PhalanxProximal),
            XRHandConstants::RING_PHALANX_INTERMEDIATE => {
                Joint::Ring(FingerJoint::PhalanxIntermediate)
            },
            XRHandConstants::RING_PHALANX_DISTAL => Joint::Ring(FingerJoint::PhalanxDistal),
            XRHandConstants::RING_PHALANX_TIP => Joint::Ring(FingerJoint::PhalanxTip),
            XRHandConstants::LITTLE_METACARPAL => Joint::Little(FingerJoint::Metacarpal),
            XRHandConstants::LITTLE_PHALANX_PROXIMAL => Joint::Little(FingerJoint::PhalanxProximal),
            XRHandConstants::LITTLE_PHALANX_INTERMEDIATE => {
                Joint::Little(FingerJoint::PhalanxIntermediate)
            },
            XRHandConstants::LITTLE_PHALANX_DISTAL => Joint::Little(FingerJoint::PhalanxDistal),
            XRHandConstants::LITTLE_PHALANX_TIP => Joint::Little(FingerJoint::PhalanxTip),
            // XXXManishearth should this be a TypeError?
            _ => return None,
        };
        self.spaces.get(joint).map(|j| DomRoot::from_ref(&**j))
    }
}
