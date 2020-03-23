use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ImageBitmapBinding;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use dom_struct::dom_struct;
use js::jsapi::JSObject;
use std::vec::Vec;

#[dom_struct]
pub struct ImageBitmap {
    reflector_: Reflector,
    width: u32,
    height: u32,
    ibm_vector: DomRefCell<Vec<u8>>,
}


impl ImageBitmap {
    fn new_inherited(width_arg: u32, height_arg: u32) -> ImageBitmap {
        ImageBitmap {
            reflector_: Reflector::new(),
            width: width_arg,
            height: height_arg,
            bitmap_data: DomRefCell::new(vec![]),
        }
    }

	#[allow (dead_code)]
    pub fn new(global: &GlobalScope, width: u32, height: u32) -> Fallible<DomRoot<ImageBitmap>> {
		let imagebitmap = Box::new(ImageBitmap::new_inherited(width, height));

        Ok(reflect_dom_object(
            imagebitmap,
            global,
        ))
    }
}

impl ImageBitmapMethods for ImageBitmap {
    // https://html.spec.whatwg.org/multipage/#dom-imagebitmap-height
    fn Height(&self) -> u32 {
        //to do: add a condition for checking detached internal slot
        //and return 0 if set to true
        self.height
    }

    // https://html.spec.whatwg.org/multipage/#dom-imagebitmap-width
    fn Width(&self) -> u32 {
        //to do: add a condition to check detached internal slot
        ////and return 0 if set to true
        self.width
    }
}
