/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use webxr_api::HitTestResult;

use crate::dom::bindings::codegen::Bindings::XRHitTestResultBinding::XRHitTestResultMethods;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::window::Window;
use crate::dom::xrframe::XRFrame;
use crate::dom::xrpose::XRPose;
use crate::dom::xrspace::XRSpace;

#[dom_struct]
pub(crate) struct XRHitTestResult {
    reflector_: Reflector,
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
        cx: &mut JSContext,
        window: &Window,
        result: HitTestResult,
        frame: &XRFrame,
    ) -> DomRoot<XRHitTestResult> {
        reflect_dom_object_with_cx(
            Box::new(XRHitTestResult::new_inherited(result, frame)),
            window,
            cx,
        )
    }
}

impl XRHitTestResultMethods<crate::DomTypeHolder> for XRHitTestResult {
    /// <https://immersive-web.github.io/hit-test/#dom-xrhittestresult-getpose>
    fn GetPose(&self, cx: &mut JSContext, base: &XRSpace) -> Option<DomRoot<XRPose>> {
        let base = self.frame.get_pose(base)?;
        let pose = self.result.space.then(&base.inverse());
        Some(XRPose::new(cx, self.global().as_window(), pose.cast_unit()))
    }
}
