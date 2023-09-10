/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::BorderRadius;
use api::units::*;
use euclid::{Point2D, Rect, Size2D, Vector2D, point2, size2};
use euclid::{default, Transform2D, Transform3D, Scale};
use malloc_size_of::{MallocShallowSizeOf, MallocSizeOf, MallocSizeOfOps};
use plane_split::{Clipper, Polygon};
use std::{i32, f32, fmt, ptr};
use std::borrow::Cow;
use std::num::NonZeroUsize;
use std::os::raw::c_void;
use std::sync::Arc;
use std::mem::replace;


// Matches the definition of SK_ScalarNearlyZero in Skia.
const NEARLY_ZERO: f32 = 1.0 / 4096.0;

/// A typesafe helper that separates new value construction from
/// vector growing, allowing LLVM to ideally construct the element in place.
pub struct Allocation<'a, T: 'a> {
    vec: &'a mut Vec<T>,
    index: usize,
}

impl<'a, T> Allocation<'a, T> {
    // writing is safe because alloc() ensured enough capacity
    // and `Allocation` holds a mutable borrow to prevent anyone else
    // from breaking this invariant.
    #[inline(always)]
    pub fn init(self, value: T) -> usize {
        unsafe {
            ptr::write(self.vec.as_mut_ptr().add(self.index), value);
            self.vec.set_len(self.index + 1);
        }
        self.index
    }
}

