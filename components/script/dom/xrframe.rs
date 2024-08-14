/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::gc::CustomAutoRooterGuard;
use js::typedarray::Float32Array;
use webxr_api::{Frame, LayerId, SubImages};

use crate::dom::bindings::codegen::Bindings::XRFrameBinding::XRFrameMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrhittestresult::XRHitTestResult;
use crate::dom::xrhittestsource::XRHitTestSource;
use crate::dom::xrjointpose::XRJointPose;
use crate::dom::xrjointspace::XRJointSpace;
use crate::dom::xrpose::XRPose;
use crate::dom::xrreferencespace::XRReferenceSpace;
use crate::dom::xrsession::{ApiPose, XRSession};
use crate::dom::xrspace::XRSpace;
use crate::dom::xrviewerpose::XRViewerPose;

#[dom_struct]
pub struct XRFrame {
    reflector_: Reflector,
    session: Dom<XRSession>,
    #[ignore_malloc_size_of = "defined in webxr_api"]
    #[no_trace]
    data: Frame,
    active: Cell<bool>,
    animation_frame: Cell<bool>,
}

impl XRFrame {
    fn new_inherited(session: &XRSession, data: Frame) -> XRFrame {
        XRFrame {
            reflector_: Reflector::new(),
            session: Dom::from_ref(session),
            data,
            active: Cell::new(false),
            animation_frame: Cell::new(false),
        }
    }

    pub fn new(global: &GlobalScope, session: &XRSession, data: Frame) -> DomRoot<XRFrame> {
        reflect_dom_object(Box::new(XRFrame::new_inherited(session, data)), global)
    }

    /// <https://immersive-web.github.io/webxr/#xrframe-active>
    pub fn set_active(&self, active: bool) {
        self.active.set(active);
    }

    /// <https://immersive-web.github.io/webxr/#xrframe-animationframe>
    pub fn set_animation_frame(&self, animation_frame: bool) {
        self.animation_frame.set(animation_frame);
    }

    pub fn get_pose(&self, space: &XRSpace) -> Option<ApiPose> {
        space.get_pose(&self.data)
    }

    pub fn get_sub_images(&self, layer_id: LayerId) -> Option<&SubImages> {
        self.data
            .sub_images
            .iter()
            .find(|sub_images| sub_images.layer_id == layer_id)
    }
}

