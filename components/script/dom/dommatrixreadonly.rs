/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DOMMatrixBinding::{DOMMatrixInit, DOMMatrixMethods};
use dom::bindings::codegen::Bindings::DOMMatrixReadOnlyBinding::{DOMMatrixReadOnlyMethods, Wrap};
use dom::bindings::codegen::Bindings::DOMPointBinding::DOMPointInit;
use dom::bindings::error;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{reflect_dom_object, Reflectable, Reflector};
use dom::dommatrix::DOMMatrix;
use dom::dompoint::DOMPoint;
use euclid::Matrix4D;
use euclid::Point4D;
use std::cell::{Cell, RefCell};
use std::f64;

#[dom_struct]
pub struct DOMMatrixReadOnly {
    reflector_: Reflector,
    matrix: RefCell<Matrix4D<f64>>,
    is2D: Cell<bool>,
}

pub struct MatrixParts {
    pub matrix: Matrix4D<f64>,
    pub is2D: bool,
}

pub fn dommatrixinit_to_matrix(dict: &DOMMatrixInit) -> Fallible<MatrixParts> {
    let mut d = dict.clone();
    validate_and_fixup_dommatrixinit(&mut d)
        .map(|dict| {
            MatrixParts {
                matrix: Matrix4D::new(dict.m11.unwrap_or(1.0), dict.m12.unwrap_or(0.0), dict.m13, dict.m14,
                                      dict.m21.unwrap_or(0.0), dict.m22.unwrap_or(1.0), dict.m23, dict.m24,
                                      dict.m31,                dict.m32,                dict.m33, dict.m34,
                                      dict.m41.unwrap_or(0.0), dict.m42.unwrap_or(0.0), dict.m43, dict.m44),
                is2D: dict.is2D.unwrap(),
            }
        })
}


pub fn entries_to_matrix(entries: &[f64]) -> Fallible<MatrixParts> {
    if entries.len() == 6 {
        Ok(MatrixParts {
            matrix: Matrix4D::new(entries[0], entries[1], 0.0, 0.0,
                                  entries[2], entries[3], 0.0, 0.0,
                                  0.0,        0.0,        1.0, 0.0,
                                  entries[4], entries[5], 0.0, 1.0),
            is2D: true,
        })
    } else if entries.len() == 16 {
        Ok(MatrixParts {
            matrix: Matrix4D::new(entries[0],  entries[1],  entries[2],  entries[3],
                                  entries[4],  entries[5],  entries[6],  entries[7],
                                  entries[8],  entries[9],  entries[10], entries[11],
                                  entries[12], entries[13], entries[14], entries[15]),
            is2D: false,
        })
    } else {
        let err_msg = format!("Expected 6 or 16 entries, but found {}.", entries.len());
        Err(error::Error::Type(err_msg.to_owned()))
    }
}


impl DOMMatrixReadOnly {
    #[allow(unrooted_must_root)]
    pub fn new(global: GlobalRef, is2D: bool, matrix: Matrix4D<f64>) -> Root<Self> {
        let dommatrix = Self::new_inherited(is2D, matrix);
        reflect_dom_object(box dommatrix, global, Wrap)
    }

