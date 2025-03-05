/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use canvas_traits::canvas::PathSegment;
use dom_struct::dom_struct;
use js::rust::HandleObject;
use script_bindings::str::DOMString;
use svgtypes::SimplifyingPathParser;

use crate::dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::Path2DMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[derive(Clone, Copy, JSTraceable)]
struct PathSegmentWrapper(#[no_trace = "Does not contain managed objects"] PathSegment);
malloc_size_of::malloc_size_of_is_0!(PathSegmentWrapper);

#[dom_struct]
pub(crate) struct Path2D {
    reflector_: Reflector,
    path: RefCell<Vec<PathSegmentWrapper>>,
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
        for result in SimplifyingPathParser::from(path) {
            let Ok(seg) = result else { break };
            let seg = match seg {
                svgtypes::SimplePathSegment::MoveTo { x, y } => PathSegment::MoveTo {
                    x: x as f32,
                    y: y as f32,
                },
                svgtypes::SimplePathSegment::LineTo { x, y } => PathSegment::LineTo {
                    x: x as f32,
                    y: y as f32,
                },
                svgtypes::SimplePathSegment::CurveTo {
                    x1,
                    y1,
                    x2,
                    y2,
                    x,
                    y,
                } => PathSegment::Bezier {
                    cp1x: x1 as f32,
                    cp1y: y1 as f32,
                    cp2x: x2 as f32,
                    cp2y: y2 as f32,
                    x: x as f32,
                    y: y as f32,
                },
                svgtypes::SimplePathSegment::Quadratic { x1, y1, x, y } => PathSegment::Quadratic {
                    cpx: x1 as f32,
                    cpy: y1 as f32,
                    x: x as f32,
                    y: y as f32,
                },
                svgtypes::SimplePathSegment::ClosePath => PathSegment::ClosePath,
            };
            path_segments.push(seg);
        }
        Self {
            reflector_: Reflector::new(),
            path: RefCell::new(path_segments.into_iter().map(PathSegmentWrapper).collect()),
        }
    }
    pub(crate) fn push(&self, seg: PathSegment) {
        self.path.borrow_mut().push(PathSegmentWrapper(seg));
    }
    pub(crate) fn segments(&self) -> Vec<PathSegment> {
        self.path
            .borrow()
            .clone()
            .into_iter()
            .map(|PathSegmentWrapper(seg)| seg)
            .collect()
    }
}

impl Path2DMethods<crate::DomTypeHolder> for Path2D {
    // https://html.spec.whatwg.org/multipage/#dom-path2d-addpath
    fn AddPath(&self, other: &Path2D) {
        let mut dest = self.path.borrow_mut();
        dest.extend(other.path.borrow().iter().copied());
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-closepath
    fn ClosePath(&self) {
        self.push(PathSegment::ClosePath);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-moveto
    fn MoveTo(&self, x: f64, y: f64) {
        if !([x, y].iter().all(|x| x.is_finite())) {
            return;
        }

        self.push(PathSegment::MoveTo {
            x: x as f32,
            y: y as f32,
        });
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-lineto
    fn LineTo(&self, x: f64, y: f64) {
        if !([x, y].iter().all(|x| x.is_finite())) {
            return;
        }

        self.push(PathSegment::LineTo {
            x: x as f32,
            y: y as f32,
        });
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-quadraticcurveto
    fn QuadraticCurveTo(&self, cpx: f64, cpy: f64, x: f64, y: f64) {
        if !([cpx, cpy, x, y].iter().all(|x| x.is_finite())) {
            return;
        }

        self.push(PathSegment::Quadratic {
            cpx: cpx as f32,
            cpy: cpy as f32,
            x: x as f32,
            y: y as f32,
        });
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-beziercurveto
    fn BezierCurveTo(&self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, x: f64, y: f64) {
        if !([cp1x, cp1y, cp2x, cp2y, x, y].iter().all(|x| x.is_finite())) {
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

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-arcto
    fn ArcTo(&self, x1: f64, y1: f64, x2: f64, y2: f64, r: f64) -> Fallible<()> {
        if !([x1, y1, x1, y2, r].iter().all(|x| x.is_finite())) {
            return Ok(());
        }

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

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-rect
    fn Rect(&self, x: f64, y: f64, w: f64, h: f64) {
        if !([x, y, w, h].iter().all(|x| x.is_finite())) {
            return;
        }

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
        self.push(PathSegment::ClosePath);

        self.push(PathSegment::MoveTo {
            x: x as f32,
            y: y as f32,
        });
        self.push(PathSegment::ClosePath);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-arc
    fn Arc(
        &self,
        x: f64,
        y: f64,
        r: f64,
        start: f64,
        end: f64,
        anticlockwise: bool,
    ) -> Fallible<()> {
        if !([x, y, r, start, end].iter().all(|x| x.is_finite())) {
            return Ok(());
        }

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

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-ellipse
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
        if !([x, y, rx, ry, rotation, start, end]
            .iter()
            .all(|x| x.is_finite()))
        {
            return Ok(());
        }
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

    // https://html.spec.whatwg.org/multipage/#dom-path2d-dev
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<Path2D> {
        reflect_dom_object_with_proto(Box::new(Self::new()), global, proto, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-path2d-dev
    fn Constructor_(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        other: &Path2D,
    ) -> DomRoot<Path2D> {
        reflect_dom_object_with_proto(Box::new(Self::new_with_path(other)), global, proto, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-path2d-dev
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
