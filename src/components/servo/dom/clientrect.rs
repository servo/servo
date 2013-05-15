/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::WrapperCache;

pub struct ClientRect {
    wrapper: WrapperCache,
    top: f32,
    bottom: f32,
    left: f32,
    right: f32,
}

impl ClientRect {
    pub fn new(top: f32, bottom: f32, left: f32, right: f32) -> @mut ClientRect {
        let rect = @mut ClientRect {
            top: top,
            bottom: bottom,
            left: left,
            right: right,
            wrapper: WrapperCache::new()
        };
        rect.init_wrapper();
        rect
    }

    pub fn Top(&self) -> f32 {
        self.top
    }

    pub fn Bottom(&self) -> f32 {
        self.bottom
    }

    pub fn Left(&self) -> f32 {
        self.left
    }

    pub fn Right(&self) -> f32 {
        self.right
    }

    pub fn Width(&self) -> f32 {
        f32::abs(self.right - self.left)
    }

    pub fn Height(&self) -> f32 {
        f32::abs(self.bottom - self.top)
    }
}

