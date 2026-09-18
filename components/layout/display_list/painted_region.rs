/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use euclid::{Box2D, Point2D};
use webrender_api::units::LayoutPixel;

/// A set of disjoint, axis-aligned rectangles that tracks exactly which area has
/// already been painted for a Container Timing container, so that overlapping
/// fragments (e.g. two overlapping `<img>`s) aren't double-counted.
///
/// Mirrors the approach Chromium's Container Timing implementation takes with
/// `cc::Region` in `ContainerTiming::Record::MaybeUpdateLastNewPaintedArea`: keep a
/// region of everything painted so far, skip anything already fully covered, and
/// only ever add the non-overlapping remainder of a new rect
#[derive(Clone, Debug, Default)]
pub(crate) struct PaintedRegion {
    /// Invariant: no two rects in this list overlap.
    rects: Vec<Box2D<f32, LayoutPixel>>,
}

impl PaintedRegion {
    /// The total painted area, i.e. the sum of the (disjoint) rects' areas.
    pub(crate) fn area(&self) -> f32 {
        self.rects.iter().map(Box2D::area).sum()
    }

    /// The smallest box containing every rect in this region.
    pub(crate) fn bounds(&self) -> Box2D<f32, LayoutPixel> {
        // Seeded from the first rect rather than `Box2D::zero()`: union-ing with the
        // zero box would drag the bounds back to the origin for any region that does
        // not already reach it.
        self.rects
            .iter()
            .copied()
            .reduce(|bounds, rect| bounds.union(&rect))
            .unwrap_or_else(Box2D::zero)
    }

    /// Returns `true` if `rect` is already entirely covered by this region, i.e.
    /// merging it in would add no new area.
    pub(crate) fn contains(&self, rect: Box2D<f32, LayoutPixel>) -> bool {
        if rect.is_empty() {
            return true;
        }
        // Fast path: a single existing rect fully containing the new one is the
        // common case (the same fragment, unchanged, revisited on
        // an unrelated repaint).
        if self
            .rects
            .iter()
            .any(|existing| existing.contains_box(&rect))
        {
            return true;
        }
        subtract_all(rect, &self.rects).is_empty()
    }

    /// Merges `rect` into the region, splitting it against existing rects so the
    /// region remains disjoint. Pixel-snaps outward first, both to bound the
    /// number of distinct rects that can accumulate and to avoid spurious area
    /// growth from sub-pixel jitter between builds (matching Chromium's use of
    /// `gfx::ToEnclosingRect` before touching `cc::Region`).
    pub(crate) fn union(&mut self, rect: Box2D<f32, LayoutPixel>) {
        let rect = rect.round_out();
        if rect.is_empty() || self.contains(rect) {
            return;
        }
        self.rects.extend(subtract_all(rect, &self.rects));
    }

    /// Merges every rect of `other` into this region.
    pub(crate) fn merge(&mut self, other: &PaintedRegion) {
        for &rect in &other.rects {
            self.union(rect);
        }
    }
}

/// Returns the portion of `rect` not covered by any rect in `others`, as a set of
/// disjoint rects (possibly empty, if `others` fully covers `rect`).
fn subtract_all(
    rect: Box2D<f32, LayoutPixel>,
    others: &[Box2D<f32, LayoutPixel>],
) -> Vec<Box2D<f32, LayoutPixel>> {
    let mut pieces = vec![rect];
    for other in others {
        if pieces.is_empty() {
            break;
        }
        pieces = pieces
            .into_iter()
            .flat_map(|piece| subtract_one(piece, *other))
            .collect();
    }
    pieces
}

/// Splits `a` into the (up to four) disjoint rects that remain once `b` is removed
/// from it.
fn subtract_one(
    a: Box2D<f32, LayoutPixel>,
    b: Box2D<f32, LayoutPixel>,
) -> Vec<Box2D<f32, LayoutPixel>> {
    let Some(i) = a.intersection(&b) else {
        return vec![a];
    };
    if i.is_empty() {
        return vec![a];
    }

    let mut out = Vec::with_capacity(4);
    // Above the intersection, full width of `a`.
    if i.min.y > a.min.y {
        out.push(Box2D::new(
            Point2D::new(a.min.x, a.min.y),
            Point2D::new(a.max.x, i.min.y),
        ));
    }
    // Below the intersection, full width of `a`.
    if i.max.y < a.max.y {
        out.push(Box2D::new(
            Point2D::new(a.min.x, i.max.y),
            Point2D::new(a.max.x, a.max.y),
        ));
    }
    // Left of the intersection, clipped to the intersection's height so it
    // doesn't overlap the strips above/below.
    if i.min.x > a.min.x {
        out.push(Box2D::new(
            Point2D::new(a.min.x, i.min.y),
            Point2D::new(i.min.x, i.max.y),
        ));
    }
    // Right of the intersection, likewise clipped.
    if i.max.x < a.max.x {
        out.push(Box2D::new(
            Point2D::new(i.max.x, i.min.y),
            Point2D::new(a.max.x, i.max.y),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use euclid::Box2D;

    use super::PaintedRegion;

    fn rect(x0: f32, y0: f32, x1: f32, y1: f32) -> Box2D<f32, webrender_api::units::LayoutPixel> {
        Box2D::new((x0, y0).into(), (x1, y1).into())
    }

    #[test]
    fn repeated_identical_rect_does_not_grow() {
        let mut region = PaintedRegion::default();
        region.union(rect(0.0, 0.0, 10.0, 10.0));
        assert_eq!(region.area(), 100.0);
        for _ in 0..5 {
            assert!(region.contains(rect(0.0, 0.0, 10.0, 10.0)));
            region.union(rect(0.0, 0.0, 10.0, 10.0));
        }
        assert_eq!(region.area(), 100.0);
    }

    #[test]
    fn overlapping_rects_are_not_double_counted() {
        let mut region = PaintedRegion::default();
        region.union(rect(0.0, 0.0, 10.0, 10.0));
        region.union(rect(5.0, 5.0, 15.0, 15.0));
        // Union of two 10x10 squares overlapping in a 5x5 corner: 100 + 100 - 25.
        assert_eq!(region.area(), 175.0);
        assert_eq!(region.bounds(), rect(0.0, 0.0, 15.0, 15.0));
    }

    #[test]
    fn bounds_do_not_reach_back_to_the_origin() {
        let mut region = PaintedRegion::default();
        region.union(rect(100.0, 100.0, 200.0, 200.0));
        assert_eq!(region.bounds(), rect(100.0, 100.0, 200.0, 200.0));
    }

    #[test]
    fn disjoint_rects_sum() {
        let mut region = PaintedRegion::default();
        region.union(rect(0.0, 0.0, 10.0, 10.0));
        region.union(rect(20.0, 20.0, 30.0, 30.0));
        assert_eq!(region.area(), 200.0);
    }

    #[test]
    fn fully_contained_rect_is_a_noop() {
        let mut region = PaintedRegion::default();
        region.union(rect(0.0, 0.0, 100.0, 100.0));
        assert!(region.contains(rect(10.0, 10.0, 20.0, 20.0)));
        region.union(rect(10.0, 10.0, 20.0, 20.0));
        assert_eq!(region.area(), 10_000.0);
    }

    #[test]
    fn empty_rect_is_a_noop() {
        let mut region = PaintedRegion::default();
        region.union(rect(5.0, 5.0, 5.0, 5.0));
        assert_eq!(region.area(), 0.0);
        assert!(region.contains(rect(5.0, 5.0, 5.0, 5.0)));
    }
}
