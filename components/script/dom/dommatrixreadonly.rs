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
use euclid::Matrix4;
use euclid::Point4D;
use std::cell::{Cell, RefCell};
use std::f32;
use std::f64;

#[dom_struct]
pub struct DOMMatrixReadOnly {
    reflector_: Reflector,
    is2D: Cell<bool>,
    // TODO(peterjoel): Update this to Matrix4D<f64> when euclid suppots generics
    matrix: RefCell<Matrix4>,
}

impl DOMMatrixReadOnly {

    pub fn new_from_vec(entries: Vec<f64>) -> Fallible<DOMMatrixReadOnly> {
        // TODO this conversion (and others in this file) will be unnecessary when generic Matrix4D is used
        let entries: Vec<f32> = entries.into_iter().map(|v| v as f32).collect();
        if entries.len() == 6 {
            Ok(DOMMatrixReadOnly::new_from_matrix4(
                true,
                Matrix4::new(
                    entries[0], entries[1], 0.0, 0.0,
                    entries[2], entries[3], 0.0, 0.0,
                    0.0,        0.0,        1.0, 0.0,
                    entries[4], entries[5], 0.0, 1.0)))
        } else if entries.len() == 16 {
            Ok(DOMMatrixReadOnly::new_from_matrix4(
                false,
                Matrix4::new(
                    entries[0],  entries[1],  entries[2],  entries[3],
                    entries[4],  entries[5],  entries[6],  entries[7],
                    entries[8],  entries[9],  entries[10], entries[11],
                    entries[12], entries[13], entries[14], entries[15])))
        } else {
            let err_msg = format!("Expected 6 or 16 entries, but found {}.", entries.len());
            Err(error::Error::Type(err_msg.to_owned()))
        }
    }

    pub fn new_from_matrix4(is2D: bool, matrix: Matrix4) -> DOMMatrixReadOnly {
        DOMMatrixReadOnly {
            matrix: RefCell::new(matrix),
            reflector_: Reflector::new(),
            is2D: Cell::new(is2D),
         }
    }

    pub fn new_from_init(init: &DOMMatrixInit) -> DOMMatrixReadOnly {
        DOMMatrixReadOnly::new_from_matrix4(init.is2D.unwrap_or(false), init.to_matrix4())
    }

    #[allow(unrooted_must_root)]
    pub fn Constructor(global: GlobalRef) -> Fallible<Root<DOMMatrixReadOnly>> {
        let entries = vec![1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        DOMMatrixReadOnly::new_from_vec(entries)
            .map(|dommatrix| reflect_dom_object(box dommatrix, global, Wrap))
    }

    #[allow(unrooted_must_root)]
    pub fn Constructor_(global: GlobalRef, entries: Vec<f64>) -> Fallible<Root<DOMMatrixReadOnly>> {
        DOMMatrixReadOnly::new_from_vec(entries)
            .map(|dommatrix| reflect_dom_object(box dommatrix, global, Wrap))
    }

    pub fn FromMatrix(global: GlobalRef, other: &DOMMatrixInit) -> Root<DOMMatrixReadOnly> {
        reflect_dom_object(box DOMMatrixReadOnly::new_from_init(other), global, Wrap)
    }

    fn to_DOMMatrix(&self) -> Root<DOMMatrix> {
        let matrix = self.matrix.borrow().clone();
        let is2D = self.is2D.get();
        DOMMatrix::new_from_matrix4_rooted(self.global().r(), is2D, matrix)
    }
}


impl DOMMatrixReadOnlyMethods for DOMMatrixReadOnly {

