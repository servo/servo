use dom::bindings::codegen::Bindings::DOMMatrixBinding::{Wrap, DOMMatrixMethods, DOMMatrixInit};
use dom::bindings::codegen::Bindings::DOMMatrixReadOnlyBinding::{DOMMatrixReadOnlyMethods};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::reflector::{reflect_dom_object, Reflectable};
use euclid::Matrix4;

use super::dommatrixreadonly::{DOMMatrixMutateMethods, DOMMatrixReadOnly, DOMMatrixWriteMethods};

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

    fn new_from_matrix4(is2D: bool, matrix: Matrix4) -> DOMMatrix {
        DOMMatrix::new_inherited(DOMMatrixReadOnly::new_from_matrix4(is2D, matrix))
    }

    pub fn new_from_matrix4_rooted(global: GlobalRef, is2D: bool, matrix: Matrix4) -> Root<DOMMatrix> {
        reflect_dom_object(box DOMMatrix::new_from_matrix4(is2D, matrix), global, Wrap)
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
        // Field accessors
        fn M11(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M11()
        }
        fn SetM11(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM11(value);
        }

        fn M12(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M12()
        }
        fn SetM12(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM12(value);
        }

        fn M13(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M13()
        }
        fn SetM13(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM13(value);
        }

        fn M14(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M14()
        }
        fn SetM14(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM14(value);
        }

        fn M21(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M21()
        }
        fn SetM21(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM21(value);
        }

        fn M22(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M22()
        }
        fn SetM22(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM22(value);
        }

        fn M23(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M23()
        }
        fn SetM23(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM23(value);
        }

        fn M24(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M24()
        }
        fn SetM24(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM24(value);
        }

        fn M31(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M31()
        }
        fn SetM31(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM31(value);
        }

        fn M32(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M32()
        }
        fn SetM32(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM32(value);
        }

        fn M33(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M33()
        }
        fn SetM33(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM33(value);
        }

        fn M34(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M34()
        }
        fn SetM34(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM34(value);
        }

        fn M41(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M41()
        }
        fn SetM41(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM41(value);
        }

        fn M42(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M42()
        }
        fn SetM42(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM42(value);
        }

        fn M43(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M43()
        }
        fn SetM43(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM43(value);
        }

        fn M44(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().M44()
        }
        fn SetM44(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetM44(value);
        }
        // Aliases
        fn A(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().A()
        }
        fn SetA(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetA(value);
        }

        fn B(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().B()
        }
        fn SetB(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetB(value);
        }

        fn C(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().C()
        }
        fn SetC(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetC(value);
        }

        fn D(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().D()
        }
        fn SetD(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetD(value);
        }

        fn E(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().E()
        }
        fn SetE(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetE(value);
        }

        fn F(&self) -> f64 {
            self.upcast::<DOMMatrixReadOnly>().F()
        }
        fn SetF(&self, value: f64) {
            self.upcast::<DOMMatrixReadOnly>().SetF(value);
        }

        // Operations
        fn MultiplySelf(&self, other:&DOMMatrixInit) -> Root<DOMMatrix> {
            self.upcast::<DOMMatrixReadOnly>().MultiplySelf(other);
            Root::from_ref(&self)
        }

        fn PreMultiplySelf(&self, other:&DOMMatrixInit) -> Root<DOMMatrix> {
            self.upcast::<DOMMatrixReadOnly>().PreMultiplySelf(other);
            Root::from_ref(&self)
        }

        fn TranslateSelf(&self, tx: f64, ty: f64, tz: f64) -> Root<DOMMatrix> {
            self.upcast::<DOMMatrixReadOnly>().TranslateSelf(tx, ty, tz);
            Root::from_ref(&self)
        }

        fn ScaleSelf(&self, scaleX: f64, scaleY: Option<f64>, scaleZ: f64, originX: f64, originY: f64, originZ: f64) -> Root<DOMMatrix> {
            self.upcast::<DOMMatrixReadOnly>().ScaleSelf(scaleX, scaleY, scaleZ, originX, originY, originZ);
            Root::from_ref(&self)
        }

        fn Scale3dSelf(&self, scale: f64, originX: f64, originY: f64, originZ: f64) -> Root<DOMMatrix> {
            self.upcast::<DOMMatrixReadOnly>().Scale3dSelf(scale, originX, originY, originZ);
            Root::from_ref(&self)
        }

        fn RotateSelf(&self, rotX: f64, rotY: Option<f64>, rotZ: Option<f64>) -> Root<DOMMatrix> {
            self.upcast::<DOMMatrixReadOnly>().RotateSelf(rotX, rotY, rotZ);
            Root::from_ref(&self)
        }

        fn RotateFromVectorSelf(&self, x: f64, y: f64) -> Root<DOMMatrix> {
            self.upcast::<DOMMatrixReadOnly>().RotateFromVectorSelf(x, y);
            Root::from_ref(&self)
        }

        fn RotateAxisAngleSelf(&self, x: f64, y: f64, z: f64, angle: f64) -> Root<DOMMatrix> {
            self.upcast::<DOMMatrixReadOnly>().RotateAxisAngleSelf(x, y, z, angle);
            Root::from_ref(&self)
        }

        fn SkewXSelf(&self, sx: f64) -> Root<DOMMatrix> {
            self.upcast::<DOMMatrixReadOnly>().SkewXSelf(sx);
            Root::from_ref(&self)
        }

        fn SkewYSelf(&self, sy: f64) -> Root<DOMMatrix> {
            self.upcast::<DOMMatrixReadOnly>().SkewYSelf(sy);
            Root::from_ref(&self)
        }

        fn InvertSelf(&self) -> Root<DOMMatrix> {
            self.upcast::<DOMMatrixReadOnly>().InvertSelf();
            Root::from_ref(&self)
        }
}
