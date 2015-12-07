/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DOMPointBinding::{DOMPointInit, DOMPointMethods};
use dom::bindings::codegen::Bindings::DOMPointReadOnlyBinding::DOMPointReadOnlyMethods;
use dom::bindings::codegen::Bindings::DOMQuadBinding::{DOMQuadInit, DOMQuadMethods, Wrap};
use dom::bindings::codegen::Bindings::DOMRectReadOnlyBinding::{DOMRectInit, DOMRectReadOnlyMethods};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{Root, JS};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::dompoint::DOMPoint;
use dom::domrectreadonly::DOMRectReadOnly;

fn create_dom_point_init(x: f64, y: f64, z: f64, w: f64) -> DOMPointInit {
    DOMPointInit {
        x: x,
        y: y,
        z: z,
        w: w,
    }
}

// https://drafts.fxtf.org/geometry/#DOMQuad
#[dom_struct]
pub struct DOMQuad {
    reflector_: Reflector,
    p1: JS<DOMPoint>,
    p2: JS<DOMPoint>,
    p3: JS<DOMPoint>,
    p4: JS<DOMPoint>,
    bounds: JS<DOMRectReadOnly>,
}

impl DOMQuad {
    fn new_inherited(global: GlobalRef,
                     p1: &DOMPoint,
                     p2: &DOMPoint,
                     p3: &DOMPoint,
                     p4: &DOMPoint) -> DOMQuad {
        let left = p1.X().min(p2.X()).min(p3.X()).min(p4.X());
        let top = p1.Y().min(p2.Y()).min(p3.Y()).min(p4.Y());
        let right = p1.X().max(p2.X()).max(p3.X()).max(p4.X());
        let bottom = p1.Y().max(p2.Y()).max(p3.Y()).max(p4.Y());

        DOMQuad {
            reflector_: Reflector::new(),
            p1: JS::from_ref(p1),
            p2: JS::from_ref(p2),
            p3: JS::from_ref(p3),
            p4: JS::from_ref(p4),
            bounds: JS::from_ref(&DOMRectReadOnly::new(global, left, top, right - left, bottom - top)),
        }
    }

    pub fn new(global: GlobalRef,
               p1: &DOMPoint,
               p2: &DOMPoint,
               p3: &DOMPoint,
               p4: &DOMPoint) -> Root<DOMQuad> {
        reflect_dom_object(box DOMQuad::new_inherited(global, p1, p2, p3, p4),
                           global,
                           Wrap)
    }

    pub fn Constructor(global: GlobalRef,
                       p1: &DOMPointInit,
                       p2: &DOMPointInit,
                       p3: &DOMPointInit,
                       p4: &DOMPointInit)
                       -> Fallible<Root<DOMQuad>> {
        Ok(DOMQuad::new(global,
                        &*DOMPoint::new_from_init(global, p1),
                        &*DOMPoint::new_from_init(global, p2),
                        &*DOMPoint::new_from_init(global, p3),
                        &*DOMPoint::new_from_init(global, p4)))
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-fromrect
    pub fn FromRect(global: GlobalRef, other: &DOMRectInit) -> Root<DOMQuad> {
        DOMQuad::new(global,
                     &*DOMPoint::new(global, other.x, other.y, 0f64, 1f64),
                     &*DOMPoint::new(global, other.x + other.width, other.y, 0f64, 1f64),
                     &*DOMPoint::new(global, other.x + other.width, other.y + other.height, 0f64, 1f64),
                     &*DOMPoint::new(global, other.x, other.y + other.height, 0f64, 1f64))
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-fromquad
    pub fn FromQuad(global: GlobalRef, other: &DOMQuadInit) -> Root<DOMQuad> {
        DOMQuad::new(global,
                     &DOMPoint::new_from_init(global, &other.p1),
                     &DOMPoint::new_from_init(global, &other.p2),
                     &DOMPoint::new_from_init(global, &other.p3),
                     &DOMPoint::new_from_init(global, &other.p4))
    }
}

impl DOMQuadMethods for DOMQuad {
    // https://drafts.fxtf.org/geometry/#dom-domquad-p1
    fn P1(&self) -> Root<DOMPoint> {
        Root::from_ref(&*self.p1)
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p2
    fn P2(&self) -> Root<DOMPoint> {
        Root::from_ref(&*self.p2)
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p3
    fn P3(&self) -> Root<DOMPoint> {
        Root::from_ref(&*self.p3)
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p4
    fn P4(&self) -> Root<DOMPoint> {
        Root::from_ref(&*self.p4)
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p4
    fn Bounds(&self) -> Root<DOMRectReadOnly> {
        Root::from_ref(&*self.bounds)
    }
}
