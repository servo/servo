/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use euclid::RigidTransform3D;
use js::conversions::ToJSValConvertible;
use js::jsapi::Heap;
use js::jsval::{JSVal, UndefinedValue};
use webxr_api::{Viewer, ViewerPose, Views};

use crate::dom::bindings::codegen::Bindings::XRViewBinding::XREye;
use crate::dom::bindings::codegen::Bindings::XRViewerPoseBinding::XRViewerPoseMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrpose::XRPose;
use crate::dom::xrrigidtransform::XRRigidTransform;
use crate::dom::xrsession::{cast_transform, BaseSpace, BaseTransform, XRSession};
use crate::dom::xrview::XRView;
use crate::realms::enter_realm;
use crate::script_runtime::JSContext;

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
        to_base: BaseTransform,
        viewer_pose: &ViewerPose,
    ) -> DomRoot<XRViewerPose> {
        let _ac = enter_realm(global);
        rooted_vec!(let mut views);
        match &viewer_pose.views {
            Views::Inline => views.push(XRView::new(
                global,
                session,
                &session.inline_view(),
                XREye::None,
                0,
                &to_base,
            )),
            Views::Mono(view) => {
                views.push(XRView::new(global, session, view, XREye::None, 0, &to_base))
            },
            Views::Stereo(left, right) => {
                views.push(XRView::new(global, session, left, XREye::Left, 0, &to_base));
                views.push(XRView::new(
                    global,
                    session,
                    right,
                    XREye::Right,
                    1,
                    &to_base,
                ));
            },
            Views::StereoCapture(left, right, third_eye) => {
                views.push(XRView::new(global, session, left, XREye::Left, 0, &to_base));
                views.push(XRView::new(
                    global,
                    session,
                    right,
                    XREye::Right,
                    1,
                    &to_base,
                ));
                views.push(XRView::new(
                    global,
                    session,
                    third_eye,
                    XREye::None,
                    2,
                    &to_base,
                ));
            },
            Views::Cubemap(front, left, right, top, bottom, back) => {
                views.push(XRView::new(
                    global,
                    session,
                    front,
                    XREye::None,
                    0,
                    &to_base,
                ));
                views.push(XRView::new(global, session, left, XREye::None, 1, &to_base));
                views.push(XRView::new(
                    global,
                    session,
                    right,
                    XREye::None,
                    2,
                    &to_base,
                ));
                views.push(XRView::new(global, session, top, XREye::None, 3, &to_base));
                views.push(XRView::new(
                    global,
                    session,
                    bottom,
                    XREye::None,
                    4,
                    &to_base,
                ));
                views.push(XRView::new(global, session, back, XREye::None, 5, &to_base));
            },
        };
        let transform: RigidTransform3D<f32, Viewer, BaseSpace> =
            viewer_pose.transform.then(&to_base);
        let transform = XRRigidTransform::new(global, cast_transform(transform));
        let pose = reflect_dom_object(Box::new(XRViewerPose::new_inherited(&transform)), global);

        let cx = GlobalScope::get_cx();
        unsafe {
            rooted!(in(*cx) let mut jsval = UndefinedValue());
            views.to_jsval(*cx, jsval.handle_mut());
            pose.views.set(jsval.get());
        }

        pose
    }
}

impl XRViewerPoseMethods for XRViewerPose {
    /// <https://immersive-web.github.io/webxr/#dom-xrviewerpose-views>
    fn Views(&self, _cx: JSContext) -> JSVal {
        self.views.get()
    }
}
