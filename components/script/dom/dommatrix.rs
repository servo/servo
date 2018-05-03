/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DOMMatrixBinding::{Wrap, DOMMatrixMethods, DOMMatrixInit};
use dom::bindings::codegen::Bindings::DOMMatrixReadOnlyBinding::DOMMatrixReadOnlyMethods;
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::dommatrixreadonly::{dommatrixinit_to_matrix, DOMMatrixReadOnly, entries_to_matrix};
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use euclid::Transform3D;
use js::rust::CustomAutoRooterGuard;
use js::typedarray::{Float32Array, Float64Array};
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct DOMMatrix<TH: TypeHolderTrait> {
    parent: DOMMatrixReadOnly<TH>
}

impl<TH: TypeHolderTrait> DOMMatrix<TH> {
    #[allow(unrooted_must_root)]
    pub fn new(global: &GlobalScope<TH>, is2D: bool, matrix: Transform3D<f64>) -> DomRoot<Self> {
        let dommatrix = Self::new_inherited(is2D, matrix);
        reflect_dom_object(Box::new(dommatrix), global, Wrap)
    }

    pub fn new_inherited(is2D: bool, matrix: Transform3D<f64>) -> Self {
        DOMMatrix {
            parent: DOMMatrixReadOnly::new_inherited(is2D, matrix)
        }
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-dommatrix
    pub fn Constructor(global: &GlobalScope<TH>) -> Fallible<DomRoot<Self>> {
        Self::Constructor_(global, vec![1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-dommatrix-numbersequence
    pub fn Constructor_(global: &GlobalScope<TH>, entries: Vec<f64>) -> Fallible<DomRoot<Self>> {
        entries_to_matrix(&entries[..])
            .map(|(is2D, matrix)| {
                Self::new(global, is2D, matrix)
            })
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-frommatrix
    pub fn FromMatrix(global: &GlobalScope<TH>, other: &DOMMatrixInit) -> Fallible<DomRoot<Self>> {
        dommatrixinit_to_matrix(&other)
            .map(|(is2D, matrix)| {
                Self::new(global, is2D, matrix)
            })
    }

    pub fn from_readonly(global: &GlobalScope<TH>, ro: &DOMMatrixReadOnly<TH>) -> DomRoot<Self> {
        Self::new(global, ro.is_2d(), ro.matrix().clone())
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-fromfloat32array
    pub fn FromFloat32Array(
        global: &GlobalScope<TH>,
        array: CustomAutoRooterGuard<Float32Array>,
    ) -> Fallible<DomRoot<DOMMatrix<TH>>> {
        let vec: Vec<f64> = array.to_vec().iter().map(|&x| x as f64).collect();
        DOMMatrix::Constructor_(global, vec)
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-fromfloat64array
    pub fn FromFloat64Array(
        global: &GlobalScope<TH>,
        array: CustomAutoRooterGuard<Float64Array>,
    ) -> Fallible<DomRoot<DOMMatrix<TH>>> {
        let vec: Vec<f64> = array.to_vec();
        DOMMatrix::Constructor_(global, vec)
    }
}

impl<TH: TypeHolderTrait> DOMMatrixMethods<TH> for DOMMatrix<TH> {
    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m11
    fn M11(&self) -> f64 {
        self.upcast::<DOMMatrixReadOnly<TH>>().M11()
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m11
    fn SetM11(&self, value: f64) {
        self.upcast::<DOMMatrixReadOnly<TH>>().set_m11(value);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m12
    fn M12(&self) -> f64 {
        self.upcast::<DOMMatrixReadOnly<TH>>().M12()
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m12
    fn SetM12(&self, value: f64) {
        self.upcast::<DOMMatrixReadOnly<TH>>().set_m12(value);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m13
    fn M13(&self) -> f64 {
        self.upcast::<DOMMatrixReadOnly<TH>>().M13()
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m13
    fn SetM13(&self, value: f64) {
        self.upcast::<DOMMatrixReadOnly<TH>>().set_m13(value);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m14
    fn M14(&self) -> f64 {
        self.upcast::<DOMMatrixReadOnly<TH>>().M14()
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m14
    fn SetM14(&self, value: f64) {
        self.upcast::<DOMMatrixReadOnly<TH>>().set_m14(value);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m21
    fn M21(&self) -> f64 {
        self.upcast::<DOMMatrixReadOnly<TH>>().M21()
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m21
    fn SetM21(&self, value: f64) {
        self.upcast::<DOMMatrixReadOnly<TH>>().set_m21(value);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m22
    fn M22(&self) -> f64 {
        self.upcast::<DOMMatrixReadOnly<TH>>().M22()
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m22
    fn SetM22(&self, value: f64) {
        self.upcast::<DOMMatrixReadOnly<TH>>().set_m22(value);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m23
    fn M23(&self) -> f64 {
        self.upcast::<DOMMatrixReadOnly<TH>>().M23()
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m23
    fn SetM23(&self, value: f64) {
        self.upcast::<DOMMatrixReadOnly<TH>>().set_m23(value);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m24
    fn M24(&self) -> f64 {
        self.upcast::<DOMMatrixReadOnly<TH>>().M24()
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m24
    fn SetM24(&self, value: f64) {
        self.upcast::<DOMMatrixReadOnly<TH>>().set_m24(value);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m31
    fn M31(&self) -> f64 {
        self.upcast::<DOMMatrixReadOnly<TH>>().M31()
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m31
    fn SetM31(&self, value: f64) {
        self.upcast::<DOMMatrixReadOnly<TH>>().set_m31(value);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m32
    fn M32(&self) -> f64 {
        self.upcast::<DOMMatrixReadOnly<TH>>().M32()
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m32
    fn SetM32(&self, value: f64) {
        self.upcast::<DOMMatrixReadOnly<TH>>().set_m32(value);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m33
    fn M33(&self) -> f64 {
        self.upcast::<DOMMatrixReadOnly<TH>>().M33()
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m33
    fn SetM33(&self, value: f64) {
        self.upcast::<DOMMatrixReadOnly<TH>>().set_m33(value);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m34
    fn M34(&self) -> f64 {
        self.upcast::<DOMMatrixReadOnly<TH>>().M34()
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m34
    fn SetM34(&self, value: f64) {
        self.upcast::<DOMMatrixReadOnly<TH>>().set_m34(value);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m41
    fn M41(&self) -> f64 {
        self.upcast::<DOMMatrixReadOnly<TH>>().M41()
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m41
    fn SetM41(&self, value: f64) {
        self.upcast::<DOMMatrixReadOnly<TH>>().set_m41(value);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m42
    fn M42(&self) -> f64 {
        self.upcast::<DOMMatrixReadOnly<TH>>().M42()
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m42
    fn SetM42(&self, value: f64) {
        self.upcast::<DOMMatrixReadOnly<TH>>().set_m42(value);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m43
    fn M43(&self) -> f64 {
        self.upcast::<DOMMatrixReadOnly<TH>>().M43()
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m43
    fn SetM43(&self, value: f64) {
        self.upcast::<DOMMatrixReadOnly<TH>>().set_m43(value);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m44
    fn M44(&self) -> f64 {
        self.upcast::<DOMMatrixReadOnly<TH>>().M44()
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m44
    fn SetM44(&self, value: f64) {
        self.upcast::<DOMMatrixReadOnly<TH>>().set_m44(value);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-a
    fn A(&self) -> f64 {
        self.upcast::<DOMMatrixReadOnly<TH>>().A()
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-a
    fn SetA(&self, value: f64) {
        self.upcast::<DOMMatrixReadOnly<TH>>().set_m11(value);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-b
    fn B(&self) -> f64 {
        self.upcast::<DOMMatrixReadOnly<TH>>().B()
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-b
    fn SetB(&self, value: f64) {
        self.upcast::<DOMMatrixReadOnly<TH>>().set_m12(value);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-c
    fn C(&self) -> f64 {
        self.upcast::<DOMMatrixReadOnly<TH>>().C()
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-c
    fn SetC(&self, value: f64) {
        self.upcast::<DOMMatrixReadOnly<TH>>().set_m21(value);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-d
    fn D(&self) -> f64 {
        self.upcast::<DOMMatrixReadOnly<TH>>().D()
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-d
    fn SetD(&self, value: f64) {
        self.upcast::<DOMMatrixReadOnly<TH>>().set_m22(value);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-e
    fn E(&self) -> f64 {
        self.upcast::<DOMMatrixReadOnly<TH>>().E()
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-e
    fn SetE(&self, value: f64) {
        self.upcast::<DOMMatrixReadOnly<TH>>().set_m41(value);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-f
    fn F(&self) -> f64 {
        self.upcast::<DOMMatrixReadOnly<TH>>().F()
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-f
    fn SetF(&self, value: f64) {
        self.upcast::<DOMMatrixReadOnly<TH>>().set_m42(value);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-multiplyself
    fn MultiplySelf(&self, other:&DOMMatrixInit) -> Fallible<DomRoot<DOMMatrix<TH>>> {
        // Steps 1-3.
        self.upcast::<DOMMatrixReadOnly<TH>>().multiply_self(other)
            // Step 4.
            .and(Ok(DomRoot::from_ref(&self)))
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-premultiplyself
    fn PreMultiplySelf(&self, other:&DOMMatrixInit) -> Fallible<DomRoot<DOMMatrix<TH>>> {
        // Steps 1-3.
        self.upcast::<DOMMatrixReadOnly<TH>>().pre_multiply_self(other)
            // Step 4.
            .and(Ok(DomRoot::from_ref(&self)))
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-translateself
    fn TranslateSelf(&self, tx: f64, ty: f64, tz: f64) -> DomRoot<DOMMatrix<TH>> {
        // Steps 1-2.
        self.upcast::<DOMMatrixReadOnly<TH>>().translate_self(tx, ty, tz);
        // Step 3.
        DomRoot::from_ref(&self)
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-scaleself
    fn ScaleSelf(&self, scaleX: f64, scaleY: Option<f64>, scaleZ: f64,
                 originX: f64, originY: f64, originZ: f64) -> DomRoot<DOMMatrix<TH>> {
        // Steps 1-6.
        self.upcast::<DOMMatrixReadOnly<TH>>().scale_self(scaleX, scaleY, scaleZ, originX, originY, originZ);
        // Step 7.
        DomRoot::from_ref(&self)
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-scale3dself
    fn Scale3dSelf(&self, scale: f64, originX: f64, originY: f64, originZ: f64) -> DomRoot<DOMMatrix<TH>> {
        // Steps 1-4.
        self.upcast::<DOMMatrixReadOnly<TH>>().scale_3d_self(scale, originX, originY, originZ);
        // Step 5.
        DomRoot::from_ref(&self)
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-rotateself
    fn RotateSelf(&self, rotX: f64, rotY: Option<f64>, rotZ: Option<f64>) -> DomRoot<DOMMatrix<TH>> {
        // Steps 1-7.
        self.upcast::<DOMMatrixReadOnly<TH>>().rotate_self(rotX, rotY, rotZ);
        // Step 8.
        DomRoot::from_ref(&self)
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-rotatefromvectorself
    fn RotateFromVectorSelf(&self, x: f64, y: f64) -> DomRoot<DOMMatrix<TH>> {
        // Step 1.
        self.upcast::<DOMMatrixReadOnly<TH>>().rotate_from_vector_self(x, y);
        // Step 2.
        DomRoot::from_ref(&self)
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-rotateaxisangleself
    fn RotateAxisAngleSelf(&self, x: f64, y: f64, z: f64, angle: f64) -> DomRoot<DOMMatrix<TH>> {
        // Steps 1-2.
        self.upcast::<DOMMatrixReadOnly<TH>>().rotate_axis_angle_self(x, y, z, angle);
        // Step 3.
        DomRoot::from_ref(&self)
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-skewxself
    fn SkewXSelf(&self, sx: f64) -> DomRoot<DOMMatrix<TH>> {
        // Step 1.
        self.upcast::<DOMMatrixReadOnly<TH>>().skew_x_self(sx);
        // Step 2.
        DomRoot::from_ref(&self)
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-skewyself
    fn SkewYSelf(&self, sy: f64) -> DomRoot<DOMMatrix<TH>> {
        // Step 1.
        self.upcast::<DOMMatrixReadOnly<TH>>().skew_y_self(sy);
        // Step 2.
        DomRoot::from_ref(&self)
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-invertself
    fn InvertSelf(&self) -> DomRoot<DOMMatrix<TH>> {
        // Steps 1-2.
        self.upcast::<DOMMatrixReadOnly<TH>>().invert_self();
        // Step 3.
        DomRoot::from_ref(&self)
    }
}