    fn M11(&self) -> f64 {
        self.matrix.borrow().m11 as f64
    }
    fn M12(&self) -> f64 {
        self.matrix.borrow().m12 as f64
    }
    fn M13(&self) -> f64 {
        self.matrix.borrow().m13 as f64
    }
    fn M14(&self) -> f64 {
        self.matrix.borrow().m14 as f64
    }
    fn M21(&self) -> f64 {
        self.matrix.borrow().m21 as f64
    }
    fn M22(&self) -> f64 {
        self.matrix.borrow().m22 as f64
    }
    fn M23(&self) -> f64 {
        self.matrix.borrow().m23 as f64
    }
    fn M24(&self) -> f64 {
        self.matrix.borrow().m24 as f64
    }
    fn M31(&self) -> f64 {
        self.matrix.borrow().m31 as f64
    }
    fn M32(&self) -> f64 {
        self.matrix.borrow().m32 as f64
    }
    fn M33(&self) -> f64 {
        self.matrix.borrow().m33 as f64
    }
    fn M34(&self) -> f64 {
        self.matrix.borrow().m34 as f64
    }
    fn M41(&self) -> f64 {
        self.matrix.borrow().m41 as f64
    }
    fn M42(&self) -> f64 {
        self.matrix.borrow().m42 as f64
    }
    fn M43(&self) -> f64 {
        self.matrix.borrow().m43 as f64
    }
    fn M44(&self) -> f64 {
        self.matrix.borrow().m44 as f64
    }
    // Aliases
    fn A(&self) -> f64 {
        self.matrix.borrow().m11 as f64
    }
    fn B(&self) -> f64 {
        self.matrix.borrow().m12 as f64
    }
    fn C(&self) -> f64 {
        self.matrix.borrow().m21 as f64
    }
    fn D(&self) -> f64 {
        self.matrix.borrow().m22 as f64
    }
    fn E(&self) -> f64 {
        self.matrix.borrow().m41 as f64
    }
    fn F(&self) -> f64 {
        self.matrix.borrow().m42 as f64
    }
    //
    fn Is2D(&self) -> bool {
        self.is2D.get()
    }
    fn IsIdentity(&self) -> bool {
        let matrix = self.matrix.borrow();
        matrix.m12 == 0.0 && matrix.m13 == 0.0 && matrix.m14 == 0.0 && matrix.m21 == 0.0
            && matrix.m23 == 0.0 && matrix.m24 == 0.0 && matrix.m31 == 0.0 && matrix.m32 == 0.0
            && matrix.m34 == 0.0 && matrix.m41 == 0.0 && matrix.m42 == 0.0 && matrix.m43 == 0.0
            && matrix.m11 == 1.0 && matrix.m22 == 1.0 && matrix.m33 == 1.0 && matrix.m44 == 1.0
    }

    fn Translate(&self, tx:f64, ty:f64, tz:f64) -> Root<DOMMatrix> {
        self.to_DOMMatrix().TranslateSelf(tx, ty, tz)
    }

    fn Scale(&self, scaleX:f64, scaleY:Option<f64>, scaleZ:f64, originX: f64, originY: f64, originZ: f64) -> Root<DOMMatrix> {
        self.to_DOMMatrix().ScaleSelf(scaleX, scaleY, scaleZ, originX, originY, originZ)
    }

    fn Scale3d(&self, scale: f64, originX: f64, originY: f64, originZ: f64) -> Root<DOMMatrix> {
        self.to_DOMMatrix().Scale3dSelf(scale, originX, originY, originZ)
    }

    fn Rotate(&self, rotX: f64, rotY: Option<f64>, rotZ: Option<f64>) -> Root<DOMMatrix> {
        self.to_DOMMatrix().RotateSelf(rotX, rotY, rotZ)
    }

    fn RotateFromVector(&self, x: f64, y:f64) -> Root<DOMMatrix> {
        self.to_DOMMatrix().RotateFromVectorSelf(x, y)
    }

    fn RotateAxisAngle(&self, x: f64, y: f64, z: f64, angle: f64) -> Root<DOMMatrix> {
        self.to_DOMMatrix().RotateAxisAngleSelf(x, y, z, angle)
    }

    fn SkewX(&self, sx: f64) -> Root<DOMMatrix> {
        self.to_DOMMatrix().SkewXSelf(sx)
    }

    fn SkewY(&self, sy: f64) -> Root<DOMMatrix> {
        self.to_DOMMatrix().SkewYSelf(sy)
    }

    fn Multiply(&self, other: &DOMMatrixInit) -> Root<DOMMatrix> {
        self.to_DOMMatrix().MultiplySelf(&other)
    }

