
use dom::bindings::utils::{Reflectable,Reflector};
use dom::bindings::trace::JSTraceable;
use util::mem::HeapSizeOf;
use js::jsapi::JSTracer;

#[derive(HeapSizeOf)]
#[must_root]
pub struct ServoXMLParser{
	reflector_: Reflector,
}

impl ServoXMLParser {
pub fn new(){
       
}
}

impl Reflectable for ServoXMLParser{
	fn reflector<'a>(&'a self) -> &'a Reflector{
		&self.reflector_
	}

	fn init_reflector(&mut self, obj: *mut JSObject){
	}
}

impl JSTraceable for ServoXMLParser{
	fn trace(&self, trc: *mut JSTracer){
	}
}

