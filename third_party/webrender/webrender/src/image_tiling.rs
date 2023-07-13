/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::api::TileSize;
use crate::api::units::*;
use crate::segment::EdgeAaSegmentMask;
use euclid::{point2, size2};
use std::i32;
use std::ops::Range;

/// If repetitions are far enough apart that only one is within
/// the primitive rect, then we can simplify the parameters and
/// treat the primitive as not repeated.
/// This can let us avoid unnecessary work later to handle some
/// of the parameters.
pub fn simplify_repeated_primitive(
    stretch_size: &LayoutSize,
    tile_spacing: &mut LayoutSize,
    prim_rect: &mut LayoutRect,
) {
    let stride = *stretch_size + *tile_spacing;

    if stride.width >= prim_rect.size.width {
        tile_spacing.width = 0.0;
        prim_rect.size.width = f32::min(prim_rect.size.width, stretch_size.width);
    }
    if stride.height >= prim_rect.size.height {
        tile_spacing.height = 0.0;
        prim_rect.size.height = f32::min(prim_rect.size.height, stretch_size.height);
    }
}

pub struct Repetition {
    pub origin: LayoutPoint,
    pub edge_flags: EdgeAaSegmentMask,
}

pub struct RepetitionIterator {
    current_x: i32,
    x_count: i32,
    current_y: i32,
    y_count: i32,
    row_flags: EdgeAaSegmentMask,
    current_origin: LayoutPoint,
    initial_origin: LayoutPoint,
    stride: LayoutSize,
}

impl Iterator for RepetitionIterator {
    type Item = Repetition;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_x == self.x_count {
            self.current_y += 1;
            if self.current_y >= self.y_count {
                return None;
            }
            self.current_x = 0;

            self.row_flags = EdgeAaSegmentMask::empty();
            if self.current_y == self.y_count - 1 {
                self.row_flags |= EdgeAaSegmentMask::BOTTOM;
            }

            self.current_origin.x = self.initial_origin.x;
            self.current_origin.y += self.stride.height;
        }

        let mut edge_flags = self.row_flags;
        if self.current_x == 0 {
            edge_flags |= EdgeAaSegmentMask::LEFT;
        }

        if self.current_x == self.x_count - 1 {
            edge_flags |= EdgeAaSegmentMask::RIGHT;
        }

        let repetition = Repetition {
            origin: self.current_origin,
            edge_flags,
        };

        self.current_origin.x += self.stride.width;
        self.current_x += 1;

        Some(repetition)
    }
}

pub fn repetitions(
    prim_rect: &LayoutRect,
    visible_rect: &LayoutRect,
    stride: LayoutSize,
) -> RepetitionIterator {
    let visible_rect = match prim_rect.intersection(&visible_rect) {
        Some(rect) => rect,
        None => {
            return RepetitionIterator {
                current_origin: LayoutPoint::zero(),
                initial_origin: LayoutPoint::zero(),
                current_x: 0,
                current_y: 0,
                x_count: 0,
                y_count: 0,
                stride,
                row_flags: EdgeAaSegmentMask::empty(),
            }
        }
    };

    assert!(stride.width > 0.0);
    assert!(stride.height > 0.0);

    let nx = if visible_rect.origin.x > prim_rect.origin.x {
        f32::floor((visible_rect.origin.x - prim_rect.origin.x) / stride.width)
    } else {
        0.0
    };

    let ny = if visible_rect.origin.y > prim_rect.origin.y {
        f32::floor((visible_rect.origin.y - prim_rect.origin.y) / stride.height)
    } else {
        0.0
    };

    let x0 = prim_rect.origin.x + nx * stride.width;
    let y0 = prim_rect.origin.y + ny * stride.height;

    let x_most = visible_rect.max_x();
    let y_most = visible_rect.max_y();

    let x_count = f32::ceil((x_most - x0) / stride.width) as i32;
    let y_count = f32::ceil((y_most - y0) / stride.height) as i32;

    let mut row_flags = EdgeAaSegmentMask::TOP;
    if y_count == 1 {
        row_flags |= EdgeAaSegmentMask::BOTTOM;
    }

    RepetitionIterator {
        current_origin: LayoutPoint::new(x0, y0),
        initial_origin: LayoutPoint::new(x0, y0),
        current_x: 0,
        current_y: 0,
        x_count,
        y_count,
        row_flags,
        stride,
    }
}

