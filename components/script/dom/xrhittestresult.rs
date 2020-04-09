/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrframe::XRFrame;
use dom_struct::dom_struct;
use webxr_api::HitTestResult;

#[dom_struct]
pub struct XRHitTestResult {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "defined in webxr"]
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

    pub fn new(
        global: &GlobalScope,
        result: HitTestResult,
        frame: &XRFrame,
    ) -> DomRoot<XRHitTestResult> {
        reflect_dom_object(
            Box::new(XRHitTestResult::new_inherited(result, frame)),
            global,
        )
    }
}
