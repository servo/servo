// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use geom::{Size2D, Point2D, SideOffsets2D, Rect};
use util::logical_geometry::{WritingMode, LogicalSize, LogicalPoint, LogicalMargin, LogicalRect,
                             FLAG_RTL, FLAG_VERTICAL, FLAG_VERTICAL_LR, FLAG_SIDEWAYS_LEFT};

#[cfg(test)]
fn modes() -> [WritingMode; 10] {
    [
        WritingMode::empty(),
        FLAG_VERTICAL,
        FLAG_VERTICAL | FLAG_VERTICAL_LR,
        FLAG_VERTICAL | FLAG_VERTICAL_LR | FLAG_SIDEWAYS_LEFT,
        FLAG_VERTICAL | FLAG_SIDEWAYS_LEFT,
        FLAG_RTL,
        FLAG_VERTICAL | FLAG_RTL,
        FLAG_VERTICAL | FLAG_VERTICAL_LR | FLAG_RTL,
        FLAG_VERTICAL | FLAG_VERTICAL_LR | FLAG_SIDEWAYS_LEFT | FLAG_RTL,
        FLAG_VERTICAL | FLAG_SIDEWAYS_LEFT | FLAG_RTL,
    ]
}

#[test]
fn test_size_round_trip() {
    let physical = Size2D(1u32, 2u32);
    for &mode in modes().iter() {
        let logical = LogicalSize::from_physical(mode, physical);
        assert!(logical.to_physical(mode) == physical);
        assert!(logical.width(mode) == 1);
        assert!(logical.height(mode) == 2);
    }
}

#[test]
fn test_point_round_trip() {
    let physical = Point2D(1u32, 2u32);
    let container = Size2D(100, 200);
    for &mode in modes().iter() {
        let logical = LogicalPoint::from_physical(mode, physical, container);
        assert!(logical.to_physical(mode, container) == physical);
        assert!(logical.x(mode, container) == 1);
        assert!(logical.y(mode, container) == 2);
    }
}

#[test]
fn test_margin_round_trip() {
    let physical = SideOffsets2D::new(1u32, 2u32, 3u32, 4u32);
    for &mode in modes().iter() {
        let logical = LogicalMargin::from_physical(mode, physical);
        assert!(logical.to_physical(mode) == physical);
        assert!(logical.top(mode) == 1);
        assert!(logical.right(mode) == 2);
        assert!(logical.bottom(mode) == 3);
        assert!(logical.left(mode) == 4);
    }
}

#[test]
fn test_rect_round_trip() {
    let physical = Rect(Point2D(1u32, 2u32), Size2D(3u32, 4u32));
    let container = Size2D(100, 200);
    for &mode in modes().iter() {
        let logical = LogicalRect::from_physical(mode, physical, container);
        assert!(logical.to_physical(mode, container) == physical);
    }
}