#[derive(Debug)]
pub struct Tile {
    pub rect: LayoutRect,
    pub offset: TileOffset,
    pub edge_flags: EdgeAaSegmentMask,
}

#[derive(Debug)]
pub struct TileIteratorExtent {
    /// Range of visible tiles to iterate over in number of tiles.
    tile_range: Range<i32>,
    /// Range of tiles of the full image including tiles that are culled out.
    image_tiles: Range<i32>,
    /// Size of the first tile in layout space.
    first_tile_layout_size: f32,
    /// Size of the last tile in layout space.
    last_tile_layout_size: f32,
    /// Position of blob point (0, 0) in layout space.
    layout_tiling_origin: f32,
    /// Position of the top-left corner of the primitive rect in layout space.
    layout_prim_start: f32,
}

#[derive(Debug)]
pub struct TileIterator {
    current_tile: TileOffset,
    x: TileIteratorExtent,
    y: TileIteratorExtent,
    regular_tile_size: LayoutSize,
}

impl Iterator for TileIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        // If we reach the end of a row, reset to the beginning of the next row.
        if self.current_tile.x >= self.x.tile_range.end {
            self.current_tile.y += 1;
            self.current_tile.x = self.x.tile_range.start;
        }

        // Stop iterating if we reach the last tile. We may start here if there
        // were no tiles to iterate over.
        if self.current_tile.x >= self.x.tile_range.end || self.current_tile.y >= self.y.tile_range.end {
            return None;
        }

        let tile_offset = self.current_tile;

        let mut segment_rect = LayoutRect {
            origin: LayoutPoint::new(
                self.x.layout_tiling_origin + tile_offset.x as f32 * self.regular_tile_size.width,
                self.y.layout_tiling_origin + tile_offset.y as f32 * self.regular_tile_size.height,
            ),
            size: self.regular_tile_size,
        };

        let mut edge_flags = EdgeAaSegmentMask::empty();

        if tile_offset.x == self.x.image_tiles.start {
            edge_flags |= EdgeAaSegmentMask::LEFT;
            segment_rect.size.width = self.x.first_tile_layout_size;
            segment_rect.origin.x = self.x.layout_prim_start;
        }
        if tile_offset.x == self.x.image_tiles.end - 1 {
            edge_flags |= EdgeAaSegmentMask::RIGHT;
            segment_rect.size.width = self.x.last_tile_layout_size;
        }

        if tile_offset.y == self.y.image_tiles.start {
            segment_rect.size.height = self.y.first_tile_layout_size;
            segment_rect.origin.y = self.y.layout_prim_start;
            edge_flags |= EdgeAaSegmentMask::TOP;
        }
        if tile_offset.y == self.y.image_tiles.end - 1 {
            segment_rect.size.height = self.y.last_tile_layout_size;
            edge_flags |= EdgeAaSegmentMask::BOTTOM;
        }

        assert!(tile_offset.y < self.y.tile_range.end);
        let tile = Tile {
            rect: segment_rect,
            offset: tile_offset,
            edge_flags,
        };

        self.current_tile.x += 1;

        Some(tile)
    }
}