    fn FlipX(&self) -> Root<DOMMatrix> {
        let is2D = self.is2D.get();
        let flip = Matrix4::new(-1.0, 0.0, 0.0, 0.0,
                                 0.0, 1.0, 0.0, 0.0,
                                 0.0, 0.0, 1.0, 0.0,
                                 0.0, 0.0, 0.0, 1.0);
        let matrix = self.matrix.borrow().mul(&flip);
        DOMMatrix::new_from_matrix4_rooted(self.global().r(), is2D, matrix)
    }

    fn FlipY(&self) -> Root<DOMMatrix> {
        let is2D = self.is2D.get();
        let flip = Matrix4::new(1.0,  0.0, 0.0, 0.0,
                                0.0, -1.0, 0.0, 0.0,
                                0.0,  0.0, 1.0, 0.0,
                                0.0,  0.0, 0.0, 1.0);
        let matrix = self.matrix.borrow().mul(&flip);
        DOMMatrix::new_from_matrix4_rooted(self.global().r(), is2D, matrix)
    }

    fn Inverse(&self) -> Root<DOMMatrix> {
        self.to_DOMMatrix().InvertSelf()
    }

    fn TransformPoint(&self, point: &DOMPointInit) -> Root<DOMPoint> {
        let matrix = self.matrix.borrow();
        let result = matrix.transform_point4d(&Point4D::new(point.x as f32, point.y as f32, point.z as f32, point.w as f32));
        DOMPoint::new(self.global().r(), result.x as f64, result.y as f64, result.z as f64, result.w as f64)
    }
}


pub trait DOMMatrixWriteMethods {
    fn SetM11(&self, value: f64);
    fn SetM12(&self, value: f64);
    fn SetM13(&self, value: f64);
    fn SetM14(&self, value: f64);
    fn SetM21(&self, value: f64);
    fn SetM22(&self, value: f64);
    fn SetM23(&self, value: f64);
    fn SetM24(&self, value: f64);
    fn SetM31(&self, value: f64);
    fn SetM32(&self, value: f64);
    fn SetM33(&self, value: f64);
    fn SetM34(&self, value: f64);
    fn SetM41(&self, value: f64);
    fn SetM42(&self, value: f64);
    fn SetM43(&self, value: f64);
    fn SetM44(&self, value: f64);
    fn SetA(&self, value: f64);
    fn SetB(&self, value: f64);
    fn SetC(&self, value: f64);
    fn SetD(&self, value: f64);
    fn SetE(&self, value: f64);
    fn SetF(&self, value: f64);
}

impl DOMMatrixWriteMethods for DOMMatrixReadOnly {
    fn SetM11(&self, value: f64){
        self.matrix.borrow_mut().m11 = value as f32;
    }
    fn SetM12(&self, value: f64){
        self.matrix.borrow_mut().m12 = value as f32;
    }
    fn SetM13(&self, value: f64){
        self.matrix.borrow_mut().m13 = value as f32;
    }
    fn SetM14(&self, value: f64){
        self.matrix.borrow_mut().m14 = value as f32;
    }
    fn SetM21(&self, value: f64){
        self.matrix.borrow_mut().m21 = value as f32;
    }
    fn SetM22(&self, value: f64){
        self.matrix.borrow_mut().m22 = value as f32;
    }
    fn SetM23(&self, value: f64){
        self.matrix.borrow_mut().m23 = value as f32;
    }
    fn SetM24(&self, value: f64){
        self.matrix.borrow_mut().m24 = value as f32;
    }
    fn SetM31(&self, value: f64){
        self.matrix.borrow_mut().m31 = value as f32;
    }
    fn SetM32(&self, value: f64){
        self.matrix.borrow_mut().m32 = value as f32;
    }
    fn SetM33(&self, value: f64){
        self.matrix.borrow_mut().m33 = value as f32;
    }
    fn SetM34(&self, value: f64){
        self.matrix.borrow_mut().m34 = value as f32;
    }
    fn SetM41(&self, value: f64){
        self.matrix.borrow_mut().m41 = value as f32;
    }
    fn SetM42(&self, value: f64){
        self.matrix.borrow_mut().m42 = value as f32;
    }
    fn SetM43(&self, value: f64){
        self.matrix.borrow_mut().m43 = value as f32;
    }
    fn SetM44(&self, value: f64){
        self.matrix.borrow_mut().m44 = value as f32;
    }
    fn SetA(&self, value: f64){
        self.matrix.borrow_mut().m11 = value as f32;
    }
    fn SetB(&self, value: f64){
        self.matrix.borrow_mut().m12 = value as f32;
    }
    fn SetC(&self, value: f64){
        self.matrix.borrow_mut().m21 = value as f32;
    }
    fn SetD(&self, value: f64){
        self.matrix.borrow_mut().m22 = value as f32;
    }
    fn SetE(&self, value: f64){
        self.matrix.borrow_mut().m41 = value as f32;
    }
    fn SetF(&self, value: f64){
        self.matrix.borrow_mut().m42 = value as f32;
    }
}


pub trait DOMMatrixMutateMethods {
    fn MultiplySelf(&self, other: &DOMMatrixInit);
    fn PreMultiplySelf(&self, other: &DOMMatrixInit);
    fn TranslateSelf(&self, tx: f64, ty: f64, tz: f64);
    fn ScaleSelf(&self, scaleX: f64, scaleY: Option<f64>, scaleZ: f64, originX: f64, originY: f64, originZ: f64);
    fn Scale3dSelf(&self, scale: f64, originX: f64, originY: f64, originZ: f64);
    fn RotateSelf(&self, rotX: f64, rotY: Option<f64>, rotZ: Option<f64>);
    fn RotateFromVectorSelf(&self, x: f64, y: f64);
    fn RotateAxisAngleSelf(&self, x: f64, y: f64, z: f64, angle: f64);
    fn SkewXSelf(&self, sx: f64);
    fn SkewYSelf(&self, sy: f64);
    fn InvertSelf(&self);
    // fn SetMatrixValue(&self, transformList: DOMString);
}

impl DOMMatrixMutateMethods for DOMMatrixReadOnly {
    fn MultiplySelf(&self, other: &DOMMatrixInit) {
        let mut matrix = self.matrix.borrow_mut();
        *matrix = matrix.mul(&other.to_matrix4());
    }