/// An entry into a vector, similar to `std::collections::hash_map::Entry`.
pub enum VecEntry<'a, T: 'a> {
    Vacant(Allocation<'a, T>),
    Occupied(&'a mut T),
}

impl<'a, T> VecEntry<'a, T> {
    #[inline(always)]
    pub fn set(self, value: T) {
        match self {
            VecEntry::Vacant(alloc) => { alloc.init(value); }
            VecEntry::Occupied(slot) => { *slot = value; }
        }
    }
}

pub trait VecHelper<T> {
    /// Growns the vector by a single entry, returning the allocation.
    fn alloc(&mut self) -> Allocation<T>;
    /// Either returns an existing elemenet, or grows the vector by one.
    /// Doesn't expect indices to be higher than the current length.
    fn entry(&mut self, index: usize) -> VecEntry<T>;

    /// Equivalent to `mem::replace(&mut vec, Vec::new())`
    fn take(&mut self) -> Self;

    /// Call clear and return self (useful for chaining with calls that move the vector).
    fn cleared(self) -> Self;

    /// Functionally equivalent to `mem::replace(&mut vec, Vec::new())` but tries
    /// to keep the allocation in the caller if it is empty or replace it with a
    /// pre-allocated vector.
    fn take_and_preallocate(&mut self) -> Self;
}

impl<T> VecHelper<T> for Vec<T> {
    fn alloc(&mut self) -> Allocation<T> {
        let index = self.len();
        if self.capacity() == index {
            self.reserve(1);
        }
        Allocation {
            vec: self,
            index,
        }
    }

    fn entry(&mut self, index: usize) -> VecEntry<T> {
        if index < self.len() {
            VecEntry::Occupied(unsafe {
                self.get_unchecked_mut(index)
            })
        } else {
            assert_eq!(index, self.len());
            VecEntry::Vacant(self.alloc())
        }
    }

    fn take(&mut self) -> Self {
        replace(self, Vec::new())
    }

    fn cleared(mut self) -> Self {
        self.clear();

        self
    }

    fn take_and_preallocate(&mut self) -> Self {
        let len = self.len();
        if len == 0 {
            self.clear();
            return Vec::new();
        }
        replace(self, Vec::with_capacity(len + 8))
    }
}


// Represents an optimized transform where there is only
// a scale and translation (which are guaranteed to maintain
// an axis align rectangle under transformation). The
// scaling is applied first, followed by the translation.
// TODO(gw): We should try and incorporate F <-> T units here,
//           but it's a bit tricky to do that now with the
//           way the current spatial tree works.
#[derive(Debug, Clone, Copy, MallocSizeOf)]
#[cfg_attr(feature = "capture", derive(Serialize))]
pub struct ScaleOffset {
    pub scale: default::Vector2D<f32>,
    pub offset: default::Vector2D<f32>,
}

impl ScaleOffset {
    pub fn identity() -> Self {
        ScaleOffset {
            scale: Vector2D::new(1.0, 1.0),
            offset: Vector2D::zero(),
        }
    }

    // Construct a ScaleOffset from a transform. Returns
    // None if the matrix is not a pure scale / translation.
    pub fn from_transform<F, T>(
        m: &Transform3D<f32, F, T>,
    ) -> Option<ScaleOffset> {

        // To check that we have a pure scale / translation:
        // Every field must match an identity matrix, except:
        //  - Any value present in tx,ty
        //  - Any value present in sx,sy

        if m.m12.abs() > NEARLY_ZERO ||
           m.m13.abs() > NEARLY_ZERO ||
           m.m14.abs() > NEARLY_ZERO ||
           m.m21.abs() > NEARLY_ZERO ||
           m.m23.abs() > NEARLY_ZERO ||
           m.m24.abs() > NEARLY_ZERO ||
           m.m31.abs() > NEARLY_ZERO ||
           m.m32.abs() > NEARLY_ZERO ||
           (m.m33 - 1.0).abs() > NEARLY_ZERO ||
           m.m34.abs() > NEARLY_ZERO ||
           m.m43.abs() > NEARLY_ZERO ||
           (m.m44 - 1.0).abs() > NEARLY_ZERO {
            return None;
        }

        Some(ScaleOffset {
            scale: Vector2D::new(m.m11, m.m22),
            offset: Vector2D::new(m.m41, m.m42),
        })
    }

    pub fn from_offset(offset: default::Vector2D<f32>) -> Self {
        ScaleOffset {
            scale: Vector2D::new(1.0, 1.0),
            offset,
        }
    }

    pub fn inverse(&self) -> Self {
        ScaleOffset {
            scale: Vector2D::new(
                1.0 / self.scale.x,
                1.0 / self.scale.y,
            ),
            offset: Vector2D::new(
                -self.offset.x / self.scale.x,
                -self.offset.y / self.scale.y,
            ),
        }
    }

    pub fn offset(&self, offset: default::Vector2D<f32>) -> Self {
        self.accumulate(
            &ScaleOffset {
                scale: Vector2D::new(1.0, 1.0),
                offset,
            }
        )
    }

    pub fn scale(&self, scale: f32) -> Self {
        self.accumulate(
            &ScaleOffset {
                scale: Vector2D::new(scale, scale),
                offset: Vector2D::zero(),
            }
        )
    }

    /// Produce a ScaleOffset that includes both self and other.
    /// The 'self' ScaleOffset is applied after other.
    /// This is equivalent to `Transform3D::pre_transform`.
    pub fn accumulate(&self, other: &ScaleOffset) -> Self {
        ScaleOffset {
            scale: Vector2D::new(
                self.scale.x * other.scale.x,
                self.scale.y * other.scale.y,
            ),
            offset: Vector2D::new(
                self.offset.x + self.scale.x * other.offset.x,
                self.offset.y + self.scale.y * other.offset.y,
            ),
        }
    }

    pub fn map_rect<F, T>(&self, rect: &Rect<f32, F>) -> Rect<f32, T> {
        // TODO(gw): The logic below can return an unexpected result if the supplied
        //           rect is invalid (has size < 0). Since Gecko currently supplied
        //           invalid rects in some cases, adding a max(0) here ensures that
        //           mapping an invalid rect retains the property that rect.is_empty()
        //           will return true (the mapped rect output will have size 0 instead
        //           of a negative size). In future we could catch / assert / fix
        //           these invalid rects earlier, and assert here instead.

        let w = rect.size.width.max(0.0);
        let h = rect.size.height.max(0.0);

        let mut x0 = rect.origin.x * self.scale.x + self.offset.x;
        let mut y0 = rect.origin.y * self.scale.y + self.offset.y;

        let mut sx = w * self.scale.x;
        let mut sy = h * self.scale.y;

        // Handle negative scale. Previously, branchless float math was used to find the
        // min / max vertices and size. However, that sequence of operations was producind
        // additional floating point accuracy on android emulator builds, causing one test
        // to fail an assert. Instead, we retain the same math as previously, and adjust
        // the origin / size if required.

        if self.scale.x < 0.0 {
            x0 += sx;
            sx = -sx;
        }
        if self.scale.y < 0.0 {
            y0 += sy;
            sy = -sy;
        }

        Rect::new(
            Point2D::new(x0, y0),
            Size2D::new(sx, sy),
        )
    }

    pub fn unmap_rect<F, T>(&self, rect: &Rect<f32, F>) -> Rect<f32, T> {
        // TODO(gw): The logic below can return an unexpected result if the supplied
        //           rect is invalid (has size < 0). Since Gecko currently supplied
        //           invalid rects in some cases, adding a max(0) here ensures that
        //           mapping an invalid rect retains the property that rect.is_empty()
        //           will return true (the mapped rect output will have size 0 instead
        //           of a negative size). In future we could catch / assert / fix
        //           these invalid rects earlier, and assert here instead.

        let w = rect.size.width.max(0.0);
        let h = rect.size.height.max(0.0);

        let mut x0 = (rect.origin.x - self.offset.x) / self.scale.x;
        let mut y0 = (rect.origin.y - self.offset.y) / self.scale.y;

        let mut sx = w / self.scale.x;
        let mut sy = h / self.scale.y;

        // Handle negative scale. Previously, branchless float math was used to find the
        // min / max vertices and size. However, that sequence of operations was producind
        // additional floating point accuracy on android emulator builds, causing one test
        // to fail an assert. Instead, we retain the same math as previously, and adjust
        // the origin / size if required.

        if self.scale.x < 0.0 {
            x0 += sx;
            sx = -sx;
        }
        if self.scale.y < 0.0 {
            y0 += sy;
            sy = -sy;
        }

        Rect::new(
            Point2D::new(x0, y0),
            Size2D::new(sx, sy),
        )
    }

    pub fn map_vector<F, T>(&self, vector: &Vector2D<f32, F>) -> Vector2D<f32, T> {
        Vector2D::new(
            vector.x * self.scale.x,
            vector.y * self.scale.y,
        )
    }

    pub fn unmap_vector<F, T>(&self, vector: &Vector2D<f32, F>) -> Vector2D<f32, T> {
        Vector2D::new(
            vector.x / self.scale.x,
            vector.y / self.scale.y,
        )
    }

    pub fn map_point<F, T>(&self, point: &Point2D<f32, F>) -> Point2D<f32, T> {
        Point2D::new(
            point.x * self.scale.x + self.offset.x,
            point.y * self.scale.y + self.offset.y,
        )
    }

    pub fn unmap_point<F, T>(&self, point: &Point2D<f32, F>) -> Point2D<f32, T> {
        Point2D::new(
            (point.x - self.offset.x) / self.scale.x,
            (point.y - self.offset.y) / self.scale.y,
        )
    }

    pub fn to_transform<F, T>(&self) -> Transform3D<f32, F, T> {
        Transform3D::new(
            self.scale.x,
            0.0,
            0.0,
            0.0,

            0.0,
            self.scale.y,
            0.0,
            0.0,

            0.0,
            0.0,
            1.0,
            0.0,

            self.offset.x,
            self.offset.y,
            0.0,
            1.0,
        )
    }
}

// TODO: Implement these in euclid!
pub trait MatrixHelpers<Src, Dst> {
    /// A port of the preserves2dAxisAlignment function in Skia.
    /// Defined in the SkMatrix44 class.
    fn preserves_2d_axis_alignment(&self) -> bool;
    fn has_perspective_component(&self) -> bool;
    fn has_2d_inverse(&self) -> bool;
    /// Check if the matrix post-scaling on either the X or Y axes could cause geometry
    /// transformed by this matrix to have scaling exceeding the supplied limit.
    fn exceeds_2d_scale(&self, limit: f64) -> bool;
    fn inverse_project(&self, target: &Point2D<f32, Dst>) -> Option<Point2D<f32, Src>>;
    fn inverse_rect_footprint(&self, rect: &Rect<f32, Dst>) -> Option<Rect<f32, Src>>;
    fn transform_kind(&self) -> TransformedRectKind;
    fn is_simple_translation(&self) -> bool;
    fn is_simple_2d_translation(&self) -> bool;
    fn is_2d_scale_translation(&self) -> bool;
    /// Return the determinant of the 2D part of the matrix.
    fn determinant_2d(&self) -> f32;
    /// This function returns a point in the `Src` space that projects into zero XY.
    /// It ignores the Z coordinate and is usable for "flattened" transformations,
    /// since they are not generally inversible.
    fn inverse_project_2d_origin(&self) -> Option<Point2D<f32, Src>>;
    /// Turn Z transformation into identity. This is useful when crossing "flat"
    /// transform styled stacking contexts upon traversing the coordinate systems.
    fn flatten_z_output(&mut self);

    fn cast_unit<NewSrc, NewDst>(&self) -> Transform3D<f32, NewSrc, NewDst>;
}

impl<Src, Dst> MatrixHelpers<Src, Dst> for Transform3D<f32, Src, Dst> {
    fn preserves_2d_axis_alignment(&self) -> bool {
        if self.m14 != 0.0 || self.m24 != 0.0 {
            return false;
        }

        let mut col0 = 0;
        let mut col1 = 0;
        let mut row0 = 0;
        let mut row1 = 0;

        if self.m11.abs() > NEARLY_ZERO {
            col0 += 1;
            row0 += 1;
        }
        if self.m12.abs() > NEARLY_ZERO {
            col1 += 1;
            row0 += 1;
        }
        if self.m21.abs() > NEARLY_ZERO {
            col0 += 1;
            row1 += 1;
        }
        if self.m22.abs() > NEARLY_ZERO {
            col1 += 1;
            row1 += 1;
        }

        col0 < 2 && col1 < 2 && row0 < 2 && row1 < 2
    }

    fn has_perspective_component(&self) -> bool {
         self.m14.abs() > NEARLY_ZERO ||
         self.m24.abs() > NEARLY_ZERO ||
         self.m34.abs() > NEARLY_ZERO ||
         (self.m44 - 1.0).abs() > NEARLY_ZERO
    }

    fn has_2d_inverse(&self) -> bool {
        self.determinant_2d() != 0.0
    }

    fn exceeds_2d_scale(&self, limit: f64) -> bool {
        let limit2 = (limit * limit) as f32;
        self.m11 * self.m11 + self.m12 * self.m12 > limit2 ||
        self.m21 * self.m21 + self.m22 * self.m22 > limit2
    }

    /// Find out a point in `Src` that would be projected into the `target`.
    fn inverse_project(&self, target: &Point2D<f32, Dst>) -> Option<Point2D<f32, Src>> {
        // form the linear equation for the hyperplane intersection
        let m = Transform2D::<f32, Src, Dst>::new(
            self.m11 - target.x * self.m14, self.m12 - target.y * self.m14,
            self.m21 - target.x * self.m24, self.m22 - target.y * self.m24,
            self.m41 - target.x * self.m44, self.m42 - target.y * self.m44,
        );
        let inv = m.inverse()?;
        // we found the point, now check if it maps to the positive hemisphere
        if inv.m31 * self.m14 + inv.m32 * self.m24 + self.m44 > 0.0 {
            Some(Point2D::new(inv.m31, inv.m32))
        } else {
            None
        }
    }

    fn inverse_rect_footprint(&self, rect: &Rect<f32, Dst>) -> Option<Rect<f32, Src>> {
        Some(Rect::from_points(&[
            self.inverse_project(&rect.origin)?,
            self.inverse_project(&rect.top_right())?,
            self.inverse_project(&rect.bottom_left())?,
            self.inverse_project(&rect.bottom_right())?,
        ]))
    }

    fn transform_kind(&self) -> TransformedRectKind {
        if self.preserves_2d_axis_alignment() {
            TransformedRectKind::AxisAligned
        } else {
            TransformedRectKind::Complex
        }
    }

    fn is_simple_translation(&self) -> bool {
        if (self.m11 - 1.0).abs() > NEARLY_ZERO ||
            (self.m22 - 1.0).abs() > NEARLY_ZERO ||
            (self.m33 - 1.0).abs() > NEARLY_ZERO ||
            (self.m44 - 1.0).abs() > NEARLY_ZERO {
            return false;
        }

        self.m12.abs() < NEARLY_ZERO && self.m13.abs() < NEARLY_ZERO &&
            self.m14.abs() < NEARLY_ZERO && self.m21.abs() < NEARLY_ZERO &&
            self.m23.abs() < NEARLY_ZERO && self.m24.abs() < NEARLY_ZERO &&
            self.m31.abs() < NEARLY_ZERO && self.m32.abs() < NEARLY_ZERO &&
            self.m34.abs() < NEARLY_ZERO
    }

    fn is_simple_2d_translation(&self) -> bool {
        if !self.is_simple_translation() {
            return false;
        }

        self.m43.abs() < NEARLY_ZERO
    }

    /*  is this...
     *  X  0  0  0
     *  0  Y  0  0
     *  0  0  1  0
     *  a  b  0  1
     */
    fn is_2d_scale_translation(&self) -> bool {
        (self.m33 - 1.0).abs() < NEARLY_ZERO &&
            (self.m44 - 1.0).abs() < NEARLY_ZERO &&
            self.m12.abs() < NEARLY_ZERO && self.m13.abs() < NEARLY_ZERO && self.m14.abs() < NEARLY_ZERO &&
            self.m21.abs() < NEARLY_ZERO && self.m23.abs() < NEARLY_ZERO && self.m24.abs() < NEARLY_ZERO &&
            self.m31.abs() < NEARLY_ZERO && self.m32.abs() < NEARLY_ZERO && self.m34.abs() < NEARLY_ZERO &&
            self.m43.abs() < NEARLY_ZERO
    }

    fn determinant_2d(&self) -> f32 {
        self.m11 * self.m22 - self.m12 * self.m21
    }

    fn inverse_project_2d_origin(&self) -> Option<Point2D<f32, Src>> {
        let det = self.determinant_2d();
        if det != 0.0 {
            let x = (self.m21 * self.m42 - self.m41 * self.m22) / det;
            let y = (self.m12 * self.m41 - self.m11 * self.m42) / det;
            Some(Point2D::new(x, y))
        } else {
            None
        }
    }

    fn flatten_z_output(&mut self) {
        self.m13 = 0.0;
        self.m23 = 0.0;
        self.m33 = 1.0;
        self.m43 = 0.0;
        //Note: we used to zero out m3? as well, see "reftests/flatten-all-flat.yaml" test
    }

    fn cast_unit<NewSrc, NewDst>(&self) -> Transform3D<f32, NewSrc, NewDst> {
        Transform3D::new(
            self.m11, self.m12, self.m13, self.m14,
            self.m21, self.m22, self.m23, self.m24,
            self.m31, self.m32, self.m33, self.m34,
            self.m41, self.m42, self.m43, self.m44,
        )
    }
}

pub trait PointHelpers<U>
where
    Self: Sized,
{
    fn snap(&self) -> Self;
}

impl<U> PointHelpers<U> for Point2D<f32, U> {
    fn snap(&self) -> Self {
        Point2D::new(
            (self.x + 0.5).floor(),
            (self.y + 0.5).floor(),
        )
    }
}

pub trait RectHelpers<U>
where
    Self: Sized,
{
    fn from_floats(x0: f32, y0: f32, x1: f32, y1: f32) -> Self;
    fn snap(&self) -> Self;
}

impl<U> RectHelpers<U> for Rect<f32, U> {
    fn from_floats(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Rect::new(
            Point2D::new(x0, y0),
            Size2D::new(x1 - x0, y1 - y0),
        )
    }

    fn snap(&self) -> Self {
        let origin = Point2D::new(
            (self.origin.x + 0.5).floor(),
            (self.origin.y + 0.5).floor(),
        );
        Rect::new(
            origin,
            Size2D::new(
                (self.origin.x + self.size.width + 0.5).floor() - origin.x,
                (self.origin.y + self.size.height + 0.5).floor() - origin.y,
            ),
        )
    }
}

pub trait VectorHelpers<U>
where
    Self: Sized,
{
    fn snap(&self) -> Self;
}

impl<U> VectorHelpers<U> for Vector2D<f32, U> {
    fn snap(&self) -> Self {
        Vector2D::new(
            (self.x + 0.5).floor(),
            (self.y + 0.5).floor(),
        )
    }
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (b - a) * t + a
}

#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum TransformedRectKind {
    AxisAligned = 0,
    Complex = 1,
}

#[inline(always)]
pub fn pack_as_float(value: u32) -> f32 {
    value as f32 + 0.5
}

#[inline]
fn extract_inner_rect_impl<U>(
    rect: &Rect<f32, U>,
    radii: &BorderRadius,
    k: f32,
) -> Option<Rect<f32, U>> {
    // `k` defines how much border is taken into account
    // We enforce the offsets to be rounded to pixel boundaries
    // by `ceil`-ing and `floor`-ing them

    let xl = (k * radii.top_left.width.max(radii.bottom_left.width)).ceil();
    let xr = (rect.size.width - k * radii.top_right.width.max(radii.bottom_right.width)).floor();
    let yt = (k * radii.top_left.height.max(radii.top_right.height)).ceil();
    let yb =
        (rect.size.height - k * radii.bottom_left.height.max(radii.bottom_right.height)).floor();

    if xl <= xr && yt <= yb {
        Some(Rect::new(
            Point2D::new(rect.origin.x + xl, rect.origin.y + yt),
            Size2D::new(xr - xl, yb - yt),
        ))
    } else {
        None
    }
}

/// Return an aligned rectangle that is inside the clip region and doesn't intersect
/// any of the bounding rectangles of the rounded corners.
pub fn extract_inner_rect_safe<U>(
    rect: &Rect<f32, U>,
    radii: &BorderRadius,
) -> Option<Rect<f32, U>> {
    // value of `k==1.0` is used for extraction of the corner rectangles
    // see `SEGMENT_CORNER_*` in `clip_shared.glsl`
    extract_inner_rect_impl(rect, radii, 1.0)
}

#[cfg(test)]
use euclid::vec3;

#[cfg(test)]
pub mod test {
    use super::*;
    use euclid::default::{Point2D, Rect, Size2D, Transform3D};
    use euclid::{Angle, approxeq::ApproxEq};
    use std::f32::consts::PI;
    use crate::clip::{is_left_of_line, polygon_contains_point};
    use crate::prim_store::PolygonKey;
    use api::FillRule;

