/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use euclid::RigidTransform3D;
use webxr_api::{BaseSpace, Frame, Space};

use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrinputsource::XRInputSource;
use crate::dom::xrjointspace::XRJointSpace;
use crate::dom::xrreferencespace::XRReferenceSpace;
use crate::dom::xrsession::{cast_transform, ApiPose, XRSession};
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct XRSpace {
    eventtarget: EventTarget,
    session: Dom<XRSession>,
    input_source: MutNullableDom<XRInputSource>,
    /// If we're an input space, are we an aim space or a grip space?
    is_grip_space: bool,
}

impl XRSpace {
    pub(crate) fn new_inherited(session: &XRSession) -> XRSpace {
        XRSpace {
            eventtarget: EventTarget::new_inherited(),
            session: Dom::from_ref(session),
            input_source: Default::default(),
            is_grip_space: false,
        }
    }

    fn new_inputspace_inner(
        session: &XRSession,
        input: &XRInputSource,
        is_grip_space: bool,
    ) -> XRSpace {
        XRSpace {
            eventtarget: EventTarget::new_inherited(),
            session: Dom::from_ref(session),
            input_source: MutNullableDom::new(Some(input)),
            is_grip_space,
        }
    }

    pub(crate) fn new_inputspace(
        global: &GlobalScope,
        session: &XRSession,
        input: &XRInputSource,
        is_grip_space: bool,
    ) -> DomRoot<XRSpace> {
        reflect_dom_object(
            Box::new(XRSpace::new_inputspace_inner(session, input, is_grip_space)),
            global,
            CanGc::note(),
        )
    }

    pub(crate) fn space(&self) -> Space {
        if let Some(rs) = self.downcast::<XRReferenceSpace>() {
            rs.space()
        } else if let Some(j) = self.downcast::<XRJointSpace>() {
            j.space()
        } else if let Some(source) = self.input_source.get() {
            let base = if self.is_grip_space {
                BaseSpace::Grip(source.id())
            } else {
                BaseSpace::TargetRay(source.id())
            };
            Space {
                base,
                offset: RigidTransform3D::identity(),
            }
        } else {
            panic!("invalid space found")
        }
    }
}

impl XRSpace {
    /// Gets pose represented by this space
    ///
    /// The reference origin used is common between all
    /// get_pose calls for spaces from the same device, so this can be used to compare
    /// with other spaces
    pub(crate) fn get_pose(&self, base_pose: &Frame) -> Option<ApiPose> {
        if let Some(reference) = self.downcast::<XRReferenceSpace>() {
            reference.get_pose(base_pose)
        } else if let Some(joint) = self.downcast::<XRJointSpace>() {
            joint.get_pose(base_pose)
        } else if let Some(source) = self.input_source.get() {
            // XXXManishearth we should be able to request frame information
            // for inputs when necessary instead of always loading it
            //
            // Also, the below code is quadratic, so this API may need an overhaul anyway
            let id = source.id();
            // XXXManishearth once we have dynamic inputs we'll need to handle this better
            let frame = base_pose
                .inputs
                .iter()
                .find(|i| i.id == id)
                .expect("no input found");
            if self.is_grip_space {
                frame.grip_origin.map(cast_transform)
            } else {
                frame.target_ray_origin.map(cast_transform)
            }
        } else {
            unreachable!()
        }
    }

    pub(crate) fn session(&self) -> &XRSession {
        &self.session
    }
}
