/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRSpaceBinding;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrinputsource::XRInputSource;
use crate::dom::xrreferencespace::XRReferenceSpace;
use crate::dom::xrsession::{cast_transform, ApiPose, XRSession};
use dom_struct::dom_struct;
use webxr_api::Frame;

#[dom_struct]
pub struct XRSpace {
    eventtarget: EventTarget,
    session: Dom<XRSession>,
    input_source: MutNullableDom<XRInputSource>,
}

impl XRSpace {
    pub fn new_inherited(session: &XRSession) -> XRSpace {
        XRSpace {
            eventtarget: EventTarget::new_inherited(),
            session: Dom::from_ref(session),
            input_source: Default::default(),
        }
    }

    fn new_inputspace_inner(session: &XRSession, input: &XRInputSource) -> XRSpace {
        XRSpace {
            eventtarget: EventTarget::new_inherited(),
            session: Dom::from_ref(session),
            input_source: MutNullableDom::new(Some(input)),
        }
    }

    pub fn new_inputspace(
        global: &GlobalScope,
        session: &XRSession,
        input: &XRInputSource,
    ) -> DomRoot<XRSpace> {
        reflect_dom_object(
            Box::new(XRSpace::new_inputspace_inner(session, input)),
            global,
            XRSpaceBinding::Wrap,
        )
    }
}

impl XRSpace {
    /// Gets pose represented by this space
    ///
    /// The reference origin used is common between all
    /// get_pose calls for spaces from the same device, so this can be used to compare
    /// with other spaces
    pub fn get_pose(&self, base_pose: &Frame) -> ApiPose {
        if let Some(reference) = self.downcast::<XRReferenceSpace>() {
            reference.get_pose(base_pose)
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
            cast_transform(frame.target_ray_origin)
        } else {
            unreachable!()
        }
    }

    pub fn session(&self) -> &XRSession {
        &self.session
    }
}