    pub fn new_inherited(is2D: bool, matrix: Matrix4D<f64>) -> Self {
        DOMMatrixReadOnly {
            reflector_: Reflector::new(),
            matrix: RefCell::new(matrix),
            is2D: Cell::new(is2D),
         }
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-dommatrixreadonly
    pub fn Constructor(global: GlobalRef) -> Fallible<Root<Self>> {
        entries_to_matrix(&[1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
            .map(|MatrixParts { matrix, is2D }| Self::new(global, is2D, matrix))
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-dommatrixreadonly-numbersequence
    pub fn Constructor_(global: GlobalRef, entries: Vec<f64>) -> Fallible<Root<Self>> {
        entries_to_matrix(&entries[..])
            .map(|MatrixParts { matrix, is2D }| Self::new(global, is2D, matrix))
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-frommatrix
    pub fn FromMatrix(global: GlobalRef, other: &DOMMatrixInit) -> Root<Self> {
        let MatrixParts { matrix, is2D } = dommatrixinit_to_matrix(&other).unwrap(); // TODO handle failure
        Self::new(global, is2D, matrix)
    }

    fn to_DOMMatrix(&self) -> Root<DOMMatrix> {
        let matrix = self.matrix.borrow().clone();
        let is2D = self.is2D.get();
        DOMMatrix::new(self.global().r(), is2D, matrix)
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m11
    pub fn SetM11(&self, value: f64) {
        self.matrix.borrow_mut().m11 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m12
    pub fn SetM12(&self, value: f64) {
        self.matrix.borrow_mut().m12 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m13
    pub fn SetM13(&self, value: f64) {
        self.matrix.borrow_mut().m13 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m14
    pub fn SetM14(&self, value: f64) {
        self.matrix.borrow_mut().m14 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m21
    pub fn SetM21(&self, value: f64) {
        self.matrix.borrow_mut().m21 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m22
    pub fn SetM22(&self, value: f64) {
        self.matrix.borrow_mut().m22 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m23
    pub fn SetM23(&self, value: f64) {
        self.matrix.borrow_mut().m23 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m24
    pub fn SetM24(&self, value: f64) {
        self.matrix.borrow_mut().m24 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m31
    pub fn SetM31(&self, value: f64) {
        self.matrix.borrow_mut().m31 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m32
    pub fn SetM32(&self, value: f64) {
        self.matrix.borrow_mut().m32 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m33
    pub fn SetM33(&self, value: f64) {
        self.matrix.borrow_mut().m33 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m34
    pub fn SetM34(&self, value: f64) {
        self.matrix.borrow_mut().m34 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m41
    pub fn SetM41(&self, value: f64) {
        self.matrix.borrow_mut().m41 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m42
    pub fn SetM42(&self, value: f64) {
        self.matrix.borrow_mut().m42 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m43
    pub fn SetM43(&self, value: f64) {
        self.matrix.borrow_mut().m43 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m44
    pub fn SetM44(&self, value: f64) {
        self.matrix.borrow_mut().m44 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-a
    pub fn SetA(&self, value: f64) {
        self.matrix.borrow_mut().m11 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-b
    pub fn SetB(&self, value: f64) {
        self.matrix.borrow_mut().m12 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-c
    pub fn SetC(&self, value: f64) {
        self.matrix.borrow_mut().m21 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-d
    pub fn SetD(&self, value: f64) {
        self.matrix.borrow_mut().m22 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-e
    pub fn SetE(&self, value: f64) {
        self.matrix.borrow_mut().m41 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-f
    pub fn SetF(&self, value: f64) {
        self.matrix.borrow_mut().m42 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-multiplyself
    pub fn MultiplySelf(&self, other: &DOMMatrixInit) -> Fallible<&Self> {
        let mut matrix = self.matrix.borrow_mut();
        dommatrixinit_to_matrix(&other).map(|parts| {
            *matrix = matrix.mul(&parts.matrix);
            self
        })
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-premultiplyself
    pub fn PreMultiplySelf(&self, other: &DOMMatrixInit) -> Fallible<&Self> {
        let mut matrix = self.matrix.borrow_mut();
        dommatrixinit_to_matrix(&other).map(|parts| {
            *matrix = parts.matrix.mul(&matrix);
            self
        })
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-translateself
    pub fn TranslateSelf(&self, tx: f64, ty: f64, tz: f64) {
        let mut matrix = self.matrix.borrow_mut();
        let translation = Matrix4D::create_translation(tx, ty, tz);
        *matrix = translation.mul(&matrix);
        if tz != 0.0 {
            self.is2D.set(false);
        }
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-scaleself
    pub fn ScaleSelf(&self, scaleX: f64, scaleY: Option<f64>, scaleZ: f64, originX: f64, originY: f64, originZ: f64) {
        let mut matrix = self.matrix.borrow_mut();
        let translation = Matrix4D::create_translation(originX, originY, originZ);
        *matrix = translation.mul(&matrix);
        let scale3D = Matrix4D::create_scale(scaleX, scaleY.unwrap_or(scaleX), scaleZ);
        *matrix = scale3D.mul(&matrix);
        let translation_rev = Matrix4D::create_translation(-originX, -originY, -originZ);
        *matrix = translation_rev.mul(&matrix);
        if scaleZ != 1.0 || originZ != 0.0 {
            self.is2D.set(false);
        }
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-scale3dself
    pub fn Scale3dSelf(&self, scale: f64, originX: f64, originY: f64, originZ: f64) {
        let mut matrix = self.matrix.borrow_mut();
        let translation = Matrix4D::create_translation(originX, originY, originZ);
        *matrix = translation.mul(&matrix);
        let scale3D = Matrix4D::create_scale(scale, scale, scale);
        *matrix = scale3D.mul(&matrix);
        let translation_rev = Matrix4D::create_translation(-originX, -originY, -originZ);
        *matrix = translation_rev.mul(&matrix);
        if scale != 1.0 {
            self.is2D.set(false);
        }
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-rotateself
    pub fn RotateSelf(&self, rotX: f64, rotY: Option<f64>, rotZ: Option<f64>) {
        let mut matrix = self.matrix.borrow_mut();
        let (rotX, rotY, rotZ) = match rotY {
            None     => (0.0, 0.0, rotX),
            Some(ry) => match rotZ {
                    None     => (rotX, ry, 0.0),
                    Some(rz) => (rotX, ry, rz)
            }
        };
        if rotZ != 0.0 {
            *matrix = Matrix4D::create_rotation(0.0, 0.0, 1.0, rotZ.to_radians()).mul(&matrix);
        }
        if rotY != 0.0 {
            *matrix = Matrix4D::create_rotation(0.0, 1.0, 0.0, rotY.to_radians()).mul(&matrix);
        }
        if rotX != 0.0 {
            *matrix = Matrix4D::create_rotation(1.0, 0.0, 0.0, rotX.to_radians()).mul(&matrix);
        }
        if rotX != 0.0 || rotY != 0.0 {
            self.is2D.set(false);
        }
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-rotatefromvectorself
    pub fn RotateFromVectorSelf(&self, x: f64, y: f64) {
        let mut matrix = self.matrix.borrow_mut();
        // don't do anything when the rotation angle is zero or undefined
        if y != 0.0 || x < 0.0 {
            let rotZ = f64::atan2(y, x);
            *matrix = Matrix4D::create_rotation(0.0, 0.0, 1.0, rotZ).mul(&matrix);
        }
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-rotateaxisangleself
    pub fn RotateAxisAngleSelf(&self, x: f64, y: f64, z: f64, angle: f64) {
        let mut matrix = self.matrix.borrow_mut();
        let (norm_x, norm_y, norm_z) = normalize_point(x, y, z);
        let rotation = Matrix4D::create_rotation(norm_x, norm_y, norm_z, angle.to_radians());
        *matrix = rotation.mul(&matrix);
        if x != 0.0 || y != 0.0 {
            self.is2D.set(false);
        }
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-skewxself
    pub fn SkewXSelf(&self, sx: f64) {
        let mut matrix = self.matrix.borrow_mut();
        let skew_x = Matrix4D::create_skew(sx.to_radians(), 0.0);
        *matrix = skew_x.mul(&matrix);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-skewyself
    pub fn SkewYSelf(&self, sy: f64) {
        let mut matrix = self.matrix.borrow_mut();
        let skew_y = Matrix4D::create_skew(0.0, sy.to_radians());
        *matrix = skew_y.mul(&matrix);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-invertself
    pub fn InvertSelf(&self) {
        let mut matrix = self.matrix.borrow_mut();
        if matrix.determinant() == 0.0 {
            *matrix = Matrix4D::new(f64::NAN, f64::NAN, f64::NAN, f64::NAN,
                                    f64::NAN, f64::NAN, f64::NAN, f64::NAN,
                                    f64::NAN, f64::NAN, f64::NAN, f64::NAN,
                                    f64::NAN, f64::NAN, f64::NAN, f64::NAN);
            self.is2D.set(false);
        } else {
            *matrix = matrix.invert();
        }
    }
}


impl DOMMatrixReadOnlyMethods for DOMMatrixReadOnly {
    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m11
    fn M11(&self) -> f64 {
        self.matrix.borrow().m11 as f64
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m12
    fn M12(&self) -> f64 {
        self.matrix.borrow().m12 as f64
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m13
    fn M13(&self) -> f64 {
        self.matrix.borrow().m13 as f64
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m14
    fn M14(&self) -> f64 {
        self.matrix.borrow().m14 as f64
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m21
    fn M21(&self) -> f64 {
        self.matrix.borrow().m21 as f64
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m22
    fn M22(&self) -> f64 {
        self.matrix.borrow().m22 as f64
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m23
    fn M23(&self) -> f64 {
        self.matrix.borrow().m23 as f64
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m24
    fn M24(&self) -> f64 {
        self.matrix.borrow().m24 as f64
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m31
    fn M31(&self) -> f64 {
        self.matrix.borrow().m31 as f64
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m32
    fn M32(&self) -> f64 {
        self.matrix.borrow().m32 as f64
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m33
    fn M33(&self) -> f64 {
        self.matrix.borrow().m33 as f64
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m34
    fn M34(&self) -> f64 {
        self.matrix.borrow().m34 as f64
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m41
    fn M41(&self) -> f64 {
        self.matrix.borrow().m41 as f64
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m42
    fn M42(&self) -> f64 {
        self.matrix.borrow().m42 as f64
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m43
    fn M43(&self) -> f64 {
        self.matrix.borrow().m43 as f64
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m44
    fn M44(&self) -> f64 {
        self.matrix.borrow().m44 as f64
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-a
    fn A(&self) -> f64 {
        self.matrix.borrow().m11 as f64
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-b
    fn B(&self) -> f64 {
        self.matrix.borrow().m12 as f64
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-c
    fn C(&self) -> f64 {
        self.matrix.borrow().m21 as f64
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-d
    fn D(&self) -> f64 {
        self.matrix.borrow().m22 as f64
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-e
    fn E(&self) -> f64 {
        self.matrix.borrow().m41 as f64
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-f
    fn F(&self) -> f64 {
        self.matrix.borrow().m42 as f64
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-is2d
    fn Is2D(&self) -> bool {
        self.is2D.get()
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-isidentity
    fn IsIdentity(&self) -> bool {
        let matrix = self.matrix.borrow();
        matrix.m12 == 0.0 && matrix.m13 == 0.0 && matrix.m14 == 0.0 && matrix.m21 == 0.0 &&
            matrix.m23 == 0.0 && matrix.m24 == 0.0 && matrix.m31 == 0.0 && matrix.m32 == 0.0 &&
            matrix.m34 == 0.0 && matrix.m41 == 0.0 && matrix.m42 == 0.0 && matrix.m43 == 0.0 &&
            matrix.m11 == 1.0 && matrix.m22 == 1.0 && matrix.m33 == 1.0 && matrix.m44 == 1.0
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-translate
    fn Translate(&self, tx: f64, ty: f64, tz: f64) -> Root<DOMMatrix> {
        self.to_DOMMatrix().TranslateSelf(tx, ty, tz)
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-scale
    fn Scale(&self, scaleX: f64, scaleY: Option<f64>, scaleZ: f64,
                    originX: f64, originY: f64, originZ: f64) -> Root<DOMMatrix> {
        self.to_DOMMatrix().ScaleSelf(scaleX, scaleY, scaleZ, originX, originY, originZ)
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-scale3d
    fn Scale3d(&self, scale: f64, originX: f64, originY: f64, originZ: f64) -> Root<DOMMatrix> {
        self.to_DOMMatrix().Scale3dSelf(scale, originX, originY, originZ)
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-rotate
    fn Rotate(&self, rotX: f64, rotY: Option<f64>, rotZ: Option<f64>) -> Root<DOMMatrix> {
        self.to_DOMMatrix().RotateSelf(rotX, rotY, rotZ)
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-rotatefromvector
    fn RotateFromVector(&self, x: f64, y: f64) -> Root<DOMMatrix> {
        self.to_DOMMatrix().RotateFromVectorSelf(x, y)
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-rotateaxisangle
    fn RotateAxisAngle(&self, x: f64, y: f64, z: f64, angle: f64) -> Root<DOMMatrix> {
        self.to_DOMMatrix().RotateAxisAngleSelf(x, y, z, angle)
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-skewx
    fn SkewX(&self, sx: f64) -> Root<DOMMatrix> {
        self.to_DOMMatrix().SkewXSelf(sx)
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-skewy
    fn SkewY(&self, sy: f64) -> Root<DOMMatrix> {
        self.to_DOMMatrix().SkewYSelf(sy)
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-multiply
    fn Multiply(&self, other: &DOMMatrixInit) -> Root<DOMMatrix> {
        self.to_DOMMatrix().MultiplySelf(&other)
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-flipx
    fn FlipX(&self) -> Root<DOMMatrix> {
        let is2D = self.is2D.get();
        let flip = Matrix4D::new(-1.0, 0.0, 0.0, 0.0,
                                  0.0, 1.0, 0.0, 0.0,
                                  0.0, 0.0, 1.0, 0.0,
                                  0.0, 0.0, 0.0, 1.0);
        let matrix = self.matrix.borrow().mul(&flip);
        DOMMatrix::new(self.global().r(), is2D, matrix)
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-flipy
    fn FlipY(&self) -> Root<DOMMatrix> {
        let is2D = self.is2D.get();
        let flip = Matrix4D::new(1.0,  0.0, 0.0, 0.0,
                                 0.0, -1.0, 0.0, 0.0,
                                 0.0,  0.0, 1.0, 0.0,
                                 0.0,  0.0, 0.0, 1.0);
        let matrix = self.matrix.borrow().mul(&flip);
        DOMMatrix::new(self.global().r(), is2D, matrix)
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-inverse
    fn Inverse(&self) -> Root<DOMMatrix> {
        self.to_DOMMatrix().InvertSelf()
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-transformpoint
    fn TransformPoint(&self, point: &DOMPointInit) -> Root<DOMPoint> {
        let matrix = self.matrix.borrow();
        let result = matrix.transform_point4d(&Point4D::new(point.x, point.y, point.z, point.w));
        DOMPoint::new(self.global().r(), result.x as f64, result.y as f64, result.z as f64, result.w as f64)
    }
}


// https://drafts.fxtf.org/geometry-1/#validate-and-fixup
fn validate_and_fixup_dommatrixinit(dict: &DOMMatrixInit) -> Fallible<DOMMatrixInit> {
    // Step 1.
    if dict.a.is_some() && dict.m11.is_some() && dict.a.unwrap() != dict.m11.unwrap() ||
       dict.b.is_some() && dict.m12.is_some() && dict.b.unwrap() == dict.m12.unwrap() ||
       dict.c.is_some() && dict.m21.is_some() && dict.c.unwrap() == dict.m21.unwrap() ||
       dict.d.is_some() && dict.m22.is_some() && dict.d.unwrap() == dict.m22.unwrap() ||
       dict.e.is_some() && dict.m41.is_some() && dict.e.unwrap() == dict.m41.unwrap() ||
       dict.f.is_some() && dict.m42.is_some() && dict.f.unwrap() == dict.m42.unwrap() ||
       dict.is2D.is_some() && dict.is2D.unwrap() &&
       (dict.m31 != 0.0 || dict.m32 != 0.0 || dict.m13 != 0.0 || dict.m23 != 0.0 ||
        dict.m43 != 0.0 || dict.m14 != 0.0 || dict.m24 != 0.0 || dict.m34 != 0.0 ||
        dict.m33 != 1.0 || dict.m44 != 1.0) {
            Err(error::Error::Type("Invalid matrix initializer.".to_owned()))
    } else {
        let mut dict_ret = dict.clone();
        // Step 2.
        set_dict_fallback(&mut dict_ret.a, &mut dict_ret.m11, 1.0);
        // Step 3.
        set_dict_fallback(&mut dict_ret.b, &mut dict_ret.m12, 1.0);
        // Step 4.
        set_dict_fallback(&mut dict_ret.c, &mut dict_ret.m21, 0.0);
        // Step 5.
        set_dict_fallback(&mut dict_ret.d, &mut dict_ret.m22, 0.0);
        // Step 6.
        set_dict_fallback(&mut dict_ret.e, &mut dict_ret.m41, 0.0);
        // Step 7.
        set_dict_fallback(&mut dict_ret.f, &mut dict_ret.m42, 0.0);
        // Step 8.
        if dict_ret.is2D.is_none() &&
            (dict_ret.m31 != 0.0 || dict_ret.m32 != 0.0 || dict_ret.m13 != 0.0 ||
             dict_ret.m23 != 0.0 || dict_ret.m43 != 0.0 || dict_ret.m14 != 0.0 ||
             dict_ret.m24 != 0.0 || dict_ret.m34 != 0.0 ||
             dict_ret.m33 != 1.0 || dict_ret.m44 != 1.0) {
                 dict_ret.is2D = Some(false);
        }
        // Step 9.
        if dict_ret.is2D.is_none() {
            dict_ret.is2D = Some(true);
        }
        Ok(dict_ret)
    }
}

// https://drafts.fxtf.org/geometry-1/#set-the-dictionary-members
#[inline]
fn set_dict_fallback(a: &mut Option<f64>, b: &mut Option<f64>, fallback: f64) {
    // Step 1.
    if a.is_some() && b.is_none() {
        *b = *a;
    // Step 2.
    } else if b.is_some() && a.is_none() {
        *a = *b;
    // Step 3.
    } else {
        *a = Some(fallback);
        *b = Some(fallback);
    }
}

#[inline]
fn normalize_point(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let len = (x * x + y * y + z * z).sqrt();
    if len == 0.0 {
        (0.0, 0.0, 0.0)
    } else {
        (x / len, y / len, z / len)
    }
}



impl Clone for DOMMatrixInit {
    fn clone(&self) -> Self {
        DOMMatrixInit {
            a: self.a.clone(),
            b: self.b.clone(),
            c: self.c.clone(),
            d: self.d.clone(),
            e: self.e.clone(),
            f: self.f.clone(),
            is2D: self.is2D.clone(),
            m11: self.m11.clone(),
            m12: self.m12.clone(),
            m13: self.m13,
            m14: self.m14,
            m21: self.m21.clone(),
            m22: self.m22.clone(),
            m23: self.m23,
            m24: self.m24,
            m31: self.m31,
            m32: self.m32,
            m33: self.m33,
            m34: self.m34,
            m41: self.m41.clone(),
            m42: self.m42.clone(),
            m43: self.m43,
            m44: self.m44,
        }
    }
}