impl XRFrameMethods for XRFrame {
    /// <https://immersive-web.github.io/webxr/#dom-xrframe-session>
    fn Session(&self) -> DomRoot<XRSession> {
        DomRoot::from_ref(&self.session)
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrframe-getviewerpose>
    fn GetViewerPose(
        &self,
        reference: &XRReferenceSpace,
    ) -> Result<Option<DomRoot<XRViewerPose>>, Error> {
        if self.session != reference.upcast::<XRSpace>().session() {
            return Err(Error::InvalidState);
        }

        if !self.active.get() || !self.animation_frame.get() {
            return Err(Error::InvalidState);
        }

        let to_base = if let Some(to_base) = reference.get_base_transform(&self.data) {
            to_base
        } else {
            return Ok(None);
        };
        let viewer_pose = if let Some(pose) = self.data.pose.as_ref() {
            pose
        } else {
            return Ok(None);
        };
        Ok(Some(XRViewerPose::new(
            &self.global(),
            &self.session,
            to_base,
            viewer_pose,
        )))
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrframe-getpose>
    fn GetPose(
        &self,
        space: &XRSpace,
        relative_to: &XRSpace,
    ) -> Result<Option<DomRoot<XRPose>>, Error> {
        if self.session != space.session() || self.session != relative_to.session() {
            return Err(Error::InvalidState);
        }
        if !self.active.get() {
            return Err(Error::InvalidState);
        }
        let space = if let Some(space) = self.get_pose(space) {
            space
        } else {
            return Ok(None);
        };
        let relative_to = if let Some(r) = self.get_pose(relative_to) {
            r
        } else {
            return Ok(None);
        };
        let pose = space.then(&relative_to.inverse());
        Ok(Some(XRPose::new(&self.global(), pose)))
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrframe-getpose>
    fn GetJointPose(
        &self,
        space: &XRJointSpace,
        relative_to: &XRSpace,
    ) -> Result<Option<DomRoot<XRJointPose>>, Error> {
        if self.session != space.upcast::<XRSpace>().session() ||
            self.session != relative_to.session()
        {
            return Err(Error::InvalidState);
        }
        if !self.active.get() {
            return Err(Error::InvalidState);
        }
        let joint_frame = if let Some(frame) = space.frame(&self.data) {
            frame
        } else {
            return Ok(None);
        };
        let relative_to = if let Some(r) = self.get_pose(relative_to) {
            r
        } else {
            return Ok(None);
        };
        let pose = joint_frame.pose.then(&relative_to.inverse());
        Ok(Some(XRJointPose::new(
            &self.global(),
            pose.cast_unit(),
            Some(joint_frame.radius),
        )))
    }

    /// <https://immersive-web.github.io/hit-test/#dom-xrframe-gethittestresults>
    fn GetHitTestResults(&self, source: &XRHitTestSource) -> Vec<DomRoot<XRHitTestResult>> {
        self.data
            .hit_test_results
            .iter()
            .filter(|r| r.id == source.id())
            .map(|r| XRHitTestResult::new(&self.global(), *r, self))
            .collect()
    }

    #[allow(unsafe_code)]
    /// <https://www.w3.org/TR/webxr-hand-input-1/#dom-xrframe-filljointradii>
    fn FillJointRadii(
        &self,
        joint_spaces: Vec<DomRoot<XRJointSpace>>,
        mut radii: CustomAutoRooterGuard<Float32Array>,
    ) -> Result<bool, Error> {
        if !self.active.get() {
            return Err(Error::InvalidState);
        }

        for joint_space in &joint_spaces {
            if self.session != joint_space.upcast::<XRSpace>().session() {
                return Err(Error::InvalidState);
            }
        }

        if joint_spaces.len() > radii.len() {
            return Err(Error::Type(
                "Length of radii does not match length of joint spaces".to_string(),
            ));
        }

        let mut radii_vec = radii.to_vec();
        let mut all_valid = true;
        radii_vec.iter_mut().enumerate().for_each(|(i, radius)| {
            if let Some(joint_frame) = joint_spaces
                .get(i)
                .and_then(|joint_space| joint_space.frame(&self.data))
            {
                *radius = joint_frame.radius;
            } else {
                all_valid = false;
            }
        });

        if !all_valid {
            radii_vec.fill(f32::NAN);
        }

        unsafe {
            radii.update(&radii_vec);
        }

        Ok(all_valid)
    }

    #[allow(unsafe_code)]
    /// <https://www.w3.org/TR/webxr-hand-input-1/#dom-xrframe-fillposes>
    fn FillPoses(
        &self,
        spaces: Vec<DomRoot<XRSpace>>,
        base_space: &XRSpace,
        mut transforms: CustomAutoRooterGuard<Float32Array>,
    ) -> Result<bool, Error> {
        if !self.active.get() {
            return Err(Error::InvalidState);
        }

        for space in &spaces {
            if self.session != space.session() {
                return Err(Error::InvalidState);
            }
        }

        if self.session != base_space.session() {
            return Err(Error::InvalidState);
        }

        if spaces.len() * 16 > transforms.len() {
            return Err(Error::Type(
                "Transforms array length does not match 16 * spaces length".to_string(),
            ));
        }

        let mut transforms_vec = transforms.to_vec();
        let mut all_valid = true;
        spaces.iter().enumerate().for_each(|(i, space)| {
            let Some(joint_pose) = self.get_pose(space) else {
                all_valid = false;
                return;
            };
            let Some(base_pose) = self.get_pose(base_space) else {
                all_valid = false;
                return;
            };
            let pose = joint_pose.then(&base_pose.inverse());
            let elements = pose.to_transform();
            let elements_arr = elements.to_array();
            transforms_vec[i * 16..(i + 1) * 16].copy_from_slice(&elements_arr);
        });

        if !all_valid {
            transforms_vec.fill(f32::NAN);
        }

        unsafe {
            transforms.update(&transforms_vec);
        }

        Ok(all_valid)
    }
}
