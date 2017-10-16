/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DOMRectReadOnlyBinding::{DOMRectReadOnlyMethods, Wrap};
use dom::bindings::error::Fallible;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use std::cell::Cell;

#[dom_struct]
pub struct DOMRectReadOnly {
    reflector_: Reflector,
    x: Cell<f64>,
    y: Cell<f64>,
    width: Cell<f64>,
    height: Cell<f64>,
}

impl DOMRectReadOnly {
    pub fn new_inherited(x: f64, y: f64, width: f64, height: f64) -> DOMRectReadOnly {
        DOMRectReadOnly {
            x: Cell::new(x),
            y: Cell::new(y),
            width: Cell::new(width),
            height: Cell::new(height),
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: &GlobalScope,
               x: f64,
               y: f64,
               width: f64,
               height: f64)
               -> DomRoot<DOMRectReadOnly> {
        reflect_dom_object(Box::new(DOMRectReadOnly::new_inherited(x, y, width, height)),
                           global,
                           Wrap)
    }

    pub fn Constructor(global: &GlobalScope,
                       x: f64,
                       y: f64,
                       width: f64,
                       height: f64)
                       -> Fallible<DomRoot<DOMRectReadOnly>> {
        Ok(DOMRectReadOnly::new(global, x, y, width, height))
    }

    pub fn set_x(&self, value: f64) {
        self.x.set(value);
    }

    pub fn set_y(&self, value: f64) {
        self.y.set(value);
    }

    pub fn set_width(&self, value: f64) {
        self.width.set(value);
    }

    pub fn set_height(&self, value: f64) {
        self.height.set(value);
    }
}

impl DOMRectReadOnlyMethods for DOMRectReadOnly {
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
