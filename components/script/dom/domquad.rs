/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use rustc_hash::FxHashMap;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_proto_and_cx};
use servo_base::id::{DomQuadId, DomQuadIndex};
use servo_constellation_traits::{DomPoint, DomQuad};

use crate::dom::bindings::codegen::Bindings::DOMPointBinding::{DOMPointInit, DOMPointMethods};
use crate::dom::bindings::codegen::Bindings::DOMQuadBinding::{DOMQuadInit, DOMQuadMethods};
use crate::dom::bindings::codegen::Bindings::DOMRectReadOnlyBinding::DOMRectInit;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::serializable::Serializable;
use crate::dom::bindings::structuredclone::StructuredData;
use crate::dom::dompoint::DOMPoint;
use crate::dom::domrect::DOMRect;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

/// <https://drafts.fxtf.org/geometry/#DOMQuad>
#[dom_struct]
pub(crate) struct DOMQuad {
    reflector_: Reflector,
    p1: Dom<DOMPoint>,
    p2: Dom<DOMPoint>,
    p3: Dom<DOMPoint>,
    p4: Dom<DOMPoint>,
}

impl DOMQuad {
    fn new_inherited(p1: &DOMPoint, p2: &DOMPoint, p3: &DOMPoint, p4: &DOMPoint) -> DOMQuad {
        DOMQuad {
            reflector_: Reflector::new(),
            p1: Dom::from_ref(p1),
            p2: Dom::from_ref(p2),
            p3: Dom::from_ref(p3),
            p4: Dom::from_ref(p4),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        p1: &DOMPoint,
        p2: &DOMPoint,
        p3: &DOMPoint,
        p4: &DOMPoint,
    ) -> DomRoot<DOMQuad> {
        Self::new_with_proto(cx, global, None, p1, p2, p3, p4)
    }

    fn new_with_proto(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        p1: &DOMPoint,
        p2: &DOMPoint,
        p3: &DOMPoint,
        p4: &DOMPoint,
    ) -> DomRoot<DOMQuad> {
        reflect_dom_object_with_proto_and_cx(
            Box::new(DOMQuad::new_inherited(p1, p2, p3, p4)),
            global,
            proto,
            cx,
        )
    }
}

impl DOMQuadMethods<crate::DomTypeHolder> for DOMQuad {
    /// <https://drafts.fxtf.org/geometry/#dom-domquad-domquad>
    fn Constructor(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        p1: &DOMPointInit,
        p2: &DOMPointInit,
        p3: &DOMPointInit,
        p4: &DOMPointInit,
    ) -> Fallible<DomRoot<DOMQuad>> {
        let p1 = DOMPoint::new_from_init(cx, global, p1);
        let p2 = DOMPoint::new_from_init(cx, global, p2);
        let p3 = DOMPoint::new_from_init(cx, global, p3);
        let p4 = DOMPoint::new_from_init(cx, global, p4);
        Ok(DOMQuad::new_with_proto(
            cx, global, proto, &p1, &p2, &p3, &p4,
        ))
    }

    /// <https://drafts.fxtf.org/geometry/#dom-domquad-fromrect>
    fn FromRect(cx: &mut JSContext, global: &GlobalScope, other: &DOMRectInit) -> DomRoot<DOMQuad> {
        let p1 = DOMPoint::new(cx, global, other.x, other.y, 0f64, 1f64);
        let p2 = DOMPoint::new(cx, global, other.x + other.width, other.y, 0f64, 1f64);
        let p3 = DOMPoint::new(
            cx,
            global,
            other.x + other.width,
            other.y + other.height,
            0f64,
            1f64,
        );
        let p4 = DOMPoint::new(cx, global, other.x, other.y + other.height, 0f64, 1f64);
        DOMQuad::new(cx, global, &p1, &p2, &p3, &p4)
    }

    /// <https://drafts.fxtf.org/geometry/#dom-domquad-fromquad>
    fn FromQuad(cx: &mut JSContext, global: &GlobalScope, other: &DOMQuadInit) -> DomRoot<DOMQuad> {
        let p1 = DOMPoint::new_from_init(cx, global, &other.p1);
        let p2 = DOMPoint::new_from_init(cx, global, &other.p2);
        let p3 = DOMPoint::new_from_init(cx, global, &other.p3);
        let p4 = DOMPoint::new_from_init(cx, global, &other.p4);
        DOMQuad::new(cx, global, &p1, &p2, &p3, &p4)
    }

    /// <https://drafts.fxtf.org/geometry/#dom-domquad-p1>
    fn P1(&self) -> DomRoot<DOMPoint> {
        DomRoot::from_ref(&self.p1)
    }

    /// <https://drafts.fxtf.org/geometry/#dom-domquad-p2>
    fn P2(&self) -> DomRoot<DOMPoint> {
        DomRoot::from_ref(&self.p2)
    }

    /// <https://drafts.fxtf.org/geometry/#dom-domquad-p3>
    fn P3(&self) -> DomRoot<DOMPoint> {
        DomRoot::from_ref(&self.p3)
    }