    #[test]
    fn inverse_project() {
        let m0 = Transform3D::identity();
        let p0 = Point2D::new(1.0, 2.0);
        // an identical transform doesn't need any inverse projection
        assert_eq!(m0.inverse_project(&p0), Some(p0));
        let m1 = Transform3D::rotation(0.0, 1.0, 0.0, Angle::radians(-PI / 3.0));
        // rotation by 60 degrees would imply scaling of X component by a factor of 2
        assert_eq!(m1.inverse_project(&p0), Some(Point2D::new(2.0, 2.0)));
    }

    #[test]
    fn inverse_project_footprint() {
        let m = Transform3D::new(
            0.477499992, 0.135000005, -1.0, 0.000624999986,
            -0.642787635, 0.766044438, 0.0, 0.0,
            0.766044438, 0.642787635, 0.0, 0.0,
            1137.10986, 113.71286, 402.0, 0.748749971,
        );
        let r = Rect::new(Point2D::zero(), Size2D::new(804.0, 804.0));
        {
            let points = &[
                r.origin,
                r.top_right(),
                r.bottom_left(),
                r.bottom_right(),
            ];
            let mi = m.inverse().unwrap();
            // In this section, we do the forward and backward transformation
            // to confirm that its bijective.
            // We also do the inverse projection path, and confirm it functions the same way.
            println!("Points:");
            for p in points {
                let pp = m.transform_point2d_homogeneous(*p);
                let p3 = pp.to_point3d().unwrap();
                let pi = mi.transform_point3d_homogeneous(p3);
                let px = pi.to_point2d().unwrap();
                let py = m.inverse_project(&pp.to_point2d().unwrap()).unwrap();
                println!("\t{:?} -> {:?} -> {:?} -> ({:?} -> {:?}, {:?})", p, pp, p3, pi, px, py);
                assert!(px.approx_eq_eps(p, &Point2D::new(0.001, 0.001)));
                assert!(py.approx_eq_eps(p, &Point2D::new(0.001, 0.001)));
            }
        }
        // project
        let rp = project_rect(&m, &r, &Rect::new(Point2D::zero(), Size2D::new(1000.0, 1000.0))).unwrap();
        println!("Projected {:?}", rp);
        // one of the points ends up in the negative hemisphere
        assert_eq!(m.inverse_project(&rp.origin), None);
        // inverse
        if let Some(ri) = m.inverse_rect_footprint(&rp) {
            // inverse footprint should be larger, since it doesn't know the original Z
            assert!(ri.contains_rect(&r), "Inverse {:?}", ri);
        }
    }

