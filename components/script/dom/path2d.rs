/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use canvas_traits::canvas::PathSegment;
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
use crate::svgpath::PathParser;

#[dom_struct]
pub(crate) struct Path2D {
    reflector_: Reflector,
    #[no_trace]
    path: RefCell<Vec<PathSegment>>,
}

impl Path2D {
    pub(crate) fn new() -> Path2D {
        Self {
            reflector_: Reflector::new(),
            path: RefCell::new(vec![]),
        }
    }
    pub(crate) fn new_with_path(other: &Path2D) -> Path2D {
        Self {
            reflector_: Reflector::new(),
            path: other.path.clone(),
        }
    }
    pub(crate) fn new_with_str(path: &str) -> Path2D {
        let mut path_segments = Vec::new();

        for segment in PathParser::new(path) {
            if let Ok(segment) = segment {
                path_segments.push(segment);
            } else {
                break;
            }
        }

        Self {
            reflector_: Reflector::new(),
            path: RefCell::new(path_segments),
        }
    }
    pub(crate) fn push(&self, seg: PathSegment) {
        self.path.borrow_mut().push(seg);
    }
    pub(crate) fn segments(&self) -> Vec<PathSegment> {
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
        self.push(PathSegment::ClosePath);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-moveto>
    fn MoveTo(&self, x: f64, y: f64) {
        // Step 1. If either of the arguments are infinite or NaN, then return.
        if !(x.is_finite() && y.is_finite()) {
            return;
        }

        // Step 2. Create a new subpath with the specified point as its first (and only) point.
        self.push(PathSegment::MoveTo {
            x: x as f32,
            y: y as f32,
        });
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-lineto>
    fn LineTo(&self, x: f64, y: f64) {
        // Step 1. If either of the arguments are infinite or NaN, then return.
        if !(x.is_finite() && y.is_finite()) {
            return;
        }

        self.push(PathSegment::LineTo {
            x: x as f32,
            y: y as f32,
        });
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-quadraticcurveto>
    fn QuadraticCurveTo(&self, cpx: f64, cpy: f64, x: f64, y: f64) {
        // Step 1. If any of the arguments are infinite or NaN, then return.
        if !(cpx.is_finite() && cpy.is_finite() && x.is_finite() && y.is_finite()) {
            return;
        }

        self.push(PathSegment::Quadratic {
            cpx: cpx as f32,
            cpy: cpy as f32,
            x: x as f32,
            y: y as f32,
        });
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-beziercurveto>
    fn BezierCurveTo(&self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, x: f64, y: f64) {
        // Step 1. If any of the arguments are infinite or NaN, then return.
        if !(cp1x.is_finite() &&
            cp1y.is_finite() &&
            cp2x.is_finite() &&
            cp2y.is_finite() &&
            x.is_finite() &&
            y.is_finite())
        {
            return;
        }

        self.push(PathSegment::Bezier {
            cp1x: cp1x as f32,
            cp1y: cp1y as f32,
            cp2x: cp2x as f32,
            cp2y: cp2y as f32,
            x: x as f32,
            y: y as f32,
        });
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-arcto>
    fn ArcTo(&self, x1: f64, y1: f64, x2: f64, y2: f64, r: f64) -> Fallible<()> {
        // Step 1. If any of the arguments are infinite or NaN, then return.
        if !(x1.is_finite() && y1.is_finite() && x2.is_finite() && y2.is_finite() && r.is_finite())
        {
            return Ok(());
        }

        // Step 3. If radius is negative, then throw an "IndexSizeError" DOMException.
        if r < 0.0 {
            return Err(Error::IndexSize);
        }

        self.push(PathSegment::ArcTo {
            cp1x: x1 as f32,
            cp1y: y1 as f32,
            cp2x: x2 as f32,
            cp2y: y2 as f32,
            radius: r as f32,
        });
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-rect>
    fn Rect(&self, x: f64, y: f64, w: f64, h: f64) {
        // Step 1. If any of the arguments are infinite or NaN, then return.
        if !(x.is_finite() && y.is_finite() && w.is_finite() && h.is_finite()) {
            return;
        }
        // Step 2. Create a new subpath containing just the four points
        // (x, y), (x+w, y), (x+w, y+h), (x, y+h), in that order,
        // with those four points connected by straight lines.
        self.push(PathSegment::MoveTo {
            x: x as f32,
            y: y as f32,
        });
        self.push(PathSegment::LineTo {
            x: (x + w) as f32,
            y: y as f32,
        });
        self.push(PathSegment::LineTo {
            x: (x + w) as f32,
            y: (y + h) as f32,
        });
        self.push(PathSegment::LineTo {
            x: x as f32,
            y: (y + h) as f32,
        });
        // Step 3. Mark the subpath as closed.
        self.push(PathSegment::ClosePath);

        // Step 4. Create a new subpath with the point (x, y) as the only point in the subpath.
        self.push(PathSegment::MoveTo {
            x: x as f32,
            y: y as f32,
        });
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-arc>
    fn Arc(
        &self,
        x: f64,
        y: f64,
        r: f64,
        start: f64,
        end: f64,
        anticlockwise: bool,
    ) -> Fallible<()> {
        // Step 1. If any of the arguments are infinite or NaN, then return.
        if !(x.is_finite() &&
            y.is_finite() &&
            r.is_finite() &&
            start.is_finite() &&
            end.is_finite())
        {
            return Ok(());
        }

        // Step 2. If either radiusX or radiusY are negative, then throw an "IndexSizeError" DOMException.
        if r < 0.0 {
            return Err(Error::IndexSize);
        }

        self.push(PathSegment::Ellipse {
            x: x as f32,
            y: y as f32,
            radius_x: r as f32,
            radius_y: r as f32,
            rotation: 0.,
            start_angle: start as f32,
            end_angle: end as f32,
            anticlockwise,
        });
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-ellipse>
    fn Ellipse(
        &self,
        x: f64,
        y: f64,
        rx: f64,
        ry: f64,
        rotation: f64,
        start: f64,
        end: f64,
        anticlockwise: bool,
    ) -> Fallible<()> {
        // Step 1. If any of the arguments are infinite or NaN, then return.
        if !(x.is_finite() &&
            y.is_finite() &&
            rx.is_finite() &&
            ry.is_finite() &&
            rotation.is_finite() &&
            start.is_finite() &&
            end.is_finite())
        {
            return Ok(());
        }

        // Step 2. If either radiusX or radiusY are negative, then throw an "IndexSizeError" DOMException.
        if rx < 0.0 || ry < 0.0 {
            return Err(Error::IndexSize);
        }

        self.push(PathSegment::Ellipse {
            x: x as f32,
            y: y as f32,
            radius_x: rx as f32,
            radius_y: ry as f32,
            rotation: rotation as f32,
            start_angle: start as f32,
            end_angle: end as f32,
            anticlockwise,
        });
        Ok(())
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
