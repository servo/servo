/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Defines data structures which are consumed by the Compositor.

use embedder_traits::Cursor;

/// Information that Servo keeps alongside WebRender display items
/// in order to add more context to hit test results.
#[derive(Debug, Deserialize, Serialize)]
pub struct HitTestInfo {
    /// The id of the node of this hit test item.
    pub node: u64,

    /// The cursor of this node's hit test item.
    pub cursor: Option<Cursor>,
}

/// A data structure which stores compositor-side information about
/// display lists sent to the compositor.
/// by a WebRender display list.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct CompositorDisplayListInfo {
    /// An array of `HitTestInfo` which is used to store information
    /// to assist the compositor to take various actions (set the cursor,
    /// scroll without layout) using a WebRender hit test result.
    pub hit_test_info: Vec<HitTestInfo>,
}

impl CompositorDisplayListInfo {
    /// Add or re-use a duplicate HitTestInfo entry in this `CompositorHitTestInfo`
    /// and return the index.
    pub fn add_hit_test_info(&mut self, node: u64, cursor: Option<Cursor>) -> usize {
        if let Some(last) = self.hit_test_info.last() {
            if node == last.node && cursor == last.cursor {
                return self.hit_test_info.len() - 1;
            }
        }

        self.hit_test_info.push(HitTestInfo { node, cursor });
        self.hit_test_info.len() - 1
    }
}