pub fn tiles(
    prim_rect: &LayoutRect,
    visible_rect: &LayoutRect,
    image_rect: &DeviceIntRect,
    device_tile_size: i32,
) -> TileIterator {
    // The image resource is tiled. We have to generate an image primitive
    // for each tile.
    // We need to do this because the image is broken up into smaller tiles in the texture
    // cache and the image shader is not able to work with this type of sparse representation.

    // The tiling logic works as follows:
    //
    //  +-#################-+  -+
    //  | #//|    |    |//# |   | image size
    //  | #//|    |    |//# |   |
    //  +-#--+----+----+--#-+   |  -+
    //  | #//|    |    |//# |   |   | regular tile size
    //  | #//|    |    |//# |   |   |
    //  +-#--+----+----+--#-+   |  -+-+
    //  | #//|////|////|//# |   |     | "leftover" height
    //  | ################# |  -+  ---+
    //  +----+----+----+----+
    //
    // In the ascii diagram above, a large image is split into tiles of almost regular size.
    // The tiles on the edges (hatched in the diagram) can be smaller than the regular tiles
    // and are handled separately in the code (we'll call them boundary tiles).
    //
    // Each generated segment corresponds to a tile in the texture cache, with the
    // assumption that the boundary tiles are sized to fit their own irregular size in the
    // texture cache.
    //
    // Because we can have very large virtual images we iterate over the visible portion of
    // the image in layer space instead of iterating over all device tiles.

    let visible_rect = match prim_rect.intersection(&visible_rect) {
        Some(rect) => rect,
        None => {
            return TileIterator {
                current_tile: TileOffset::zero(),
                x: TileIteratorExtent {
                    tile_range: 0..0,
                    image_tiles: 0..0,
                    first_tile_layout_size: 0.0,
                    last_tile_layout_size: 0.0,
                    layout_tiling_origin: 0.0,
                    layout_prim_start: prim_rect.origin.x,
                },
                y: TileIteratorExtent {
                    tile_range: 0..0,
                    image_tiles: 0..0,
                    first_tile_layout_size: 0.0,
                    last_tile_layout_size: 0.0,
                    layout_tiling_origin: 0.0,
                    layout_prim_start: prim_rect.origin.y,
                },
                regular_tile_size: LayoutSize::zero(),
            }
        }
    };

    // Size of regular tiles in layout space.
    let layout_tile_size = LayoutSize::new(
        device_tile_size as f32 / image_rect.size.width as f32 * prim_rect.size.width,
        device_tile_size as f32 / image_rect.size.height as f32 * prim_rect.size.height,
    );

    // The decomposition logic is exactly the same on each axis so we reduce
    // this to a 1-dimensional problem in an attempt to make the code simpler.

    let x_extent = tiles_1d(
        layout_tile_size.width,
        visible_rect.x_range(),
        prim_rect.min_x(),
        image_rect.x_range(),
        device_tile_size,
    );

    let y_extent = tiles_1d(
        layout_tile_size.height,
        visible_rect.y_range(),
        prim_rect.min_y(),
        image_rect.y_range(),
        device_tile_size,
    );

    TileIterator {
        current_tile: point2(
            x_extent.tile_range.start,
            y_extent.tile_range.start,
        ),
        x: x_extent,
        y: y_extent,
        regular_tile_size: layout_tile_size,
    }
}

/// Decompose tiles along an arbitrary axis.
///
/// This does most of the heavy lifting needed for `tiles` but in a single dimension for
/// the sake of simplicity since the problem is independent on the x and y axes.
fn tiles_1d(
    layout_tile_size: f32,
    layout_visible_range: Range<f32>,
    layout_prim_start: f32,
    device_image_range: Range<i32>,
    device_tile_size: i32,
) -> TileIteratorExtent {
    // A few sanity checks.
    debug_assert!(layout_tile_size > 0.0);
    debug_assert!(layout_visible_range.end >= layout_visible_range.start);
    debug_assert!(device_image_range.end > device_image_range.start);
    debug_assert!(device_tile_size > 0);

    // Sizes of the boundary tiles in pixels.
    let first_tile_device_size = first_tile_size_1d(&device_image_range, device_tile_size);
    let last_tile_device_size = last_tile_size_1d(&device_image_range, device_tile_size);

    // [start..end[ Range of tiles of this row/column (in number of tiles) without
    // taking culling into account.
    let image_tiles = tile_range_1d(&device_image_range, device_tile_size);

    // Layout offset of tile (0, 0) with respect to the top-left corner of the display item.
    let layout_offset = device_image_range.start as f32 * layout_tile_size / device_tile_size as f32;
    // Position in layout space of tile (0, 0).
    let layout_tiling_origin = layout_prim_start - layout_offset;

    // [start..end[ Range of the visible tiles (because of culling).
    let visible_tiles_start = f32::floor((layout_visible_range.start - layout_tiling_origin) / layout_tile_size) as i32;
    let visible_tiles_end = f32::ceil((layout_visible_range.end - layout_tiling_origin) / layout_tile_size) as i32;

    // Combine the above two to get the tiles in the image that are visible this frame.
    let mut tiles_start = i32::max(image_tiles.start, visible_tiles_start);
    let tiles_end = i32::min(image_tiles.end, visible_tiles_end);
    if tiles_start > tiles_end {
        tiles_start = tiles_end;
    }

    // The size in layout space of the boundary tiles.
    let first_tile_layout_size = if tiles_start == image_tiles.start {
        first_tile_device_size as f32 * layout_tile_size / device_tile_size as f32
    } else {
        // boundary tile was culled out, so the new first tile is a regularly sized tile.
        layout_tile_size
    };

    // Same here.
    let last_tile_layout_size = if tiles_end == image_tiles.end {
        last_tile_device_size as f32 * layout_tile_size / device_tile_size as f32
    } else {
        layout_tile_size
    };

    TileIteratorExtent {
        tile_range: tiles_start..tiles_end,
        image_tiles,
        first_tile_layout_size,
        last_tile_layout_size,
        layout_tiling_origin,
        layout_prim_start,
    }
}