    fn validate_convert(xref: &LayoutTransform) {
        let so = ScaleOffset::from_transform(xref).unwrap();
        let xf = so.to_transform();
        assert!(xref.approx_eq(&xf));
    }

    #[test]
    fn negative_scale_map_unmap() {
        let xref = LayoutTransform::scale(1.0, -1.0, 1.0)
                        .pre_translate(LayoutVector3D::new(124.0, 38.0, 0.0));
        let so = ScaleOffset::from_transform(&xref).unwrap();
        let local_rect = LayoutRect::new(
            LayoutPoint::new(50.0, -100.0),
            LayoutSize::new(200.0, 400.0),
        );

        let mapped_rect: LayoutRect = so.map_rect(&local_rect);
        let xf_rect = project_rect(
            &xref,
            &local_rect,
            &LayoutRect::max_rect(),
        ).unwrap();

        assert!(mapped_rect.origin.x.approx_eq(&xf_rect.origin.x));
        assert!(mapped_rect.origin.y.approx_eq(&xf_rect.origin.y));
        assert!(mapped_rect.size.width.approx_eq(&xf_rect.size.width));
        assert!(mapped_rect.size.height.approx_eq(&xf_rect.size.height));

        let unmapped_rect: LayoutRect = so.unmap_rect(&mapped_rect);
        assert!(unmapped_rect.origin.x.approx_eq(&local_rect.origin.x));
        assert!(unmapped_rect.origin.y.approx_eq(&local_rect.origin.y));
        assert!(unmapped_rect.size.width.approx_eq(&local_rect.size.width));
        assert!(unmapped_rect.size.height.approx_eq(&local_rect.size.height));
    }

    #[test]
    fn scale_offset_convert() {
        let xref = LayoutTransform::translation(130.0, 200.0, 0.0);
        validate_convert(&xref);

        let xref = LayoutTransform::scale(13.0, 8.0, 1.0);
        validate_convert(&xref);

        let xref = LayoutTransform::scale(0.5, 0.5, 1.0)
                        .pre_translate(LayoutVector3D::new(124.0, 38.0, 0.0));
        validate_convert(&xref);

        let xref = LayoutTransform::scale(30.0, 11.0, 1.0)
            .then_translate(vec3(50.0, 240.0, 0.0));
        validate_convert(&xref);
    }

    fn validate_inverse(xref: &LayoutTransform) {
        let s0 = ScaleOffset::from_transform(xref).unwrap();
        let s1 = s0.inverse().accumulate(&s0);
        assert!((s1.scale.x - 1.0).abs() < NEARLY_ZERO &&
                (s1.scale.y - 1.0).abs() < NEARLY_ZERO &&
                s1.offset.x.abs() < NEARLY_ZERO &&
                s1.offset.y.abs() < NEARLY_ZERO,
                "{:?}",
                s1);
    }

    #[test]
    fn scale_offset_inverse() {
        let xref = LayoutTransform::translation(130.0, 200.0, 0.0);
        validate_inverse(&xref);

        let xref = LayoutTransform::scale(13.0, 8.0, 1.0);
        validate_inverse(&xref);

        let xref = LayoutTransform::translation(124.0, 38.0, 0.0).
            then_scale(0.5, 0.5, 1.0);

        validate_inverse(&xref);

        let xref = LayoutTransform::scale(30.0, 11.0, 1.0)
            .then_translate(vec3(50.0, 240.0, 0.0));
        validate_inverse(&xref);
    }

    fn validate_accumulate(x0: &LayoutTransform, x1: &LayoutTransform) {
        let x = x1.then(&x0);

        let s0 = ScaleOffset::from_transform(x0).unwrap();
        let s1 = ScaleOffset::from_transform(x1).unwrap();

        let s = s0.accumulate(&s1).to_transform();

        assert!(x.approx_eq(&s), "{:?}\n{:?}", x, s);
    }

    #[test]
    fn scale_offset_accumulate() {
        let x0 = LayoutTransform::translation(130.0, 200.0, 0.0);
        let x1 = LayoutTransform::scale(7.0, 3.0, 1.0);

        validate_accumulate(&x0, &x1);
    }

    #[test]
    fn inverse_project_2d_origin() {
        let mut m = Transform3D::identity();
        assert_eq!(m.inverse_project_2d_origin(), Some(Point2D::zero()));
        m.m11 = 0.0;
        assert_eq!(m.inverse_project_2d_origin(), None);
        m.m21 = -2.0;
        m.m22 = 0.0;
        m.m12 = -0.5;
        m.m41 = 1.0;
        m.m42 = 0.5;
        let origin = m.inverse_project_2d_origin().unwrap();
        assert_eq!(origin, Point2D::new(1.0, 0.5));
        assert_eq!(m.transform_point2d(origin), Some(Point2D::zero()));
    }

