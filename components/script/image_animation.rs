/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use fxhash::FxHashMap;
use script_layout_interface::ImageAnimationState;
use style::dom::OpaqueNode;

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

    pub fn restore_image_animate_set(&mut self, map: FxHashMap<OpaqueNode, ImageAnimationState>) {
        let _ = std::mem::replace(&mut self.node_to_image_map, map);
    }
}
