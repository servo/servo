/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use compositing_traits::{ImageUpdate, SerializableImageData};
use embedder_traits::UntrustedNodeAddress;
use fxhash::FxHashMap;
use ipc_channel::ipc::IpcSharedMemory;
use layout_api::ImageAnimationState;
use libc::c_void;
use malloc_size_of::MallocSizeOf;
use parking_lot::RwLock;
use script_bindings::root::Dom;
use style::dom::OpaqueNode;
use webrender_api::units::DeviceIntSize;
use webrender_api::{ImageDescriptor, ImageDescriptorFlags, ImageFormat};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::trace::NoTrace;
use crate::dom::node::{Node, from_untrusted_node_address};
use crate::dom::window::Window;

#[derive(Clone, Debug, Default, JSTraceable)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub struct ImageAnimationManager {
    #[no_trace]
    node_to_image_map: Arc<RwLock<FxHashMap<OpaqueNode, ImageAnimationState>>>,

    /// A list of nodes with in-progress image animations.
    ///
    /// TODO(mrobinson): This does not properly handle animating images that are in pseudo-elements.
    rooted_nodes: DomRefCell<FxHashMap<NoTrace<OpaqueNode>, Dom<Node>>>,
}

impl MallocSizeOf for ImageAnimationManager {
    fn size_of(&self, ops: &mut malloc_size_of::MallocSizeOfOps) -> usize {
        (*self.node_to_image_map.read()).size_of(ops) + self.rooted_nodes.size_of(ops)
    }
}

impl ImageAnimationManager {
    pub(crate) fn node_to_image_map(
        &self,
    ) -> Arc<RwLock<FxHashMap<OpaqueNode, ImageAnimationState>>> {
        self.node_to_image_map.clone()
    }

    pub(crate) fn next_scheduled_time(&self, now: f64) -> Option<f64> {
        self.node_to_image_map
            .read()
            .values()
            .map(|state| state.time_to_next_frame(now))
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    }

    pub(crate) fn image_animations_present(&self) -> bool {
        !self.node_to_image_map.read().is_empty()
    }

    pub(crate) fn update_active_frames(&self, window: &Window, now: f64) {
        let rooted_nodes = self.rooted_nodes.borrow();
        let updates = self
            .node_to_image_map
            .write()
            .iter_mut()
            .filter_map(|(node, state)| {
                if !state.update_frame_for_animation_timeline_value(now) {
                    return None;
                }

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
            })
            .collect();
        window.compositor_api().update_images(updates);
    }
    /// Ensure that all nodes with animating images are rooted and unroots any nodes that
    /// no longer have an animating image. This should be called immediately after a
    /// restyle, to ensure that these addresses are still valid.
    #[allow(unsafe_code)]
    pub(crate) fn update_rooted_dom_nodes(&self) {
        let mut rooted_nodes = self.rooted_nodes.borrow_mut();
        let node_to_image_map = self.node_to_image_map.read();

        for opaque_node in node_to_image_map.keys() {
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

        rooted_nodes.retain(|node, _| node_to_image_map.contains_key(&node.0));
    }
}