    #[test]
    fn polygon_clip_is_left_of_point() {
        // Define points of a line through (1, -3) and (-2, 6) to test against.
        // If the triplet consisting of these two points and the test point
        // form a counter-clockwise triangle, then the test point is on the
        // left. The easiest way to visualize this is with an "ascending"
        // line from low-Y to high-Y.
        let p0_x = 1.0;
        let p0_y = -3.0;
        let p1_x = -2.0;
        let p1_y = 6.0;

        // Test some points to the left of the line.
        assert!(is_left_of_line(-9.0, 0.0, p0_x, p0_y, p1_x, p1_y) > 0.0);
        assert!(is_left_of_line(-1.0, 1.0, p0_x, p0_y, p1_x, p1_y) > 0.0);
        assert!(is_left_of_line(1.0, -4.0, p0_x, p0_y, p1_x, p1_y) > 0.0);

        // Test some points on the line.
        assert!(is_left_of_line(-3.0, 9.0, p0_x, p0_y, p1_x, p1_y) == 0.0);
        assert!(is_left_of_line(0.0, 0.0, p0_x, p0_y, p1_x, p1_y) == 0.0);
        assert!(is_left_of_line(100.0, -300.0, p0_x, p0_y, p1_x, p1_y) == 0.0);

        // Test some points to the right of the line.
        assert!(is_left_of_line(0.0, 1.0, p0_x, p0_y, p1_x, p1_y) < 0.0);
        assert!(is_left_of_line(-4.0, 13.0, p0_x, p0_y, p1_x, p1_y) < 0.0);
        assert!(is_left_of_line(5.0, -12.0, p0_x, p0_y, p1_x, p1_y) < 0.0);
    }

    #[test]
    fn polygon_clip_contains_point() {
        // We define the points of a self-overlapping polygon, which we will
        // use to create polygons with different windings and fill rules.
        let p0 = LayoutPoint::new(4.0, 4.0);
        let p1 = LayoutPoint::new(6.0, 4.0);
        let p2 = LayoutPoint::new(4.0, 7.0);
        let p3 = LayoutPoint::new(2.0, 1.0);
        let p4 = LayoutPoint::new(8.0, 1.0);
        let p5 = LayoutPoint::new(6.0, 7.0);

        let poly_clockwise_nonzero = PolygonKey::new(
            &[p5, p4, p3, p2, p1, p0].to_vec(), FillRule::Nonzero
        );
        let poly_clockwise_evenodd = PolygonKey::new(
            &[p5, p4, p3, p2, p1, p0].to_vec(), FillRule::Evenodd
        );
        let poly_counter_clockwise_nonzero = PolygonKey::new(
            &[p0, p1, p2, p3, p4, p5].to_vec(), FillRule::Nonzero
        );
        let poly_counter_clockwise_evenodd = PolygonKey::new(
            &[p0, p1, p2, p3, p4, p5].to_vec(), FillRule::Evenodd
        );

        // We define a rect that provides a bounding clip area of
        // the polygon.
        let rect = LayoutRect::new(LayoutPoint::new(0.0, 0.0),
                                   LayoutSize::new(10.0, 10.0));

        // And we'll test three points of interest.
        let p_inside_once = LayoutPoint::new(5.0, 3.0);
        let p_inside_twice = LayoutPoint::new(5.0, 5.0);
        let p_outside = LayoutPoint::new(9.0, 9.0);

        // We should get the same results for both clockwise and
        // counter-clockwise polygons.
        // For nonzero polygons, the inside twice point is considered inside.
        for poly_nonzero in vec![poly_clockwise_nonzero, poly_counter_clockwise_nonzero].iter() {
            assert_eq!(polygon_contains_point(&p_inside_once, &rect, &poly_nonzero), true);
            assert_eq!(polygon_contains_point(&p_inside_twice, &rect, &poly_nonzero), true);
            assert_eq!(polygon_contains_point(&p_outside, &rect, &poly_nonzero), false);
        }
        // For evenodd polygons, the inside twice point is considered outside.
        for poly_evenodd in vec![poly_clockwise_evenodd, poly_counter_clockwise_evenodd].iter() {
            assert_eq!(polygon_contains_point(&p_inside_once, &rect, &poly_evenodd), true);
            assert_eq!(polygon_contains_point(&p_inside_twice, &rect, &poly_evenodd), false);
            assert_eq!(polygon_contains_point(&p_outside, &rect, &poly_evenodd), false);
        }
    }
}

pub trait MaxRect {
    fn max_rect() -> Self;
}

impl MaxRect for DeviceIntRect {
    fn max_rect() -> Self {
        DeviceIntRect::new(
            DeviceIntPoint::new(i32::MIN / 2, i32::MIN / 2),
            DeviceIntSize::new(i32::MAX, i32::MAX),
        )
    }
}

impl<U> MaxRect for Rect<f32, U> {
    fn max_rect() -> Self {
        // Having an unlimited bounding box is fine up until we try
        // to cast it to `i32`, where we get `-2147483648` for any
        // values larger than or equal to 2^31.
        //
        // Note: clamping to i32::MIN and i32::MAX is not a solution,
        // with explanation left as an exercise for the reader.
        const MAX_COORD: f32 = 1.0e9;

        Rect::new(
            Point2D::new(-MAX_COORD, -MAX_COORD),
            Size2D::new(2.0 * MAX_COORD, 2.0 * MAX_COORD),
        )
    }
}

/// An enum that tries to avoid expensive transformation matrix calculations
/// when possible when dealing with non-perspective axis-aligned transformations.
#[derive(Debug, MallocSizeOf)]
pub enum FastTransform<Src, Dst> {
    /// A simple offset, which can be used without doing any matrix math.
    Offset(Vector2D<f32, Src>),

    /// A 2D transformation with an inverse.
    Transform {
        transform: Transform3D<f32, Src, Dst>,
        inverse: Option<Transform3D<f32, Dst, Src>>,
        is_2d: bool,
    },
}

impl<Src, Dst> Clone for FastTransform<Src, Dst> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Src, Dst> Copy for FastTransform<Src, Dst> { }

impl<Src, Dst> FastTransform<Src, Dst> {
    pub fn identity() -> Self {
        FastTransform::Offset(Vector2D::zero())
    }

    pub fn with_vector(offset: Vector2D<f32, Src>) -> Self {
        FastTransform::Offset(offset)
    }

    pub fn with_scale_offset(scale_offset: ScaleOffset) -> Self {
        if scale_offset.scale == Vector2D::new(1.0, 1.0) {
            FastTransform::Offset(Vector2D::from_untyped(scale_offset.offset))
        } else {
            FastTransform::Transform {
                transform: scale_offset.to_transform(),
                inverse: Some(scale_offset.inverse().to_transform()),
                is_2d: true,
            }
        }
    }

    #[inline(always)]
    pub fn with_transform(transform: Transform3D<f32, Src, Dst>) -> Self {
        if transform.is_simple_2d_translation() {
            return FastTransform::Offset(Vector2D::new(transform.m41, transform.m42));
        }
        let inverse = transform.inverse();
        let is_2d = transform.is_2d();
        FastTransform::Transform { transform, inverse, is_2d}
    }

    pub fn to_transform(&self) -> Cow<Transform3D<f32, Src, Dst>> {
        match *self {
            FastTransform::Offset(offset) => Cow::Owned(
                Transform3D::translation(offset.x, offset.y, 0.0)
            ),
            FastTransform::Transform { ref transform, .. } => Cow::Borrowed(transform),
        }
    }

    /// Return true if this is an identity transform
    #[allow(unused)]
    pub fn is_identity(&self)-> bool {
        match *self {
            FastTransform::Offset(offset) => {
                offset == Vector2D::zero()
            }
            FastTransform::Transform { ref transform, .. } => {
                *transform == Transform3D::identity()
            }
        }
    }

