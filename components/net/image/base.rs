/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::iter::range_step;
use servo_image;
use png;

// FIXME: Images must not be copied every frame. Instead we should atomically
// reference count them.
pub type Image = png::Image;
pub type DynamicImage = servo_image::DynamicImage;


static TEST_IMAGE: &'static [u8] = include_bin!("test.jpeg");

pub fn test_image_bin() -> Vec<u8> {
    TEST_IMAGE.iter().map(|&x| x).collect()
}

// TODO(pcwalton): Speed up with SIMD, or better yet, find some way to not do this.
fn byte_swap(data: &mut [u8]) {
    let length = data.len();
    for i in range_step(0, length, 4) {
        let r = data[i + 2];
        data[i + 2] = data[i + 0];
        data[i + 0] = r;
    }
}

// TODO(pcwalton): Speed up with SIMD, or better yet, find some way to not do this.
fn byte_swap_and_premultiply(data: &mut [u8]) {
    let length = data.len();
    for i in range_step(0, length, 4) {
        let r = data[i + 2];
        let g = data[i + 1];
        let b = data[i + 0];
        let a = data[i + 3];
        data[i + 0] = ((r as u32) * (a as u32) / 255) as u8;
        data[i + 1] = ((g as u32) * (a as u32) / 255) as u8;
        data[i + 2] = ((b as u32) * (a as u32) / 255) as u8;
    }
}

pub fn load_from_memory(buffer: &[u8],ext: &str) -> Option<DynamicImage> {
    if buffer.len() == 0 {
        return None;
    }
   else {
	//let new_image_type: servo_image::ImageFormat = servo_image::ImageFormat::PNG;
   
        let image_type: Option< servo_image::ImageFormat > = get_format(ext);
	if image_type == None
	{
	panic!("Image format not supported!");
	}
	else{
        let new_image_type: servo_image::ImageFormat = image_type.unwrap();
        
        
	let result = servo_image::load_from_memory(buffer,new_image_type);
	if (result.is_ok()) {
  	    let v = result.unwrap();
  	    return Some(v);
	}
	else  {	
	    return None;
	}		
   }
   }
   }
fn get_format(ext: &str) -> Option<servo_image::ImageFormat> {
		match ext.to_ascii().to_uppercase().as_str_ascii() {
    "PNG" => {(return Some(servo_image::ImageFormat::PNG));},
    "JPEG" => {(return Some(servo_image::ImageFormat::JPEG));},
    "JPG" => {(return Some(servo_image::ImageFormat::JPEG));},
    "GIF"=> {(return Some(servo_image::ImageFormat::GIF));},
    "WEBP" => {(return Some(servo_image::ImageFormat::WEBP));},
    "PPM" => {(return Some(servo_image::ImageFormat::PPM));},
    _ => {return None ;},
		}
	}

