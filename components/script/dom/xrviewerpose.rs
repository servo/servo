/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::compartments::enter_realm;
use crate::dom::bindings::codegen::Bindings::XRViewBinding::XREye;
use crate::dom::bindings::codegen::Bindings::XRViewerPoseBinding;
use crate::dom::bindings::codegen::Bindings::XRViewerPoseBinding::XRViewerPoseMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrpose::XRPose;
use crate::dom::xrrigidtransform::XRRigidTransform;
use crate::dom::xrsession::{cast_transform, ApiViewerPose, XRSession};
use crate::dom::xrview::XRView;
use crate::script_runtime::JSContext;
use dom_struct::dom_struct;
use js::conversions::ToJSValConvertible;
use js::jsapi::Heap;
use js::jsval::{JSVal, UndefinedValue};
use webxr_api::Views;

#[dom_struct]
pub struct XRViewerPose {
    pose: XRPose,
    #[ignore_malloc_size_of = "mozjs"]
    views: Heap<JSVal>,
}

impl XRViewerPose {
    fn new_inherited(transform: &XRRigidTransform) -> XRViewerPose {
        XRViewerPose {
            pose: XRPose::new_inherited(transform),
            views: Heap::default(),
        }
    }

    #[allow(unsafe_code)]
    pub fn new(
        global: &GlobalScope,
        session: &XRSession,
        pose: ApiViewerPose,
    ) -> DomRoot<XRViewerPose> {
        let _ac = enter_realm(&*global);
        rooted_vec!(let mut views);
        session.with_session(|s| match s.views() {
            Views::Inline => views.push(XRView::new(
                global,
                session,
                &session.inline_view(),
                XREye::None,
                &pose,
            )),
            Views::Mono(view) => {
                views.push(XRView::new(global, session, &view, XREye::None, &pose))
            },
            Views::Stereo(left, right) => {
                views.push(XRView::new(global, session, &left, XREye::Left, &pose));
                views.push(XRView::new(global, session, &right, XREye::Right, &pose));
            },
        });
        let transform = XRRigidTransform::new(global, cast_transform(pose));
        let pose = reflect_dom_object(
            Box::new(XRViewerPose::new_inherited(&transform)),
            global,
            XRViewerPoseBinding::Wrap,
        );

        let cx = global.get_cx();
        unsafe {
            rooted!(in(*cx) let mut jsval = UndefinedValue());
            views.to_jsval(*cx, jsval.handle_mut());
            pose.views.set(jsval.get());
        }

        pose
    }
}

impl XRViewerPoseMethods for XRViewerPose {
    /// https://immersive-web.github.io/webxr/#dom-xrviewerpose-views
    fn Views(&self, _cx: JSContext) -> JSVal {
        self.views.get()
    }
}
