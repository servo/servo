/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use euclid::{Size2D, Point2D, SideOffsets2D, Rect};
use style::logical_geometry::{WritingMode, LogicalSize, LogicalPoint, LogicalMargin, LogicalRect};

#[cfg(test)]
fn modes() -> [WritingMode; 13] {
    [
        WritingMode::empty(),
        WritingMode::VERTICAL,
        WritingMode::VERTICAL | WritingMode::VERTICAL_LR,
        WritingMode::VERTICAL | WritingMode::VERTICAL_LR | WritingMode::SIDEWAYS,
        WritingMode::VERTICAL | WritingMode::SIDEWAYS,
        WritingMode::VERTICAL | WritingMode::UPRIGHT,
        WritingMode::RTL,
        WritingMode::VERTICAL | WritingMode::RTL,
        WritingMode::VERTICAL | WritingMode::VERTICAL_LR | WritingMode::RTL,
        WritingMode::VERTICAL | WritingMode::VERTICAL_LR | WritingMode::SIDEWAYS | WritingMode::RTL,
        WritingMode::VERTICAL | WritingMode::VERTICAL_LR | WritingMode::UPRIGHT | WritingMode::RTL,
        WritingMode::VERTICAL | WritingMode::SIDEWAYS | WritingMode::RTL,
        WritingMode::VERTICAL | WritingMode::UPRIGHT | WritingMode::RTL,
    ]
}

#[test]
fn test_size_round_trip() {
    let physical = Size2D::new(1u32, 2u32);
    for &mode in modes().iter() {
        let logical = LogicalSize::from_physical(mode, physical);
        assert!(logical.to_physical(mode) == physical);
        assert!(logical.width(mode) == 1);
        assert!(logical.height(mode) == 2);
    }
}

#[test]
fn test_point_round_trip() {
    let physical = Point2D::new(1u32, 2u32);
    let container = Size2D::new(100, 200);
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
    let physical = Rect::new(Point2D::new(1u32, 2u32), Size2D::new(3u32, 4u32));
    let container = Size2D::new(100, 200);
    for &mode in modes().iter() {
        let logical = LogicalRect::from_physical(mode, physical, container);
        assert!(logical.to_physical(mode, container) == physical);
    }
}
