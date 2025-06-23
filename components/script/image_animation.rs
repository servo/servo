/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use compositing_traits::{ImageUpdate, SerializableImageData};
use embedder_traits::UntrustedNodeAddress;
use fxhash::{FxHashMap, FxHashSet};
use ipc_channel::ipc::IpcSharedMemory;
use layout_api::ImageAnimationState;
use libc::c_void;
use script_bindings::root::Dom;
use style::dom::OpaqueNode;
use webrender_api::units::DeviceIntSize;
use webrender_api::{ImageDescriptor, ImageDescriptorFlags, ImageFormat};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::trace::NoTrace;
use crate::dom::node::{Node, from_untrusted_node_address};
use crate::dom::window::Window;

#[derive(Clone, Debug, Default, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub struct ImageAnimationManager {
    #[no_trace]
    pub node_to_image_map: FxHashMap<OpaqueNode, ImageAnimationState>,

    /// A list of nodes with in-progress image animations.
    rooted_nodes: DomRefCell<FxHashMap<NoTrace<OpaqueNode>, Dom<Node>>>,
}

impl ImageAnimationManager {
    pub fn new() -> Self {
        ImageAnimationManager {
            node_to_image_map: Default::default(),
            rooted_nodes: DomRefCell::new(FxHashMap::default()),
        }
    }

    pub fn take_image_animate_set(&mut self) -> FxHashMap<OpaqueNode, ImageAnimationState> {
        std::mem::take(&mut self.node_to_image_map)
    }

    pub fn restore_image_animate_set(&mut self, map: FxHashMap<OpaqueNode, ImageAnimationState>) {
        let _ = std::mem::replace(&mut self.node_to_image_map, map);
        self.root_newly_animating_dom_nodes();
        self.unroot_unused_nodes();
    }

    pub fn next_schedule_time(&self, now: f64) -> Option<f64> {
        self.node_to_image_map
            .values()
            .map(|state| state.time_to_next_frame(now))
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    }

    pub fn image_animations_present(&self) -> bool {
        !self.node_to_image_map.is_empty()
    }

    pub fn update_active_frames(&mut self, window: &Window, now: f64) {
        let rooted_nodes = self.rooted_nodes.borrow();
        let updates: Vec<ImageUpdate> = self
            .node_to_image_map
            .iter_mut()
            .filter_map(|(node, state)| {
                if state.update_frame_for_animation_timeline_value(now) {
                    let image = &state.image;
                    let frame = image
                        .frames()
                        .nth(state.active_frame)
                        .expect("active_frame should within range of frames");

                    if let Some(node) = rooted_nodes.get(&NoTrace(*node)) {
                        node.dirty(crate::dom::node::NodeDamage::Other);
                    }
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
                    ))
                } else {
                    None
                }
            })
            .collect();
        window.compositor_api().update_images(updates);
    }

    // Unroot any nodes that we have rooted but no longer have animating images.
    fn unroot_unused_nodes(&self) {
        let nodes: FxHashSet<&OpaqueNode> = self.node_to_image_map.keys().collect();
        self.rooted_nodes
            .borrow_mut()
            .retain(|node, _| nodes.contains(&node.0));
    }

    /// Ensure that all nodes with Image animations are rooted. This should be called
    /// immediately after a restyle, to ensure that these addresses are still valid.
    #[allow(unsafe_code)]
    fn root_newly_animating_dom_nodes(&self) {
        let mut rooted_nodes = self.rooted_nodes.borrow_mut();
        for opaque_node in self.node_to_image_map.keys() {
            let opaque_node = *opaque_node;
            if rooted_nodes.contains_key(&NoTrace(opaque_node)) {
                continue;
            }
            let address = UntrustedNodeAddress(opaque_node.0 as *const c_void);
            unsafe {
                rooted_nodes.insert(
                    NoTrace(opaque_node),
                    Dom::from_ref(&*from_untrusted_node_address(address)),
                )
            };
        }
    }
}
