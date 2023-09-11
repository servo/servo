/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A simple occlusion culling algorithm for axis-aligned rectangles.
//!
//! ## Output
//!
//! Occlusion culling results in two lists of rectangles:
//! 
//! - The opaque list should be rendered first. None of its rectangles overlap so order doesn't matter
//!   within the opaque pass.
//! - The non-opaque list (or alpha list) which should be rendered in back-to-front order after the opaque pass.
//!
//! The output has minimal overdraw (no overdraw at all for opaque items and as little as possible for alpha ones).
//!
//! ## Algorithm overview
//!
//! The occlusion culling algorithm works in front-to-back order, accumulating rectangle in opaque and non-opaque lists.
//! Each time a rectangle is added, it is first tested against existing opaque rectangles and potentially split into visible
//! sub-rectangles, or even discarded completely. The front-to-back order ensures that once a rectangle is added it does not
//! have to be modified again, making the underlying data structure trivial (append-only).
//!
//! ## splitting
//!
//! Partially visible rectangles are split into up to 4 visible sub-rectangles by each intersecting occluder.
//!
//! ```ascii
//!  +----------------------+       +----------------------+
//!  | rectangle            |       |                      |
//!  |                      |       |                      |
//!  |  +-----------+       |       +--+-----------+-------+
//!  |  |occluder   |       |  -->  |  |\\\\\\\\\\\|       |
//!  |  +-----------+       |       +--+-----------+-------+
//!  |                      |       |                      |
//!  +----------------------+       +----------------------+
//! ```
//!
//! In the example above the rectangle is split into 4 visible parts with the central occluded part left out.
//!
//! This implementation favors longer horizontal bands instead creating nine-patches to deal with the corners.
//! The advantage is that it produces less rectangles which is good for the performance of the algorithm and
//! for SWGL which likes long horizontal spans, however it would cause artifacts if the resulting rectangles
//! were to be drawn with a non-axis-aligned transformation.
//!
//! ## Performance
//!
//! The cost of the algorithm grows with the number of opaque rectangle as each new rectangle is tested against
//! all previously added opaque rectangles.
//!
//! Note that opaque rectangles can either be added as opaque or non-opaque. This means a trade-off between
//! overdraw and number of rectangles can be explored to adjust performance: Small opaque rectangles, especially
//! towards the front of the scene, could be added as non-opaque to avoid causing many splits while adding only 
//! a small amount of overdraw.
//!
//! This implementation is intended to be used with a small number of (opaque) items. A similar implementation
//! could use a spatial acceleration structure for opaque rectangles to perform better with a large amount of
//! occluders.
//!

use euclid::point2;
use smallvec::SmallVec;
use api::units::*;

/// A visible part of a rectangle after occlusion culling.
#[derive(Debug, PartialEq)]
pub struct Item {
    pub rectangle: DeviceBox2D,
    pub key: usize,
}

/// A builder that applies occlusion culling with rectangles provided in front-to-back order.
pub struct FrontToBackBuilder {
    opaque_items: Vec<Item>,
    alpha_items: Vec<Item>,
}

impl FrontToBackBuilder {

    /// Pre-allocating constructor.
    pub fn with_capacity(opaque: usize, alpha: usize) -> Self {
        FrontToBackBuilder {
            opaque_items: Vec::with_capacity(opaque),
            alpha_items: Vec::with_capacity(alpha),
        }
    }

    /// Add a rectangle, potentially splitting it and discarding the occluded parts if any.
    ///
    /// Returns true the rectangle is at least partially visible.
    pub fn add(&mut self, rect: &DeviceBox2D, is_opaque: bool, key: usize) -> bool {
        let mut fragments: SmallVec<[DeviceBox2D; 16]> = SmallVec::new();
        fragments.push(*rect);

        for item in &self.opaque_items {
            if fragments.is_empty() {
                break;
            }
            if item.rectangle.intersects(rect) {
                apply_occluder(&item.rectangle, &mut fragments);
            }
        }

        let list = if is_opaque {
            &mut self.opaque_items
        } else {
            &mut self.alpha_items
        };

        for rect in &fragments {
            list.push(Item {
                rectangle: *rect,
                key,
            });
        }

        !fragments.is_empty()
    }

    /// Returns true if the provided rect is at least partially visible, without adding it.
    pub fn test(&self, rect: &DeviceBox2D) -> bool {
        let mut fragments: SmallVec<[DeviceBox2D; 16]> = SmallVec::new();
        fragments.push(*rect);

        for item in &self.opaque_items {
            if item.rectangle.intersects(rect) {
                apply_occluder(&item.rectangle, &mut fragments);
            }
        }

        !fragments.is_empty()
    }

    /// The visible opaque rectangles (front-to-back order).
    pub fn opaque_items(&self) -> &[Item] {
        &self.opaque_items
    }

    /// The visible non-opaque rectangles (front-to-back order).
    pub fn alpha_items(&self) -> &[Item] {
        &self.alpha_items
    }
}


// Split out the parts of the rects in the provided vector
fn apply_occluder(occluder: &DeviceBox2D, rects: &mut SmallVec<[DeviceBox2D; 16]>) {
    // Iterate in reverse order so that we can push new rects at the back without
    // visiting them;
    let mut i = rects.len() - 1;
    loop {
        let r = rects[i];

        if r.intersects(occluder) {
            let top = r.min.y < occluder.min.y;
            let bottom = r.max.y > occluder.max.y;
            let left = r.min.x < occluder.min.x;
            let right = r.max.x > occluder.max.x;

            if top {
                rects.push(DeviceBox2D {
                    min: r.min,
                    max: point2(r.max.x, occluder.min.y),
                });
            }

            if bottom {
                rects.push(DeviceBox2D {
                    min: point2(r.min.x, occluder.max.y),
                    max: r.max,
                });
            }

            if left {
                let min_y = r.min.y.max(occluder.min.y);
                let max_y = r.max.y.min(occluder.max.y);
                rects.push(DeviceBox2D {
                    min: point2(r.min.x, min_y),
                    max: point2(occluder.min.x, max_y),
                });
            }

            if right {
                let min_y = r.min.y.max(occluder.min.y);
                let max_y = r.max.y.min(occluder.max.y);
                rects.push(DeviceBox2D {
                    min: point2(occluder.max.x, min_y),
                    max: point2(r.max.x, max_y),
                });
            }

            // Remove the original rectangle, replacing it with
            // one of the new ones we just added, or popping it
            // if it is the last item.
            if i == rects.len() {
                rects.pop();
            } else {
                rects.swap_remove(i);
            }
        }

        if i == 0 {
            break;
        }

        i -= 1;
    }
}
