/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::ffi::c_void;
use std::sync::Arc;
use std::time::Duration;

use embedder_traits::UntrustedNodeAddress;
use layout_api::AnimatingImages;
use paint_api::ImageUpdate;
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::root::Dom;
use style::dom::OpaqueNode;
use timers::{TimerEventRequest, TimerId};

use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::trace::NoTrace;
use crate::dom::from_untrusted_node_address;
use crate::dom::node::Node;
use crate::dom::window::Window;
use crate::script_thread::with_script_thread;

#[derive(Clone, Default, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub struct ImageAnimationManager {
    /// The set of [`AnimatingImages`] which is used to communicate the addition
    /// and removal of animating images from layout.
    #[no_trace]
    #[conditional_malloc_size_of]
    animating_images: Arc<RwLock<AnimatingImages>>,

    /// The [`TimerId`] of the currently scheduled animated image update callback.
    #[no_trace]
    callback_timer_id: Cell<Option<TimerId>>,

    /// A map of nodes with in-progress image animations. This is kept outside
    /// of [`Self::animating_images`] as that data structure is shared with layout.
    rooted_nodes: FxHashMap<NoTrace<OpaqueNode>, Dom<Node>>,
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
                    .frame_data(state.active_frame)
                    .expect("No frame found")
                    .clone();
                if let Some(mut descriptor) =
                    image.webrender_image_descriptor_and_offset_for_frame()
                {
                    descriptor.offset = frame.byte_range.start as i32;
                    Some(ImageUpdate::UpdateImageForAnimation(
                        image.id.unwrap(),
                        descriptor,
                    ))
                } else {
                    error!("Doing normal image update which will be slow!");
                    None
                }
            })
            .collect();
        window
            .paint_api()
            .update_images(window.webview_id().into(), updates);

        self.maybe_schedule_update(window, now);
    }

    /// This does three things:
    ///  - Root any nodes with newly animating images
    ///  - Schedule an image update for newly animating images
    ///  - Cancel animations for any nodes that no longer have layout boxes.
    pub(crate) fn do_post_reflow_update(&mut self, window: &Window, now: f64) {
        // Cancel animations for any images that are no longer rendering.
        self.rooted_nodes.retain(|opaque_node, node| {
            if node.is_being_rendered(None) {
                return true;
            }
            self.animating_images.write().remove(opaque_node.0);
            false
        });

        if self.animating_images().write().clear_dirty() {
            self.root_nodes_with_newly_animating_images();
            self.maybe_schedule_update(window, now);
        }
    }

    fn root_nodes_with_newly_animating_images(&mut self) {
        for opaque_node in self.animating_images().read().node_to_state_map.keys() {
            #[expect(unsafe_code)]
            self.rooted_nodes
                .entry(NoTrace(*opaque_node))
                .or_insert_with(|| {
                    // SAFETY: This should be safe as this method is run directly after layout,
                    // which should not remove any nodes.
                    let address = UntrustedNodeAddress(opaque_node.0 as *const c_void);
                    unsafe { Dom::from_ref(&*from_untrusted_node_address(address)) }
                });
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

    pub(crate) fn cancel_animations_for_node(&mut self, node: &Node) {
        let opaque_node = node.to_opaque();
        self.animating_images().write().remove(opaque_node);
        self.rooted_nodes.remove(&NoTrace(opaque_node));
    }
}
