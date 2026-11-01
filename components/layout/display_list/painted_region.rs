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
/// `cc::Region`
#[derive(Clone, Debug, Default)]
pub(crate) struct PaintedRegion {
    /// No two rects in this list overlap.
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

    /// Merges `rect` into the region, splitting it against existing rects so the
    /// region remains disjoint.
    pub(crate) fn union(&mut self, rect: Box2D<f32, LayoutPixel>) {
        let rect = rect.round_out();
        if rect.is_empty() {
            return;
        }

        // Only existing rects that overlap `rect` can take anything away from it. For
        // typical content (e.g. a page of text), new fragments rarely overlap what is
        // already painted, so this is usually empty and `rect` can be added as is,
        // without any allocation.
        let overlapping: Vec<_> = self
            .rects
            .iter()
            .copied()
            .filter(|existing| existing.intersects(&rect))
            .collect();
        if overlapping.is_empty() {
            self.rects.push(rect);
            return;
        }

        // Fast path: a single existing rect fully containing the new one is the
        // common overlapping case (the same fragment, unchanged, revisited on an
        // unrelated repaint).
        if overlapping
            .iter()
            .any(|existing| existing.contains_box(&rect))
        {
            return;
        }
        self.rects.extend(subtract_all(rect, &overlapping));
    }

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
    let mut remaining = Vec::new();
    for other in others {
        if pieces.is_empty() {
            break;
        }
        for piece in pieces.drain(..) {
            subtract_one(piece, *other, &mut remaining);
        }
        std::mem::swap(&mut pieces, &mut remaining);
    }
    pieces
}

/// Pushes onto `out` the (up to four) disjoint rects that remain once `b` is removed
/// from `a`.
fn subtract_one(
    a: Box2D<f32, LayoutPixel>,
    b: Box2D<f32, LayoutPixel>,
    out: &mut Vec<Box2D<f32, LayoutPixel>>,
) {
    let Some(i) = a.intersection(&b) else {
        out.push(a);
        return;
    };
    if i.is_empty() {
        out.push(a);
        return;
    }

    if i.min.y > a.min.y {
        out.push(Box2D::new(
            Point2D::new(a.min.x, a.min.y),
            Point2D::new(a.max.x, i.min.y),
        ));
    }
    if i.max.y < a.max.y {
        out.push(Box2D::new(
            Point2D::new(a.min.x, i.max.y),
            Point2D::new(a.max.x, a.max.y),
        ));
    }
    if i.min.x > a.min.x {
        out.push(Box2D::new(
            Point2D::new(a.min.x, i.min.y),
            Point2D::new(i.min.x, i.max.y),
        ));
    }
    if i.max.x < a.max.x {
        out.push(Box2D::new(
            Point2D::new(i.max.x, i.min.y),
            Point2D::new(a.max.x, i.max.y),
        ));
    }
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
        region.union(rect(10.0, 10.0, 20.0, 20.0));
        assert_eq!(region.area(), 10_000.0);
    }

    #[test]
    fn empty_rect_is_a_noop() {
        let mut region = PaintedRegion::default();
        region.union(rect(5.0, 5.0, 5.0, 5.0));
        assert_eq!(region.area(), 0.0);
    }

    #[test]
    fn rect_partially_covered_by_several_rects() {
        let mut region = PaintedRegion::default();
        region.union(rect(0.0, 0.0, 10.0, 10.0));
        region.union(rect(20.0, 0.0, 30.0, 10.0));
        // Far away, so it is not one of the rects overlapping the next one.
        region.union(rect(100.0, 100.0, 110.0, 110.0));
        // Spans both of the first two rects and the gap between them.
        region.union(rect(5.0, 5.0, 25.0, 15.0));
        // 3 * 100 + the 20x10 rect minus its two 5x5 overlaps.
        assert_eq!(region.area(), 300.0 + 200.0 - 50.0);
    }

    #[test]
    fn touching_rects_are_not_overlapping() {
        let mut region = PaintedRegion::default();
        region.union(rect(0.0, 0.0, 10.0, 10.0));
        region.union(rect(10.0, 0.0, 20.0, 10.0));
        assert_eq!(region.area(), 200.0);
        assert_eq!(region.bounds(), rect(0.0, 0.0, 20.0, 10.0));
    }

    #[test]
    fn many_disjoint_rects() {
        // A grid of small rects, like a page of text. Each rect is added unsplit.
        let mut region = PaintedRegion::default();
        for row in 0..100 {
            for column in 0..40 {
                let (x, y) = (column as f32 * 12.0, row as f32 * 8.0);
                region.union(rect(x, y, x + 10.0, y + 6.0));
                // Revisiting the same fragment adds nothing.
                region.union(rect(x, y, x + 10.0, y + 6.0));
            }
        }
        assert_eq!(region.rects.len(), 4000);
        assert_eq!(region.area(), 4000.0 * 60.0);
    }
}
