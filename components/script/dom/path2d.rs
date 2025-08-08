/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use canvas_traits::canvas::Path;
use dom_struct::dom_struct;
use euclid::default::Point2D;
use js::rust::HandleObject;
use kurbo::{PathEl, Point};
use script_bindings::codegen::GenericBindings::DOMMatrixBinding::DOMMatrix2DInit;
use script_bindings::error::ErrorResult;
use script_bindings::str::DOMString;

use crate::dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::Path2DMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::dommatrixreadonly::dommatrix2dinit_to_matrix;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct Path2D {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "Defined in kurbo."]
    #[no_trace]
    path: RefCell<Path>,
}

impl Path2D {
    pub(crate) fn new() -> Path2D {
        Self {
            reflector_: Reflector::new(),
            path: RefCell::new(Path::new()),
        }
    }
    pub(crate) fn new_with_path(other: &Path2D) -> Path2D {
        Self {
            reflector_: Reflector::new(),
            path: other.path.clone(),
        }
    }

    pub(crate) fn new_with_str(path: &str) -> Path2D {
        Self {
            reflector_: Reflector::new(),
            path: RefCell::new(Path::from_svg(path)),
        }
    }

    pub(crate) fn segments(&self) -> Path {
        self.path.borrow().clone()
    }

    pub(crate) fn is_path_empty(&self) -> bool {
        self.path.borrow().0.is_empty()
    }

    fn add_path(&self, other: &Path2D, transform: &DOMMatrix2DInit) -> ErrorResult {
        // Step 1. If the Path2D object path has no subpaths, then return.
        if other.is_path_empty() {
            return Ok(());
        }

        // Step 2 Let matrix be the result of creating a DOMMatrix from the 2D dictionary transform.
        let matrix = dommatrix2dinit_to_matrix(transform)?;

        // Step 3. If one or more of matrix's m11 element, m12 element, m21
        // element, m22 element, m41 element, or m42 element are infinite or
        // NaN, then return.
        if !matrix.m11.is_finite() ||
            !matrix.m12.is_finite() ||
            !matrix.m21.is_finite() ||
            !matrix.m22.is_finite() ||
            !matrix.m31.is_finite() ||
            !matrix.m32.is_finite()
        {
            return Ok(());
        }

        // Step 4. Create a copy of all the subpaths in path. Let c be this copy.
        let mut c = other.segments();

        // Step 5. Transform all the coordinates and lines in c by the transform matrix `matrix`.
        let apply_matrix_transform =
            |point: Point2D<f64>| -> Point2D<f64> { matrix.transform_point(point) };

        for segment in c.0.elements_mut() {
            match *segment {
                PathEl::MoveTo(point) => {
                    let transformed_point = apply_matrix_transform(Point2D::new(point.x, point.y));
                    *segment = PathEl::MoveTo(Point::new(transformed_point.x, transformed_point.y));
                },
                PathEl::LineTo(point) => {
                    let transformed_point = apply_matrix_transform(Point2D::new(point.x, point.y));
                    *segment = PathEl::LineTo(Point::new(transformed_point.x, transformed_point.y));
                },
                PathEl::QuadTo(point, point_1) => {
                    let transformed_point = apply_matrix_transform(Point2D::new(point.x, point.y));
                    let transformed_point_1 =
                        apply_matrix_transform(Point2D::new(point_1.x, point_1.y));
                    *segment = PathEl::QuadTo(
                        Point::new(transformed_point.x, transformed_point.y),
                        Point::new(transformed_point_1.x, transformed_point_1.y),
                    );
                },
                PathEl::CurveTo(point, point_1, point_2) => {
                    let transformed_point = apply_matrix_transform(Point2D::new(point.x, point.y));
                    let transformed_point_1 =
                        apply_matrix_transform(Point2D::new(point_1.x, point_1.y));
                    let transformed_point_2 =
                        apply_matrix_transform(Point2D::new(point_2.x, point_2.y));
                    *segment = PathEl::CurveTo(
                        Point::new(transformed_point.x, transformed_point.y),
                        Point::new(transformed_point_1.x, transformed_point_1.y),
                        Point::new(transformed_point_2.x, transformed_point_2.y),
                    );
                },
                PathEl::ClosePath => {},
            }
        }

        // Step 6. Let (x, y) be the last point in the last subpath of c
        let last_point = self.path.borrow_mut().last_point();

        // Step 7. Add all the subpaths in c to a.
        self.path.borrow_mut().0.extend(c.0);

        // Step 8. Create a new subpath in `a` with (x, y) as the only point in the subpath.
        if let Some(last_point) = last_point {
            self.path.borrow_mut().move_to(last_point.x, last_point.y);
        }

        Ok(())
    }
}

impl Path2DMethods<crate::DomTypeHolder> for Path2D {
    /// <https://html.spec.whatwg.org/multipage/#dom-path2d-addpath>
    fn AddPath(&self, other: &Path2D, transform: &DOMMatrix2DInit) -> ErrorResult {
        self.add_path(other, transform)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-closepath>
    fn ClosePath(&self) {
        self.path.borrow_mut().close_path();
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-moveto>
    fn MoveTo(&self, x: f64, y: f64) {
        self.path.borrow_mut().move_to(x, y);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-lineto>
    fn LineTo(&self, x: f64, y: f64) {
        self.path.borrow_mut().line_to(x, y);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-quadraticcurveto>
    fn QuadraticCurveTo(&self, cpx: f64, cpy: f64, x: f64, y: f64) {
        self.path.borrow_mut().quadratic_curve_to(cpx, cpy, x, y);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-beziercurveto>
    fn BezierCurveTo(&self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, x: f64, y: f64) {
        self.path
            .borrow_mut()
            .bezier_curve_to(cp1x, cp1y, cp2x, cp2y, x, y);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-arcto>
    fn ArcTo(&self, x1: f64, y1: f64, x2: f64, y2: f64, radius: f64) -> Fallible<()> {
        self.path
            .borrow_mut()
            .arc_to(x1, y1, x2, y2, radius)
            .map_err(|_| Error::IndexSize)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-rect>
    fn Rect(&self, x: f64, y: f64, w: f64, h: f64) {
        self.path.borrow_mut().rect(x, y, w, h);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-arc>
    fn Arc(
        &self,
        x: f64,
        y: f64,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        counterclockwise: bool,
    ) -> Fallible<()> {
        self.path
            .borrow_mut()
            .arc(x, y, radius, start_angle, end_angle, counterclockwise)
            .map_err(|_| Error::IndexSize)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-ellipse>
    fn Ellipse(
        &self,
        x: f64,
        y: f64,
        radius_x: f64,
        radius_y: f64,
        rotation_angle: f64,
        start_angle: f64,
        end_angle: f64,
        counterclockwise: bool,
    ) -> Fallible<()> {
        self.path
            .borrow_mut()
            .ellipse(
                x,
                y,
                radius_x,
                radius_y,
                rotation_angle,
                start_angle,
                end_angle,
                counterclockwise,
            )
            .map_err(|_| Error::IndexSize)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-path2d-dev>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<Path2D> {
        reflect_dom_object_with_proto(Box::new(Self::new()), global, proto, can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-path2d-dev>
    fn Constructor_(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        other: &Path2D,
    ) -> DomRoot<Path2D> {
        reflect_dom_object_with_proto(Box::new(Self::new_with_path(other)), global, proto, can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-path2d-dev>
    fn Constructor__(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        path_string: DOMString,
    ) -> DomRoot<Path2D> {
        reflect_dom_object_with_proto(
            Box::new(Self::new_with_str(path_string.str())),
            global,
            proto,
            can_gc,
        )
    }
}
