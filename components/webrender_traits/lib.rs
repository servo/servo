/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

use euclid::default::Size2D;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use webrender_api::units::TexelRect;

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

/// List of Webrender external images to be shared among all external image
/// consumers (WebGL, Media).
/// It ensures that external image identifiers are unique.
pub struct WebrenderExternalImageRegistry {
    /// Map of all generated external images.
    external_images: HashMap<webrender_api::ExternalImageId, WebrenderImageHandlerType>,
    /// Id generator for the next external image identifier.
    next_image_id: u64,
}

impl WebrenderExternalImageRegistry {
    pub fn new() -> Self {
        Self {
            external_images: HashMap::new(),
            next_image_id: 0,
        }
    }

    pub fn next_id(
        &mut self,
        handler_type: WebrenderImageHandlerType,
    ) -> webrender_api::ExternalImageId {
        self.next_image_id += 1;
        let key = webrender_api::ExternalImageId(self.next_image_id);
        self.external_images.insert(key, handler_type);
        key
    }

    pub fn remove(&mut self, key: &webrender_api::ExternalImageId) {
        self.external_images.remove(key);
    }

    pub fn get(&self, key: &webrender_api::ExternalImageId) -> Option<&WebrenderImageHandlerType> {
        self.external_images.get(key)
    }
}

/// WebRender External Image Handler implementation.
pub struct WebrenderExternalImageHandlers {
    /// WebGL handler.
    webgl_handler: Option<Box<dyn WebrenderExternalImageApi>>,
    /// Media player handler.
    media_handler: Option<Box<dyn WebrenderExternalImageApi>>,
    /// Webrender external images.
    external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
}

impl WebrenderExternalImageHandlers {
    pub fn new() -> (Self, Arc<Mutex<WebrenderExternalImageRegistry>>) {
        let external_images = Arc::new(Mutex::new(WebrenderExternalImageRegistry::new()));
        (
            Self {
                webgl_handler: None,
                media_handler: None,
                external_images: external_images.clone(),
            },
            external_images,
        )
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

impl webrender_api::ExternalImageHandler for WebrenderExternalImageHandlers {
    /// Lock the external image. Then, WR could start to read the
    /// image content.
    /// The WR client should not change the image content until the
    /// unlock() call.
    fn lock(
        &mut self,
        key: webrender_api::ExternalImageId,
        _channel_index: u8,
        _rendering: webrender_api::ImageRendering,
    ) -> webrender_api::ExternalImage {
        let external_images = self.external_images.lock().unwrap();
        let handler_type = external_images
            .get(&key)
            .expect("Tried to get unknown external image");
        let (texture_id, uv) = match handler_type {
            WebrenderImageHandlerType::WebGL => {
                let (texture_id, size) = self.webgl_handler.as_mut().unwrap().lock(key.0);
                (
                    texture_id,
                    TexelRect::new(0.0, size.height as f32, size.width as f32, 0.0),
                )
            },
            WebrenderImageHandlerType::Media => {
                let (texture_id, size) = self.media_handler.as_mut().unwrap().lock(key.0);
                (
                    texture_id,
                    TexelRect::new(0.0, 0.0, size.width as f32, size.height as f32),
                )
            },
        };
        webrender_api::ExternalImage {
            uv,
            source: webrender_api::ExternalImageSource::NativeTexture(texture_id),
        }
    }

    /// Unlock the external image. The WR should not read the image
    /// content after this call.
    fn unlock(&mut self, key: webrender_api::ExternalImageId, _channel_index: u8) {
        let external_images = self.external_images.lock().unwrap();
        let handler_type = external_images
            .get(&key)
            .expect("Tried to get unknown external image");
        match handler_type {
            WebrenderImageHandlerType::WebGL => self.webgl_handler.as_mut().unwrap().unlock(key.0),
            WebrenderImageHandlerType::Media => self.media_handler.as_mut().unwrap().unlock(key.0),
        };
    }
}
