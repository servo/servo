/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![crate_name = "webrender_traits"]
#![crate_type = "rlib"]
#![deny(unsafe_code)]

use euclid::Size2D;
use std::collections::HashMap;

/// This trait is used as a bridge between the different GL clients
/// in Servo that handles WebRender ExternalImages and the WebRender
/// ExternalImageHandler API.
//
/// This trait is used to notify lock/unlock messages and get the
/// required info that WR needs.
pub trait WebrenderExternalImageApi {
    fn lock(&mut self, id: u64) -> (u32, Size2D<i32>);
    fn unlock(&mut self, id: u64);
}

/// Type of Webrender External Image Handler.
pub enum WebrenderImageHandlerType {
    WebGL,
    Media,
}

/// WebRender External Image Handler implementation.
pub struct WebrenderExternalImageHandler {
    webgl_handler: Option<Box<dyn WebrenderExternalImageApi>>,
    media_handler: Option<Box<dyn WebrenderExternalImageApi>>,
    //XXX(ferjm) register external images.
    external_images: HashMap<webrender_api::ExternalImageId, WebrenderImageHandlerType>,
}

impl WebrenderExternalImageHandler {
    pub fn new() -> Self {
        Self {
            webgl_handler: None,
            media_handler: None,
            external_images: HashMap::new(),
        }
    }

    pub fn set_handler(
        &mut self,
        handler: Box<dyn WebrenderExternalImageApi>,
        handler_type: WebrenderImageHandlerType,
    ) {
        match handler_type {
            WebrenderImageHandlerType::WebGL => self.webgl_handler = Some(handler),
            WebrenderImageHandlerType::Media => self.media_handler = Some(handler),
        }
    }
}

impl webrender::ExternalImageHandler for WebrenderExternalImageHandler {
    /// Lock the external image. Then, WR could start to read the
    /// image content.
    /// The WR client should not change the image content until the
    /// unlock() call.
    fn lock(
        &mut self,
        key: webrender_api::ExternalImageId,
        _channel_index: u8,
        _rendering: webrender_api::ImageRendering,
    ) -> webrender::ExternalImage {
        if let Some(handler_type) = self.external_images.get(&key) {
            // It is safe to unwrap the handlers here because we forbid registration
            // for specific types that has no handler set.
            // XXX(ferjm) make this ^ true.
            let (texture_id, size) = match handler_type {
                WebrenderImageHandlerType::WebGL => {
                    self.webgl_handler.as_mut().unwrap().lock(key.0)
                },
                WebrenderImageHandlerType::Media => {
                    self.media_handler.as_mut().unwrap().lock(key.0)
                },
            };
            webrender::ExternalImage {
                uv: webrender_api::TexelRect::new(0.0, 0.0, size.width as f32, size.height as f32),
                source: webrender::ExternalImageSource::NativeTexture(texture_id),
            }
        } else {
            unreachable!()
        }
    }

    /// Unlock the external image. The WR should not read the image
    /// content after this call.
    fn unlock(&mut self, key: webrender_api::ExternalImageId, _channel_index: u8) {
        if let Some(handler_type) = self.external_images.get(&key) {
            // It is safe to unwrap the handlers here because we forbid registration
            // for specific types that has no handler set.
            match handler_type {
                WebrenderImageHandlerType::WebGL => {
                    self.webgl_handler.as_mut().unwrap().unlock(key.0)
                },
                WebrenderImageHandlerType::Media => {
                    self.media_handler.as_mut().unwrap().unlock(key.0)
                },
            };
        } else {
            unreachable!();
        }
    }
}
