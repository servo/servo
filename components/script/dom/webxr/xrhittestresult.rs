/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use webxr_api::HitTestResult;

use crate::dom::bindings::codegen::Bindings::XRHitTestResultBinding::XRHitTestResultMethods;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::window::Window;
use crate::dom::xrframe::XRFrame;
use crate::dom::xrpose::XRPose;
use crate::dom::xrspace::XRSpace;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct XRHitTestResult {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webxr"]
    #[no_trace]
    result: HitTestResult,
    frame: Dom<XRFrame>,
}

impl XRHitTestResult {
    fn new_inherited(result: HitTestResult, frame: &XRFrame) -> XRHitTestResult {
        XRHitTestResult {
            reflector_: Reflector::new(),
            result,
            frame: Dom::from_ref(frame),
        }
    }

    pub(crate) fn new(
        window: &Window,
        result: HitTestResult,
        frame: &XRFrame,
        can_gc: CanGc,
    ) -> DomRoot<XRHitTestResult> {
        reflect_dom_object(
            Box::new(XRHitTestResult::new_inherited(result, frame)),
            window,
            can_gc,
        )
    }
}

impl XRHitTestResultMethods<crate::DomTypeHolder> for XRHitTestResult {
    // https://immersive-web.github.io/hit-test/#dom-xrhittestresult-getpose
    fn GetPose(&self, base: &XRSpace, can_gc: CanGc) -> Option<DomRoot<XRPose>> {
        let base = self.frame.get_pose(base)?;
        let pose = self.result.space.then(&base.inverse());
        Some(XRPose::new(
            self.global().as_window(),
            pose.cast_unit(),
            can_gc,
        ))
    }
}
