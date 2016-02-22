/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DOMMatrixBinding::{Wrap, DOMMatrixMethods, DOMMatrixInit};
use dom::bindings::codegen::Bindings::DOMMatrixReadOnlyBinding::DOMMatrixReadOnlyMethods;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::dommatrixreadonly::DOMMatrixReadOnly;
use euclid::Matrix4D;


#[dom_struct]
pub struct DOMMatrix {
    parent: DOMMatrixReadOnly
}

impl DOMMatrix {
    #[allow(unrooted_must_root)]
    fn new_inherited(parent: DOMMatrixReadOnly) -> DOMMatrix {
        DOMMatrix {
            parent: parent
        }
    }

    fn new_from_matrix4D(is2D: bool, matrix: Matrix4D<f64>) -> DOMMatrix {
        DOMMatrix::new_inherited(DOMMatrixReadOnly::new_from_matrix4D(is2D, matrix))
    }

    pub fn new_from_matrix4D_rooted(global: GlobalRef, is2D: bool, matrix: Matrix4D<f64>) -> Root<DOMMatrix> {
        reflect_dom_object(box DOMMatrix::new_from_matrix4D(is2D, matrix), global, Wrap)
    }

    #[allow(unrooted_must_root)]
    fn new_from_vec(entries: Vec<f64>) -> Fallible<DOMMatrix> {
        DOMMatrixReadOnly::new_from_vec(entries)
            .map(|dommatrix| DOMMatrix::new_inherited(dommatrix))
    }

    pub fn Constructor(global: GlobalRef) -> Fallible<Root<DOMMatrix>> {
        let entries = vec![1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        DOMMatrix::Constructor_(global, entries)
    }

    #[allow(unrooted_must_root)]
    pub fn Constructor_(global: GlobalRef, entries: Vec<f64>) -> Fallible<Root<DOMMatrix>> {
        DOMMatrix::new_from_vec(entries)
            .map(|dommatrix| reflect_dom_object(box dommatrix, global, Wrap))
    }

    #[allow(unrooted_must_root)]
    pub fn FromMatrix(global: GlobalRef, other: &DOMMatrixInit) -> Root<DOMMatrix> {
        let parent = DOMMatrixReadOnly::new_from_init(other);
        reflect_dom_object(box DOMMatrix::new_inherited(parent), global, Wrap)
    }
}

impl DOMMatrixMethods for DOMMatrix {
        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m11
        fn M11(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M11()
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m11
        fn SetM11(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM11(value);
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m12
        fn M12(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M12()
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m12
        fn SetM12(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM12(value);
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m13
        fn M13(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M13()
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m13
        fn SetM13(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM13(value);
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m14
        fn M14(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M14()
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m14
        fn SetM14(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM14(value);
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m21
        fn M21(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M21()
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m21
        fn SetM21(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM21(value);
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m22
        fn M22(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M22()
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m22
        fn SetM22(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM22(value);
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m23
        fn M23(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M23()
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m23
        fn SetM23(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM23(value);
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m24
        fn M24(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M24()
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m24
        fn SetM24(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM24(value);
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m31
        fn M31(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M31()
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m31
        fn SetM31(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM31(value);
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m32
        fn M32(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M32()
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m32
        fn SetM32(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM32(value);
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m33
        fn M33(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M33()
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m33
        fn SetM33(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM33(value);
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m34
        fn M34(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M34()
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m34
        fn SetM34(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM34(value);
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m41
        fn M41(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M41()
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m41
        fn SetM41(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM41(value);
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m42
        fn M42(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M42()
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m42
        fn SetM42(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM42(value);
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m43
        fn M43(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M43()
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m43
        fn SetM43(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM43(value);
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m44
        fn M44(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M44()
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m44
        fn SetM44(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM44(value);
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-a
        fn A(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().A()
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-a
        fn SetA(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetA(value);
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-b
        fn B(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().B()
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-b
        fn SetB(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetB(value);
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-c
        fn C(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().C()
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-c
        fn SetC(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetC(value);
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-d
        fn D(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().D()
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-d
        fn SetD(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetD(value);
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-e
        fn E(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().E()
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-e
        fn SetE(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetE(value);
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-f
        fn F(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().F()
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-f
        fn SetF(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetF(value);
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-multiplyself
        fn MultiplySelf(&self, other:&DOMMatrixInit) -> Root<DOMMatrix> {
            self.upcast::<DOMMatrixReadOnly>().MultiplySelf(other);
            Root::from_ref(&self)
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-premultiplyself
        fn PreMultiplySelf(&self, other:&DOMMatrixInit) -> Root<DOMMatrix> {
            self.upcast::<DOMMatrixReadOnly>().PreMultiplySelf(other);
            Root::from_ref(&self)
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-translateself
        fn TranslateSelf(&self, tx: f64, ty: f64, tz: f64) -> Root<DOMMatrix> {
            self.upcast::<DOMMatrixReadOnly>().TranslateSelf(tx, ty, tz);
            Root::from_ref(&self)
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-scaleself
        fn ScaleSelf(&self, scaleX: f64, scaleY: Option<f64>, scaleZ: f64,
                            originX: f64, originY: f64, originZ: f64) -> Root<DOMMatrix> {
            self.upcast::<DOMMatrixReadOnly>().ScaleSelf(scaleX, scaleY, scaleZ, originX, originY, originZ);
            Root::from_ref(&self)
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-scale3dself
        fn Scale3dSelf(&self, scale: f64, originX: f64, originY: f64, originZ: f64) -> Root<DOMMatrix> {
            self.upcast::<DOMMatrixReadOnly>().Scale3dSelf(scale, originX, originY, originZ);
            Root::from_ref(&self)
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-rotateself
        fn RotateSelf(&self, rotX: f64, rotY: Option<f64>, rotZ: Option<f64>) -> Root<DOMMatrix> {
            self.upcast::<DOMMatrixReadOnly>().RotateSelf(rotX, rotY, rotZ);
            Root::from_ref(&self)
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-rotatefromvectorself
        fn RotateFromVectorSelf(&self, x: f64, y: f64) -> Root<DOMMatrix> {
            self.upcast::<DOMMatrixReadOnly>().RotateFromVectorSelf(x, y);
            Root::from_ref(&self)
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-rotateaxisangleself
        fn RotateAxisAngleSelf(&self, x: f64, y: f64, z: f64, angle: f64) -> Root<DOMMatrix> {
            self.upcast::<DOMMatrixReadOnly>().RotateAxisAngleSelf(x, y, z, angle);
            Root::from_ref(&self)
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-skewxself
        fn SkewXSelf(&self, sx: f64) -> Root<DOMMatrix> {
            self.upcast::<DOMMatrixReadOnly>().SkewXSelf(sx);
            Root::from_ref(&self)
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-skewyself
        fn SkewYSelf(&self, sy: f64) -> Root<DOMMatrix> {
            self.upcast::<DOMMatrixReadOnly>().SkewYSelf(sy);
            Root::from_ref(&self)
        }

        // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-invertself
        fn InvertSelf(&self) -> Root<DOMMatrix> {
            self.upcast::<DOMMatrixReadOnly>().InvertSelf();
            Root::from_ref(&self)
        }
}
