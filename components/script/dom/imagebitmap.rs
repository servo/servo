/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// the following are the statements
use crate::dom::bindings::codegen::Bindings::CanvasGradientBinding;
use crate::dom::bindings::error::Error;
use crate::dom::globalscope::GlobalScope;
use crate::dom::bindings::serializable::Serializable;
// There should be an implmentation under this like in blob.rs
use script_traits::serializable::Serializable;
use crate dom::domexception::DOMException;
use crate::dom::bindings::callback::ExceptionHandling;
// for the reflector to be used
use crate::dom::bindings::reflector::Reflector;
//not sure if these two work
use crate::dom::bindings::codegen::Bindings::CanvasPatternBinding;
use crate::dom::bindings::codegen::Bindings::ImageBitMapBinding;
// in the serializable with the imagebitmap_impls
use crate::dom::bindings::structuredclone::StructuredDataHolder;
// dom_struct things
// use crate::dom::bindings::ImageOrientation;
// use crate::dom::bindings::PremultiplyAlpha;
// use crate::dom::bindings::ColorSpaceConversion;
// use crate::dom::bindings::ResizeQuality;
// use dom_struct::dom_struct;
use std::vec::Vec;


#[dom_struct]
pub struct ImageBitMap{
	reflector_:Reflector,
	width: u64,
	height: u64,
	// removing them as I don't think these are mentioned in HTML
	//origin_clean: bool,
	//imagebitmap_id: ImageBitMapId,
}

//#[allow (dead_code)]
impl ImageBitMap {
	pub fn new_inherited(width: u64, height: u64, imagebitmap_impl: &ImageBitMapImpl) -> ImageBitMap {
		ImageBitMap{
			reflector_: Reflector::new(),
			imagebitmap_id: imagebitmap_impl.imagebitmap_id(),
		}
	}

	//Note: might throw an error as the DOMRoot does not have id
	pub fn new (global: &GlobalScope, resizeWidth: u64, resizeHeight :u64) -> DOMRoot<ImageBitMap> {
		let dom_imagebitmap = reflect_dom_object(
			//it should be new_inherited instead but I am not sure what will take imagebitmap_impl and blob has it the new way
			//box ImageBitMap::new_inherited(resizeWidth, resizeHeight), global, ImageBitMapBinding::Wrap);
			Box::new(ImageBitMap {
						reflect_: Reflector::new(),
						imagebitmap_id: imagebitmap_impl.imagebitmap_id()
			}),
			global,
			ImageBitMapBinding::Wrap,
		);
		global.track_imagebitmap(&dom_imagebitmap, imagebitmap_impl);
		dom_imagebitmap
	}
}