    fn PreMultiplySelf(&self, other: &DOMMatrixInit) {
        let mut matrix = self.matrix.borrow_mut();
        *matrix = other.to_matrix4().mul(&matrix);
    }

    fn TranslateSelf(&self, tx: f64, ty: f64, tz: f64) {
        let mut matrix = self.matrix.borrow_mut();
        let translation: Matrix4 = Matrix4::create_translation(tx as f32, ty as f32, tz as f32);
        *matrix = translation.mul(&matrix);
        if tz != 0.0 {
            self.is2D.set(false);
        }
    }

    fn ScaleSelf(&self, scaleX: f64, scaleY: Option<f64>, scaleZ: f64, originX: f64, originY: f64, originZ: f64) {
        let mut matrix = self.matrix.borrow_mut();
        let translation: Matrix4 = Matrix4::create_translation(originX as f32, originY as f32, originZ as f32);
        *matrix = translation.mul(&matrix);
        let scale3D = Matrix4::create_scale(scaleX as f32, scaleY.unwrap_or(scaleX) as f32, scaleZ as f32);
        *matrix = scale3D.mul(&matrix);
        let translation_rev: Matrix4 = Matrix4::create_translation(-originX as f32, -originY as f32, -originZ as f32);
        *matrix = translation_rev.mul(&matrix);
        if scaleZ != 1.0 || originZ != 0.0 {
            self.is2D.set(false);
        }
    }

    fn Scale3dSelf(&self, scale: f64, originX: f64, originY: f64, originZ: f64) {
        let mut matrix = self.matrix.borrow_mut();
        let translation: Matrix4 = Matrix4::create_translation(originX as f32, originY as f32, originZ as f32);
        *matrix = translation.mul(&matrix);
        let scale3D = Matrix4::create_scale(scale as f32, scale as f32, scale as f32);
        *matrix = scale3D.mul(&matrix);
        let translation_rev: Matrix4 = Matrix4::create_translation(-originX as f32, -originY as f32, -originZ as f32);
        *matrix = translation_rev.mul(&matrix);
        if scale != 1.0 {
            self.is2D.set(false);
        }
    }

