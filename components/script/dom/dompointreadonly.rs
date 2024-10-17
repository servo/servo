/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::HashMap;
use std::num::NonZeroU32;

use base::id::{CoordinatesId, CoordinatesIndex, PipelineNamespaceId};
use dom_struct::dom_struct;
use js::rust::HandleObject;
use script_traits::serializable::Coordinates;

use crate::dom::bindings::codegen::Bindings::DOMPointBinding::DOMPointInit;
use crate::dom::bindings::codegen::Bindings::DOMPointReadOnlyBinding::DOMPointReadOnlyMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::serializable::{Serializable, StorageKey};
use crate::dom::bindings::structuredclone::StructuredDataHolder;
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
}

#[allow(non_snake_case)]
impl DOMPointReadOnlyMethods for DOMPointReadOnly {
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
    fn FromPoint(global: &GlobalScope, init: &DOMPointInit) -> DomRoot<Self> {
        Self::new(global, init.x, init.y, init.z, init.w)
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

impl Serializable for DOMPointReadOnly {
    /// <https://w3c.github.io/FileAPI/#ref-for-serialization-steps>
    fn serialize(&self, sc_holder: &mut StructuredDataHolder) -> Result<StorageKey, ()> {
        let coordinates_impl = match sc_holder {
            StructuredDataHolder::Write { coordinates, .. } => coordinates,
            _ => panic!("Unexpected variant of StructuredDataHolder"),
        };

        // We clone the data, but the clone gets its own Id.
        let (coordinates, id) =
            Coordinates::new_from_points(self.X(), self.Y(), self.Z(), self.W());

        // 2. Store the object at a given key.
        let store = coordinates_impl.get_or_insert_with(HashMap::new);
        store.insert(id, coordinates);

        let PipelineNamespaceId(name_space) = id.namespace_id;
        let CoordinatesIndex(index) = id.index;
        let index = index.get();

        let name_space = name_space.to_ne_bytes();
        let index = index.to_ne_bytes();

        let storage_key = StorageKey {
            index: u32::from_ne_bytes(index),
            name_space: u32::from_ne_bytes(name_space),
        };

        // 3. Return the storage key.
        Ok(storage_key)
    }

    /// <https://w3c.github.io/FileAPI/#ref-for-deserialization-steps>
    fn deserialize(
        owner: &GlobalScope,
        sc_holder: &mut StructuredDataHolder,
        storage_key: StorageKey,
    ) -> Result<(), ()> {
        // 1. Re-build the key for the storage location
        // of the serialized object.
        let namespace_id = PipelineNamespaceId(storage_key.name_space);
        let index = CoordinatesIndex(
            NonZeroU32::new(storage_key.index).expect("Deserialized index is zero"),
        );

        let id = CoordinatesId {
            namespace_id,
            index,
        };

        let (dom_point_read_only, coordinates) = match sc_holder {
            StructuredDataHolder::Read {
                dom_point_read_only,
                coordinates,
                ..
            } => (dom_point_read_only, coordinates),
            _ => panic!("Unexpected variant of StructuredDataHolder"),
        };

        // 2. Get the transferred object from its storage, using the key.
        let coordinates_map = coordinates
            .as_mut()
            .expect("The SC holder does not have any coordinates");
        let coordinates_impl = coordinates_map
            .remove(&id)
            .expect("No coordinates to be deserialized found.");
        if coordinates_map.is_empty() {
            *coordinates = None;
        }

        let deserialized = DOMPointReadOnly::new(
            owner,
            coordinates_impl.x,
            coordinates_impl.y,
            coordinates_impl.z,
            coordinates_impl.w,
        );

        let points = dom_point_read_only.get_or_insert_with(HashMap::new);
        points.insert(storage_key, deserialized);

        Ok(())
    }
}
