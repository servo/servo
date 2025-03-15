/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use base::id::PipelineId;
use fnv::FnvHashMap;
use fonts::FontContext;
use net_traits::image_cache::{
    ImageCache, ImageCacheResult, ImageOrMetadataAvailable, UsePlaceholder,
};
use parking_lot::{Mutex, RwLock};
use pixels::Image;
use script_layout_interface::{
    IFrameSizes, ImageAnimateRegisterItem, ImageAnimationAction, ImageAnimationCancelItem,
    LayoutImageAnimateSet, PendingImage, PendingImageState,
};
use servo_url::{ImmutableOrigin, ServoUrl};
use style::context::SharedStyleContext;
use style::dom::OpaqueNode;

use crate::display_list::WebRenderImageInfo;

pub struct LayoutContext<'a> {
    pub id: PipelineId,
    pub use_rayon: bool,
    pub origin: ImmutableOrigin,

    /// Bits shared by the layout and style system.
    pub style_context: SharedStyleContext<'a>,

    /// A FontContext to be used during layout.
    pub font_context: Arc<FontContext>,

    /// Reference to the script thread image cache.
    pub image_cache: Arc<dyn ImageCache>,

    /// A list of in-progress image loads to be shared with the script thread.
    pub pending_images: Mutex<Vec<PendingImage>>,

    /// A collection of `<iframe>` sizes to send back to script.
    pub iframe_sizes: Mutex<IFrameSizes>,

    pub webrender_image_cache:
        Arc<RwLock<FnvHashMap<(ServoUrl, UsePlaceholder), WebRenderImageInfo>>>,

    /// A list of animated image to be handle to register/cancel update frame in ImageAnimationManager.
    pub pending_image_animation_actions: Mutex<Vec<ImageAnimationAction>>,

    pub layout_image_animation_set: LayoutImageAnimateSet,
}

impl Drop for LayoutContext<'_> {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            assert!(self.pending_images.lock().is_empty());
        }
    }
}

impl LayoutContext<'_> {
    #[inline(always)]
    pub fn shared_context(&self) -> &SharedStyleContext {
        &self.style_context
    }

    pub fn get_or_request_image_or_meta(
        &self,
        node: OpaqueNode,
        url: ServoUrl,
        use_placeholder: UsePlaceholder,
    ) -> Option<ImageOrMetadataAvailable> {
        // Check for available image or start tracking.
        let cache_result = self.image_cache.get_cached_image_status(
            url.clone(),
            self.origin.clone(),
            None,
            use_placeholder,
        );

        match cache_result {
            ImageCacheResult::Available(img_or_meta) => Some(img_or_meta),
            // Image has been requested, is still pending. Return no image for this paint loop.
            // When the image loads it will trigger a reflow and/or repaint.
            ImageCacheResult::Pending(id) => {
                let image = PendingImage {
                    state: PendingImageState::PendingResponse,
                    node: node.into(),
                    id,
                    origin: self.origin.clone(),
                };
                self.pending_images.lock().push(image);
                None
            },
            // Not yet requested - request image or metadata from the cache
            ImageCacheResult::ReadyForRequest(id) => {
                let image = PendingImage {
                    state: PendingImageState::Unrequested(url),
                    node: node.into(),
                    id,
                    origin: self.origin.clone(),
                };
                self.pending_images.lock().push(image);
                None
            },
            // Image failed to load, so just return nothing
            ImageCacheResult::LoadError => None,
        }
    }
    // TODO: Do some image animation registering here.
    /*
        There are two things need to be stored in layoutcontext:
            1. info used to determine whether this <node,ImageIdentifier> is in our ImageAnimationManager:
                a. If the image exist in the imageAnimationManager
                    1. Is the image inside the entry the same as the one outside?
                        a. is the same then do nothing.
                        b. if not the same, see if the new one is animated image or not.
                            1. if the new one is animated image, register a add action.
                            2. if the new one is not animted iamge, register a cancel action.
                b. If the image does not exist in the imageAnimationManager
                    1. Is the image is animated?
                        a. if yes, register an add action.
                        b. if not, do nothing.

            2. list of <Action: to add/update/delete <node,ImageIdentifier> in ImageAnimationManager >
                a. for add action, what information do we need: <Node, ImageIdentifier, Arc<Image> >
                b. for update action: <Node, NewImageIdentifier, Arc<Image>>
                c. for delete action: <Node>
    */
    pub fn handle_animated_image(&self, node: OpaqueNode, url: &ServoUrl, image: Arc<Image>) {
        let store = self.layout_image_animation_set.node_to_image_key.read();
        if let Some(image_id) = store.get(&node) {
            if &image_id.0 != url {
                // TODO: For now just check the url. see if there are any side effect?
                if image.should_animate() {
                    // Register a add action
                    let action = ImageAnimationAction::Register(ImageAnimateRegisterItem {
                        node,
                        identifier: (url.clone(), self.origin.clone(), None),
                        image,
                    });
                    self.pending_image_animation_actions.lock().push(action);
                } else {
                    let action = ImageAnimationAction::Cancel(ImageAnimationCancelItem { node });
                    self.pending_image_animation_actions.lock().push(action);
                }
            }
        } else if image.should_animate() {
            let action = ImageAnimationAction::Register(ImageAnimateRegisterItem {
                node,
                identifier: (url.clone(), self.origin.clone(), None),
                image,
            });
            self.pending_image_animation_actions.lock().push(action);
        }
    }

    pub fn get_webrender_image_for_url(
        &self,
        node: OpaqueNode,
        url: ServoUrl,
        use_placeholder: UsePlaceholder,
    ) -> Option<WebRenderImageInfo> {
        if let Some(existing_webrender_image) = self
            .webrender_image_cache
            .read()
            .get(&(url.clone(), use_placeholder))
        {
            return Some(*existing_webrender_image);
        }

        match self.get_or_request_image_or_meta(node, url.clone(), use_placeholder) {
            Some(ImageOrMetadataAvailable::ImageAvailable { image, .. }) => {
                self.handle_animated_image(node, &url, image.clone());
                let image_info = WebRenderImageInfo {
                    width: image.width,
                    height: image.height,
                    key: image.id,
                };
                if image_info.key.is_none() {
                    Some(image_info)
                } else {
                    let mut webrender_image_cache = self.webrender_image_cache.write();
                    webrender_image_cache.insert((url, use_placeholder), image_info);
                    Some(image_info)
                }
            },
            None | Some(ImageOrMetadataAvailable::MetadataAvailable(..)) => None,
        }
    }
}
