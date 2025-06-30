/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use canvas_traits::canvas::Path;
use dom_struct::dom_struct;
use euclid::default::Point2D;
use js::rust::HandleObject;
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

    fn has_segments(&self) -> bool {
        !self.path.borrow().is_empty()
    }

    fn add_path_(&self, other: &Path2D, transform: &DOMMatrix2DInit) -> ErrorResult {
        // Step 1. If the Path@D object path has no subpaths, then return.
        if !other.has_segments() {
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
            !matrix.m41.is_finite() ||
            !matrix.m42.is_finite()
        {
            return Ok(());
        }

        // Step 4. Create a copy of all the subpaths in path. Let c be this copy.
        let mut c = other.segments();

        // Step 5. Transform all the coordinates and lines in c by the transform matrix matrix.
        let apply_matrix_transform = |x: f32, y: f32| -> Fallible<Point2D<f64>> {
            matrix
                .transform_point2d(Point2D::new(x.into(), y.into()))
                .ok_or(Error::InvalidState)
        };

        for segment in c.iter_mut() {
            match *segment {
                PathSegment::ClosePath => todo!(),
                PathSegment::MoveTo { x, y } => {
                    let transformed_point = apply_matrix_transform(x, y)?;
                    *segment = PathSegment::MoveTo {
                        x: transformed_point.x as f32,
                        y: transformed_point.y as f32,
                    };
                },
                PathSegment::LineTo { x, y } => {
                    let transformed_point = apply_matrix_transform(x, y)?;
                    *segment = PathSegment::LineTo {
                        x: transformed_point.x as f32,
                        y: transformed_point.y as f32,
                    }
                },
                PathSegment::Quadratic { cpx, cpy, x, y } => {
                    let transformed_cp = apply_matrix_transform(cpx, cpy)?;
                    let transformed_point = apply_matrix_transform(x, y)?;
                    *segment = PathSegment::Quadratic {
                        cpx: transformed_cp.x as f32,
                        cpy: transformed_cp.y as f32,
                        x: transformed_point.x as f32,
                        y: transformed_point.y as f32,
                    }
                },
                PathSegment::Bezier {
                    cp1x,
                    cp1y,
                    cp2x,
                    cp2y,
                    x,
                    y,
                } => {
                    let transformed_cp1 = apply_matrix_transform(cp1x, cp1y)?;
                    let transformed_cp2 = apply_matrix_transform(cp2x, cp2y)?;
                    let transformed_point = apply_matrix_transform(x, y)?;
                    *segment = PathSegment::Bezier {
                        cp1x: transformed_cp1.x as f32,
                        cp1y: transformed_cp1.y as f32,
                        cp2x: transformed_cp2.x as f32,
                        cp2y: transformed_cp2.y as f32,
                        x: transformed_point.x as f32,
                        y: transformed_point.y as f32,
                    }
                },
                PathSegment::ArcTo {
                    cp1x,
                    cp1y,
                    cp2x,
                    cp2y,
                    radius,
                } => {
                    let transformed_cp1 = apply_matrix_transform(cp1x, cp1y)?;
                    let transformed_cp2 = apply_matrix_transform(cp2x, cp2y)?;
                    *segment = PathSegment::ArcTo {
                        cp1x: transformed_cp1.x as f32,
                        cp1y: transformed_cp1.y as f32,
                        cp2x: transformed_cp2.x as f32,
                        cp2y: transformed_cp2.y as f32,
                        radius,
                    }
                },
                PathSegment::Ellipse {
                    x,
                    y,
                    radius_x,
                    radius_y,
                    rotation,
                    start_angle,
                    end_angle,
                    anticlockwise,
                } => {
                    let transformed_point = apply_matrix_transform(x, y)?;
                    *segment = PathSegment::Ellipse {
                        x: transformed_point.x as f32,
                        y: transformed_point.y as f32,
                        radius_x,
                        radius_y,
                        rotation,
                        start_angle,
                        end_angle,
                        anticlockwise,
                    }
                },
                PathSegment::SvgArc {
                    radius_x,
                    radius_y,
                    rotation,
                    large_arc,
                    sweep,
                    x,
                    y,
                } => {
                    let transformed_point = apply_matrix_transform(x, y)?;
                    *segment = PathSegment::SvgArc {
                        x: transformed_point.x as f32,
                        y: transformed_point.y as f32,
                        radius_x,
                        radius_y,
                        rotation,
                        large_arc,
                        sweep,
                    }
                },
            }
        }

        // Step 6. Let (x, y) be the last point in the last subpath of c

        // Step 7. Add all the subpaths in c to a.
        if std::ptr::eq(&self.path, &other.path) {
            // Note: this is not part of the spec, but it is a workaround to
            // avoids borrow conflict when path is same as other.path
            self.path.borrow_mut().extend_from_within(..);
        } else {
            let mut dest = self.path.borrow_mut();
            dest.extend(other.path.borrow().iter().copied());
        }

        Ok(())
    }
}

impl Path2DMethods<crate::DomTypeHolder> for Path2D {
    /// <https://html.spec.whatwg.org/multipage/#dom-path2d-addpath>
    fn AddPath(&self, other: &Path2D, transform: &DOMMatrix2DInit) -> ErrorResult {
        self.add_path_(other, transform)
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
