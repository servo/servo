/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, Reflectable, Reflector, reflect_dom_object};
use dom::bindings::utils::Fallible;
use dom::bindings::codegen::CanvasRenderingContext2DBinding;
use dom::window::Window;
use dom::bindings::utils::{ErrorResult};

pub struct CanvasRenderingContext2D {
    reflector_: Reflector,
    window: @mut Window,
}

impl CanvasRenderingContext2D {
    pub fn new_inherited(window: @mut Window) -> CanvasRenderingContext2D {
        CanvasRenderingContext2D {
            reflector_: Reflector::new(),
            window: window,
        }
    }

    pub fn new(window: @mut Window) -> @mut CanvasRenderingContext2D {
        reflect_dom_object(@mut CanvasRenderingContext2D::new_inherited(window), window, CanvasRenderingContext2DBinding::Wrap)
    }

    pub fn Width(&self) -> u32 {                    
        0
    }

    pub fn SetWidth(&mut self, _width: u32) -> ErrorResult {
        Ok(())
    }

    pub fn Height(&self) -> u32 {                
        0
    }

    pub fn SetHeight(&mut self, _height: u32) -> ErrorResult {
        Ok(())
    }

}

impl Reflectable for CanvasRenderingContext2D {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}
