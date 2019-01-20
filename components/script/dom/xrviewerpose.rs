/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRViewerPoseBinding;
use crate::dom::bindings::codegen::Bindings::XRViewerPoseBinding::XRViewerPoseMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrview::XRView;
use dom_struct::dom_struct;
use js::conversions::ToJSValConvertible;
use js::jsapi::{Heap, JSContext};
use js::jsval::{JSVal, UndefinedValue};

#[dom_struct]
pub struct XRViewerPose {
    reflector_: Reflector,
    views: Heap<JSVal>,
}

impl XRViewerPose {
    fn new_inherited() -> XRViewerPose {
        XRViewerPose {
            reflector_: Reflector::new(),
            views: Heap::default(),
        }
    }

    #[allow(unsafe_code)]
    pub fn new(global: &GlobalScope, left: &XRView, right: &XRView) -> DomRoot<XRViewerPose> {
        let pose = reflect_dom_object(
            Box::new(XRViewerPose::new_inherited()),
            global,
            XRViewerPoseBinding::Wrap,
        );

        unsafe {
            let cx = global.get_cx();
            rooted!(in(cx) let mut jsval = UndefinedValue());
            let vec = vec![DomRoot::from_ref(left), DomRoot::from_ref(right)];
            vec.to_jsval(cx, jsval.handle_mut());
            pose.views.set(jsval.get());
        }

        pose
    }
}

impl XRViewerPoseMethods for XRViewerPose {
    /// https://immersive-web.github.io/webxr/#dom-xrviewerpose-views
    #[allow(unsafe_code)]
    unsafe fn Views(&self, _cx: *mut JSContext) -> JSVal {
        self.views.get()
    }
}