/// Compute the range of tiles (in number of tiles) that intersect the provided
/// image range (in pixels) in an arbitrary dimension.
///
/// ```ignore
///
///         0
///         :
///   #-+---+---+---+---+---+--#
///   # |   |   |   |   |   |  #
///   #-+---+---+---+---+---+--#
/// ^       :                   ^
///
///  +------------------------+  image_range
///        +---+  regular_tile_size
///
/// ```
fn tile_range_1d(
    image_range: &Range<i32>,
    regular_tile_size: i32,
) -> Range<i32> {
    // Integer division truncates towards zero so with negative values if the first/last
    // tile isn't a full tile we can get offset by one which we account for here.

    let mut start = image_range.start / regular_tile_size;
    if image_range.start % regular_tile_size < 0 {
        start -= 1;
    }

    let mut end = image_range.end / regular_tile_size;
    if image_range.end % regular_tile_size > 0 {
        end += 1;
    }

    start..end
}

// Sizes of the first boundary tile in pixels.
//
// It can be smaller than the regular tile size if the image is not a multiple
// of the regular tile size.
fn first_tile_size_1d(
    image_range: &Range<i32>,
    regular_tile_size: i32,
) -> i32 {
    // We have to account for how the % operation behaves for negative values.
    let image_size = image_range.end - image_range.start;
    i32::min(
        match image_range.start % regular_tile_size {
            //             .      #------+------+      .
            //             .      #//////|      |      .
            0 => regular_tile_size,
            //   (zero) -> 0      .   #--+------+      .
            //             .      .   #//|      |      .
            // %(m):                  ~~>
            m if m > 0 => regular_tile_size - m,
            //             .      .   #--+------+      0 <- (zero)
            //             .      .   #//|      |      .
            // %(m):                  <~~
            m => -m,
        },
        image_size
    )
}

// Sizes of the last boundary tile in pixels.
//
// It can be smaller than the regular tile size if the image is not a multiple
// of the regular tile size.
fn last_tile_size_1d(
    image_range: &Range<i32>,
    regular_tile_size: i32,
) -> i32 {
    // We have to account for how the modulo operation behaves for negative values.
    let image_size = image_range.end - image_range.start;
    i32::min(
        match image_range.end % regular_tile_size {
            //                    +------+------#      .
            // tiles:      .      |      |//////#      .
            0 => regular_tile_size,
            //             .      +------+--#   .      0 <- (zero)
            //             .      |      |//#   .      .
            // modulo (m):                   <~~
            m if m < 0 => regular_tile_size + m,
            //   (zero) -> 0      +------+--#   .      .
            //             .      |      |//#   .      .
            // modulo (m):                ~~>
            m => m,
        },
        image_size,
    )
}

