/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use compositing_traits::{ImageUpdate, SerializableImageData};
use fxhash::FxHashMap;
use ipc_channel::ipc::IpcSharedMemory;
use script_layout_interface::ImageAnimationState;
use style::dom::OpaqueNode;
use webrender_api::units::DeviceIntSize;
use webrender_api::{ImageDescriptor, ImageDescriptorFlags, ImageFormat};

use crate::dom::window::Window;

#[derive(Clone, Debug, Default, JSTraceable, MallocSizeOf)]
pub struct ImageAnimationManager {
    #[no_trace]
    pub node_to_image_map: FxHashMap<OpaqueNode, ImageAnimationState>,
}

impl ImageAnimationManager {
    pub fn new() -> Self {
        ImageAnimationManager {
            node_to_image_map: Default::default(),
        }
    }

    pub fn take_image_animate_set(&mut self) -> FxHashMap<OpaqueNode, ImageAnimationState> {
        std::mem::take(&mut self.node_to_image_map)
    }

    pub fn update_image_animations_post_reflow(
        &mut self,
        map: FxHashMap<OpaqueNode, ImageAnimationState>,
    ) {
        self.restore_image_animate_set(map);
    }

    pub fn image_animations_present(&self) -> bool {
        !self.node_to_image_map.is_empty()
    }

    fn restore_image_animate_set(&mut self, map: FxHashMap<OpaqueNode, ImageAnimationState>) {
        let _ = std::mem::replace(&mut self.node_to_image_map, map);
    }

    pub fn update_active_frames(&mut self, window: &Window, now: f64) {
        let mut updates: Vec<ImageUpdate> = vec![];
        for state in self.node_to_image_map.values_mut() {
            if state.update(now) {
                let image = &state.image;
                let frame = image.frames.get(state.active_frame).unwrap();
                updates.push(ImageUpdate::UpdateImage(
                    image.id.unwrap(),
                    ImageDescriptor {
                        format: ImageFormat::BGRA8,
                        size: DeviceIntSize::new(image.width as i32, image.height as i32),
                        stride: None,
                        offset: 0,
                        flags: ImageDescriptorFlags::ALLOW_MIPMAPS,
                    },
                    SerializableImageData::Raw(IpcSharedMemory::from_bytes(&frame.bytes)),
                ));
            }
        }
        window.compositor_api().update_images(updates);
    }
}
