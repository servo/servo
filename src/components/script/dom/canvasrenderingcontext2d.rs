use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::utils::Fallible;
use dom::bindings::codegen::CanvasRenderingContext2DBinding;
use dom::window::Window;
use js::jsapi::{JSContext, JSObject};
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

impl CanvasRenderingContext2D {
    pub fn Constructor(window: @mut Window) -> Fallible<@mut CanvasRenderingContext2D> {
        Ok(CanvasRenderingContext2D::new(window))
    }
}



impl Reflectable for CanvasRenderingContext2D {           
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }

    fn wrap_object_shared(@mut self, _cx: *JSContext, _scope: *JSObject) -> *JSObject {
        unreachable!();
    }

    fn GetParentObject(&self, _cx: *JSContext) -> Option<@mut Reflectable> {
        Some(self.window as @mut Reflectable)
    }

}