    /// <https://drafts.fxtf.org/geometry/#dom-domquad-p4>
    fn P4(&self) -> DomRoot<DOMPoint> {
        DomRoot::from_ref(&self.p4)
    }

    /// <https://drafts.fxtf.org/geometry/#dom-domquad-getbounds>
    fn GetBounds(&self, cx: &mut JSContext) -> DomRoot<DOMRect> {
        // https://drafts.fxtf.org/geometry/#nan-safe-minimum
        let nan_safe_minimum = |a: f64, b: f64| {
            if a.is_nan() || b.is_nan() {
                f64::NAN
            } else {
                a.min(b)
            }
        };

        // https://drafts.fxtf.org/geometry/#nan-safe-maximum
        let nan_safe_maximum = |a: f64, b: f64| {
            if a.is_nan() || b.is_nan() {
                f64::NAN
            } else {
                a.max(b)
            }
        };

        // Step 1. Let bounds be a DOMRect object.
        // NOTE: We construct the object at the end

        // Step 2. Let left be the NaN-safe minimum of point 1’s x coordinate,
        // point 2’s x coordinate, point 3’s x coordinate and point 4’s x coordinate.
        let left = nan_safe_minimum(
            nan_safe_minimum(self.p1.X(), self.p2.X()),
            nan_safe_minimum(self.p3.X(), self.p4.X()),
        );

        // Step 3. Let top be the NaN-safe minimum of point 1’s y coordinate,
        // point 2’s y coordinate, point 3’s y coordinate and point 4’s y coordinate.
        let top = nan_safe_minimum(
            nan_safe_minimum(self.p1.Y(), self.p2.Y()),
            nan_safe_minimum(self.p3.Y(), self.p4.Y()),
        );

        // Step 4. Let right be the NaN-safe maximum of point 1’s x coordinate,
        // point 2’s x coordinate, point 3’s x coordinate and point 4’s x coordinate.
        let right = nan_safe_maximum(
            nan_safe_maximum(self.p1.X(), self.p2.X()),
            nan_safe_maximum(self.p3.X(), self.p4.X()),
        );

        // Step 5. Let bottom be the NaN-safe maximum of point 1’s y coordinate,
        // point 2’s y coordinate, point 3’s y coordinate and point 4’s y coordinate.
        let bottom = nan_safe_maximum(
            nan_safe_maximum(self.p1.Y(), self.p2.Y()),
            nan_safe_maximum(self.p3.Y(), self.p4.Y()),
        );

        // Step 6. Set x coordinate of bounds to left, y coordinate of bounds to top,
        // width dimension of bounds to right - left and height dimension of bounds to bottom - top.
        // NOTE: Combined with Step 1.
        DOMRect::new(
            &self.global(),
            left,
            top,
            right - left,
            bottom - top,
            CanGc::from_cx(cx),
        )
    }
}

impl Serializable for DOMQuad {
    type Index = DomQuadIndex;
    type Data = DomQuad;

    fn serialize(&self) -> Result<(DomQuadId, Self::Data), ()> {
        let make_point = |src: DomRoot<DOMPoint>| -> DomPoint {
            DomPoint {
                x: src.X(),
                y: src.Y(),
                z: src.Z(),
                w: src.W(),
            }
        };
        let serialized = DomQuad {
            p1: make_point(self.P1()),
            p2: make_point(self.P2()),
            p3: make_point(self.P3()),
            p4: make_point(self.P4()),
        };
        Ok((DomQuadId::new(), serialized))
    }

    #[expect(unsafe_code)]
    fn deserialize(
        owner: &GlobalScope,
        serialized: Self::Data,
        _can_gc: CanGc,
    ) -> Result<DomRoot<Self>, ()>
    where
        Self: Sized,
    {
        // TODO: https://github.com/servo/servo/issues/44588
        let mut cx = unsafe { script_bindings::script_runtime::temp_cx() };
        let p1 = DOMPoint::new(
            &mut cx,
            owner,
            serialized.p1.x,
            serialized.p1.y,
            serialized.p1.z,
            serialized.p1.w,
        );
        let p2 = DOMPoint::new(
            &mut cx,
            owner,
            serialized.p2.x,
            serialized.p2.y,
            serialized.p2.z,
            serialized.p2.w,
        );
        let p3 = DOMPoint::new(
            &mut cx,
            owner,
            serialized.p3.x,
            serialized.p3.y,
            serialized.p3.z,
            serialized.p3.w,
        );
        let p4 = DOMPoint::new(
            &mut cx,
            owner,
            serialized.p4.x,
            serialized.p4.y,
            serialized.p4.z,
            serialized.p4.w,
        );
        Ok(Self::new(&mut cx, owner, &p1, &p2, &p3, &p4))
    }

    fn serialized_storage<'a>(
        data: StructuredData<'a, '_>,
    ) -> &'a mut Option<FxHashMap<DomQuadId, Self::Data>> {
        match data {
            StructuredData::Reader(reader) => &mut reader.quads,
            StructuredData::Writer(writer) => &mut writer.quads,
        }
    }
}
