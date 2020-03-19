/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// the following are the statements
use crate::dom::bindings::codegen::Bindings::CanvasGradientBinding;
use crate::dom::bindings::error::Error;
use crate::dom::globalscope::GlobalScope;
use crate::dom::bindings::cell::DOMRefCell;
use crate::dom::bindings::ImageBitMapBinding::{ImageBitMapMethods}
//use crate::dom::bindings::serializable::Serializable;
//use crate::dom::bindings::transferable::Transferable;
// There should be an implmentation under this like in blob.rs
//use script_traits::serializable::Serializable;
//use script_traits::transferable;
use crate dom::domexception::DOMException;
use crate::dom::bindings::callback::ExceptionHandling;
// for the reflector to be used
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
//not sure if these two work
use crate::dom::bindings::codegen::Bindings::CanvasPatternBinding;
use crate::dom::bindings::codegen::Bindings::ImageBitMapBinding;
// in the serializable with the imagebitmap_impls
//use crate::dom::bindings::structuredclone::StructuredDataHolder;
// dom_struct things
// use crate::dom::bindings::ImageOrientation;
// use crate::dom::bindings::PremultiplyAlpha;
// use crate::dom::bindings::ColorSpaceConversion;
// use crate::dom::bindings::ResizeQuality;
// use dom_struct::dom_struct;
use dom_struct::dom_struct; 
use std::vec::Vec;

//as mentioned in bluetoothuuid.rs
//pub type ImageBitMapSource = 

#[dom_struct]
pub struct ImageBitMap{
	reflector_:Reflector,
	width: u64,
	height: u64,
	ibm_vector: DOMRefCell
}

//#[allow (dead_code)]
impl ImageBitMap {
	pub fn new_inherited(width: u64, height: u64) -> ImageBitMap {
		ImageBitMap{
			reflector_: Reflector::new(),
		}
	}

	pub fn new (global: &GlobalScope, width: u64, height :u64) -> DOMRoot<ImageBitMap> {
		reflect_dom_object(box ImageBitMap::new_inherited(width, height), global, ImageBitMapBinding::Wrap);
	}
}

// uncomment when working on it
impl ImageBitMapMethods for ImageBitMap{
	fn Height(&self) -> u64 {
		//to do: add a condition for checking detached internal slot
        return self.height.get();
    }

	fn Width(&self) -> u64 {
		//to do: add a condition to check detached internal slot
        return self.width.get();
    }


}