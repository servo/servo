/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::sync::Arc;
use std::time::Duration;

use compositing_traits::{ImageUpdate, SerializableImageData};
use ipc_channel::ipc::IpcSharedMemory;
use layout_api::AnimatingImages;
use malloc_size_of::MallocSizeOf;
use parking_lot::RwLock;
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use timers::{TimerEventRequest, TimerId};
use webrender_api::units::DeviceIntSize;
use webrender_api::{ImageDescriptor, ImageDescriptorFlags, ImageFormat};

use crate::dom::bindings::refcounted::Trusted;
use crate::dom::node::Node;
use crate::dom::window::Window;
use crate::script_thread::with_script_thread;

#[derive(Clone, Default, JSTraceable)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub struct ImageAnimationManager {
    /// The set of [`AnimatingImages`] which is used to communicate the addition
    /// and removal of animating images from layout.
    #[no_trace]
    animating_images: Arc<RwLock<AnimatingImages>>,

    /// The [`TimerId`] of the currently scheduled animated image update callback.
    #[no_trace]
    callback_timer_id: Cell<Option<TimerId>>,
}

impl MallocSizeOf for ImageAnimationManager {
    fn size_of(&self, ops: &mut malloc_size_of::MallocSizeOfOps) -> usize {
        (*self.animating_images.read()).size_of(ops)
    }
}

impl ImageAnimationManager {
    pub(crate) fn animating_images(&self) -> Arc<RwLock<AnimatingImages>> {
        self.animating_images.clone()
    }

    fn duration_to_next_frame(&self, now: f64) -> Option<Duration> {
        self.animating_images
            .read()
            .node_to_state_map
            .values()
            .map(|state| state.duration_to_next_frame(now))
            .min()
    }

    pub(crate) fn update_active_frames(&self, window: &Window, now: f64) {
        if self.animating_images.read().is_empty() {
            return;
        }

        let updates = self
            .animating_images
            .write()
            .node_to_state_map
            .values_mut()
            .filter_map(|state| {
                if !state.update_frame_for_animation_timeline_value(now) {
                    return None;
                }

                let image = &state.image;
                let frame = image
                    .frame(state.active_frame)
                    .expect("active_frame should within range of frames");

                Some(ImageUpdate::UpdateImage(
                    image.id.unwrap(),
                    ImageDescriptor {
                        format: ImageFormat::BGRA8,
                        size: DeviceIntSize::new(
                            image.metadata.width as i32,
                            image.metadata.height as i32,
                        ),
                        stride: None,
                        offset: 0,
                        flags: ImageDescriptorFlags::ALLOW_MIPMAPS,
                    },
                    SerializableImageData::Raw(IpcSharedMemory::from_bytes(frame.bytes)),
                    None,
                ))
            })
            .collect();
        window.compositor_api().update_images(updates);

        self.maybe_schedule_update(window, now);
    }

    /// After doing a layout, if the set of animating images was updated in some way,
    /// schedule a new animation update.
    pub(crate) fn maybe_schedule_update_after_layout(&self, window: &Window, now: f64) {
        if self.animating_images().write().clear_dirty() {
            self.maybe_schedule_update(window, now);
        }
    }

    fn maybe_schedule_update(&self, window: &Window, now: f64) {
        with_script_thread(|script_thread| {
            if let Some(current_timer_id) = self.callback_timer_id.take() {
                self.callback_timer_id.set(None);
                script_thread.cancel_timer(current_timer_id);
            }

            if let Some(duration) = self.duration_to_next_frame(now) {
                let trusted_window = Trusted::new(window);
                let timer_id = script_thread.schedule_timer(TimerEventRequest {
                    callback: Box::new(move || {
                        let window = trusted_window.root();
                        window.Document().set_has_pending_animated_image_update();
                    }),
                    duration,
                });
                self.callback_timer_id.set(Some(timer_id));
            }
        })
    }

    pub(crate) fn cancel_animations_for_node(&self, node: &Node) {
        self.animating_images().write().remove(node.to_opaque());
    }
}
