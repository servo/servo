/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::{f64, ptr};

use base::id::{DomMatrixId, DomMatrixIndex};
use constellation_traits::DomMatrix;
use cssparser::{Parser, ParserInput};
use dom_struct::dom_struct;
use euclid::Angle;
use euclid::default::{Transform2D, Transform3D};
use js::conversions::jsstr_to_string;
use js::jsapi::JSObject;
use js::jsval;
use js::rust::{CustomAutoRooterGuard, HandleObject, ToString};
use js::typedarray::{Float32Array, Float64Array, HeapFloat32Array, HeapFloat64Array};
use rustc_hash::FxHashMap;
use script_bindings::trace::RootedTraceableBox;
use style::parser::ParserContext;
use url::Url;

use crate::dom::bindings::buffer_source::create_buffer_source;
use crate::dom::bindings::cell::{DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::DOMMatrixBinding::{
    DOMMatrix2DInit, DOMMatrixInit, DOMMatrixMethods,
};
use crate::dom::bindings::codegen::Bindings::DOMMatrixReadOnlyBinding::DOMMatrixReadOnlyMethods;
use crate::dom::bindings::codegen::Bindings::DOMPointBinding::DOMPointInit;
use crate::dom::bindings::codegen::UnionTypes::StringOrUnrestrictedDoubleSequence;
use crate::dom::bindings::error;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::serializable::Serializable;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::structuredclone::StructuredData;
use crate::dom::dommatrix::DOMMatrix;
use crate::dom::dompoint::DOMPoint;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use crate::script_runtime::{CanGc, JSContext};

#[dom_struct]
#[expect(non_snake_case)]
pub(crate) struct DOMMatrixReadOnly {
    reflector_: Reflector,
    #[no_trace]
    matrix: DomRefCell<Transform3D<f64>>,
    is2D: Cell<bool>,
}

#[expect(non_snake_case)]
impl DOMMatrixReadOnly {
    pub(crate) fn new(
        global: &GlobalScope,
        is2D: bool,
        matrix: Transform3D<f64>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        Self::new_with_proto(global, None, is2D, matrix, can_gc)
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        is2D: bool,
        matrix: Transform3D<f64>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        let dommatrix = Self::new_inherited(is2D, matrix);
        reflect_dom_object_with_proto(Box::new(dommatrix), global, proto, can_gc)
    }

    pub(crate) fn new_inherited(is2D: bool, matrix: Transform3D<f64>) -> Self {
        DOMMatrixReadOnly {
            reflector_: Reflector::new(),
            matrix: DomRefCell::new(matrix),
            is2D: Cell::new(is2D),
        }
    }

    pub(crate) fn matrix(&self) -> Ref<'_, Transform3D<f64>> {
        self.matrix.borrow()
    }

    pub(crate) fn set_matrix(&self, value: Transform3D<f64>) {
        self.set_m11(value.m11);
        self.set_m12(value.m12);
        self.set_m13(value.m13);
        self.set_m14(value.m14);
        self.set_m21(value.m21);
        self.set_m22(value.m22);
        self.set_m23(value.m23);
        self.set_m24(value.m24);
        self.set_m31(value.m31);
        self.set_m32(value.m32);
        self.set_m33(value.m33);
        self.set_m34(value.m34);
        self.set_m41(value.m41);
        self.set_m42(value.m42);
        self.set_m43(value.m43);
        self.set_m44(value.m44);
    }

    pub(crate) fn is2D(&self) -> bool {
        self.is2D.get()
    }

    pub(crate) fn set_is2D(&self, value: bool) {
        self.is2D.set(value);
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m11
    pub(crate) fn set_m11(&self, value: f64) {
        self.matrix.borrow_mut().m11 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m12
    pub(crate) fn set_m12(&self, value: f64) {
        self.matrix.borrow_mut().m12 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m13
    pub(crate) fn set_m13(&self, value: f64) {
        // For the DOMMatrix interface, setting the m13 attribute must set the
        // m13 element to the new value and, if the new value is not 0 or -0, set is 2D to false.

        self.matrix.borrow_mut().m13 = value;
        if value.abs() != 0. {
            self.is2D.set(false);
        }
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m14
    pub(crate) fn set_m14(&self, value: f64) {
        // For the DOMMatrix interface, setting the m14 attribute must set the
        // m14 element to the new value and, if the new value is not 0 or -0, set is 2D to false.
        self.matrix.borrow_mut().m14 = value;

        if value.abs() != 0. {
            self.is2D.set(false);
        }
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m21
    pub(crate) fn set_m21(&self, value: f64) {
        self.matrix.borrow_mut().m21 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m22
    pub(crate) fn set_m22(&self, value: f64) {
        self.matrix.borrow_mut().m22 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m23
    pub(crate) fn set_m23(&self, value: f64) {
        // For the DOMMatrix interface, setting the m23 attribute must set the
        // m23 element to the new value and, if the new value is not 0 or -0, set is 2D to false.
        self.matrix.borrow_mut().m23 = value;

        if value.abs() != 0. {
            self.is2D.set(false);
        }
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m24
    pub(crate) fn set_m24(&self, value: f64) {
        // For the DOMMatrix interface, setting the m24 attribute must set the
        // m24 element to the new value and, if the new value is not 0 or -0, set is 2D to false.
        self.matrix.borrow_mut().m24 = value;

        if value.abs() != 0. {
            self.is2D.set(false);
        }
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m31
    pub(crate) fn set_m31(&self, value: f64) {
        // For the DOMMatrix interface, setting the m31 attribute must set the
        // m31 element to the new value and, if the new value is not 0 or -0, set is 2D to false.
        self.matrix.borrow_mut().m31 = value;

        if value.abs() != 0. {
            self.is2D.set(false);
        }
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m32
    pub(crate) fn set_m32(&self, value: f64) {
        // For the DOMMatrix interface, setting the m32 attribute must set the
        // m32 element to the new value and, if the new value is not 0 or -0, set is 2D to false.
        self.matrix.borrow_mut().m32 = value;

        if value.abs() != 0. {
            self.is2D.set(false);
        }
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m33
    pub(crate) fn set_m33(&self, value: f64) {
        // For the DOMMatrix interface, setting the m33 attribute must set the
        // m33 element to the new value and, if the new value is not 1, set is 2D to false.
        self.matrix.borrow_mut().m33 = value;

        if value != 1. {
            self.is2D.set(false);
        }
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m34
    pub(crate) fn set_m34(&self, value: f64) {
        // For the DOMMatrix interface, setting the m34 attribute must set the
        // m34 element to the new value and, if the new value is not 0 or -0, set is 2D to false.
        self.matrix.borrow_mut().m34 = value;

        if value.abs() != 0. {
            self.is2D.set(false);
        }
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m41
    pub(crate) fn set_m41(&self, value: f64) {
        self.matrix.borrow_mut().m41 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m42
    pub(crate) fn set_m42(&self, value: f64) {
        self.matrix.borrow_mut().m42 = value;
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m43
    pub(crate) fn set_m43(&self, value: f64) {
        // For the DOMMatrix interface, setting the m43 attribute must set the
        // m43 element to the new value and, if the new value is not 0 or -0, set is 2D to false.
        self.matrix.borrow_mut().m43 = value;

        if value.abs() != 0. {
            self.is2D.set(false);
        }
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m44
    pub(crate) fn set_m44(&self, value: f64) {
        // For the DOMMatrix interface, setting the m44 attribute must set the
        // m44 element to the new value and, if the new value is not 1, set is 2D to false.
        self.matrix.borrow_mut().m44 = value;

        if value != 1. {
            self.is2D.set(false);
        }
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-multiplyself
    pub(crate) fn multiply_self(&self, other: &DOMMatrixInit) -> Fallible<()> {
        // Step 1.
        dommatrixinit_to_matrix(other).map(|(is2D, other_matrix)| {
            // Step 2.
            let mut matrix = self.matrix.borrow_mut();
            *matrix = other_matrix.then(&matrix);
            // Step 3.
            if !is2D {
                self.is2D.set(false);
            }
            // Step 4 in DOMMatrix.MultiplySelf
        })
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-premultiplyself
    pub(crate) fn pre_multiply_self(&self, other: &DOMMatrixInit) -> Fallible<()> {
        // Step 1.
        dommatrixinit_to_matrix(other).map(|(is2D, other_matrix)| {
            // Step 2.
            let mut matrix = self.matrix.borrow_mut();
            *matrix = matrix.then(&other_matrix);
            // Step 3.
            if !is2D {
                self.is2D.set(false);
            }
            // Step 4 in DOMMatrix.PreMultiplySelf
        })
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-translateself
    pub(crate) fn translate_self(&self, tx: f64, ty: f64, tz: f64) {
        // Step 1.
        let translation = Transform3D::translation(tx, ty, tz);
        let mut matrix = self.matrix.borrow_mut();
        *matrix = translation.then(&matrix);
        // Step 2.
        if tz != 0.0 {
            self.is2D.set(false);
        }
        // Step 3 in DOMMatrix.TranslateSelf
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-scaleself
    pub(crate) fn scale_self(
        &self,
        scaleX: f64,
        scaleY: Option<f64>,
        scaleZ: f64,
        mut originX: f64,
        mut originY: f64,
        mut originZ: f64,
    ) {
        // Step 1.
        self.translate_self(originX, originY, originZ);
        // Step 2.
        let scaleY = scaleY.unwrap_or(scaleX);
        // Step 3.
        {
            let scale3D = Transform3D::scale(scaleX, scaleY, scaleZ);
            let mut matrix = self.matrix.borrow_mut();
            *matrix = scale3D.then(&matrix);
        }
        // Step 4.
        originX = -originX;
        originY = -originY;
        originZ = -originZ;
        // Step 5.
        self.translate_self(originX, originY, originZ);
        // Step 6.
        if scaleZ != 1.0 || originZ != 0.0 {
            self.is2D.set(false);
        }
        // Step 7 in DOMMatrix.ScaleSelf
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-scale3dself
    pub(crate) fn scale_3d_self(&self, scale: f64, originX: f64, originY: f64, originZ: f64) {
        // Step 1.
        self.translate_self(originX, originY, originZ);
        // Step 2.
        {
            let scale3D = Transform3D::scale(scale, scale, scale);
            let mut matrix = self.matrix.borrow_mut();
            *matrix = scale3D.then(&matrix);
        }
        // Step 3.
        self.translate_self(-originX, -originY, -originZ);
        // Step 4.
        if scale != 1.0 {
            self.is2D.set(false);
        }
        // Step 5 in DOMMatrix.Scale3dSelf
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-rotateself
    pub(crate) fn rotate_self(&self, mut rotX: f64, mut rotY: Option<f64>, mut rotZ: Option<f64>) {
        // Step 1.
        if rotY.is_none() && rotZ.is_none() {
            rotZ = Some(rotX);
            rotX = 0.0;
            rotY = Some(0.0);
        }
        // Step 2.
        let rotY = rotY.unwrap_or(0.0);
        // Step 3.
        let rotZ = rotZ.unwrap_or(0.0);
        // Step 4.
        if rotX != 0.0 || rotY != 0.0 {
            self.is2D.set(false);
        }
        if rotZ != 0.0 {
            // Step 5.
            let rotation = Transform3D::rotation(0.0, 0.0, 1.0, Angle::radians(rotZ.to_radians()));
            let mut matrix = self.matrix.borrow_mut();
            *matrix = rotation.then(&matrix);
        }
        if rotY != 0.0 {
            // Step 6.
            let rotation = Transform3D::rotation(0.0, 1.0, 0.0, Angle::radians(rotY.to_radians()));
            let mut matrix = self.matrix.borrow_mut();
            *matrix = rotation.then(&matrix);
        }
        if rotX != 0.0 {
            // Step 7.
            let rotation = Transform3D::rotation(1.0, 0.0, 0.0, Angle::radians(rotX.to_radians()));
            let mut matrix = self.matrix.borrow_mut();
            *matrix = rotation.then(&matrix);
        }
        // Step 8 in DOMMatrix.RotateSelf
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-rotatefromvectorself
    pub(crate) fn rotate_from_vector_self(&self, x: f64, y: f64) {
        // don't do anything when the rotation angle is zero or undefined
        if y != 0.0 || x < 0.0 {
            // Step 1.
            let rotZ = Angle::radians(f64::atan2(y, x));
            let rotation = Transform3D::rotation(0.0, 0.0, 1.0, rotZ);
            let mut matrix = self.matrix.borrow_mut();
            *matrix = rotation.then(&matrix);
        }
        // Step 2 in DOMMatrix.RotateFromVectorSelf
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-rotateaxisangleself
    pub(crate) fn rotate_axis_angle_self(&self, x: f64, y: f64, z: f64, angle: f64) {
        // Step 1.
        let (norm_x, norm_y, norm_z) = normalize_point(x, y, z);
        // Beware: pass negated value until https://github.com/servo/euclid/issues/354
        let rotation =
            Transform3D::rotation(norm_x, norm_y, norm_z, Angle::radians(angle.to_radians()));
        let mut matrix = self.matrix.borrow_mut();
        *matrix = rotation.then(&matrix);
        // Step 2.
        if x != 0.0 || y != 0.0 {
            self.is2D.set(false);
        }
        // Step 3 in DOMMatrix.RotateAxisAngleSelf
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-skewxself
    pub(crate) fn skew_x_self(&self, sx: f64) {
        // Step 1.
        let skew = Transform3D::skew(Angle::radians(sx.to_radians()), Angle::radians(0.0));
        let mut matrix = self.matrix.borrow_mut();
        *matrix = skew.then(&matrix);
        // Step 2 in DOMMatrix.SkewXSelf
    }

    // https://drafts.fxtf.org/geometry-1/#dom-dommatrix-skewyself
    pub(crate) fn skew_y_self(&self, sy: f64) {
        // Step 1.
        let skew = Transform3D::skew(Angle::radians(0.0), Angle::radians(sy.to_radians()));
        let mut matrix = self.matrix.borrow_mut();
        *matrix = skew.then(&matrix);
        // Step 2 in DOMMatrix.SkewYSelf
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrix-invertself>
    pub(crate) fn invert_self(&self) {
        let mut matrix = self.matrix.borrow_mut();
        // Step 1. Invert the current matrix.
        let inverted = match self.is2D() {
            true => matrix.to_2d().inverse().map(|m| m.to_3d()),
            false => matrix.inverse(),
        };

        // Step 2. If the current matrix is not invertible set all attributes to NaN
        // and set is 2D to false.
        *matrix = inverted.unwrap_or_else(|| -> Transform3D<f64> {
            self.is2D.set(false);
            Transform3D::new(
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
                f64::NAN,
            )
        });
        // Step 3 in DOMMatrix.InvertSelf
    }
}

#[expect(non_snake_case)]
impl DOMMatrixReadOnlyMethods<crate::DomTypeHolder> for DOMMatrixReadOnly {
    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-dommatrixreadonly>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        init: Option<StringOrUnrestrictedDoubleSequence>,
    ) -> Fallible<DomRoot<Self>> {
        if init.is_none() {
            return Ok(Self::new_with_proto(
                global,
                proto,
                true,
                Transform3D::identity(),
                can_gc,
            ));
        }
        match init.unwrap() {
            StringOrUnrestrictedDoubleSequence::String(ref s) => {
                if !global.is::<Window>() {
                    return Err(error::Error::Type(
                        "String constructor is only supported in the main thread.".to_owned(),
                    ));
                }
                if s.is_empty() {
                    return Ok(Self::new(global, true, Transform3D::identity(), can_gc));
                }
                transform_to_matrix(s.to_string())
                    .map(|(is2D, matrix)| Self::new_with_proto(global, proto, is2D, matrix, can_gc))
            },
            StringOrUnrestrictedDoubleSequence::UnrestrictedDoubleSequence(ref entries) => {
                entries_to_matrix(&entries[..])
                    .map(|(is2D, matrix)| Self::new_with_proto(global, proto, is2D, matrix, can_gc))
            },
        }
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-frommatrix>
    fn FromMatrix(
        global: &GlobalScope,
        other: &DOMMatrixInit,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<Self>> {
        dommatrixinit_to_matrix(other).map(|(is2D, matrix)| Self::new(global, is2D, matrix, can_gc))
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-fromfloat32array>
    fn FromFloat32Array(
        global: &GlobalScope,
        array: CustomAutoRooterGuard<Float32Array>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<DOMMatrixReadOnly>> {
        let vec: Vec<f64> = array.to_vec().iter().map(|&x| x as f64).collect();
        DOMMatrixReadOnly::Constructor(
            global,
            None,
            can_gc,
            Some(StringOrUnrestrictedDoubleSequence::UnrestrictedDoubleSequence(vec)),
        )
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-fromfloat64array>
    fn FromFloat64Array(
        global: &GlobalScope,
        array: CustomAutoRooterGuard<Float64Array>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<DOMMatrixReadOnly>> {
        let vec: Vec<f64> = array.to_vec();
        DOMMatrixReadOnly::Constructor(
            global,
            None,
            can_gc,
            Some(StringOrUnrestrictedDoubleSequence::UnrestrictedDoubleSequence(vec)),
        )
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m11>
    fn M11(&self) -> f64 {
        self.matrix.borrow().m11
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m12>
    fn M12(&self) -> f64 {
        self.matrix.borrow().m12
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m13>
    fn M13(&self) -> f64 {
        self.matrix.borrow().m13
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m14>
    fn M14(&self) -> f64 {
        self.matrix.borrow().m14
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m21>
    fn M21(&self) -> f64 {
        self.matrix.borrow().m21
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m22>
    fn M22(&self) -> f64 {
        self.matrix.borrow().m22
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m23>
    fn M23(&self) -> f64 {
        self.matrix.borrow().m23
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m24>
    fn M24(&self) -> f64 {
        self.matrix.borrow().m24
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m31>
    fn M31(&self) -> f64 {
        self.matrix.borrow().m31
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m32>
    fn M32(&self) -> f64 {
        self.matrix.borrow().m32
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m33>
    fn M33(&self) -> f64 {
        self.matrix.borrow().m33
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m34>
    fn M34(&self) -> f64 {
        self.matrix.borrow().m34
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m41>
    fn M41(&self) -> f64 {
        self.matrix.borrow().m41
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m42>
    fn M42(&self) -> f64 {
        self.matrix.borrow().m42
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m43>
    fn M43(&self) -> f64 {
        self.matrix.borrow().m43
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-m44>
    fn M44(&self) -> f64 {
        self.matrix.borrow().m44
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-a>
    fn A(&self) -> f64 {
        self.M11()
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-b>
    fn B(&self) -> f64 {
        self.M12()
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-c>
    fn C(&self) -> f64 {
        self.M21()
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-d>
    fn D(&self) -> f64 {
        self.M22()
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-e>
    fn E(&self) -> f64 {
        self.M41()
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-f>
    fn F(&self) -> f64 {
        self.M42()
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-is2d>
    fn Is2D(&self) -> bool {
        self.is2D.get()
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-isidentity>
    fn IsIdentity(&self) -> bool {
        let matrix = self.matrix.borrow();
        matrix.m12 == 0.0 &&
            matrix.m13 == 0.0 &&
            matrix.m14 == 0.0 &&
            matrix.m21 == 0.0 &&
            matrix.m23 == 0.0 &&
            matrix.m24 == 0.0 &&
            matrix.m31 == 0.0 &&
            matrix.m32 == 0.0 &&
            matrix.m34 == 0.0 &&
            matrix.m41 == 0.0 &&
            matrix.m42 == 0.0 &&
            matrix.m43 == 0.0 &&
            matrix.m11 == 1.0 &&
            matrix.m22 == 1.0 &&
            matrix.m33 == 1.0 &&
            matrix.m44 == 1.0
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-translate>
    fn Translate(&self, tx: f64, ty: f64, tz: f64, can_gc: CanGc) -> DomRoot<DOMMatrix> {
        DOMMatrix::from_readonly(&self.global(), self, can_gc).TranslateSelf(tx, ty, tz)
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-scale>
    fn Scale(
        &self,
        scaleX: f64,
        scaleY: Option<f64>,
        scaleZ: f64,
        originX: f64,
        originY: f64,
        originZ: f64,
        can_gc: CanGc,
    ) -> DomRoot<DOMMatrix> {
        DOMMatrix::from_readonly(&self.global(), self, can_gc)
            .ScaleSelf(scaleX, scaleY, scaleZ, originX, originY, originZ)
    }

    /// <https://drafts.fxtf.org/geometry/#dom-dommatrixreadonly-scalenonuniform>
    fn ScaleNonUniform(&self, scaleX: f64, scaleY: f64, can_gc: CanGc) -> DomRoot<DOMMatrix> {
        DOMMatrix::from_readonly(&self.global(), self, can_gc).ScaleSelf(
            scaleX,
            Some(scaleY),
            1.0,
            0.0,
            0.0,
            0.0,
        )
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-scale3d>
    fn Scale3d(
        &self,
        scale: f64,
        originX: f64,
        originY: f64,
        originZ: f64,
        can_gc: CanGc,
    ) -> DomRoot<DOMMatrix> {
        DOMMatrix::from_readonly(&self.global(), self, can_gc)
            .Scale3dSelf(scale, originX, originY, originZ)
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-rotate>
    fn Rotate(
        &self,
        rotX: f64,
        rotY: Option<f64>,
        rotZ: Option<f64>,
        can_gc: CanGc,
    ) -> DomRoot<DOMMatrix> {
        DOMMatrix::from_readonly(&self.global(), self, can_gc).RotateSelf(rotX, rotY, rotZ)
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-rotatefromvector>
    fn RotateFromVector(&self, x: f64, y: f64, can_gc: CanGc) -> DomRoot<DOMMatrix> {
        DOMMatrix::from_readonly(&self.global(), self, can_gc).RotateFromVectorSelf(x, y)
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-rotateaxisangle>
    fn RotateAxisAngle(
        &self,
        x: f64,
        y: f64,
        z: f64,
        angle: f64,
        can_gc: CanGc,
    ) -> DomRoot<DOMMatrix> {
        DOMMatrix::from_readonly(&self.global(), self, can_gc).RotateAxisAngleSelf(x, y, z, angle)
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-skewx>
    fn SkewX(&self, sx: f64, can_gc: CanGc) -> DomRoot<DOMMatrix> {
        DOMMatrix::from_readonly(&self.global(), self, can_gc).SkewXSelf(sx)
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-skewy>
    fn SkewY(&self, sy: f64, can_gc: CanGc) -> DomRoot<DOMMatrix> {
        DOMMatrix::from_readonly(&self.global(), self, can_gc).SkewYSelf(sy)
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-multiply>
    fn Multiply(&self, other: &DOMMatrixInit, can_gc: CanGc) -> Fallible<DomRoot<DOMMatrix>> {
        DOMMatrix::from_readonly(&self.global(), self, can_gc).MultiplySelf(other)
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-flipx>
    fn FlipX(&self, can_gc: CanGc) -> DomRoot<DOMMatrix> {
        let is2D = self.is2D.get();
        let flip = Transform3D::new(
            -1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        );
        let matrix = flip.then(&self.matrix.borrow());
        DOMMatrix::new(&self.global(), is2D, matrix, can_gc)
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-flipy>
    fn FlipY(&self, can_gc: CanGc) -> DomRoot<DOMMatrix> {
        let is2D = self.is2D.get();
        let flip = Transform3D::new(
            1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        );
        let matrix = flip.then(&self.matrix.borrow());
        DOMMatrix::new(&self.global(), is2D, matrix, can_gc)
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-inverse>
    fn Inverse(&self, can_gc: CanGc) -> DomRoot<DOMMatrix> {
        DOMMatrix::from_readonly(&self.global(), self, can_gc).InvertSelf()
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-transformpoint>
    fn TransformPoint(&self, point: &DOMPointInit, can_gc: CanGc) -> DomRoot<DOMPoint> {
        // Euclid always normalizes the homogeneous coordinate which is usually the right
        // thing but may (?) not be compliant with the CSS matrix spec (or at least is
        // probably not the behavior web authors will expect even if it is mathematically
        // correct in the context of geometry computations).
        // Since this is the only place where this is needed, better implement it here
        // than in euclid (which does not have a notion of 4d points).
        let mat = self.matrix.borrow();
        let x = point.x * mat.m11 + point.y * mat.m21 + point.z * mat.m31 + point.w * mat.m41;
        let y = point.x * mat.m12 + point.y * mat.m22 + point.z * mat.m32 + point.w * mat.m42;
        let z = point.x * mat.m13 + point.y * mat.m23 + point.z * mat.m33 + point.w * mat.m43;
        let w = point.x * mat.m14 + point.y * mat.m24 + point.z * mat.m34 + point.w * mat.m44;

        DOMPoint::new(&self.global(), x, y, z, w, can_gc)
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-tofloat32array>
    fn ToFloat32Array(&self, cx: JSContext, can_gc: CanGc) -> RootedTraceableBox<HeapFloat32Array> {
        let vec: Vec<f32> = self
            .matrix
            .borrow()
            .to_array()
            .iter()
            .map(|&x| x as f32)
            .collect();
        rooted!(in (*cx) let mut array = ptr::null_mut::<JSObject>());
        create_buffer_source(cx, &vec, array.handle_mut(), can_gc)
            .expect("Converting matrix to float32 array should never fail")
    }

    /// <https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-tofloat64array>
    fn ToFloat64Array(&self, cx: JSContext, can_gc: CanGc) -> RootedTraceableBox<HeapFloat64Array> {
        rooted!(in (*cx) let mut array = ptr::null_mut::<JSObject>());
        create_buffer_source(
            cx,
            &self.matrix.borrow().to_array(),
            array.handle_mut(),
            can_gc,
        )
        .expect("Converting matrix to float64 array should never fail")
    }

    // https://drafts.fxtf.org/geometry/#dommatrixreadonly-stringification-behavior
    #[expect(unsafe_code)]
    fn Stringifier(&self) -> Fallible<DOMString> {
        // Step 1. If one or more of m11 element through m44 element are a non-finite value,
        // then throw an "InvalidStateError" DOMException.
        let mat = self.matrix.borrow();
        if !mat.m11.is_finite() ||
            !mat.m12.is_finite() ||
            !mat.m13.is_finite() ||
            !mat.m14.is_finite() ||
            !mat.m21.is_finite() ||
            !mat.m22.is_finite() ||
            !mat.m23.is_finite() ||
            !mat.m24.is_finite() ||
            !mat.m31.is_finite() ||
            !mat.m32.is_finite() ||
            !mat.m33.is_finite() ||
            !mat.m34.is_finite() ||
            !mat.m41.is_finite() ||
            !mat.m42.is_finite() ||
            !mat.m43.is_finite() ||
            !mat.m44.is_finite()
        {
            return Err(error::Error::InvalidState(None));
        }

        let cx = GlobalScope::get_cx();
        let to_string = |f: f64| {
            let value = jsval::DoubleValue(f);

            unsafe {
                rooted!(in(*cx) let mut rooted_value = value);
                let serialization = std::ptr::NonNull::new(ToString(*cx, rooted_value.handle()))
                    .expect("Pointer cannot be null");
                jsstr_to_string(*cx, serialization)
            }
        };

        // Step 2. Let string be the empty string.
        // Step 3. If is 2D is true, then:
        let string = if self.is2D() {
            // Step 3.1 Append "matrix(" to string.
            // Step 3.2 Append ! ToString(m11 element) to string.
            // Step 3.3 Append ", " to string.
            // Step 3.4 Append ! ToString(m12 element) to string.
            // Step 3.5 Append ", " to string.
            // Step 3.6 Append ! ToString(m21 element) to string.
            // Step 3.7 Append ", " to string.
            // Step 3.8 Append ! ToString(m22 element) to string.
            // Step 3.9 Append ", " to string.
            // Step 3.10 Append ! ToString(m41 element) to string.
            // Step 3.11 Append ", " to string.
            // Step 3.12 Append ! ToString(m42 element) to string.
            // Step 3.13 Append ")" to string.
            format!(
                "matrix({}, {}, {}, {}, {}, {})",
                to_string(mat.m11),
                to_string(mat.m12),
                to_string(mat.m21),
                to_string(mat.m22),
                to_string(mat.m41),
                to_string(mat.m42)
            )
            .into()
        }
        // Step 4. Otherwise:
        else {
            // Step 4.1 Append "matrix3d(" to string.
            // Step 4.2 Append ! ToString(m11 element) to string.
            // Step 4.3 Append ", " to string.
            // Step 4.4 Append ! ToString(m12 element) to string.
            // Step 4.5 Append ", " to string.
            // Step 4.6 Append ! ToString(m13 element) to string.
            // Step 4.7 Append ", " to string.
            // Step 4.8 Append ! ToString(m14 element) to string.
            // Step 4.9 Append ", " to string.
            // Step 4.10 Append ! ToString(m21 element) to string.
            // Step 4.11 Append ", " to string.
            // Step 4.12 Append ! ToString(m22 element) to string.
            // Step 4.13 Append ", " to string.
            // Step 4.14 Append ! ToString(m23 element) to string.
            // Step 4.15 Append ", " to string.
            // Step 4.16 Append ! ToString(m24 element) to string.
            // Step 4.17 Append ", " to string.
            // Step 4.18 Append ! ToString(m41 element) to string.
            // Step 4.19 Append ", " to string.
            // Step 4.20 Append ! ToString(m42 element) to string.
            // Step 4.21 Append ", " to string.
            // Step 4.22 Append ! ToString(m43 element) to string.
            // Step 4.23 Append ", " to string.
            // Step 4.24 Append ! ToString(m44 element) to string.
            // Step 4.25 Append ")" to string.

            // NOTE: The spec is wrong and missing the m3* elements.
            // (https://github.com/w3c/fxtf-drafts/issues/574)
            format!(
                "matrix3d({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {})",
                to_string(mat.m11),
                to_string(mat.m12),
                to_string(mat.m13),
                to_string(mat.m14),
                to_string(mat.m21),
                to_string(mat.m22),
                to_string(mat.m23),
                to_string(mat.m24),
                to_string(mat.m31),
                to_string(mat.m32),
                to_string(mat.m33),
                to_string(mat.m34),
                to_string(mat.m41),
                to_string(mat.m42),
                to_string(mat.m43),
                to_string(mat.m44)
            )
            .into()
        };

        Ok(string)
    }
}

impl Serializable for DOMMatrixReadOnly {
    type Index = DomMatrixIndex;
    type Data = DomMatrix;

    fn serialize(&self) -> Result<(DomMatrixId, Self::Data), ()> {
        let serialized = if self.is2D() {
            DomMatrix {
                matrix: Transform3D::new(
                    self.M11(),
                    self.M12(),
                    f64::NAN,
                    f64::NAN,
                    self.M21(),
                    self.M22(),
                    f64::NAN,
                    f64::NAN,
                    f64::NAN,
                    f64::NAN,
                    f64::NAN,
                    f64::NAN,
                    self.M41(),
                    self.M42(),
                    f64::NAN,
                    f64::NAN,
                ),
                is_2d: true,
            }
        } else {
            DomMatrix {
                matrix: *self.matrix(),
                is_2d: false,
            }
        };
        Ok((DomMatrixId::new(), serialized))
    }

    fn deserialize(
        owner: &GlobalScope,
        serialized: Self::Data,
        can_gc: CanGc,
    ) -> Result<DomRoot<Self>, ()>
    where
        Self: Sized,
    {
        if serialized.is_2d {
            Ok(Self::new(
                owner,
                true,
                Transform3D::new(
                    serialized.matrix.m11,
                    serialized.matrix.m12,
                    0.0,
                    0.0,
                    serialized.matrix.m21,
                    serialized.matrix.m22,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                    0.0,
                    serialized.matrix.m41,
                    serialized.matrix.m42,
                    0.0,
                    1.0,
                ),
                can_gc,
            ))
        } else {
            Ok(Self::new(owner, false, serialized.matrix, can_gc))
        }
    }

    fn serialized_storage<'a>(
        data: StructuredData<'a, '_>,
    ) -> &'a mut Option<FxHashMap<DomMatrixId, Self::Data>> {
        match data {
            StructuredData::Reader(reader) => &mut reader.matrices,
            StructuredData::Writer(writer) => &mut writer.matrices,
        }
    }
}

// https://drafts.fxtf.org/geometry-1/#dom-dommatrixreadonly-dommatrixreadonly-numbersequence
pub(crate) fn entries_to_matrix(entries: &[f64]) -> Fallible<(bool, Transform3D<f64>)> {
    if let Ok(array) = entries.try_into() {
        Ok((true, Transform2D::from_array(array).to_3d()))
    } else if let Ok(array) = entries.try_into() {
        Ok((false, Transform3D::from_array(array)))
    } else {
        let err_msg = format!("Expected 6 or 16 entries, but found {}.", entries.len());
        Err(error::Error::Type(err_msg.to_owned()))
    }
}

/// <https://drafts.fxtf.org/geometry-1/#matrix-validate-and-fixup-2d>
fn validate_and_fixup_2d(dict: &DOMMatrix2DInit) -> Fallible<Transform2D<f64>> {
    // <https://tc39.es/ecma262/#sec-numeric-types-number-sameValueZero>
    let same_value_zero = |x: f64, y: f64| -> bool { x.is_nan() && y.is_nan() || x == y };

    // Step 1. If if at least one of the following conditions are true for dict,
    // then throw a TypeError exception and abort these steps.
    if dict.a.is_some() &&
        dict.m11.is_some() &&
        !same_value_zero(dict.a.unwrap(), dict.m11.unwrap()) ||
        dict.b.is_some() &&
            dict.m12.is_some() &&
            !same_value_zero(dict.b.unwrap(), dict.m12.unwrap()) ||
        dict.c.is_some() &&
            dict.m21.is_some() &&
            !same_value_zero(dict.c.unwrap(), dict.m21.unwrap()) ||
        dict.d.is_some() &&
            dict.m22.is_some() &&
            !same_value_zero(dict.d.unwrap(), dict.m22.unwrap()) ||
        dict.e.is_some() &&
            dict.m41.is_some() &&
            !same_value_zero(dict.e.unwrap(), dict.m41.unwrap()) ||
        dict.f.is_some() &&
            dict.m42.is_some() &&
            !same_value_zero(dict.f.unwrap(), dict.m42.unwrap())
    {
        return Err(error::Error::Type(
            "Property mismatch on matrix initialization.".to_owned(),
        ));
    }

    // Step 2. If m11 is not present then set it to the value of member a,
    // or value 1 if a is also not present.
    let m11 = dict.m11.unwrap_or(dict.a.unwrap_or(1.0));

    // Step 3. If m12 is not present then set it to the value of member b,
    // or value 0 if b is also not present.
    let m12 = dict.m12.unwrap_or(dict.b.unwrap_or(0.0));

    // Step 4. If m21 is not present then set it to the value of member c,
    // or value 0 if c is also not present.
    let m21 = dict.m21.unwrap_or(dict.c.unwrap_or(0.0));

    // Step 5. If m22 is not present then set it to the value of member d,
    // or value 1 if d is also not present.
    let m22 = dict.m22.unwrap_or(dict.d.unwrap_or(1.0));

    // Step 6. If m41 is not present then set it to the value of member e,
    // or value 0 if e is also not present.
    let m41 = dict.m41.unwrap_or(dict.e.unwrap_or(0.0));

    // Step 7. If m42 is not present then set it to the value of member f,
    // or value 0 if f is also not present.
    let m42 = dict.m42.unwrap_or(dict.f.unwrap_or(0.0));

    Ok(Transform2D::new(m11, m12, m21, m22, m41, m42))
}

/// <https://drafts.fxtf.org/geometry-1/#matrix-validate-and-fixup>
fn validate_and_fixup(dict: &DOMMatrixInit) -> Fallible<(bool, Transform3D<f64>)> {
    // Step 1. Validate and fixup (2D) dict.
    let transform2d = validate_and_fixup_2d(&dict.parent)?;

    // Step 2. If is2D is true and: at least one of m13, m14, m23, m24, m31,
    // m32, m34, m43 are present with a value other than 0 or -0, or at least
    // one of m33, m44 are present with a value other than 1, then throw
    // a TypeError exception and abort these steps.
    if dict.is2D == Some(true) &&
        (dict.m13 != 0.0 ||
            dict.m14 != 0.0 ||
            dict.m23 != 0.0 ||
            dict.m24 != 0.0 ||
            dict.m31 != 0.0 ||
            dict.m32 != 0.0 ||
            dict.m34 != 0.0 ||
            dict.m43 != 0.0 ||
            dict.m33 != 1.0 ||
            dict.m44 != 1.0)
    {
        return Err(error::Error::Type(
            "The is2D member is set to true but the input matrix is a 3d matrix.".to_owned(),
        ));
    }

    let mut is_2d = dict.is2D;

    // Step 3. If is2D is not present and at least one of m13, m14, m23, m24,
    // m31, m32, m34, m43 are present with a value other than 0 or -0, or at
    // least one of m33, m44 are present with a value other than 1, set is2D
    // to false.
    if is_2d.is_none() &&
        (dict.m13 != 0.0 ||
            dict.m14 != 0.0 ||
            dict.m23 != 0.0 ||
            dict.m24 != 0.0 ||
            dict.m31 != 0.0 ||
            dict.m32 != 0.0 ||
            dict.m34 != 0.0 ||
            dict.m43 != 0.0 ||
            dict.m33 != 1.0 ||
            dict.m44 != 1.0)
    {
        is_2d = Some(false);
    }

    // Step 4. If is2D is still not present, set it to true.
    let is_2d = is_2d.unwrap_or(true);

    let mut transform = transform2d.to_3d();
    transform.m13 = dict.m13;
    transform.m14 = dict.m14;
    transform.m23 = dict.m23;
    transform.m24 = dict.m24;
    transform.m31 = dict.m31;
    transform.m32 = dict.m32;
    transform.m33 = dict.m33;
    transform.m34 = dict.m34;
    transform.m43 = dict.m43;
    transform.m44 = dict.m44;

    Ok((is_2d, transform))
}

/// <https://drafts.fxtf.org/geometry-1/#create-a-dommatrixreadonly-from-the-2d-dictionary>
pub(crate) fn dommatrix2dinit_to_matrix(dict: &DOMMatrix2DInit) -> Fallible<Transform2D<f64>> {
    // Step 1. Validate and fixup (2D) other.
    // Step 2. Return the result of invoking create a 2d matrix of type
    // DOMMatrixReadOnly or DOMMatrix as appropriate, with a sequence of
    // numbers, the values being the 6 elements m11, m12, m21, m22, m41 and m42
    // of other in the given order.
    validate_and_fixup_2d(dict)
}

/// <https://drafts.fxtf.org/geometry-1/#create-a-dommatrix-from-the-dictionary>
pub(crate) fn dommatrixinit_to_matrix(dict: &DOMMatrixInit) -> Fallible<(bool, Transform3D<f64>)> {
    // Step 1. Validate and fixup other.
    // Step 2. Return the result of invoking create a 3d matrix of type
    // DOMMatrixReadOnly or DOMMatrix as appropriate, with a sequence of
    // numbers, the values being the 16 elements m11, m12, m13, ..., m44
    // of other in the given order.
    validate_and_fixup(dict)
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

pub(crate) fn transform_to_matrix(value: String) -> Fallible<(bool, Transform3D<f64>)> {
    use style::properties::longhands::transform;

    let mut input = ParserInput::new(&value);
    let mut parser = Parser::new(&mut input);
    let url_data = Url::parse("about:blank").unwrap().into();
    let context = ParserContext::new(
        ::style::stylesheets::Origin::Author,
        &url_data,
        Some(::style::stylesheets::CssRuleType::Style),
        ::style_traits::ParsingMode::DEFAULT,
        ::style::context::QuirksMode::NoQuirks,
        /* namespaces = */ Default::default(),
        None,
        None,
    );

    let transform = match parser.parse_entirely(|t| transform::parse(&context, t)) {
        Ok(result) => result,
        Err(..) => return Err(error::Error::Syntax(None)),
    };

    let (m, is_3d) = match transform.to_transform_3d_matrix_f64(None) {
        Ok(result) => result,
        Err(..) => return Err(error::Error::Syntax(None)),
    };

    Ok((!is_3d, m))
}
