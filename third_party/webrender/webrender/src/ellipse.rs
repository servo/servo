/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::units::*;
use euclid::Size2D;
use std::f32::consts::FRAC_PI_2;


/// Number of steps to integrate arc length over.
const STEP_COUNT: usize = 20;

/// Represents an ellipse centred at a local space origin.
#[derive(Debug, Clone)]
pub struct Ellipse<U> {
    pub radius: Size2D<f32, U>,
    pub total_arc_length: f32,
}

impl<U> Ellipse<U> {
    pub fn new(radius: Size2D<f32, U>) -> Ellipse<U> {
        // Approximate the total length of the first quadrant of this ellipse.
        let total_arc_length = get_simpson_length(FRAC_PI_2, radius.width, radius.height);

        Ellipse {
            radius,
            total_arc_length,
        }
    }

    /// Binary search to estimate the angle of an ellipse
    /// for a given arc length. This only searches over the
    /// first quadrant of an ellipse.
    pub fn find_angle_for_arc_length(&self, arc_length: f32) -> f32 {
        // Clamp arc length to [0, pi].
        let arc_length = arc_length.max(0.0).min(self.total_arc_length);

        let epsilon = 0.01;
        let mut low = 0.0;
        let mut high = FRAC_PI_2;
        let mut theta = 0.0;
        let mut new_low = 0.0;
        let mut new_high = FRAC_PI_2;

        while low <= high {
            theta = 0.5 * (low + high);
            let length = get_simpson_length(theta, self.radius.width, self.radius.height);

            if (length - arc_length).abs() < epsilon {
                break;
            } else if length < arc_length {
                new_low = theta;
            } else {
                new_high = theta;
            }

            // If we have stopped moving down the arc, the answer that we have is as good as
            // it is going to get. We break to avoid going into an infinite loop.
            if new_low == low && new_high == high {
                break;
            }

            high = new_high;
            low = new_low;
        }

        theta
    }

    /// Get a point and tangent on this ellipse from a given angle.
    /// This only works for the first quadrant of the ellipse.
    pub fn get_point_and_tangent(&self, theta: f32) -> (LayoutPoint, LayoutPoint) {
        let (sin_theta, cos_theta) = theta.sin_cos();
        let point = LayoutPoint::new(
            self.radius.width * cos_theta,
            self.radius.height * sin_theta,
        );
        let tangent = LayoutPoint::new(
            -self.radius.width * sin_theta,
            self.radius.height * cos_theta,
        );
        (point, tangent)
    }

    pub fn contains(&self, point: LayoutPoint) -> bool {
        self.signed_distance(point.to_vector()) <= 0.0
    }

    /// Find the signed distance from this ellipse given a point.
    /// Taken from http://www.iquilezles.org/www/articles/ellipsedist/ellipsedist.htm
    fn signed_distance(&self, point: LayoutVector2D) -> f32 {
        // This algorithm fails for circles, so we handle them here.
        if self.radius.width == self.radius.height {
            return point.length() - self.radius.width;
        }

        let mut p = LayoutVector2D::new(point.x.abs(), point.y.abs());
        let mut ab = self.radius.to_vector();
        if p.x > p.y {
            p = p.yx();
            ab = ab.yx();
        }

        let l = ab.y * ab.y - ab.x * ab.x;

        let m = ab.x * p.x / l;
        let n = ab.y * p.y / l;
        let m2 = m * m;
        let n2 = n * n;

        let c = (m2 + n2 - 1.0) / 3.0;
        let c3 = c * c * c;

        let q = c3 + m2 * n2 * 2.0;
        let d = c3 + m2 * n2;
        let g = m + m * n2;

        let co = if d < 0.0 {
            let p = (q / c3).acos() / 3.0;
            let s = p.cos();
            let t = p.sin() * (3.0_f32).sqrt();
            let rx = (-c * (s + t + 2.0) + m2).sqrt();
            let ry = (-c * (s - t + 2.0) + m2).sqrt();
            (ry + l.signum() * rx + g.abs() / (rx * ry) - m) / 2.0
        } else {
            let h = 2.0 * m * n * d.sqrt();
            let s = (q + h).signum() * (q + h).abs().powf(1.0 / 3.0);
            let u = (q - h).signum() * (q - h).abs().powf(1.0 / 3.0);
            let rx = -s - u - c * 4.0 + 2.0 * m2;
            let ry = (s - u) * (3.0_f32).sqrt();
            let rm = (rx * rx + ry * ry).sqrt();
            let p = ry / (rm - rx).sqrt();
            (p + 2.0 * g / rm - m) / 2.0
        };

        let si = (1.0 - co * co).sqrt();
        let r = LayoutVector2D::new(ab.x * co, ab.y * si);
        (r - p).length() * (p.y - r.y).signum()
    }
}

/// Use Simpsons rule to approximate the arc length of
/// part of an ellipse. Note that this only works over
/// the range of [0, pi/2].
// TODO(gw): This is a simplistic way to estimate the
// arc length of an ellipse segment. We can probably use
// a faster / more accurate method!
fn get_simpson_length(theta: f32, rx: f32, ry: f32) -> f32 {
    let df = theta / STEP_COUNT as f32;
    let mut sum = 0.0;

    for i in 0 .. (STEP_COUNT + 1) {
        let (sin_theta, cos_theta) = (i as f32 * df).sin_cos();
        let a = rx * sin_theta;
        let b = ry * cos_theta;
        let y = (a * a + b * b).sqrt();
        let q = if i == 0 || i == STEP_COUNT {
            1.0
        } else if i % 2 == 0 {
            2.0
        } else {
            4.0
        };

        sum += q * y;
    }

    (df / 3.0) * sum
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn find_angle_for_arc_length_for_long_eclipse() {
        // Ensure that finding the angle on giant ellipses produces and answer and
        // doesn't send us into an infinite loop.
        let ellipse = Ellipse::new(LayoutSize::new(57500.0, 25.0));
        let _ = ellipse.find_angle_for_arc_length(55674.53);
        assert!(true);

        let ellipse = Ellipse::new(LayoutSize::new(25.0, 57500.0));
        let _ = ellipse.find_angle_for_arc_length(55674.53);
        assert!(true);
    }
}