    pub fn then<NewDst>(&self, other: &FastTransform<Dst, NewDst>) -> FastTransform<Src, NewDst> {
        match *self {
            FastTransform::Offset(offset) => match *other {
                FastTransform::Offset(other_offset) => {
                    FastTransform::Offset(offset + other_offset * Scale::<_, _, Src>::new(1.0))
                }
                FastTransform::Transform { transform: ref other_transform, .. } => {
                    FastTransform::with_transform(
                        other_transform
                            .with_source::<Src>()
                            .pre_translate(offset.to_3d())
                    )
                }
            }
            FastTransform::Transform { ref transform, ref inverse, is_2d } => match *other {
                FastTransform::Offset(other_offset) => {
                    FastTransform::with_transform(
                        transform
                            .then_translate(other_offset.to_3d())
                            .with_destination::<NewDst>()
                    )
                }
                FastTransform::Transform { transform: ref other_transform, inverse: ref other_inverse, is_2d: other_is_2d } => {
                    FastTransform::Transform {
                        transform: transform.then(other_transform),
                        inverse: inverse.as_ref().and_then(|self_inv|
                            other_inverse.as_ref().map(|other_inv| other_inv.then(self_inv))
                        ),
                        is_2d: is_2d & other_is_2d,
                    }
                }
            }
        }
    }

    pub fn pre_transform<NewSrc>(
        &self,
        other: &FastTransform<NewSrc, Src>
    ) -> FastTransform<NewSrc, Dst> {
        other.then(self)
    }

    pub fn pre_translate(&self, other_offset: Vector2D<f32, Src>) -> Self {
        match *self {
            FastTransform::Offset(offset) =>
                FastTransform::Offset(offset + other_offset),
            FastTransform::Transform { transform, .. } =>
                FastTransform::with_transform(transform.pre_translate(other_offset.to_3d()))
        }
    }

    pub fn then_translate(&self, other_offset: Vector2D<f32, Dst>) -> Self {
        match *self {
            FastTransform::Offset(offset) => {
                FastTransform::Offset(offset + other_offset * Scale::<_, _, Src>::new(1.0))
            }
            FastTransform::Transform { ref transform, .. } => {
                let transform = transform.then_translate(other_offset.to_3d());
                FastTransform::with_transform(transform)
            }
        }
    }

    #[inline(always)]
    pub fn is_backface_visible(&self) -> bool {
        match *self {
            FastTransform::Offset(..) => false,
            FastTransform::Transform { inverse: None, .. } => false,
            //TODO: fix this properly by taking "det|M33| * det|M34| > 0"
            // see https://www.w3.org/Bugs/Public/show_bug.cgi?id=23014
            FastTransform::Transform { inverse: Some(ref inverse), .. } => inverse.m33 < 0.0,
        }
    }

    #[inline(always)]
    pub fn transform_point2d(&self, point: Point2D<f32, Src>) -> Option<Point2D<f32, Dst>> {
        match *self {
            FastTransform::Offset(offset) => {
                let new_point = point + offset;
                Some(Point2D::from_untyped(new_point.to_untyped()))
            }
            FastTransform::Transform { ref transform, .. } => transform.transform_point2d(point),
        }
    }

    #[inline(always)]
    pub fn inverse(&self) -> Option<FastTransform<Dst, Src>> {
        match *self {
            FastTransform::Offset(offset) =>
                Some(FastTransform::Offset(Vector2D::new(-offset.x, -offset.y))),
            FastTransform::Transform { transform, inverse: Some(inverse), is_2d, } =>
                Some(FastTransform::Transform {
                    transform: inverse,
                    inverse: Some(transform),
                    is_2d
                }),
            FastTransform::Transform { inverse: None, .. } => None,

        }
    }
}

impl<Src, Dst> From<Transform3D<f32, Src, Dst>> for FastTransform<Src, Dst> {
    fn from(transform: Transform3D<f32, Src, Dst>) -> Self {
        FastTransform::with_transform(transform)
    }
}

impl<Src, Dst> From<Vector2D<f32, Src>> for FastTransform<Src, Dst> {
    fn from(vector: Vector2D<f32, Src>) -> Self {
        FastTransform::with_vector(vector)
    }
}

pub type LayoutFastTransform = FastTransform<LayoutPixel, LayoutPixel>;
pub type LayoutToWorldFastTransform = FastTransform<LayoutPixel, WorldPixel>;

pub fn project_rect<F, T>(
    transform: &Transform3D<f32, F, T>,
    rect: &Rect<f32, F>,
    bounds: &Rect<f32, T>,
) -> Option<Rect<f32, T>>
 where F: fmt::Debug
{
    let homogens = [
        transform.transform_point2d_homogeneous(rect.origin),
        transform.transform_point2d_homogeneous(rect.top_right()),
        transform.transform_point2d_homogeneous(rect.bottom_left()),
        transform.transform_point2d_homogeneous(rect.bottom_right()),
    ];

    // Note: we only do the full frustum collision when the polygon approaches the camera plane.
    // Otherwise, it will be clamped to the screen bounds anyway.
    if homogens.iter().any(|h| h.w <= 0.0 || h.w.is_nan()) {
        let mut clipper = Clipper::new();
        let polygon = Polygon::from_rect(*rect, 1);

        let planes = match Clipper::<_, _, usize>::frustum_planes(
            transform,
            Some(*bounds),
        ) {
            Ok(planes) => planes,
            Err(..) => return None,
        };

        for plane in planes {
            clipper.add(plane);
        }

        let results = clipper.clip(polygon);
        if results.is_empty() {
            return None
        }

        Some(Rect::from_points(results
            .into_iter()
            // filter out parts behind the view plane
            .flat_map(|poly| &poly.points)
            .map(|p| {
                let mut homo = transform.transform_point2d_homogeneous(p.to_2d());
                homo.w = homo.w.max(0.00000001); // avoid infinite values
                homo.to_point2d().unwrap()
            })
        ))
    } else {
        // we just checked for all the points to be in positive hemisphere, so `unwrap` is valid
        Some(Rect::from_points(&[
            homogens[0].to_point2d().unwrap(),
            homogens[1].to_point2d().unwrap(),
            homogens[2].to_point2d().unwrap(),
            homogens[3].to_point2d().unwrap(),
        ]))
    }
}

pub fn raster_rect_to_device_pixels(
    rect: RasterRect,
    device_pixel_scale: DevicePixelScale,
) -> DeviceRect {
    let world_rect = rect * Scale::new(1.0);
    let device_rect = world_rect * device_pixel_scale;
    device_rect.round_out()
}

/// Run the first callback over all elements in the array. If the callback returns true,
/// the element is removed from the array and moved to a second callback.
///
/// This is a simple implementation waiting for Vec::drain_filter to be stable.
/// When that happens, code like:
///
/// let filter = |op| {
///     match *op {
///         Enum::Foo | Enum::Bar => true,
///         Enum::Baz => false,
///     }
/// };
/// drain_filter(
///     &mut ops,
///     filter,
///     |op| {
///         match op {
///             Enum::Foo => { foo(); }
///             Enum::Bar => { bar(); }
///             Enum::Baz => { unreachable!(); }
///         }
///     },
/// );
///
/// Can be rewritten as:
///
/// let filter = |op| {
///     match *op {
///         Enum::Foo | Enum::Bar => true,
///         Enum::Baz => false,
///     }
/// };
/// for op in ops.drain_filter(filter) {
///     match op {
///         Enum::Foo => { foo(); }
///         Enum::Bar => { bar(); }
///         Enum::Baz => { unreachable!(); }
///     }
/// }
///
/// See https://doc.rust-lang.org/std/vec/struct.Vec.html#method.drain_filter
pub fn drain_filter<T, Filter, Action>(
    vec: &mut Vec<T>,
    mut filter: Filter,
    mut action: Action,
)
where
    Filter: FnMut(&mut T) -> bool,
    Action: FnMut(T)
{
    let mut i = 0;
    while i != vec.len() {
        if filter(&mut vec[i]) {
            action(vec.remove(i));
        } else {
            i += 1;
        }
    }
}


#[derive(Debug)]
pub struct Recycler {
    pub num_allocations: usize,
}

impl Recycler {
    /// Maximum extra capacity that a recycled vector is allowed to have. If the actual capacity
    /// is larger, we re-allocate the vector storage with lower capacity.
    const MAX_EXTRA_CAPACITY_PERCENT: usize = 200;
    /// Minimum extra capacity to keep when re-allocating the vector storage.
    const MIN_EXTRA_CAPACITY_PERCENT: usize = 20;
    /// Minimum sensible vector length to consider for re-allocation.
    const MIN_VECTOR_LENGTH: usize = 16;

