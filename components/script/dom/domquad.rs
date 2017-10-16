/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DOMPointBinding::{DOMPointInit, DOMPointMethods};
use dom::bindings::codegen::Bindings::DOMQuadBinding::{DOMQuadInit, DOMQuadMethods, Wrap};
use dom::bindings::codegen::Bindings::DOMRectReadOnlyBinding::DOMRectInit;
use dom::bindings::error::Fallible;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot};
use dom::dompoint::DOMPoint;
use dom::domrect::DOMRect;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;

// https://drafts.fxtf.org/geometry/#DOMQuad
#[dom_struct]
pub struct DOMQuad {
    reflector_: Reflector,
    p1: Dom<DOMPoint>,
    p2: Dom<DOMPoint>,
    p3: Dom<DOMPoint>,
    p4: Dom<DOMPoint>,
}

impl DOMQuad {
    fn new_inherited(p1: &DOMPoint,
                     p2: &DOMPoint,
                     p3: &DOMPoint,
                     p4: &DOMPoint)
                     -> DOMQuad {
        DOMQuad {
            reflector_: Reflector::new(),
            p1: Dom::from_ref(p1),
            p2: Dom::from_ref(p2),
            p3: Dom::from_ref(p3),
            p4: Dom::from_ref(p4),
        }
    }

    pub fn new(global: &GlobalScope,
               p1: &DOMPoint,
               p2: &DOMPoint,
               p3: &DOMPoint,
               p4: &DOMPoint) -> DomRoot<DOMQuad> {
        reflect_dom_object(Box::new(DOMQuad::new_inherited(p1, p2, p3, p4)),
                           global,
                           Wrap)
    }

    pub fn Constructor(global: &GlobalScope,
                       p1: &DOMPointInit,
                       p2: &DOMPointInit,
                       p3: &DOMPointInit,
                       p4: &DOMPointInit)
                       -> Fallible<DomRoot<DOMQuad>> {
        Ok(DOMQuad::new(global,
                        &*DOMPoint::new_from_init(global, p1),
                        &*DOMPoint::new_from_init(global, p2),
                        &*DOMPoint::new_from_init(global, p3),
                        &*DOMPoint::new_from_init(global, p4)))
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-fromrect
    pub fn FromRect(global: &GlobalScope, other: &DOMRectInit) -> DomRoot<DOMQuad> {
        DOMQuad::new(global,
                     &*DOMPoint::new(global, other.x, other.y, 0f64, 1f64),
                     &*DOMPoint::new(global, other.x + other.width, other.y, 0f64, 1f64),
                     &*DOMPoint::new(global, other.x + other.width, other.y + other.height, 0f64, 1f64),
                     &*DOMPoint::new(global, other.x, other.y + other.height, 0f64, 1f64))
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-fromquad
    pub fn FromQuad(global: &GlobalScope, other: &DOMQuadInit) -> DomRoot<DOMQuad> {
        DOMQuad::new(global,
                     &DOMPoint::new_from_init(global, &other.p1),
                     &DOMPoint::new_from_init(global, &other.p2),
                     &DOMPoint::new_from_init(global, &other.p3),
                     &DOMPoint::new_from_init(global, &other.p4))
    }
}

impl DOMQuadMethods for DOMQuad {
    // https://drafts.fxtf.org/geometry/#dom-domquad-p1
    fn P1(&self) -> DomRoot<DOMPoint> {
        DomRoot::from_ref(&self.p1)
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p2
    fn P2(&self) -> DomRoot<DOMPoint> {
        DomRoot::from_ref(&self.p2)
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p3
    fn P3(&self) -> DomRoot<DOMPoint> {
        DomRoot::from_ref(&self.p3)
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p4
    fn P4(&self) -> DomRoot<DOMPoint> {
        DomRoot::from_ref(&self.p4)
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-getbounds
    fn GetBounds(&self) -> DomRoot<DOMRect> {
        let left = self.p1.X().min(self.p2.X()).min(self.p3.X()).min(self.p4.X());
        let top = self.p1.Y().min(self.p2.Y()).min(self.p3.Y()).min(self.p4.Y());
        let right = self.p1.X().max(self.p2.X()).max(self.p3.X()).max(self.p4.X());
        let bottom = self.p1.Y().max(self.p2.Y()).max(self.p3.Y()).max(self.p4.Y());

        DOMRect::new(&self.global(),
                     left,
                     top,
                     right - left,
                     bottom - top)
    }
}