pub fn compute_tile_rect(
    image_rect: &DeviceIntRect,
    regular_tile_size: TileSize,
    tile: TileOffset,
) -> DeviceIntRect {
    let regular_tile_size = regular_tile_size as i32;
    DeviceIntRect {
        origin: point2(
            compute_tile_origin_1d(image_rect.x_range(), regular_tile_size, tile.x as i32),
            compute_tile_origin_1d(image_rect.y_range(), regular_tile_size, tile.y as i32),
        ),
        size: size2(
            compute_tile_size_1d(image_rect.x_range(), regular_tile_size, tile.x as i32),
            compute_tile_size_1d(image_rect.y_range(), regular_tile_size, tile.y as i32),
        ),
    }
}

fn compute_tile_origin_1d(
    img_range: Range<i32>,
    regular_tile_size: i32,
    tile_offset: i32,
) -> i32 {
    let tile_range = tile_range_1d(&img_range, regular_tile_size);
    if tile_offset == tile_range.start {
        img_range.start
    } else {
        tile_offset * regular_tile_size
    }
}

// Compute the width and height in pixels of a tile depending on its position in the image.
pub fn compute_tile_size(
    image_rect: &DeviceIntRect,
    regular_tile_size: TileSize,
    tile: TileOffset,
) -> DeviceIntSize {
    let regular_tile_size = regular_tile_size as i32;
    size2(
        compute_tile_size_1d(image_rect.x_range(), regular_tile_size, tile.x as i32),
        compute_tile_size_1d(image_rect.y_range(), regular_tile_size, tile.y as i32),
    )
}

fn compute_tile_size_1d(
    img_range: Range<i32>,
    regular_tile_size: i32,
    tile_offset: i32,
) -> i32 {
    let tile_range = tile_range_1d(&img_range, regular_tile_size);

    // Most tiles are going to have base_size as width and height,
    // except for tiles around the edges that are shrunk to fit the image data.
    let actual_size = if tile_offset == tile_range.start {
        first_tile_size_1d(&img_range, regular_tile_size)
    } else if tile_offset == tile_range.end - 1 {
        last_tile_size_1d(&img_range, regular_tile_size)
    } else {
        regular_tile_size
    };

    assert!(actual_size > 0);

    actual_size
}

pub fn compute_tile_range(
    visible_area: &DeviceIntRect,
    tile_size: u16,
) -> TileRange {
    let tile_size = tile_size as i32;
    let x_range = tile_range_1d(&visible_area.x_range(), tile_size);
    let y_range = tile_range_1d(&visible_area.y_range(), tile_size);

    TileRange {
        origin: point2(x_range.start, y_range.start),
        size: size2(x_range.end - x_range.start, y_range.end - y_range.start),
    }
}

pub fn for_each_tile_in_range(
    range: &TileRange,
    mut callback: impl FnMut(TileOffset),
) {
    for y in range.y_range() {
        for x in range.x_range() {
            callback(point2(x, y));
        }
    }
}