    pub fn new() -> Self {
        Recycler {
            num_allocations: 0,
        }
    }

    /// Clear a vector for re-use, while retaining the backing memory buffer. May shrink the buffer
    /// if it's currently much larger than was actually used.
    pub fn recycle_vec<T>(&mut self, vec: &mut Vec<T>) {
        let extra_capacity = (vec.capacity() - vec.len()) * 100 / vec.len().max(Self::MIN_VECTOR_LENGTH);

        if extra_capacity > Self::MAX_EXTRA_CAPACITY_PERCENT {
            // Reduce capacity of the buffer if it is a lot larger than it needs to be. This prevents
            // a frame with exceptionally large allocations to cause subsequent frames to retain
            // more memory than they need.
            //TODO: use `shrink_to` when it's stable
            *vec = Vec::with_capacity(vec.len() + vec.len() * Self::MIN_EXTRA_CAPACITY_PERCENT / 100);
            self.num_allocations += 1;
        } else {
            vec.clear();
        }
    }
}

/// Record the size of a data structure to preallocate a similar size
/// at the next frame and avoid growing it too many time.
#[derive(Copy, Clone, Debug)]
pub struct Preallocator {
    size: usize,
}

impl Preallocator {
    pub fn new(initial_size: usize) -> Self {
        Preallocator {
            size: initial_size,
        }
    }

    /// Record the size of a vector to preallocate it the next frame.
    pub fn record_vec<T>(&mut self, vec: &Vec<T>) {
        let len = vec.len();
        if len > self.size {
            self.size = len;
        } else {
            self.size = (self.size + len) / 2;
        }
    }

    /// The size that we'll preallocate the vector with.
    pub fn preallocation_size(&self) -> usize {
        // Round up to multiple of 16 to avoid small tiny
        // variations causing reallocations.
        (self.size + 15) & !15
    }

    /// Preallocate vector storage.
    ///
    /// The preallocated amount depends on the length recorded in the last
    /// record_vec call.
    pub fn preallocate_vec<T>(&self, vec: &mut Vec<T>) {
        let len = vec.len();
        let cap = self.preallocation_size();
        if len < cap {
            vec.reserve(cap - len);
        }
    }
}

impl Default for Preallocator {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Arc wrapper to support measurement via MallocSizeOf.
///
/// Memory reporting for Arcs is tricky because of the risk of double-counting.
/// One way to measure them is to keep a table of pointers that have already been
/// traversed. The other way is to use knowledge of the program structure to
/// identify which Arc instances should be measured and which should be skipped to
/// avoid double-counting.
///
/// This struct implements the second approach. It identifies the "main" pointer
/// to the Arc-ed resource, and measures the buffer as if it were an owned pointer.
/// The programmer should ensure that there is at most one PrimaryArc for a given
/// underlying ArcInner.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PrimaryArc<T>(pub Arc<T>);

impl<T> ::std::ops::Deref for PrimaryArc<T> {
    type Target = Arc<T>;

    #[inline]
    fn deref(&self) -> &Arc<T> {
        &self.0
    }
}

impl<T> MallocShallowSizeOf for PrimaryArc<T> {
    fn shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        unsafe {
            // This is a bit sketchy, but std::sync::Arc doesn't expose the
            // base pointer.
            let raw_arc_ptr: *const Arc<T> = &self.0;
            let raw_ptr_ptr: *const *const c_void = raw_arc_ptr as _;
            let raw_ptr = *raw_ptr_ptr;
            (ops.size_of_op)(raw_ptr)
        }
    }
}

impl<T: MallocSizeOf> MallocSizeOf for PrimaryArc<T> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.shallow_size_of(ops) + (**self).size_of(ops)
    }
}

/// Computes the scale factors of this matrix; that is,
/// the amounts each basis vector is scaled by.
///
/// This code comes from gecko gfx/2d/Matrix.h with the following
/// modifications:
///
/// * Removed `xMajor` parameter.
pub fn scale_factors<Src, Dst>(
    mat: &Transform3D<f32, Src, Dst>
) -> (f32, f32) {
    // Determinant is just of the 2D component.
    let det = mat.m11 * mat.m22 - mat.m12 * mat.m21;
    if det == 0.0 {
        return (0.0, 0.0);
    }

    // ignore mirroring
    let det = det.abs();

    let major = (mat.m11 * mat.m11 + mat.m12 * mat.m12).sqrt();
    let minor = if major != 0.0 { det / major } else { 0.0 };

    (major, minor)
}

/// Clamp scaling factor to a power of two.
///
/// This code comes from gecko gfx/thebes/gfxUtils.cpp with the following
/// modification:
///
/// * logs are taken in base 2 instead of base e.
pub fn clamp_to_scale_factor(val: f32, round_down: bool) -> f32 {
    // Arbitary scale factor limitation. We can increase this
    // for better scaling performance at the cost of worse
    // quality.
    const SCALE_RESOLUTION: f32 = 2.0;

    // Negative scaling is just a flip and irrelevant to
    // our resolution calculation.
    let val = val.abs();

    let (val, inverse) = if val < 1.0 {
        (1.0 / val, true)
    } else {
        (val, false)
    };

    let power = val.log2() / SCALE_RESOLUTION.log2();

    // If power is within 1e-5 of an integer, round to nearest to
    // prevent floating point errors, otherwise round up to the
    // next integer value.
    let power = if (power - power.round()).abs() < 1e-5 {
        power.round()
    } else if inverse != round_down {
        // Use floor when we are either inverted or rounding down, but
        // not both.
        power.floor()
    } else {
        // Otherwise, ceil when we are not inverted and not rounding
        // down, or we are inverted and rounding down.
        power.ceil()
    };

    let scale = SCALE_RESOLUTION.powf(power);

    if inverse {
        1.0 / scale
    } else {
        scale
    }
}

/// Rounds a value up to the nearest multiple of mul
pub fn round_up_to_multiple(val: usize, mul: NonZeroUsize) -> usize {
    match val % mul.get() {
        0 => val,
        rem => val - rem + mul.get(),
    }
}


#[macro_export]
macro_rules! c_str {
    ($lit:expr) => {
        unsafe {
            std::ffi::CStr::from_ptr(concat!($lit, "\0").as_ptr()
                                     as *const std::os::raw::c_char)
        }
    }
}

// Find a rectangle that is contained by the sum of r1 and r2.
pub fn conservative_union_rect<U>(r1: &Rect<f32, U>, r2: &Rect<f32, U>) -> Rect<f32, U> {
    //  +---+---+   +--+-+--+
    //  |   |   |   |  | |  |
    //  |   |   |   |  | |  |
    //  +---+---+   +--+-+--+
    if r1.origin.y == r2.origin.y && r1.size.height == r2.size.height {
        if r2.min_x() <= r1.max_x() && r2.max_x() >= r1.min_x() {
            let origin_x = f32::min(r1.origin.x, r2.origin.x);
            let width = f32::max(r1.max_x(), r2.max_x()) - origin_x;

            return Rect {
                origin: point2(origin_x, r1.origin.y),
                size: size2(width, r1.size.height),
            }
        }
    }

    //  +----+    +----+
    //  |    |    |    |
    //  |    |    +----+
    //  +----+    |    |
    //  |    |    +----+
    //  |    |    |    |
    //  +----+    +----+
    if r1.origin.x == r2.origin.x && r1.size.width == r2.size.width {
        if r2.min_y() <= r1.max_y() && r2.max_y() >= r1.min_y() {
            let origin_y = f32::min(r1.origin.y, r2.origin.y);
            let height = f32::max(r1.max_y(), r2.max_y()) - origin_y;

            return Rect {
                origin: point2(r1.origin.x, origin_y),
                size: size2(r1.size.width, height),
            }
        }
    }

    if r1.area() >= r2.area() { *r1 } else {*r2 }
}

