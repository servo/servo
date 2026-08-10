/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::CString;
use std::ptr;

use ohos_image_kit_sys::native_image::common::{Image_String, ImageResult as OhosImageResult};
use ohos_image_kit_sys::native_image::image_packer::{
    OH_ImagePackerNative_Create, OH_ImagePackerNative_PackToDataFromPixelmap,
    OH_ImagePackerNative_Release, OH_PackingOptions_Create, OH_PackingOptions_Release,
    OH_PackingOptions_SetMimeType, OH_PackingOptions_SetQuality,
};
use ohos_image_kit_sys::native_image::pixelmap::{
    OH_PixelmapInitializationOptions_Create, OH_PixelmapInitializationOptions_Release,
    OH_PixelmapInitializationOptions_SetHeight, OH_PixelmapInitializationOptions_SetWidth,
    OH_PixelmapNative_CreatePixelmap, OH_PixelmapNative_Release,
};

use crate::encoding::{EncodedImageType, ServoImageEncoder};

pub(crate) struct OhosImageEncoder {}

impl<W: std::io::Write> ServoImageEncoder<W> for OhosImageEncoder {
    type Error = ();
    fn encode_to_writer(
        data: &[u8],
        image_type: &crate::encoding::EncodedImageType,
        width: u32,
        height: u32,
        mut writer: W,
        quality: Option<f64>,
    ) -> Result<(), ()> {
        unsafe {
            let mut image_packer = ptr::null_mut();
            let res = OH_ImagePackerNative_Create(&raw mut image_packer);
            if res != OhosImageResult::SUCCESS || image_packer.is_null() {
                return Err(());
            }
            let mut packing_options = ptr::null_mut();
            let res = OH_PackingOptions_Create(&raw mut packing_options);
            if res != OhosImageResult::SUCCESS || packing_options.is_null() {
                OH_ImagePackerNative_Release(image_packer);
                return Err(());
            }

            let mime_type = match image_type {
                EncodedImageType::Png => CString::new("image/png").map_err(|_| ())?,
                EncodedImageType::Jpeg => CString::new("image/jpg").map_err(|_| ())?,
                EncodedImageType::Webp => CString::new("image/webp").map_err(|_| ())?,
            };

            // this should be read only
            let mut mime_type = Image_String {
                data: mime_type.as_ptr() as *mut u8,
                size: mime_type.count_bytes(),
            };

            let res = OH_PackingOptions_SetMimeType(packing_options, &raw mut mime_type);
            if res != OhosImageResult::SUCCESS {
                OH_PackingOptions_Release(packing_options);
                OH_ImagePackerNative_Release(image_packer);
                return Err(());
            }

            if let Some(quality) = quality {
                let quality = (quality * 100.0).abs().round() as u32;
                let res = OH_PackingOptions_SetQuality(packing_options, quality);
                if res != OhosImageResult::SUCCESS {
                    OH_PackingOptions_Release(packing_options);
                    OH_ImagePackerNative_Release(image_packer);
                    return Err(());
                }
            }

            let mut pixmap_options = ptr::null_mut();
            let res = OH_PixelmapInitializationOptions_Create(&raw mut pixmap_options);
            if res != OhosImageResult::SUCCESS || pixmap_options.is_null() {
                OH_PackingOptions_Release(packing_options);
                OH_ImagePackerNative_Release(image_packer);
                return Err(());
            }
            let res = OH_PixelmapInitializationOptions_SetHeight(pixmap_options, height);
            if res != OhosImageResult::SUCCESS {
                OH_PixelmapInitializationOptions_Release(pixmap_options);
                OH_PackingOptions_Release(packing_options);
                OH_ImagePackerNative_Release(image_packer);
                return Err(());
            }
            let res = OH_PixelmapInitializationOptions_SetWidth(pixmap_options, width);
            if res != OhosImageResult::SUCCESS {
                OH_PixelmapInitializationOptions_Release(pixmap_options);
                OH_PackingOptions_Release(packing_options);
                OH_ImagePackerNative_Release(image_packer);
                return Err(());
            }
            let mut pixmap = ptr::null_mut();
            // The data should be never written to and the *mut is just a binding artefact.
            let res = OH_PixelmapNative_CreatePixelmap(
                data.as_ptr() as *mut u8,
                data.len(),
                pixmap_options,
                &raw mut pixmap,
            );
            if res != OhosImageResult::SUCCESS || pixmap.is_null() {
                OH_PixelmapInitializationOptions_Release(pixmap_options);
                OH_PackingOptions_Release(packing_options);
                OH_ImagePackerNative_Release(image_packer);
                return Err(());
            }

            let mut out_buffer = vec![0_u8; (width * height * 4) as usize];
            let mut buffer_size = out_buffer.len();
            OH_ImagePackerNative_PackToDataFromPixelmap(
                image_packer,
                packing_options,
                pixmap,
                out_buffer.as_mut_ptr(),
                &raw mut buffer_size,
            );
            out_buffer.truncate(buffer_size);

            writer.write_all(&out_buffer).map_err(|_| ())?;

            OH_PixelmapInitializationOptions_Release(pixmap_options);
            OH_PixelmapNative_Release(pixmap);
            OH_PackingOptions_Release(packing_options);
            OH_ImagePackerNative_Release(image_packer);

            Ok(())
        }
    }
}
