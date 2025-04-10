/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::DOMRectReadOnlyBinding::{
    DOMRectInit, DOMRectReadOnlyMethods,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{
    Reflector, reflect_dom_object, reflect_dom_object_with_proto,
};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct DOMRectReadOnly {
    reflector_: Reflector,
    x: Cell<f64>,
    y: Cell<f64>,
    width: Cell<f64>,
    height: Cell<f64>,
}

impl DOMRectReadOnly {
    pub(crate) fn new_inherited(x: f64, y: f64, width: f64, height: f64) -> DOMRectReadOnly {
        DOMRectReadOnly {
            x: Cell::new(x),
            y: Cell::new(y),
            width: Cell::new(width),
            height: Cell::new(height),
            reflector_: Reflector::new(),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        can_gc: CanGc,
    ) -> DomRoot<DOMRectReadOnly> {
        reflect_dom_object_with_proto(
            Box::new(DOMRectReadOnly::new_inherited(x, y, width, height)),
            global,
            proto,
            can_gc,
        )
    }

    pub(crate) fn new_from_dictionary(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        dictionary: &DOMRectInit,
        can_gc: CanGc,
    ) -> DomRoot<DOMRectReadOnly> {
        reflect_dom_object_with_proto(
            Box::new(create_a_domrectreadonly_from_the_dictionary(dictionary)),
            global,
            proto,
            can_gc,
        )
    }

    pub(crate) fn set_x(&self, value: f64) {
        self.x.set(value);
    }

    pub(crate) fn set_y(&self, value: f64) {
        self.y.set(value);
    }

    pub(crate) fn set_width(&self, value: f64) {
        self.width.set(value);
    }

    pub(crate) fn set_height(&self, value: f64) {
        self.height.set(value);
    }
}

impl DOMRectReadOnlyMethods<crate::DomTypeHolder> for DOMRectReadOnly {
    // https://drafts.fxtf.org/geometry/#dom-domrectreadonly-domrectreadonly
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Fallible<DomRoot<DOMRectReadOnly>> {
        Ok(DOMRectReadOnly::new(
            global, proto, x, y, width, height, can_gc,
        ))
    }

    // https://drafts.fxtf.org/geometry/#dom-domrectreadonly-fromrect
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn FromRect(
        global: &GlobalScope,
        other: &DOMRectInit,
        can_gc: CanGc,
    ) -> DomRoot<DOMRectReadOnly> {
        let dom_rect = create_a_domrectreadonly_from_the_dictionary(other);

        reflect_dom_object(Box::new(dom_rect), global, can_gc)
    }

    // https://drafts.fxtf.org/geometry/#dom-domrectreadonly-x
    fn X(&self) -> f64 {
        self.x.get()
    }

    // https://drafts.fxtf.org/geometry/#dom-domrectreadonly-y
    fn Y(&self) -> f64 {
        self.y.get()
    }

    // https://drafts.fxtf.org/geometry/#dom-domrectreadonly-width
    fn Width(&self) -> f64 {
        self.width.get()
    }

    // https://drafts.fxtf.org/geometry/#dom-domrectreadonly-height
    fn Height(&self) -> f64 {
        self.height.get()
    }

    // https://drafts.fxtf.org/geometry/#dom-domrectreadonly-top
    fn Top(&self) -> f64 {
        let height = self.height.get();
        if height >= 0f64 {
            self.y.get()
        } else {
            self.y.get() + height
        }
    }

    // https://drafts.fxtf.org/geometry/#dom-domrectreadonly-right
    fn Right(&self) -> f64 {
        let width = self.width.get();
        if width < 0f64 {
            self.x.get()
        } else {
            self.x.get() + width
        }
    }

    // https://drafts.fxtf.org/geometry/#dom-domrectreadonly-bottom
    fn Bottom(&self) -> f64 {
        let height = self.height.get();
        if height < 0f64 {
            self.y.get()
        } else {
            self.y.get() + height
        }
    }

    // https://drafts.fxtf.org/geometry/#dom-domrectreadonly-left
    fn Left(&self) -> f64 {
        let width = self.width.get();
        if width >= 0f64 {
            self.x.get()
        } else {
            self.x.get() + width
        }
    }
}

/// <https://drafts.fxtf.org/geometry/#ref-for-create-a-domrectreadonly-from-the-dictionary>
#[cfg_attr(crown, allow(crown::unrooted_must_root))]
pub(super) fn create_a_domrectreadonly_from_the_dictionary(other: &DOMRectInit) -> DOMRectReadOnly {
    // NOTE: We trivially combine all three steps into one

    // Step 1. Let rect be a new DOMRectReadOnly or DOMRect as appropriate.

    // Step 2. Set rect’s variables x coordinate to other’s x dictionary member, y coordinate to other’s y
    // dictionary member, width dimension to other’s width dictionary member and height dimension to
    // other’s height dictionary member.

    // Step 3. Return rect.

    DOMRectReadOnly {
        reflector_: Reflector::new(),
        x: Cell::new(other.x),
        y: Cell::new(other.y),
        width: Cell::new(other.width),
        height: Cell::new(other.height),
    }
}
