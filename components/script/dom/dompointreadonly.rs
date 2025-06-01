/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::HashMap;

use base::id::{DomPointId, DomPointIndex};
use constellation_traits::DomPoint;
use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::DOMPointBinding::DOMPointInit;
use crate::dom::bindings::codegen::Bindings::DOMPointReadOnlyBinding::DOMPointReadOnlyMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::serializable::Serializable;
use crate::dom::bindings::structuredclone::StructuredData;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

// http://dev.w3.org/fxtf/geometry/Overview.html#dompointreadonly
#[dom_struct]
pub(crate) struct DOMPointReadOnly {
    reflector_: Reflector,
    x: Cell<f64>,
    y: Cell<f64>,
    z: Cell<f64>,
    w: Cell<f64>,
}

#[allow(non_snake_case)]
impl DOMPointReadOnly {
    pub(crate) fn new_inherited(x: f64, y: f64, z: f64, w: f64) -> DOMPointReadOnly {
        DOMPointReadOnly {
            x: Cell::new(x),
            y: Cell::new(y),
            z: Cell::new(z),
            w: Cell::new(w),
            reflector_: Reflector::new(),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        x: f64,
        y: f64,
        z: f64,
        w: f64,
        can_gc: CanGc,
    ) -> DomRoot<DOMPointReadOnly> {
        Self::new_with_proto(global, None, x, y, z, w, can_gc)
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        x: f64,
        y: f64,
        z: f64,
        w: f64,
        can_gc: CanGc,
    ) -> DomRoot<DOMPointReadOnly> {
        reflect_dom_object_with_proto(
            Box::new(DOMPointReadOnly::new_inherited(x, y, z, w)),
            global,
            proto,
            can_gc,
        )
    }
}

#[allow(non_snake_case)]
impl DOMPointReadOnlyMethods<crate::DomTypeHolder> for DOMPointReadOnly {
    // https://drafts.fxtf.org/geometry/#dom-dompoint-dompoint
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        x: f64,
        y: f64,
        z: f64,
        w: f64,
    ) -> Fallible<DomRoot<DOMPointReadOnly>> {
        Ok(DOMPointReadOnly::new_with_proto(
            global, proto, x, y, z, w, can_gc,
        ))
    }

    // https://drafts.fxtf.org/geometry/#dom-dompointreadonly-frompoint
    fn FromPoint(global: &GlobalScope, init: &DOMPointInit, can_gc: CanGc) -> DomRoot<Self> {
        Self::new(global, init.x, init.y, init.z, init.w, can_gc)
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-dompointreadonly-x
    fn X(&self) -> f64 {
        self.x.get()
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-dompointreadonly-y
    fn Y(&self) -> f64 {
        self.y.get()
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-dompointreadonly-z
    fn Z(&self) -> f64 {
        self.z.get()
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-dompointreadonly-w
    fn W(&self) -> f64 {
        self.w.get()
    }
}

#[allow(non_snake_case)]
pub(crate) trait DOMPointWriteMethods {
    fn SetX(&self, value: f64);
    fn SetY(&self, value: f64);
    fn SetZ(&self, value: f64);
    fn SetW(&self, value: f64);
}

impl DOMPointWriteMethods for DOMPointReadOnly {
    fn SetX(&self, value: f64) {
        self.x.set(value);
    }

    fn SetY(&self, value: f64) {
        self.y.set(value);
    }

    fn SetZ(&self, value: f64) {
        self.z.set(value);
    }

    fn SetW(&self, value: f64) {
        self.w.set(value);
    }
}

impl Serializable for DOMPointReadOnly {
    type Index = DomPointIndex;
    type Data = DomPoint;

    fn serialize(&self) -> Result<(DomPointId, Self::Data), ()> {
        let serialized = DomPoint {
            x: self.x.get(),
            y: self.y.get(),
            z: self.z.get(),
            w: self.w.get(),
        };
        Ok((DomPointId::new(), serialized))
    }

    fn deserialize(
        owner: &GlobalScope,
        serialized: Self::Data,
        can_gc: CanGc,
    ) -> Result<DomRoot<Self>, ()>
    where
        Self: Sized,
    {
        Ok(Self::new(
            owner,
            serialized.x,
            serialized.y,
            serialized.z,
            serialized.w,
            can_gc,
        ))
    }

    fn serialized_storage<'a>(
        data: StructuredData<'a, '_>,
    ) -> &'a mut Option<HashMap<DomPointId, Self::Data>> {
        match data {
            StructuredData::Reader(r) => &mut r.points,
            StructuredData::Writer(w) => &mut w.points,
        }
    }
}
