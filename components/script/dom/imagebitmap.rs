/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// the following are the statements
use crate::dom::bindings::codegen::Bindings::CanvasGradientBinding;
use crate::dom::bindings::error::Error;
use crate::dom::globalscope::GlobalScope;
use crate::dom::bindings::serializable::Serializable;
// There should be an implmentation under this like in blob.rs
//use script_traits::serializable
// for the reflector to be used
use crate::dom::bindings::reflector::Reflector;
//not sure if these two work
use crate::dom::bindings::codegen::Bindings::CanvasPatternBinding;
use crate::dom::bindings::codegen::Bindings::ImageBitMapBinding;
// dom_struct things
// use crate::dom::bindings::ImageOrientation;
// use crate::dom::bindings::PremultiplyAlpha;
use dom_struct::dom_struct;
use std::vec::Vec;


#[dom_struct]
pub struct ImageBitMap{
	// reflector connects this to the JS world
	reflector_:Reflector,
	// the following are properties
	resizeWidth: u64,
	resizeHeight: u64,
	// the following I am not sure of - from the webidl file. 
	// Guessing all of this is required here as I am guessing it is required to be in the DOM so it can be modified by JS.
	// **************QUESTIONS!! - these things are already there. When I click enter in use crates they show up***************
	/*
	ImageOrientation imageOrientation = "none";
	PremultiplyAlpha premultiplyAlpha = "default";
	ColorSpaceConversion colorSpaceConversion = "default";
	[EnforceRange] unsigned long resizeWidth;
	[EnforceRange] unsigned long resizeHeight;
	ResizeQuality resizeQuality = "low";
	*/
}

//#[allow (dead_code)]
impl ImageBitMap {
pub fn new_inherited(resizeWidth: u64, resizeHeight: u64) -> ImageBitMap{
	ImageBitMap{
		reflector_: Reflector::new(),
		// add more required things here when confirmed about dictionary here
	}
}
// this is called by new_inherited. There could be more arguments - commented out part above
pub fn new (global: &GlobalScope, resizeWidth: u64, resizeHeight :u64) -> DOMRoot<ImageBitMap>{
// change the arguments below if there are any changes
	reflect_dom_object(box ImageBitMap::new_inherited(resizeWidth, resizeHeight), global, ImageBitMapBinding::Wrap);
}
// this is called by the JS world - Constructor is probably not required as it is not mentioned in the spec
//pub fn Constructor(global: &GlobalScope){
//}
}


// uncomment on work on it when required
//impl Serializable for ImageBitMap{
//}

// uncomment when working on it
//impl ImageBitMapMethods for ImageBitMap{
//}