#[test]
fn test_conservative_union_rect() {
    // Adjacent, x axis
    let r = conservative_union_rect(
        &LayoutRect { origin: point2(1.0, 2.0), size: size2(3.0, 4.0) },
        &LayoutRect { origin: point2(4.0, 2.0), size: size2(5.0, 4.0) },
    );
    assert_eq!(r, LayoutRect { origin: point2(1.0, 2.0), size: size2(8.0, 4.0) });

    let r = conservative_union_rect(
        &LayoutRect { origin: point2(4.0, 2.0), size: size2(5.0, 4.0) },
        &LayoutRect { origin: point2(1.0, 2.0), size: size2(3.0, 4.0) },
    );
    assert_eq!(r, LayoutRect { origin: point2(1.0, 2.0), size: size2(8.0, 4.0) });

    // Averlapping adjacent, x axis
    let r = conservative_union_rect(
        &LayoutRect { origin: point2(1.0, 2.0), size: size2(3.0, 4.0) },
        &LayoutRect { origin: point2(3.0, 2.0), size: size2(5.0, 4.0) },
    );
    assert_eq!(r, LayoutRect { origin: point2(1.0, 2.0), size: size2(7.0, 4.0) });

    let r = conservative_union_rect(
        &LayoutRect { origin: point2(5.0, 2.0), size: size2(3.0, 4.0) },
        &LayoutRect { origin: point2(1.0, 2.0), size: size2(5.0, 4.0) },
    );
    assert_eq!(r, LayoutRect { origin: point2(1.0, 2.0), size: size2(7.0, 4.0) });

    // Adjacent but not touching, x axis
    let r = conservative_union_rect(
        &LayoutRect { origin: point2(1.0, 2.0), size: size2(3.0, 4.0) },
        &LayoutRect { origin: point2(6.0, 2.0), size: size2(5.0, 4.0) },
    );
    assert_eq!(r, LayoutRect { origin: point2(6.0, 2.0), size: size2(5.0, 4.0) });

    let r = conservative_union_rect(
        &LayoutRect { origin: point2(1.0, 2.0), size: size2(3.0, 4.0) },
        &LayoutRect { origin: point2(-6.0, 2.0), size: size2(1.0, 4.0) },
    );
    assert_eq!(r, LayoutRect { origin: point2(1.0, 2.0), size: size2(3.0, 4.0) });


    // Adjacent, y axis
    let r = conservative_union_rect(
        &LayoutRect { origin: point2(1.0, 2.0), size: size2(3.0, 4.0) },
        &LayoutRect { origin: point2(1.0, 6.0), size: size2(3.0, 4.0) },
    );
    assert_eq!(r, LayoutRect { origin: point2(1.0, 2.0), size: size2(3.0, 8.0) });

    let r = conservative_union_rect(
        &LayoutRect { origin: point2(1.0, 5.0), size: size2(3.0, 4.0) },
        &LayoutRect { origin: point2(1.0, 1.0), size: size2(3.0, 4.0) },
    );
    assert_eq!(r, LayoutRect { origin: point2(1.0, 1.0), size: size2(3.0, 8.0) });

    // Averlapping adjacent, y axis
    let r = conservative_union_rect(
        &LayoutRect { origin: point2(1.0, 2.0), size: size2(3.0, 4.0) },
        &LayoutRect { origin: point2(1.0, 3.0), size: size2(3.0, 4.0) },
    );
    assert_eq!(r, LayoutRect { origin: point2(1.0, 2.0), size: size2(3.0, 5.0) });

    let r = conservative_union_rect(
        &LayoutRect { origin: point2(1.0, 4.0), size: size2(3.0, 4.0) },
        &LayoutRect { origin: point2(1.0, 2.0), size: size2(3.0, 4.0) },
    );
    assert_eq!(r, LayoutRect { origin: point2(1.0, 2.0), size: size2(3.0, 6.0) });

    // Adjacent but not touching, y axis
    let r = conservative_union_rect(
        &LayoutRect { origin: point2(1.0, 2.0), size: size2(3.0, 4.0) },
        &LayoutRect { origin: point2(1.0, 10.0), size: size2(3.0, 5.0) },
    );
    assert_eq!(r, LayoutRect { origin: point2(1.0, 10.0), size: size2(3.0, 5.0) });

    let r = conservative_union_rect(
        &LayoutRect { origin: point2(1.0, 5.0), size: size2(3.0, 4.0) },
        &LayoutRect { origin: point2(1.0, 0.0), size: size2(3.0, 3.0) },
    );
    assert_eq!(r, LayoutRect { origin: point2(1.0, 5.0), size: size2(3.0, 4.0) });


    // Contained
    let r = conservative_union_rect(
        &LayoutRect { origin: point2(1.0, 2.0), size: size2(3.0, 4.0) },
        &LayoutRect { origin: point2(0.0, 1.0), size: size2(10.0, 11.0) },
    );
    assert_eq!(r, LayoutRect { origin: point2(0.0, 1.0), size: size2(10.0, 11.0) });

    let r = conservative_union_rect(
        &LayoutRect { origin: point2(0.0, 1.0), size: size2(10.0, 11.0) },
        &LayoutRect { origin: point2(1.0, 2.0), size: size2(3.0, 4.0) },
    );
    assert_eq!(r, LayoutRect { origin: point2(0.0, 1.0), size: size2(10.0, 11.0) });
}

/// This is inspired by the `weak-table` crate.
/// It holds a Vec of weak pointers that are garbage collected as the Vec
pub struct WeakTable {
    inner: Vec<std::sync::Weak<Vec<u8>>>
}

impl WeakTable {
    pub fn new() -> WeakTable {
        WeakTable { inner: Vec::new() }
    }
    pub fn insert(&mut self, x: std::sync::Weak<Vec<u8>>) {
        if self.inner.len() == self.inner.capacity() {
            self.remove_expired();

            // We want to make sure that we change capacity()
            // even if remove_expired() removes some entries
            // so that we don't repeatedly hit remove_expired()
            if self.inner.len() * 3 < self.inner.capacity() {
                // We use a different multiple for shrinking then
                // expanding so that we we don't accidentally
                // oscilate.
                self.inner.shrink_to_fit();
            } else {
                // Otherwise double our size
                self.inner.reserve(self.inner.len())
            }
        }
        self.inner.push(x);
    }

    fn remove_expired(&mut self) {
        self.inner.retain(|x| x.strong_count() > 0)
    }

    pub fn iter(&self) -> impl Iterator<Item = Arc<Vec<u8>>> + '_ {
        self.inner.iter().filter_map(|x| x.upgrade())
    }
}

#[test]
fn weak_table() {
    let mut tbl = WeakTable::new();
    let mut things = Vec::new();
    let target_count = 50;
    for _ in 0..target_count {
        things.push(Arc::new(vec![4]));
    }
    for i in &things {
        tbl.insert(Arc::downgrade(i))
    }
    assert_eq!(tbl.inner.len(), target_count);
    drop(things);
    assert_eq!(tbl.iter().count(), 0);

    // make sure that we shrink the table if it gets too big
    // by adding a bunch of dead items
    for _ in 0..target_count*2 {
        tbl.insert(Arc::downgrade(&Arc::new(vec![5])))
    }
    assert!(tbl.inner.capacity() <= 4);
}