    fn RotateSelf(&self, rotX: f64, rotY: Option<f64>, rotZ: Option<f64>) {
        let mut matrix = self.matrix.borrow_mut();
        let (rotX, rotY, rotZ) = match rotY {
            None     => (0.0, 0.0, rotX),
            Some(ry) => match rotZ {
                    None     => (rotX, ry, 0.0),
                    Some(rz) => (rotX, ry, rz)
            }
        };
        if rotZ != 0.0 {
            *matrix = Matrix4::create_rotation(0.0, 0.0, 1.0, deg_to_rad(rotZ) as f32).mul(&matrix);
        }
        if rotY != 0.0 {
            *matrix = Matrix4::create_rotation(0.0, 1.0, 0.0, deg_to_rad(rotY) as f32).mul(&matrix);
        }
        if rotX != 0.0 {
            *matrix = Matrix4::create_rotation(1.0, 0.0, 0.0, deg_to_rad(rotX) as f32).mul(&matrix);
        }
        if rotX != 0.0 || rotY != 0.0 {
            self.is2D.set(false);
        }
    }

    fn RotateFromVectorSelf(&self, x: f64, y: f64) {
        let mut matrix = self.matrix.borrow_mut();
        // don't do anything when the rotation angle is zero or undefined
        if y != 0.0 || x < 0.0 {
            let rotZ = f64::atan2(y, x);
            *matrix = Matrix4::create_rotation(0.0, 0.0, 1.0, rotZ as f32).mul(&matrix);
        }
    }

    fn RotateAxisAngleSelf(&self, x: f64, y: f64, z: f64, angle: f64) {
        let mut matrix = self.matrix.borrow_mut();
        let (norm_x, norm_y, norm_z) = normalize_point(x, y, z);
        let rotation: Matrix4 = Matrix4::create_rotation(norm_x as f32, norm_y as f32, norm_z as f32, deg_to_rad(angle) as f32);
        *matrix = rotation.mul(&matrix);
        if x != 0.0 || y != 0.0 {
            self.is2D.set(false);
        }
    }

    fn SkewXSelf(&self, sx: f64) {
        let mut matrix = self.matrix.borrow_mut();
        let skew_x = Matrix4::create_skew(deg_to_rad(sx) as f32, 0.0);
        *matrix = skew_x.mul(&matrix);
    }

    fn SkewYSelf(&self, sy: f64) {
        let mut matrix = self.matrix.borrow_mut();
        let skew_y = Matrix4::create_skew(0.0, deg_to_rad(sy) as f32);
        *matrix = skew_y.mul(&matrix);
    }

    fn InvertSelf(&self) {
        let mut matrix = self.matrix.borrow_mut();
        if matrix.determinant() == 0.0 {
            *matrix = Matrix4::new(f32::NAN, f32::NAN, f32::NAN, f32::NAN,
                                   f32::NAN, f32::NAN, f32::NAN, f32::NAN,
                                   f32::NAN, f32::NAN, f32::NAN, f32::NAN,
                                   f32::NAN, f32::NAN, f32::NAN, f32::NAN);
            self.is2D.set(false);
        } else {
            *matrix = matrix.invert();
        }
    }
}

#[inline]
fn deg_to_rad(x: f64) -> f64 {
    x * f64::consts::PI / 180.0
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

impl DOMMatrixInit {
    fn to_matrix4(&self) -> Matrix4 {
        Matrix4::new(self.m11.unwrap_or(1.0) as f32, self.m12.unwrap_or(0.0) as f32, self.m13 as f32, self.m14 as f32,
                     self.m21.unwrap_or(0.0) as f32, self.m22.unwrap_or(1.0) as f32, self.m23 as f32, self.m24 as f32,
                     self.m31 as f32,                self.m32 as f32,                self.m33 as f32, self.m34 as f32,
                     self.m41.unwrap_or(0.0) as f32, self.m42.unwrap_or(0.0) as f32, self.m43 as f32, self.m44 as f32)
    }
}
