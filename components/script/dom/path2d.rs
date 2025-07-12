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

    fn last(&self) -> Option<PathSegment> {
        self.path.borrow().last().copied()
    }

    pub(crate) fn is_path_empty(&self) -> bool {
        self.path.borrow().is_empty()
    }

    fn compute_final_arc_to_point(path: &[PathSegment]) -> Option<Point2D<f32>> {
        fn compute_arc_to_points(
            cp0: Point2D<f32>,
            cp1: Point2D<f32>,
            cp2: Point2D<f32>,
            radius: f32,
        ) -> Point2D<f32> {
            if (cp0.x == cp1.x && cp0.y == cp1.y) || cp1 == cp2 || radius == 0.0 {
                return cp1;
            }

            // if all three control points lie on a single straight line,
            // connect the first two by a straight line
            let direction = (cp2.x - cp1.x) * (cp0.y - cp1.y) + (cp2.y - cp1.y) * (cp1.x - cp0.x);
            if direction == 0.0 {
                return cp1;
            }

            // otherwise, draw the Arc
            let a2 = (cp0.x - cp1.x).powi(2) + (cp0.y - cp1.y).powi(2);
            let b2 = (cp1.x - cp2.x).powi(2) + (cp1.y - cp2.y).powi(2);
            let d = {
                let c2 = (cp0.x - cp2.x).powi(2) + (cp0.y - cp2.y).powi(2);
                let cosx = (a2 + b2 - c2) / (2.0 * (a2 * b2).sqrt());
                let sinx = (1.0 - cosx.powi(2)).sqrt();
                radius / ((1.0 - cosx) / sinx)
            };

            // second tangent point
            let bnx = (cp1.x - cp2.x) / b2.sqrt();
            let bny = (cp1.y - cp2.y) / b2.sqrt();

            Point2D::new(cp1.x - bnx * d, cp1.y - bny * d)
        }

        match path.last()? {
            PathSegment::ClosePath => {
                // find first point for this subpath
                let subpath_first_point_index = match path
                    .iter()
                    .rev()
                    .skip(1)
                    .position(|segment| *segment == PathSegment::ClosePath)
                {
                    Some(index) => index + 1,
                    None => 0,
                };

                let subpath_first_point = path.get(subpath_first_point_index)?;
                match subpath_first_point {
                    PathSegment::MoveTo { x, y } |
                    PathSegment::LineTo { x, y } |
                    PathSegment::Quadratic { x, y, .. } |
                    PathSegment::Bezier { x, y, .. } |
                    PathSegment::Ellipse { x, y, .. } |
                    PathSegment::SvgArc { x, y, .. } => Some(Point2D::new(*x, *y)),
                    PathSegment::ClosePath => None,
                    PathSegment::ArcTo {
                        cp1x,
                        cp1y,
                        cp2x,
                        cp2y,
                        radius,
                    } => {
                        let cp0 = match Path2D::compute_final_arc_to_point(
                            &path[..subpath_first_point_index],
                        ) {
                            Some(point) => point,
                            None => Point2D::new(*cp1x, *cp1y),
                        };
                        let cp1 = Point2D::new(*cp1x, *cp1y);
                        let cp2 = Point2D::new(*cp2x, *cp2y);
                        Some(compute_arc_to_points(cp0, cp1, cp2, *radius))
                    },
                }
            },
            PathSegment::MoveTo { x, y } |
            PathSegment::LineTo { x, y } |
            PathSegment::Quadratic { x, y, .. } |
            PathSegment::Bezier { x, y, .. } |
            PathSegment::Ellipse { x, y, .. } |
            PathSegment::SvgArc { x, y, .. } => Some(Point2D::new(*x, *y)),
            PathSegment::ArcTo {
                cp1x,
                cp1y,
                cp2x,
                cp2y,
                radius,
            } => {
                let last_segment = path.len() - 1;
                let cp0 = match Path2D::compute_final_arc_to_point(&path[..last_segment]) {
                    Some(point) => point,
                    None => Point2D::new(*cp1x, *cp1y),
                };
                let cp1 = Point2D::new(*cp1x, *cp1y);
                let cp2 = Point2D::new(*cp2x, *cp2y);
                Some(compute_arc_to_points(cp0, cp1, cp2, *radius))
            },
        }
    }

    fn last_point(path: &[PathSegment]) -> Option<Point2D<f32>> {
        let last_point = path.last()?;

        match last_point {
            PathSegment::MoveTo { x, y } |
            PathSegment::LineTo { x, y } |
            PathSegment::Quadratic { x, y, .. } |
            PathSegment::Bezier { x, y, .. } |
            PathSegment::Ellipse { x, y, .. } |
            PathSegment::SvgArc { x, y, .. } => Some(Point2D::new(*x, *y)),
            PathSegment::ArcTo { cp1x, cp1y, .. } => Some(
                Path2D::compute_final_arc_to_point(&path[..path.len() - 1])
                    .unwrap_or(Point2D::new(*cp1x, *cp1y)),
            ),
            PathSegment::ClosePath => {
                // find first point for the last subpath
                let first_point_index = {
                    let mut index = None;
                    for i in (0..path.len()).rev().skip(1) {
                        if matches!(path[i], PathSegment::ClosePath) {
                            index = Some(i);
                            break;
                        }
                    }

                    if let Some(index) = index {
                        index + 1
                    } else {
                        0
                    }
                };

                let first_point = path.get(first_point_index)?;
                match first_point {
                    PathSegment::MoveTo { x, y } |
                    PathSegment::LineTo { x, y } |
                    PathSegment::Quadratic { x, y, .. } |
                    PathSegment::Bezier { x, y, .. } |
                    PathSegment::Ellipse { x, y, .. } |
                    PathSegment::SvgArc { x, y, .. } => Some(Point2D::new(*x, *y)),
                    PathSegment::ClosePath => None,
                    PathSegment::ArcTo { cp1x, cp1y, .. } => {
                        match Path2D::compute_final_arc_to_point(&path[..first_point_index]) {
                            Some(point) => Some(point),
                            None => Some(Point2D::new(*cp1x, *cp1y)),
                        }
                    },
                }
            },
        }
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

        // Step 5. Transform all the coordinates and lines in c by the transform matrix matrix.
        let apply_matrix_transform = |x: f32, y: f32| -> Point2D<f64> {
            matrix.transform_point(Point2D::new(x.into(), y.into()))
        };

        for segment in c.iter_mut() {
            match *segment {
                PathSegment::MoveTo { x, y } => {
                    let transformed_point = apply_matrix_transform(x, y);
                    *segment = PathSegment::MoveTo {
                        x: transformed_point.x as f32,
                        y: transformed_point.y as f32,
                    };
                },
                PathSegment::LineTo { x, y } => {
                    let transformed_point = apply_matrix_transform(x, y);
                    *segment = PathSegment::LineTo {
                        x: transformed_point.x as f32,
                        y: transformed_point.y as f32,
                    }
                },
                PathSegment::Quadratic { cpx, cpy, x, y } => {
                    let transformed_cp = apply_matrix_transform(cpx, cpy);
                    let transformed_point = apply_matrix_transform(x, y);
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
                    let transformed_cp1 = apply_matrix_transform(cp1x, cp1y);
                    let transformed_cp2 = apply_matrix_transform(cp2x, cp2y);
                    let transformed_point = apply_matrix_transform(x, y);
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
                    let transformed_cp1 = apply_matrix_transform(cp1x, cp1y);
                    let transformed_cp2 = apply_matrix_transform(cp2x, cp2y);
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
                    let transformed_point = apply_matrix_transform(x, y);
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
                    let transformed_point = apply_matrix_transform(x, y);
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
                PathSegment::ClosePath => {},
            }
        }

        // Step 6. Let (x, y) be the last point in the last subpath of c
        let last_point = Path2D::last_point(&c);

        // Step 7. Add all the subpaths in c to a.
        self.path.borrow_mut().extend(c);

        // Step 8. Create a new subpath in `a` with (x, y) as the only point in the subpath.
        if let Some(last_point) = last_point {
            self.push(PathSegment::MoveTo {
                x: last_point.x,
                y: last_point.y,
            });
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
        if self.is_path_empty() {
            return;
        }

        if matches!(
            self.last().expect("Path should not be empty"),
            PathSegment::ClosePath
        ) {
            return;
        }

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_last_point() {
        let input = vec![
            (
                vec![PathSegment::MoveTo {
                    x: 100.,
                    y: 100.0f32,
                }],
                Some(Point2D::new(100., 100.0f32)),
            ),
            (
                vec![
                    PathSegment::MoveTo { x: 40., y: 23.0f32 },
                    PathSegment::ClosePath,
                ],
                Some(Point2D::new(40., 23.0f32)),
            ),
            (
                vec![
                    PathSegment::LineTo { x: 89.0f32, y: 33. },
                    PathSegment::MoveTo { x: 40., y: 23.0f32 },
                    PathSegment::ClosePath,
                ],
                Some(Point2D::new(89., 33.0f32)),
            ),
            (
                vec![
                    PathSegment::LineTo {
                        x: 23.0f32,
                        y: 423.,
                    },
                    PathSegment::ClosePath,
                    PathSegment::LineTo { x: 89.0f32, y: 33. },
                    PathSegment::MoveTo { x: 40., y: 23.0f32 },
                    PathSegment::ClosePath,
                ],
                Some(Point2D::new(89., 33.0f32)),
            ),
            (
                vec![PathSegment::ArcTo {
                    cp1x: 33.,
                    cp1y: 44.,
                    cp2x: 88.,
                    cp2y: 100.0f32,
                    radius: 30.,
                }],
                Some(Point2D::new(33., 44.0f32)),
            ),
            (
                vec![
                    PathSegment::MoveTo {
                        x: 89.,
                        y: 100.0f32,
                    },
                    PathSegment::ArcTo {
                        cp1x: 33.,
                        cp1y: 44.,
                        cp2x: 88.,
                        cp2y: 100.0f32,
                        radius: 30.,
                    },
                ],
                Some(Point2D::new(89., 100.0f32)),
            ),
            (
                vec![
                    PathSegment::LineTo {
                        x: 89.,
                        y: 100.0f32,
                    },
                    PathSegment::ClosePath,
                    PathSegment::ArcTo {
                        cp1x: 33.,
                        cp1y: 44.,
                        cp2x: 88.,
                        cp2y: 100.0f32,
                        radius: 30.,
                    },
                ],
                Some(Point2D::new(89., 100.0f32)),
            ),
            (
                vec![
                    PathSegment::LineTo {
                        x: 89.,
                        y: 100.0f32,
                    },
                    PathSegment::ClosePath,
                    PathSegment::ArcTo {
                        cp1x: 123.,
                        cp1y: 81.,
                        cp2x: 30.,
                        cp2y: 420.0f32,
                        radius: 30.,
                    },
                    PathSegment::ArcTo {
                        cp1x: 33.,
                        cp1y: 44.,
                        cp2x: 88.,
                        cp2y: 100.0f32,
                        radius: 30.,
                    },
                ],
                Some(Point2D::new(104.05499, 150.05759f32)),
            ),
            (
                vec![
                    PathSegment::ArcTo {
                        cp1x: 123.,
                        cp1y: 81.,
                        cp2x: 30.,
                        cp2y: 420.0f32,
                        radius: 30.,
                    },
                    PathSegment::ClosePath,
                    PathSegment::ArcTo {
                        cp1x: 838.,
                        cp1y: 132.,
                        cp2x: 21.,
                        cp2y: 180.0f32,
                        radius: 70.,
                    },
                ],
                Some(Point2D::new(123., 81.)),
            ),
            (
                vec![
                    PathSegment::ArcTo {
                        cp1x: 323.,
                        cp1y: 89.,
                        cp2x: 33.,
                        cp2y: 111.0f32,
                        radius: 30.,
                    },
                    PathSegment::ClosePath,
                    PathSegment::ArcTo {
                        cp1x: 123.,
                        cp1y: 81.,
                        cp2x: 30.,
                        cp2y: 420.0f32,
                        radius: 200.,
                    },
                    PathSegment::ClosePath,
                    PathSegment::ArcTo {
                        cp1x: 838.,
                        cp1y: 132.,
                        cp2x: 21.,
                        cp2y: 180.0f32,
                        radius: 70.,
                    },
                ],
                Some(Point2D::new(80.94957, 234.28061)),
            ),
            (
                vec![
                    PathSegment::MoveTo { x: 323., y: 89. },
                    PathSegment::ClosePath,
                    PathSegment::ArcTo {
                        cp1x: 123.,
                        cp1y: 81.,
                        cp2x: 30.,
                        cp2y: 420.0f32,
                        radius: 200.,
                    },
                    PathSegment::ClosePath,
                    PathSegment::ArcTo {
                        cp1x: 838.,
                        cp1y: 132.,
                        cp2x: 21.,
                        cp2y: 180.0f32,
                        radius: 70.,
                    },
                ],
                Some(Point2D::new(80.94957, 234.28061)),
            ),
            (
                vec![
                    PathSegment::ArcTo {
                        cp1x: 323.,
                        cp1y: 89.,
                        cp2x: 33.,
                        cp2y: 111.0f32,
                        radius: 30.,
                    },
                    PathSegment::ArcTo {
                        cp1x: 123.,
                        cp1y: 81.,
                        cp2x: 30.,
                        cp2y: 420.0f32,
                        radius: 200.,
                    },
                    PathSegment::ClosePath,
                    PathSegment::ArcTo {
                        cp1x: 838.,
                        cp1y: 132.,
                        cp2x: 21.,
                        cp2y: 180.0f32,
                        radius: 70.,
                    },
                ],
                Some(Point2D::new(323., 89.)),
            ),
            (
                vec![
                    PathSegment::ArcTo {
                        cp1x: 323.,
                        cp1y: 89.,
                        cp2x: 33.,
                        cp2y: 111.0f32,
                        radius: 30.,
                    },
                    PathSegment::ArcTo {
                        cp1x: 123.,
                        cp1y: 81.,
                        cp2x: 30.,
                        cp2y: 420.0f32,
                        radius: 200.,
                    },
                    PathSegment::ArcTo {
                        cp1x: 838.,
                        cp1y: 132.,
                        cp2x: 21.,
                        cp2y: 180.0f32,
                        radius: 70.,
                    },
                ],
                Some(Point2D::new(80.94957, 234.28061)),
            ),
            (
                vec![
                    PathSegment::ArcTo {
                        cp1x: 100.,
                        cp1y: 100.,
                        cp2x: 100.,
                        cp2y: 100.,
                        radius: 20.,
                    },
                    PathSegment::ClosePath,
                ],
                Some(Point2D::new(100., 100.)),
            ),
            (
                vec![
                    PathSegment::MoveTo { x: 111., y: 333. },
                    PathSegment::ClosePath,
                    PathSegment::ArcTo {
                        cp1x: 100.,
                        cp1y: 100.,
                        cp2x: 100.,
                        cp2y: 100.,
                        radius: 20.,
                    },
                    PathSegment::ClosePath,
                ],
                Some(Point2D::new(111., 333.)),
            ),
            (
                vec![
                    PathSegment::ArcTo {
                        cp1x: 111.,
                        cp1y: 333.,
                        cp2x: 800.,
                        cp2y: 58.,
                        radius: 100.,
                    },
                    PathSegment::ClosePath,
                    PathSegment::ArcTo {
                        cp1x: 100.,
                        cp1y: 100.,
                        cp2x: 100.,
                        cp2y: 100.,
                        radius: 20.,
                    },
                    PathSegment::ClosePath,
                ],
                Some(Point2D::new(111., 333.)),
            ),
        ];

        for (segments, expected) in input {
            let last_point = Path2D::last_point(&segments);
            assert_eq!(last_point, expected);
        }
    }
}
