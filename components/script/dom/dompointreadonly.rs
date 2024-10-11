/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::jsapi::{JSStructuredCloneReader, JS_ReadDouble};
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::DOMPointBinding::DOMPointInit;
use crate::dom::bindings::codegen::Bindings::DOMPointReadOnlyBinding::DOMPointReadOnlyMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::serializable::{
    FromStructuredClone, Serializable, SerializeOperation, ToSerializeOperations,
};
use crate::dom::bindings::structuredclone::{
    CloneableObject, StructuredReadDataHolder, StructuredWriteDataHolder,
};
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

// http://dev.w3.org/fxtf/geometry/Overview.html#dompointreadonly
#[dom_struct]
pub struct DOMPointReadOnly {
    reflector_: Reflector,
    x: Cell<f64>,
    y: Cell<f64>,
    z: Cell<f64>,
    w: Cell<f64>,
}

#[allow(non_snake_case)]
impl DOMPointReadOnly {
    pub fn new_inherited(x: f64, y: f64, z: f64, w: f64) -> DOMPointReadOnly {
        DOMPointReadOnly {
            x: Cell::new(x),
            y: Cell::new(y),
            z: Cell::new(z),
            w: Cell::new(w),
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: &GlobalScope, x: f64, y: f64, z: f64, w: f64) -> DomRoot<DOMPointReadOnly> {
        Self::new_with_proto(global, None, x, y, z, w, CanGc::note())
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

    pub fn Constructor(
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
    pub fn FromPoint(global: &GlobalScope, init: &DOMPointInit) -> DomRoot<Self> {
        Self::new(global, init.x, init.y, init.z, init.w)
    }
}

#[allow(non_snake_case)]
impl DOMPointReadOnlyMethods for DOMPointReadOnly {
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
pub trait DOMPointWriteMethods {
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

pub struct Coordinates {
    x: f64,
    y: f64,
    z: f64,
    w: f64,
}

impl ToSerializeOperations for Coordinates {
    fn to_serialize_operations(&self) -> Vec<SerializeOperation> {
        vec![
            SerializeOperation::Double(self.x),
            SerializeOperation::Double(self.y),
            SerializeOperation::Double(self.z),
            SerializeOperation::Double(self.w),
        ]
    }
}

impl FromStructuredClone for Coordinates {
    #[allow(unsafe_code)]
    unsafe fn from_structured_clone(r: *mut JSStructuredCloneReader) -> Coordinates {
        let mut coordinates = Coordinates {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        };
        assert!(JS_ReadDouble(r, &mut coordinates.x));
        assert!(JS_ReadDouble(r, &mut coordinates.y));
        assert!(JS_ReadDouble(r, &mut coordinates.z));
        assert!(JS_ReadDouble(r, &mut coordinates.w));
        coordinates
    }
}

impl Serializable for DOMPointReadOnly {
    type Data = Coordinates;
    const TAG: CloneableObject = CloneableObject::DomPointReadonly;

    /// <https://drafts.fxtf.org/geometry-1/#structured-serialization>
    fn serialize(&self, _sc_holder: &mut StructuredWriteDataHolder) -> Result<Coordinates, ()> {
        Ok(Coordinates {
            x: self.x.get(),
            y: self.y.get(),
            z: self.z.get(),
            w: self.w.get(),
        })
    }

    /// <https://drafts.fxtf.org/geometry-1/#structured-serialization>
    fn deserialize(
        owner: &GlobalScope,
        _sc_holder: &mut StructuredReadDataHolder,
        coordinates: Coordinates,
    ) -> Result<DomRoot<DOMPointReadOnly>, ()> {
        Ok(Self::new(
            owner,
            coordinates.x,
            coordinates.y,
            coordinates.z,
            coordinates.w,
        ))
    }
}
