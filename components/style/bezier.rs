/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Parametric Bézier curves.
//!
//! This is based on `WebCore/platform/graphics/UnitBezier.h` in WebKit.

#![deny(missing_docs)]

use euclid::Point2D;

const NEWTON_METHOD_ITERATIONS: u8 = 8;

/// A unit cubic Bézier curve, used for timing functions in CSS transitions and animations.
pub struct Bezier {
    ax: f64,
    bx: f64,
    cx: f64,
    ay: f64,
    by: f64,
    cy: f64,
}

impl Bezier {
    /// Create a unit cubic Bézier curve from the two middle control points.
    ///
    /// X coordinate is time, Y coordinate is function advancement.
    /// The nominal range for both is 0 to 1.
    ///
    /// The start and end points are always (0, 0) and (1, 1) so that a transition or animation
    /// starts at 0% and ends at 100%.
    #[inline]
    pub fn new(p1: Point2D<f64>, p2: Point2D<f64>) -> Bezier {
        let cx = 3.0 * p1.x;
        let bx = 3.0 * (p2.x - p1.x) - cx;

        let cy = 3.0 * p1.y;
        let by = 3.0 * (p2.y - p1.y) - cy;

        Bezier {
            ax: 1.0 - cx - bx,
            bx: bx,
            cx: cx,
            ay: 1.0 - cy - by,
            by: by,
            cy: cy,
        }
    }

    #[inline]
    fn sample_curve_x(&self, t: f64) -> f64 {
        // ax * t^3 + bx * t^2 + cx * t
        ((self.ax * t + self.bx) * t + self.cx) * t
    }

    #[inline]
    fn sample_curve_y(&self, t: f64) -> f64 {
        ((self.ay * t + self.by) * t + self.cy) * t
    }

    #[inline]
    fn sample_curve_derivative_x(&self, t: f64) -> f64 {
        (3.0 * self.ax * t + 2.0 * self.bx) * t + self.cx
    }

    #[inline]
    fn solve_curve_x(&self, x: f64, epsilon: f64) -> f64 {
        // Fast path: Use Newton's method.
        let mut t = x;
        for _ in 0..NEWTON_METHOD_ITERATIONS {
            let x2 = self.sample_curve_x(t);
            if x2.approx_eq(x, epsilon) {
                return t
            }
            let dx = self.sample_curve_derivative_x(t);
            if dx.approx_eq(0.0, 1e-6) {
                break
            }
            t -= (x2 - x) / dx;
        }

        // Slow path: Use bisection.
        let (mut lo, mut hi, mut t) = (0.0, 1.0, x);

        if t < lo {
            return lo
        }
        if t > hi {
            return hi
        }

        while lo < hi {
            let x2 = self.sample_curve_x(t);
            if x2.approx_eq(x, epsilon) {
                return t
            }
            if x > x2 {
                lo = t
            } else {
                hi = t
            }
            t = (hi - lo) / 2.0 + lo
        }

        t
    }

    /// Solve the bezier curve for a given `x` and an `epsilon`, that should be
    /// between zero and one.
    #[inline]
    pub fn solve(&self, x: f64, epsilon: f64) -> f64 {
        self.sample_curve_y(self.solve_curve_x(x, epsilon))
    }
}

trait ApproxEq {
    fn approx_eq(self, value: Self, epsilon: Self) -> bool;
}

impl ApproxEq for f64 {
    #[inline]
    fn approx_eq(self, value: f64, epsilon: f64) -> bool {
        (self - value).abs() < epsilon
    }
}

