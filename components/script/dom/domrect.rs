/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DOMRectBinding;
use dom::bindings::codegen::Bindings::DOMRectBinding::DOMRectMethods;
use dom::bindings::codegen::Bindings::DOMRectReadOnlyBinding::DOMRectReadOnlyMethods;
use dom::bindings::error::Fallible;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::domrectreadonly::DOMRectReadOnly;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct DOMRect<TH: TypeHolderTrait> {
    rect: DOMRectReadOnly<TH>,
}

impl<TH: TypeHolderTrait> DOMRect<TH> {
    fn new_inherited(x: f64, y: f64, width: f64, height: f64) -> DOMRect<TH> {
        DOMRect {
            rect: DOMRectReadOnly::new_inherited(x, y, width, height),
        }
    }

    pub fn new(global: &GlobalScope<TH>, x: f64, y: f64, width: f64, height: f64) -> DomRoot<DOMRect<TH>> {
        reflect_dom_object(Box::new(DOMRect::new_inherited(x, y, width, height)),
                           global,
                           DOMRectBinding::Wrap)
    }

    pub fn Constructor(global: &GlobalScope<TH>,
                       x: f64,
                       y: f64,
                       width: f64,
                       height: f64)
                       -> Fallible<DomRoot<DOMRect<TH>>> {
        Ok(DOMRect::new(global, x, y, width, height))
    }
}

impl<TH: TypeHolderTrait> DOMRectMethods for DOMRect<TH> {
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
