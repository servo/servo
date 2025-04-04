/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::time::{SystemTime, UNIX_EPOCH};

use fxhash::FxHashMap;
use ipc_channel::ipc::IpcSharedMemory;
use script_layout_interface::ImageAnimationState;
use style::dom::OpaqueNode;
use webrender_api::units::DeviceIntSize;
use webrender_api::{ImageDescriptor, ImageDescriptorFlags, ImageFormat};
use webrender_traits::{ImageUpdate, SerializableImageData};

use crate::dom::window::Window;

#[derive(Clone, Debug, Default, JSTraceable, MallocSizeOf)]
pub struct ImageAnimationManager {
    #[no_trace]
    pub node_to_image_map: FxHashMap<OpaqueNode, ImageAnimationState>,

    /// Whether or not there are animated image being tracked
    has_animated_image: Cell<bool>,
}

impl ImageAnimationManager {
    pub fn new() -> Self {
        ImageAnimationManager {
            node_to_image_map: Default::default(),
            has_animated_image: Cell::new(false),
        }
    }

    pub fn take_image_animate_set(&mut self) -> FxHashMap<OpaqueNode, ImageAnimationState> {
        std::mem::take(&mut self.node_to_image_map)
    }

    pub fn update_image_animation_post_reflow(
        &mut self,
        map: FxHashMap<OpaqueNode, ImageAnimationState>,
    ) {
        self.restore_image_animate_set(map);
        self.has_animated_image
            .set(!self.node_to_image_map.is_empty());
    }

    pub fn image_animations_present(&self) -> bool {
        self.has_animated_image.get()
    }

    fn restore_image_animate_set(&mut self, map: FxHashMap<OpaqueNode, ImageAnimationState>) {
        let _ = std::mem::replace(&mut self.node_to_image_map, map);
    }

    pub fn update_active_frame(&mut self, window: &Window) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        let mut updates: Vec<ImageUpdate> = vec![];
        for state in self.node_to_image_map.values_mut() {
            let image = &state.image;
            let time_interval_since_last_update = current_time - state.last_update_time;
            let mut remain_time_interval = time_interval_since_last_update -
                image
                    .frames
                    .get(state.active_frame)
                    .unwrap()
                    .delay
                    .unwrap()
                    .as_secs_f64();
            let mut next_active_frame_id = state.active_frame;
            while remain_time_interval > 0.0 {
                next_active_frame_id = (next_active_frame_id + 1) % image.frames.len();
                remain_time_interval -= image
                    .frames
                    .get(next_active_frame_id)
                    .unwrap()
                    .delay
                    .unwrap()
                    .as_secs_f64();
            }
            if next_active_frame_id != state.active_frame {
                let frame = image.frames.get(next_active_frame_id).unwrap();
                state.last_update_time = current_time;
                state.active_frame = next_active_frame_id;
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
