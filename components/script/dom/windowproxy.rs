use dom::bindings::codegen::Bindings::WindowProxyBinding::{self, WindowProxyMethods};
use dom::bindings::codegen::Bindings::WindowBinding::{WindowMethods};
use dom::bindings::global::{GlobalRef};
use dom::bindings::reflector::{Reflectable};
use dom::window::Window;
use dom::document::Document;
use dom::location::Location;
use dom::bindings::js::{Root};
use std::rc::Rc;
use js::rust::Runtime;

#[dom_struct]
pub struct WindowProxy {
    window: Rc<Window>
}

impl WindowProxy {
    pub fn new(runtime: Rc<Runtime>, window: Rc<Window> )
    -> Root<WindowProxy> {
        let mut moved = window;

        let win = box WindowProxy {
            window: moved.clone()
        };
        WindowProxyBinding::Wrap(runtime.cx(), GlobalRef::Window(moved.clone().as_ref()), win)
    }
}


impl WindowProxyMethods for WindowProxy {
    fn Document(&self) -> Root<Document> {
        self.window.Document()
    }
    fn Location(&self) -> Root<Location> {
        self.window.Location()
    }
}
