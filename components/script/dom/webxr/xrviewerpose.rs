/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use euclid::RigidTransform3D;
use js::jsapi::Heap;
use js::jsval::{JSVal, UndefinedValue};
use js::rust::MutableHandleValue;
use script_bindings::conversions::SafeToJSValConvertible;
use webxr_api::{Viewer, ViewerPose, Views};

use crate::dom::bindings::codegen::Bindings::XRViewBinding::XREye;
use crate::dom::bindings::codegen::Bindings::XRViewerPoseBinding::XRViewerPoseMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use crate::dom::xrpose::XRPose;
use crate::dom::xrrigidtransform::XRRigidTransform;
use crate::dom::xrsession::{BaseSpace, BaseTransform, XRSession, cast_transform};
use crate::dom::xrview::XRView;
use crate::realms::enter_realm;
use crate::script_runtime::{CanGc, JSContext};

#[dom_struct]
pub(crate) struct XRViewerPose {
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

    pub(crate) fn new(
        window: &Window,
        session: &XRSession,
        to_base: BaseTransform,
        viewer_pose: &ViewerPose,
        can_gc: CanGc,
    ) -> DomRoot<XRViewerPose> {
        let _ac = enter_realm(window);
        rooted_vec!(let mut views);
        match &viewer_pose.views {
            Views::Inline => views.push(XRView::new(
                window,
                session,
                &session.inline_view(),
                XREye::None,
                0,
                &to_base,
                can_gc,
            )),
            Views::Mono(view) => views.push(XRView::new(
                window,
                session,
                view,
                XREye::None,
                0,
                &to_base,
                can_gc,
            )),
            Views::Stereo(left, right) => {
                views.push(XRView::new(
                    window,
                    session,
                    left,
                    XREye::Left,
                    0,
                    &to_base,
                    can_gc,
                ));
                views.push(XRView::new(
                    window,
                    session,
                    right,
                    XREye::Right,
                    1,
                    &to_base,
                    can_gc,
                ));
            },
            Views::StereoCapture(left, right, third_eye) => {
                views.push(XRView::new(
                    window,
                    session,
                    left,
                    XREye::Left,
                    0,
                    &to_base,
                    can_gc,
                ));
                views.push(XRView::new(
                    window,
                    session,
                    right,
                    XREye::Right,
                    1,
                    &to_base,
                    can_gc,
                ));
                views.push(XRView::new(
                    window,
                    session,
                    third_eye,
                    XREye::None,
                    2,
                    &to_base,
                    can_gc,
                ));
            },
            Views::Cubemap(front, left, right, top, bottom, back) => {
                views.push(XRView::new(
                    window,
                    session,
                    front,
                    XREye::None,
                    0,
                    &to_base,
                    can_gc,
                ));
                views.push(XRView::new(
                    window,
                    session,
                    left,
                    XREye::None,
                    1,
                    &to_base,
                    can_gc,
                ));
                views.push(XRView::new(
                    window,
                    session,
                    right,
                    XREye::None,
                    2,
                    &to_base,
                    can_gc,
                ));
                views.push(XRView::new(
                    window,
                    session,
                    top,
                    XREye::None,
                    3,
                    &to_base,
                    can_gc,
                ));
                views.push(XRView::new(
                    window,
                    session,
                    bottom,
                    XREye::None,
                    4,
                    &to_base,
                    can_gc,
                ));
                views.push(XRView::new(
                    window,
                    session,
                    back,
                    XREye::None,
                    5,
                    &to_base,
                    can_gc,
                ));
            },
        };
        let transform: RigidTransform3D<f32, Viewer, BaseSpace> =
            viewer_pose.transform.then(&to_base);
        let transform = XRRigidTransform::new(window, cast_transform(transform), can_gc);
        let pose = reflect_dom_object(
            Box::new(XRViewerPose::new_inherited(&transform)),
            window,
            can_gc,
        );

        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut jsval = UndefinedValue());
        views.safe_to_jsval(cx, jsval.handle_mut());
        pose.views.set(jsval.get());

        pose
    }
}

impl XRViewerPoseMethods<crate::DomTypeHolder> for XRViewerPose {
    /// <https://immersive-web.github.io/webxr/#dom-xrviewerpose-views>
    fn Views(&self, _cx: JSContext, mut retval: MutableHandleValue) {
        retval.set(self.views.get())
    }
}
