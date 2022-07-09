/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Parametric Bézier curves.
//!
//! This is based on `WebCore/platform/graphics/UnitBezier.h` in WebKit.

#![deny(missing_docs)]

use crate::values::CSSFloat;

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
    /// Calculate the output of a unit cubic Bézier curve from the two middle control points.
    ///
    /// X coordinate is time, Y coordinate is function advancement.
    /// The nominal range for both is 0 to 1.
    ///
    /// The start and end points are always (0, 0) and (1, 1) so that a transition or animation
    /// starts at 0% and ends at 100%.
    pub fn calculate_bezier_output(
        progress: f64,
        epsilon: f64,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    ) -> f64 {
        // Check for a linear curve.
        if x1 == y1 && x2 == y2 {
            return progress;
        }

        // Ensure that we return 0 or 1 on both edges.
        if progress == 0.0 {
            return 0.0;
        }
        if progress == 1.0 {
            return 1.0;
        }

        // For negative values, try to extrapolate with tangent (p1 - p0) or,
        // if p1 is coincident with p0, with (p2 - p0).
        if progress < 0.0 {
            if x1 > 0.0 {
                return progress * y1 as f64 / x1 as f64;
            }
            if y1 == 0.0 && x2 > 0.0 {
                return progress * y2 as f64 / x2 as f64;
            }
            // If we can't calculate a sensible tangent, don't extrapolate at all.
            return 0.0;
        }

        // For values greater than 1, try to extrapolate with tangent (p2 - p3) or,
        // if p2 is coincident with p3, with (p1 - p3).
        if progress > 1.0 {
            if x2 < 1.0 {
                return 1.0 + (progress - 1.0) * (y2 as f64 - 1.0) / (x2 as f64 - 1.0);
            }
            if y2 == 1.0 && x1 < 1.0 {
                return 1.0 + (progress - 1.0) * (y1 as f64 - 1.0) / (x1 as f64 - 1.0);
            }
            // If we can't calculate a sensible tangent, don't extrapolate at all.
            return 1.0;
        }

        Bezier::new(x1, y1, x2, y2).solve(progress, epsilon)
    }

    #[inline]
    fn new(x1: CSSFloat, y1: CSSFloat, x2: CSSFloat, y2: CSSFloat) -> Bezier {
        let cx = 3. * x1 as f64;
        let bx = 3. * (x2 as f64 - x1 as f64) - cx;

        let cy = 3. * y1 as f64;
        let by = 3. * (y2 as f64 - y1 as f64) - cy;

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
                return t;
            }
            let dx = self.sample_curve_derivative_x(t);
            if dx.approx_eq(0.0, 1e-6) {
                break;
            }
            t -= (x2 - x) / dx;
        }

        // Slow path: Use bisection.
        let (mut lo, mut hi, mut t) = (0.0, 1.0, x);

        if t < lo {
            return lo;
        }
        if t > hi {
            return hi;
        }

        while lo < hi {
            let x2 = self.sample_curve_x(t);
            if x2.approx_eq(x, epsilon) {
                return t;
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
    fn solve(&self, x: f64, epsilon: f64) -> f64 {
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
