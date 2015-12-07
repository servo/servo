/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DOMPointBinding::{DOMPointInit, DOMPointMethods};
use dom::bindings::codegen::Bindings::DOMPointReadOnlyBinding::DOMPointReadOnlyMethods;
use dom::bindings::codegen::Bindings::DOMQuadBinding::{DOMQuadInit, DOMQuadMethods, Wrap};
use dom::bindings::codegen::Bindings::DOMRectReadOnlyBinding::{DOMRectInit, DOMRectReadOnlyMethods};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{Root, JS, MutHeap};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::dompoint::DOMPoint;
use dom::domrectreadonly::DOMRectReadOnly;

fn create_bound(p1: &DOMPointInit, p2: &DOMPointInit, p3: &DOMPointInit,
    p4: &DOMPointInit) -> Root<DOMRectReadOnly> {
    let left = p1.x.min(p2.x).min(p3.x).min(p4.x);
    let top = p1.y.min(p2.y).min(p3.y).min(p4.y);
    let right = p1.x.max(p2.x).max(p3.x).max(p4.x);
    let bottom = p1.y.max(p2.y).max(p3.y).max(p4.y);

    Root::from_ref(&DOMRectReadOnly::new_inherited(left, top, right - left, bottom - top))
}

fn create_bound_from_point(p1: &DOMPoint, p2: &DOMPoint, p3: &DOMPoint,
    p4: &DOMPoint) -> Root<DOMRectReadOnly> {
    let left = p1.X().min(p2.X()).min(p3.X()).min(p4.X());
    let top = p1.Y().min(p2.Y()).min(p3.Y()).min(p4.Y());
    let right = p1.X().max(p2.X()).max(p3.X()).max(p4.X());
    let bottom = p1.Y().max(p2.Y()).max(p3.Y()).max(p4.Y());

    Root::from_ref(&DOMRectReadOnly::new_inherited(left, top, right - left, bottom - top))
}

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
    p1: MutHeap<JS<DOMPoint>>,
    p2: MutHeap<JS<DOMPoint>>,
    p3: MutHeap<JS<DOMPoint>>,
    p4: MutHeap<JS<DOMPoint>>,
    bounds: JS<DOMRectReadOnly>,
}

impl DOMQuad {
    pub fn new_inherited(p1: &DOMPointInit,
                     p2: &DOMPointInit,
                     p3: &DOMPointInit,
                     p4: &DOMPointInit) -> DOMQuad {
        let bounds = create_bound(&p1, &p2, &p3, &p4);

        DOMQuad {
            reflector_: Reflector::new(),
            p1: MutHeap::new(&DOMPoint::from_init(&p1)),
            p2: MutHeap::new(&DOMPoint::from_init(&p2)),
            p3: MutHeap::new(&DOMPoint::from_init(&p3)),
            p4: MutHeap::new(&DOMPoint::from_init(&p4)),
            bounds: JS::from_ref(&bounds),
        }
    }

    pub fn new(global: GlobalRef,
               p1: &DOMPointInit,
               p2: &DOMPointInit,
               p3: &DOMPointInit,
               p4: &DOMPointInit) -> Root<DOMQuad> {
        reflect_dom_object(box DOMQuad::new_inherited(p1, p2, p3, p4),
                           global,
                           Wrap)
    }

    pub fn Constructor(global: GlobalRef,
                       p1: &DOMPointInit,
                       p2: &DOMPointInit,
                       p3: &DOMPointInit,
                       p4: &DOMPointInit)
                       -> Fallible<Root<DOMQuad>> {
        Ok(DOMQuad::new(global, p1, p2, p3, p4))
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-fromrect
    pub fn FromRect(global: GlobalRef, other: &DOMRectInit) -> Root<DOMQuad> {
        DOMQuad::new(global,
                     &create_dom_point_init(other.x, other.y, 0f64, 1f64),
                     &create_dom_point_init(other.x + other.width, other.y, 0f64, 1f64),
                     &create_dom_point_init(other.x + other.width, other.y + other.height, 0f64, 1f64),
                     &create_dom_point_init(other.x, other.y + other.height, 0f64, 1f64))
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-fromquad
    pub fn FromQuad(global: GlobalRef, other: &DOMQuadInit) -> Root<DOMQuad> {
        DOMQuad::new(global, &other.p1, &other.p2, &other.p3, &other.p4)
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p1
    pub fn SetP1(&mut self, p1: Root<DOMPoint>) {
        self.p1.set(&p1);
        self.bounds = JS::from_ref(&create_bound_from_point(&self.p1.get(), &self.p2.get(), &self.p3.get(), &self.p4.get()));
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p2
    pub fn SetP2(&mut self, p2: Root<DOMPoint>) {
        self.p2.set(&p2);
        self.bounds = JS::from_ref(&create_bound_from_point(&self.p1.get(), &self.p2.get(), &self.p3.get(), &self.p4.get()));
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p3
    pub fn SetP3(&mut self, p3: Root<DOMPoint>) {
        self.p3.set(&p3);
        self.bounds = JS::from_ref(&create_bound_from_point(&self.p1.get(), &self.p2.get(), &self.p3.get(), &self.p4.get()));
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p4
    pub fn SetP4(&mut self, p4: Root<DOMPoint>) {
        self.p4.set(&p4);
        self.bounds = JS::from_ref(&create_bound_from_point(&self.p1.get(), &self.p2.get(), &self.p3.get(), &self.p4.get()));
    }
}

impl DOMQuadMethods for DOMQuad {
    // https://drafts.fxtf.org/geometry/#dom-domquad-p1
    fn P1(&self) -> Root<DOMPoint> {
        Root::from_ref(&*self.p1.get())
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p2
    fn P2(&self) -> Root<DOMPoint> {
        Root::from_ref(&*self.p2.get())
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p3
    fn P3(&self) -> Root<DOMPoint> {
        Root::from_ref(&*self.p3.get())
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p4
    fn P4(&self) -> Root<DOMPoint> {
        Root::from_ref(&*self.p4.get())
    }

    // https://drafts.fxtf.org/geometry/#dom-domquad-p4
    fn Bounds(&self) -> Root<DOMRectReadOnly> {
        Root::from_ref(&*self.bounds)
    }
}