pub fn compute_valid_tiles_if_bounds_change(
    prev_rect: &DeviceIntRect,
    new_rect: &DeviceIntRect,
    tile_size: u16,
) -> Option<TileRange> {
    let intersection = match prev_rect.intersection(new_rect) {
        Some(rect) => rect,
        None => {
            return Some(TileRange::zero());
        }
    };

    let left = prev_rect.min_x() != new_rect.min_x();
    let right = prev_rect.max_x() != new_rect.max_x();
    let top = prev_rect.min_y() != new_rect.min_y();
    let bottom = prev_rect.max_y() != new_rect.max_y();

    if !left && !right && !top && !bottom {
        // Bounds have not changed.
        return None;
    }

    let tw = 1.0 / (tile_size as f32);
    let th = 1.0 / (tile_size as f32);

    let tiles = intersection
        .cast::<f32>()
        .scale(tw, th);

    let min_x = if left { f32::ceil(tiles.min_x()) } else { f32::floor(tiles.min_x()) };
    let min_y = if top { f32::ceil(tiles.min_y()) } else { f32::floor(tiles.min_y()) };
    let max_x = if right { f32::floor(tiles.max_x()) } else { f32::ceil(tiles.max_x()) };
    let max_y = if bottom { f32::floor(tiles.max_y()) } else { f32::ceil(tiles.max_y()) };

    Some(TileRange {
        origin: point2(min_x as i32, min_y as i32),
        size: size2((max_x - min_x) as i32, (max_y - min_y) as i32),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use euclid::rect;

    // this checks some additional invariants
    fn checked_for_each_tile(
        prim_rect: &LayoutRect,
        visible_rect: &LayoutRect,
        device_image_rect: &DeviceIntRect,
        device_tile_size: i32,
        callback: &mut dyn FnMut(&LayoutRect, TileOffset, EdgeAaSegmentMask),
    ) {
        let mut coverage = LayoutRect::zero();
        let mut seen_tiles = HashSet::new();
        for tile in tiles(
            prim_rect,
            visible_rect,
            device_image_rect,
            device_tile_size,
        ) {
            // make sure we don't get sent duplicate tiles
            assert!(!seen_tiles.contains(&tile.offset));
            seen_tiles.insert(tile.offset);
            coverage = coverage.union(&tile.rect);
            assert!(prim_rect.contains_rect(&tile.rect));
            callback(&tile.rect, tile.offset, tile.edge_flags);
        }
        assert!(prim_rect.contains_rect(&coverage));
        assert!(coverage.contains_rect(&visible_rect.intersection(&prim_rect).unwrap_or(LayoutRect::zero())));
    }

    #[test]
    fn basic() {
        let mut count = 0;
        checked_for_each_tile(&rect(0., 0., 1000., 1000.),
            &rect(75., 75., 400., 400.),
            &rect(0, 0, 400, 400),
            36,
            &mut |_tile_rect, _tile_offset, _tile_flags| {
                count += 1;
            },
        );
        assert_eq!(count, 36);
    }

    #[test]
    fn empty() {
        let mut count = 0;
        checked_for_each_tile(&rect(0., 0., 74., 74.),
            &rect(75., 75., 400., 400.),
            &rect(0, 0, 400, 400),
            36,
            &mut |_tile_rect, _tile_offset, _tile_flags| {
              count += 1;
            },
        );
        assert_eq!(count, 0);
    }

    #[test]
    fn test_tiles_1d() {
        // Exactly one full tile at positive offset.
        let result = tiles_1d(64.0, -10000.0..10000.0, 0.0, 0..64, 64);
        assert_eq!(result.tile_range.start, 0);
        assert_eq!(result.tile_range.end, 1);
        assert_eq!(result.first_tile_layout_size, 64.0);
        assert_eq!(result.last_tile_layout_size, 64.0);

        // Exactly one full tile at negative offset.
        let result = tiles_1d(64.0, -10000.0..10000.0, -64.0, -64..0, 64);
        assert_eq!(result.tile_range.start, -1);
        assert_eq!(result.tile_range.end, 0);
        assert_eq!(result.first_tile_layout_size, 64.0);
        assert_eq!(result.last_tile_layout_size, 64.0);

        // Two full tiles at negative and positive offsets.
        let result = tiles_1d(64.0, -10000.0..10000.0, -64.0, -64..64, 64);
        assert_eq!(result.tile_range.start, -1);
        assert_eq!(result.tile_range.end, 1);
        assert_eq!(result.first_tile_layout_size, 64.0);
        assert_eq!(result.last_tile_layout_size, 64.0);

        // One partial tile at positive offset, non-zero origin, culled out.
        let result = tiles_1d(64.0, -100.0..10.0, 64.0, 64..310, 64);
        assert_eq!(result.tile_range.start, result.tile_range.end);

        // Two tiles at negative and positive offsets, one of which is culled out.
        // The remaining tile is partially culled but it should still generate a full tile.
        let result = tiles_1d(64.0, 10.0..10000.0, -64.0, -64..64, 64);
        assert_eq!(result.tile_range.start, 0);
        assert_eq!(result.tile_range.end, 1);
        assert_eq!(result.first_tile_layout_size, 64.0);
        assert_eq!(result.last_tile_layout_size, 64.0);
        let result = tiles_1d(64.0, -10000.0..-10.0, -64.0, -64..64, 64);
        assert_eq!(result.tile_range.start, -1);
        assert_eq!(result.tile_range.end, 0);
        assert_eq!(result.first_tile_layout_size, 64.0);
        assert_eq!(result.last_tile_layout_size, 64.0);

        // Stretched tile in layout space device tile size is 64 and layout tile size is 128.
        // So the resulting tile sizes in layout space should be multiplied by two.
        let result = tiles_1d(128.0, -10000.0..10000.0, -64.0, -64..32, 64);
        assert_eq!(result.tile_range.start, -1);
        assert_eq!(result.tile_range.end, 1);
        assert_eq!(result.first_tile_layout_size, 128.0);
        assert_eq!(result.last_tile_layout_size, 64.0);

        // Two visible tiles (the rest is culled out).
        let result = tiles_1d(10.0, 0.0..20.0, 0.0, 0..64, 64);
        assert_eq!(result.tile_range.start, 0);
        assert_eq!(result.tile_range.end, 1);
        assert_eq!(result.first_tile_layout_size, 10.0);
        assert_eq!(result.last_tile_layout_size, 10.0);

        // Two visible tiles at negative layout offsets (the rest is culled out).
        let result = tiles_1d(10.0, -20.0..0.0, -20.0, 0..64, 64);
        assert_eq!(result.tile_range.start, 0);
        assert_eq!(result.tile_range.end, 1);
        assert_eq!(result.first_tile_layout_size, 10.0);
        assert_eq!(result.last_tile_layout_size, 10.0);
    }

    #[test]
    fn test_tile_range_1d() {
        assert_eq!(tile_range_1d(&(0..256), 256), 0..1);
        assert_eq!(tile_range_1d(&(0..257), 256), 0..2);
        assert_eq!(tile_range_1d(&(-1..257), 256), -1..2);
        assert_eq!(tile_range_1d(&(-256..256), 256), -1..1);
        assert_eq!(tile_range_1d(&(-20..-10), 6), -4..-1);
        assert_eq!(tile_range_1d(&(20..100), 256), 0..1);
    }

    #[test]
    fn test_first_last_tile_size_1d() {
        assert_eq!(first_tile_size_1d(&(0..10), 64), 10);
        assert_eq!(first_tile_size_1d(&(-20..0), 64), 20);

        assert_eq!(last_tile_size_1d(&(0..10), 64), 10);
        assert_eq!(last_tile_size_1d(&(-20..0), 64), 20);
    }

    #[test]
    fn doubly_partial_tiles() {
        // In the following tests the image is a single tile and none of the sides of the tile
        // align with the tile grid.
        // This can only happen when we have a single non-aligned partial tile and no regular
        // tiles.
        assert_eq!(first_tile_size_1d(&(300..310), 64), 10);
        assert_eq!(first_tile_size_1d(&(-20..-10), 64), 10);

        assert_eq!(last_tile_size_1d(&(300..310), 64), 10);
        assert_eq!(last_tile_size_1d(&(-20..-10), 64), 10);


        // One partial tile at positve offset, non-zero origin.
        let result = tiles_1d(64.0, -10000.0..10000.0, 0.0, 300..310, 64);
        assert_eq!(result.tile_range.start, 4);
        assert_eq!(result.tile_range.end, 5);
        assert_eq!(result.first_tile_layout_size, 10.0);
        assert_eq!(result.last_tile_layout_size, 10.0);
    }

    #[test]
    fn smaller_than_tile_size_at_origin() {
        let r = compute_tile_rect(
            &rect(0, 0, 80, 80),
            256,
            point2(0, 0),
        );

        assert_eq!(r, rect(0, 0, 80, 80));
    }

    #[test]
    fn smaller_than_tile_size_with_offset() {
        let r = compute_tile_rect(
            &rect(20, 20, 80, 80),
            256,
            point2(0, 0),
        );

        assert_eq!(r, rect(20, 20, 80, 80));
    }
}
