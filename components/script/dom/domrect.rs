/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::DOMRectBinding::DOMRectMethods;
use crate::dom::bindings::codegen::Bindings::DOMRectReadOnlyBinding::{
    DOMRectInit, DOMRectReadOnlyMethods,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::domrectreadonly::{DOMRectReadOnly, create_a_domrectreadonly_from_the_dictionary};
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct DOMRect {
    rect: DOMRectReadOnly,
}

impl DOMRect {
    fn new_inherited(x: f64, y: f64, width: f64, height: f64) -> DOMRect {
        DOMRect {
            rect: DOMRectReadOnly::new_inherited(x, y, width, height),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        can_gc: CanGc,
    ) -> DomRoot<DOMRect> {
        Self::new_with_proto(global, None, x, y, width, height, can_gc)
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        can_gc: CanGc,
    ) -> DomRoot<DOMRect> {
        reflect_dom_object_with_proto(
            Box::new(DOMRect::new_inherited(x, y, width, height)),
            global,
            proto,
            can_gc,
        )
    }
}

impl DOMRectMethods<crate::DomTypeHolder> for DOMRect {
    // https://drafts.fxtf.org/geometry/#dom-domrect-domrect
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Fallible<DomRoot<DOMRect>> {
        Ok(DOMRect::new_with_proto(
            global, proto, x, y, width, height, can_gc,
        ))
    }

    // https://drafts.fxtf.org/geometry/#dom-domrect-fromrect
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn FromRect(global: &GlobalScope, other: &DOMRectInit) -> DomRoot<DOMRect> {
        let rect = create_a_domrectreadonly_from_the_dictionary(other);

        reflect_dom_object(Box::new(Self { rect }), global, CanGc::note())
    }

    // https://drafts.fxtf.org/geometry/#dom-domrect-x
    fn X(&self) -> f64 {
        self.rect.X()
    }

    // https://drafts.fxtf.org/geometry/#dom-domrect-x
    fn SetX(&self, value: f64) {
        self.rect.set_x(value);
    }

    // https://drafts.fxtf.org/geometry/#dom-domrect-y
    fn Y(&self) -> f64 {
        self.rect.Y()
    }

    // https://drafts.fxtf.org/geometry/#dom-domrect-y
    fn SetY(&self, value: f64) {
        self.rect.set_y(value);
    }

    // https://drafts.fxtf.org/geometry/#dom-domrect-width
    fn Width(&self) -> f64 {
        self.rect.Width()
    }

    // https://drafts.fxtf.org/geometry/#dom-domrect-width
    fn SetWidth(&self, value: f64) {
        self.rect.set_width(value);
    }

    // https://drafts.fxtf.org/geometry/#dom-domrect-height
    fn Height(&self) -> f64 {
        self.rect.Height()
    }

    // https://drafts.fxtf.org/geometry/#dom-domrect-height
    fn SetHeight(&self, value: f64) {
        self.rect.set_height(value);
    }
}
