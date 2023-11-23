/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for SVG Path.

use crate::parser::{Parse, ParserContext};
use crate::values::animated::{lists, Animate, Procedure, ToAnimatedZero};
use crate::values::distance::{ComputeSquaredDistance, SquaredDistance};
use crate::values::CSSFloat;
use cssparser::Parser;
use std::fmt::{self, Write};
use std::iter::{Cloned, Peekable};
use std::slice;
use style_traits::values::SequenceWriter;
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};

/// Whether to allow empty string in the parser.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(missing_docs)]
pub enum AllowEmpty {
    Yes,
    No,
}

/// The SVG path data.
///
/// https://www.w3.org/TR/SVG11/paths.html#PathData
#[derive(
    Clone,
    Debug,
    Deserialize,
    MallocSizeOf,
    PartialEq,
    Serialize,
    SpecifiedValueInfo,
    ToAnimatedZero,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct SVGPathData(
    // TODO(emilio): Should probably measure this somehow only from the
    // specified values.
    #[ignore_malloc_size_of = "Arc"] pub crate::ArcSlice<PathCommand>,
);

impl SVGPathData {
    /// Get the array of PathCommand.
    #[inline]
    pub fn commands(&self) -> &[PathCommand] {
        &self.0
    }

    /// Create a normalized copy of this path by converting each relative
    /// command to an absolute command.
    pub fn normalize(&self) -> Self {
        let mut state = PathTraversalState {
            subpath_start: CoordPair::new(0.0, 0.0),
            pos: CoordPair::new(0.0, 0.0),
        };
        let iter = self.0.iter().map(|seg| seg.normalize(&mut state));
        SVGPathData(crate::ArcSlice::from_iter(iter))
    }

    // FIXME: Bug 1714238, we may drop this once we use the same data structure for both SVG and
    // CSS.
    /// Decode the svg path raw data from Gecko.
    #[cfg(feature = "gecko")]
    pub fn decode_from_f32_array(path: &[f32]) -> Result<Self, ()> {
        use crate::gecko_bindings::structs::dom::SVGPathSeg_Binding::*;

        let mut result: Vec<PathCommand> = Vec::new();
        let mut i: usize = 0;
        while i < path.len() {
            // See EncodeType() and DecodeType() in SVGPathSegUtils.h.
            // We are using reinterpret_cast<> to encode and decode between u32 and f32, so here we
            // use to_bits() to decode the type.
            let seg_type = path[i].to_bits() as u16;
            i = i + 1;
            match seg_type {
                PATHSEG_CLOSEPATH => result.push(PathCommand::ClosePath),
                PATHSEG_MOVETO_ABS | PATHSEG_MOVETO_REL => {
                    debug_assert!(i + 1 < path.len());
                    result.push(PathCommand::MoveTo {
                        point: CoordPair::new(path[i], path[i + 1]),
                        absolute: IsAbsolute::new(seg_type == PATHSEG_MOVETO_ABS),
                    });
                    i = i + 2;
                },
                PATHSEG_LINETO_ABS | PATHSEG_LINETO_REL => {
                    debug_assert!(i + 1 < path.len());
                    result.push(PathCommand::LineTo {
                        point: CoordPair::new(path[i], path[i + 1]),
                        absolute: IsAbsolute::new(seg_type == PATHSEG_LINETO_ABS),
                    });
                    i = i + 2;
                },
                PATHSEG_CURVETO_CUBIC_ABS | PATHSEG_CURVETO_CUBIC_REL => {
                    debug_assert!(i + 5 < path.len());
                    result.push(PathCommand::CurveTo {
                        control1: CoordPair::new(path[i], path[i + 1]),
                        control2: CoordPair::new(path[i + 2], path[i + 3]),
                        point: CoordPair::new(path[i + 4], path[i + 5]),
                        absolute: IsAbsolute::new(seg_type == PATHSEG_CURVETO_CUBIC_ABS),
                    });
                    i = i + 6;
                },
                PATHSEG_CURVETO_QUADRATIC_ABS | PATHSEG_CURVETO_QUADRATIC_REL => {
                    debug_assert!(i + 3 < path.len());
                    result.push(PathCommand::QuadBezierCurveTo {
                        control1: CoordPair::new(path[i], path[i + 1]),
                        point: CoordPair::new(path[i + 2], path[i + 3]),
                        absolute: IsAbsolute::new(seg_type == PATHSEG_CURVETO_QUADRATIC_ABS),
                    });
                    i = i + 4;
                },
                PATHSEG_ARC_ABS | PATHSEG_ARC_REL => {
                    debug_assert!(i + 6 < path.len());
                    result.push(PathCommand::EllipticalArc {
                        rx: path[i],
                        ry: path[i + 1],
                        angle: path[i + 2],
                        large_arc_flag: ArcFlag(path[i + 3] != 0.0f32),
                        sweep_flag: ArcFlag(path[i + 4] != 0.0f32),
                        point: CoordPair::new(path[i + 5], path[i + 6]),
                        absolute: IsAbsolute::new(seg_type == PATHSEG_ARC_ABS),
                    });
                    i = i + 7;
                },
                PATHSEG_LINETO_HORIZONTAL_ABS | PATHSEG_LINETO_HORIZONTAL_REL => {
                    debug_assert!(i < path.len());
                    result.push(PathCommand::HorizontalLineTo {
                        x: path[i],
                        absolute: IsAbsolute::new(seg_type == PATHSEG_LINETO_HORIZONTAL_ABS),
                    });
                    i = i + 1;
                },
                PATHSEG_LINETO_VERTICAL_ABS | PATHSEG_LINETO_VERTICAL_REL => {
                    debug_assert!(i < path.len());
                    result.push(PathCommand::VerticalLineTo {
                        y: path[i],
                        absolute: IsAbsolute::new(seg_type == PATHSEG_LINETO_VERTICAL_ABS),
                    });
                    i = i + 1;
                },
                PATHSEG_CURVETO_CUBIC_SMOOTH_ABS | PATHSEG_CURVETO_CUBIC_SMOOTH_REL => {
                    debug_assert!(i + 3 < path.len());
                    result.push(PathCommand::SmoothCurveTo {
                        control2: CoordPair::new(path[i], path[i + 1]),
                        point: CoordPair::new(path[i + 2], path[i + 3]),
                        absolute: IsAbsolute::new(seg_type == PATHSEG_CURVETO_CUBIC_SMOOTH_ABS),
                    });
                    i = i + 4;
                },
                PATHSEG_CURVETO_QUADRATIC_SMOOTH_ABS | PATHSEG_CURVETO_QUADRATIC_SMOOTH_REL => {
                    debug_assert!(i + 1 < path.len());
                    result.push(PathCommand::SmoothQuadBezierCurveTo {
                        point: CoordPair::new(path[i], path[i + 1]),
                        absolute: IsAbsolute::new(seg_type == PATHSEG_CURVETO_QUADRATIC_SMOOTH_ABS),
                    });
                    i = i + 2;
                },
                PATHSEG_UNKNOWN | _ => return Err(()),
            }
        }

        Ok(SVGPathData(crate::ArcSlice::from_iter(result.into_iter())))
    }

    /// Parse this SVG path string with the argument that indicates whether we should allow the
    /// empty string.
    // We cannot use cssparser::Parser to parse a SVG path string because the spec wants to make
    // the SVG path string as compact as possible. (i.e. The whitespaces may be dropped.)
    // e.g. "M100 200L100 200" is a valid SVG path string. If we use tokenizer, the first ident
    // is "M100", instead of "M", and this is not correct. Therefore, we use a Peekable
    // str::Char iterator to check each character.
    pub fn parse<'i, 't>(
        input: &mut Parser<'i, 't>,
        allow_empty: AllowEmpty,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        let path_string = input.expect_string()?.as_ref();

        // Parse the svg path string as multiple sub-paths.
        let mut path_parser = PathParser::new(path_string);
        while skip_wsp(&mut path_parser.chars) {
            if path_parser.parse_subpath().is_err() {
                return Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError));
            }
        }

        // The css-shapes-1 says a path data string that does conform but defines an empty path is
        // invalid and causes the entire path() to be invalid, so we use the argement to decide
        // whether we should allow the empty string.
        // https://drafts.csswg.org/css-shapes-1/#typedef-basic-shape
        if matches!(allow_empty, AllowEmpty::No) && path_parser.path.is_empty() {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        Ok(SVGPathData(crate::ArcSlice::from_iter(
            path_parser.path.into_iter(),
        )))
    }
}

impl ToCss for SVGPathData {
    #[inline]
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        dest.write_char('"')?;
        {
            let mut writer = SequenceWriter::new(dest, " ");
            for command in self.commands() {
                writer.item(command)?;
            }
        }
        dest.write_char('"')
    }
}

impl Parse for SVGPathData {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        // Note that the EBNF allows the path data string in the d property to be empty, so we
        // don't reject empty SVG path data.
        // https://svgwg.org/svg2-draft/single-page.html#paths-PathDataBNF
        SVGPathData::parse(input, AllowEmpty::Yes)
    }
}

impl Animate for SVGPathData {
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        if self.0.len() != other.0.len() {
            return Err(());
        }

        // FIXME(emilio): This allocates three copies of the path, that's not
        // great! Specially, once we're normalized once, we don't need to
        // re-normalize again.
        let left = self.normalize();
        let right = other.normalize();

        let items: Vec<_> = lists::by_computed_value::animate(&left.0, &right.0, procedure)?;
        Ok(SVGPathData(crate::ArcSlice::from_iter(items.into_iter())))
    }
}

impl ComputeSquaredDistance for SVGPathData {
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        if self.0.len() != other.0.len() {
            return Err(());
        }
        let left = self.normalize();
        let right = other.normalize();
        lists::by_computed_value::squared_distance(&left.0, &right.0)
    }
}

/// The SVG path command.
/// The fields of these commands are self-explanatory, so we skip the documents.
/// Note: the index of the control points, e.g. control1, control2, are mapping to the control
/// points of the Bézier curve in the spec.
///
/// https://www.w3.org/TR/SVG11/paths.html#PathData
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    Deserialize,
    MallocSizeOf,
    PartialEq,
    Serialize,
    SpecifiedValueInfo,
    ToAnimatedZero,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[allow(missing_docs)]
#[repr(C, u8)]
pub enum PathCommand {
    /// The unknown type.
    /// https://www.w3.org/TR/SVG/paths.html#__svg__SVGPathSeg__PATHSEG_UNKNOWN
    Unknown,
    /// The "moveto" command.
    MoveTo {
        point: CoordPair,
        absolute: IsAbsolute,
    },
    /// The "lineto" command.
    LineTo {
        point: CoordPair,
        absolute: IsAbsolute,
    },
    /// The horizontal "lineto" command.
    HorizontalLineTo { x: CSSFloat, absolute: IsAbsolute },
    /// The vertical "lineto" command.
    VerticalLineTo { y: CSSFloat, absolute: IsAbsolute },
    /// The cubic Bézier curve command.
    CurveTo {
        control1: CoordPair,
        control2: CoordPair,
        point: CoordPair,
        absolute: IsAbsolute,
    },
    /// The smooth curve command.
    SmoothCurveTo {
        control2: CoordPair,
        point: CoordPair,
        absolute: IsAbsolute,
    },
    /// The quadratic Bézier curve command.
    QuadBezierCurveTo {
        control1: CoordPair,
        point: CoordPair,
        absolute: IsAbsolute,
    },
    /// The smooth quadratic Bézier curve command.
    SmoothQuadBezierCurveTo {
        point: CoordPair,
        absolute: IsAbsolute,
    },
    /// The elliptical arc curve command.
    EllipticalArc {
        rx: CSSFloat,
        ry: CSSFloat,
        angle: CSSFloat,
        large_arc_flag: ArcFlag,
        sweep_flag: ArcFlag,
        point: CoordPair,
        absolute: IsAbsolute,
    },
    /// The "closepath" command.
    ClosePath,
}

/// For internal SVGPath normalization.
#[allow(missing_docs)]
struct PathTraversalState {
    subpath_start: CoordPair,
    pos: CoordPair,
}

impl PathCommand {
    /// Create a normalized copy of this PathCommand. Absolute commands will be copied as-is while
    /// for relative commands an equivalent absolute command will be returned.
    ///
    /// See discussion: https://github.com/w3c/svgwg/issues/321
    fn normalize(&self, state: &mut PathTraversalState) -> Self {
        use self::PathCommand::*;
        match *self {
            Unknown => Unknown,
            ClosePath => {
                state.pos = state.subpath_start;
                ClosePath
            },
            MoveTo {
                mut point,
                absolute,
            } => {
                if !absolute.is_yes() {
                    point += state.pos;
                }
                state.pos = point;
                state.subpath_start = point;
                MoveTo {
                    point,
                    absolute: IsAbsolute::Yes,
                }
            },
            LineTo {
                mut point,
                absolute,
            } => {
                if !absolute.is_yes() {
                    point += state.pos;
                }
                state.pos = point;
                LineTo {
                    point,
                    absolute: IsAbsolute::Yes,
                }
            },
            HorizontalLineTo { mut x, absolute } => {
                if !absolute.is_yes() {
                    x += state.pos.x;
                }
                state.pos.x = x;
                HorizontalLineTo {
                    x,
                    absolute: IsAbsolute::Yes,
                }
            },
            VerticalLineTo { mut y, absolute } => {
                if !absolute.is_yes() {
                    y += state.pos.y;
                }
                state.pos.y = y;
                VerticalLineTo {
                    y,
                    absolute: IsAbsolute::Yes,
                }
            },
            CurveTo {
                mut control1,
                mut control2,
                mut point,
                absolute,
            } => {
                if !absolute.is_yes() {
                    control1 += state.pos;
                    control2 += state.pos;
                    point += state.pos;
                }
                state.pos = point;
                CurveTo {
                    control1,
                    control2,
                    point,
                    absolute: IsAbsolute::Yes,
                }
            },
            SmoothCurveTo {
                mut control2,
                mut point,
                absolute,
            } => {
                if !absolute.is_yes() {
                    control2 += state.pos;
                    point += state.pos;
                }
                state.pos = point;
                SmoothCurveTo {
                    control2,
                    point,
                    absolute: IsAbsolute::Yes,
                }
            },
            QuadBezierCurveTo {
                mut control1,
                mut point,
                absolute,
            } => {
                if !absolute.is_yes() {
                    control1 += state.pos;
                    point += state.pos;
                }
                state.pos = point;
                QuadBezierCurveTo {
                    control1,
                    point,
                    absolute: IsAbsolute::Yes,
                }
            },
            SmoothQuadBezierCurveTo {
                mut point,
                absolute,
            } => {
                if !absolute.is_yes() {
                    point += state.pos;
                }
                state.pos = point;
                SmoothQuadBezierCurveTo {
                    point,
                    absolute: IsAbsolute::Yes,
                }
            },
            EllipticalArc {
                rx,
                ry,
                angle,
                large_arc_flag,
                sweep_flag,
                mut point,
                absolute,
            } => {
                if !absolute.is_yes() {
                    point += state.pos;
                }
                state.pos = point;
                EllipticalArc {
                    rx,
                    ry,
                    angle,
                    large_arc_flag,
                    sweep_flag,
                    point,
                    absolute: IsAbsolute::Yes,
                }
            },
        }
    }
}

impl ToCss for PathCommand {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        use self::PathCommand::*;
        match *self {
            Unknown => dest.write_char('X'),
            ClosePath => dest.write_char('Z'),
            MoveTo { point, absolute } => {
                dest.write_char(if absolute.is_yes() { 'M' } else { 'm' })?;
                dest.write_char(' ')?;
                point.to_css(dest)
            },
            LineTo { point, absolute } => {
                dest.write_char(if absolute.is_yes() { 'L' } else { 'l' })?;
                dest.write_char(' ')?;
                point.to_css(dest)
            },
            CurveTo {
                control1,
                control2,
                point,
                absolute,
            } => {
                dest.write_char(if absolute.is_yes() { 'C' } else { 'c' })?;
                dest.write_char(' ')?;
                control1.to_css(dest)?;
                dest.write_char(' ')?;
                control2.to_css(dest)?;
                dest.write_char(' ')?;
                point.to_css(dest)
            },
            QuadBezierCurveTo {
                control1,
                point,
                absolute,
            } => {
                dest.write_char(if absolute.is_yes() { 'Q' } else { 'q' })?;
                dest.write_char(' ')?;
                control1.to_css(dest)?;
                dest.write_char(' ')?;
                point.to_css(dest)
            },
            EllipticalArc {
                rx,
                ry,
                angle,
                large_arc_flag,
                sweep_flag,
                point,
                absolute,
            } => {
                dest.write_char(if absolute.is_yes() { 'A' } else { 'a' })?;
                dest.write_char(' ')?;
                rx.to_css(dest)?;
                dest.write_char(' ')?;
                ry.to_css(dest)?;
                dest.write_char(' ')?;
                angle.to_css(dest)?;
                dest.write_char(' ')?;
                large_arc_flag.to_css(dest)?;
                dest.write_char(' ')?;
                sweep_flag.to_css(dest)?;
                dest.write_char(' ')?;
                point.to_css(dest)
            },
            HorizontalLineTo { x, absolute } => {
                dest.write_char(if absolute.is_yes() { 'H' } else { 'h' })?;
                dest.write_char(' ')?;
                x.to_css(dest)
            },
            VerticalLineTo { y, absolute } => {
                dest.write_char(if absolute.is_yes() { 'V' } else { 'v' })?;
                dest.write_char(' ')?;
                y.to_css(dest)
            },
            SmoothCurveTo {
                control2,
                point,
                absolute,
            } => {
                dest.write_char(if absolute.is_yes() { 'S' } else { 's' })?;
                dest.write_char(' ')?;
                control2.to_css(dest)?;
                dest.write_char(' ')?;
                point.to_css(dest)
            },
            SmoothQuadBezierCurveTo { point, absolute } => {
                dest.write_char(if absolute.is_yes() { 'T' } else { 't' })?;
                dest.write_char(' ')?;
                point.to_css(dest)
            },
        }
    }
}

/// The path command absolute type.
#[allow(missing_docs)]
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    Deserialize,
    MallocSizeOf,
    PartialEq,
    Serialize,
    SpecifiedValueInfo,
    ToAnimatedZero,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum IsAbsolute {
    Yes,
    No,
}

impl IsAbsolute {
    /// Return true if this is IsAbsolute::Yes.
    #[inline]
    pub fn is_yes(&self) -> bool {
        *self == IsAbsolute::Yes
    }

    /// Return Yes if value is true. Otherwise, return No.
    #[inline]
    #[cfg(feature = "gecko")]
    fn new(value: bool) -> Self {
        if value {
            IsAbsolute::Yes
        } else {
            IsAbsolute::No
        }
    }
}

/// The path coord type.
#[allow(missing_docs)]
#[derive(
    AddAssign,
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    Deserialize,
    MallocSizeOf,
    PartialEq,
    Serialize,
    SpecifiedValueInfo,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct CoordPair {
    x: CSSFloat,
    y: CSSFloat,
}

impl CoordPair {
    /// Create a CoordPair.
    #[inline]
    pub fn new(x: CSSFloat, y: CSSFloat) -> Self {
        CoordPair { x, y }
    }
}

/// The EllipticalArc flag type.
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    MallocSizeOf,
    PartialEq,
    Serialize,
    SpecifiedValueInfo,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct ArcFlag(bool);

impl ToCss for ArcFlag {
    #[inline]
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        (self.0 as i32).to_css(dest)
    }
}

impl Animate for ArcFlag {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        (self.0 as i32)
            .animate(&(other.0 as i32), procedure)
            .map(|v| ArcFlag(v > 0))
    }
}

impl ComputeSquaredDistance for ArcFlag {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        (self.0 as i32).compute_squared_distance(&(other.0 as i32))
    }
}

impl ToAnimatedZero for ArcFlag {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        // The 2 ArcFlags in EllipticalArc determine which one of the 4 different arcs will be
        // used. (i.e. From 4 combinations). In other words, if we change the flag, we get a
        // different arc. Therefore, we return *self.
        // https://svgwg.org/svg2-draft/paths.html#PathDataEllipticalArcCommands
        Ok(*self)
    }
}

/// SVG Path parser.
struct PathParser<'a> {
    chars: Peekable<Cloned<slice::Iter<'a, u8>>>,
    path: Vec<PathCommand>,
}

macro_rules! parse_arguments {
    (
        $parser:ident,
        $abs:ident,
        $enum:ident,
        [ $para:ident => $func:ident $(, $other_para:ident => $other_func:ident)* ]
    ) => {
        {
            loop {
                let $para = $func(&mut $parser.chars)?;
                $(
                    skip_comma_wsp(&mut $parser.chars);
                    let $other_para = $other_func(&mut $parser.chars)?;
                )*
                $parser.path.push(PathCommand::$enum { $para $(, $other_para)*, $abs });

                // End of string or the next character is a possible new command.
                if !skip_wsp(&mut $parser.chars) ||
                   $parser.chars.peek().map_or(true, |c| c.is_ascii_alphabetic()) {
                    break;
                }
                skip_comma_wsp(&mut $parser.chars);
            }
            Ok(())
        }
    }
}

impl<'a> PathParser<'a> {
    /// Return a PathParser.
    #[inline]
    fn new(string: &'a str) -> Self {
        PathParser {
            chars: string.as_bytes().iter().cloned().peekable(),
            path: Vec::new(),
        }
    }

    /// Parse a sub-path.
    fn parse_subpath(&mut self) -> Result<(), ()> {
        // Handle "moveto" Command first. If there is no "moveto", this is not a valid sub-path
        // (i.e. not a valid moveto-drawto-command-group).
        self.parse_moveto()?;

        // Handle other commands.
        loop {
            skip_wsp(&mut self.chars);
            if self.chars.peek().map_or(true, |&m| m == b'M' || m == b'm') {
                break;
            }

            let command = self.chars.next().unwrap();
            let abs = if command.is_ascii_uppercase() {
                IsAbsolute::Yes
            } else {
                IsAbsolute::No
            };

            skip_wsp(&mut self.chars);
            match command {
                b'Z' | b'z' => self.parse_closepath(),
                b'L' | b'l' => self.parse_lineto(abs),
                b'H' | b'h' => self.parse_h_lineto(abs),
                b'V' | b'v' => self.parse_v_lineto(abs),
                b'C' | b'c' => self.parse_curveto(abs),
                b'S' | b's' => self.parse_smooth_curveto(abs),
                b'Q' | b'q' => self.parse_quadratic_bezier_curveto(abs),
                b'T' | b't' => self.parse_smooth_quadratic_bezier_curveto(abs),
                b'A' | b'a' => self.parse_elliptical_arc(abs),
                _ => return Err(()),
            }?;
        }
        Ok(())
    }

    /// Parse "moveto" command.
    fn parse_moveto(&mut self) -> Result<(), ()> {
        let command = match self.chars.next() {
            Some(c) if c == b'M' || c == b'm' => c,
            _ => return Err(()),
        };

        skip_wsp(&mut self.chars);
        let point = parse_coord(&mut self.chars)?;
        let absolute = if command == b'M' {
            IsAbsolute::Yes
        } else {
            IsAbsolute::No
        };
        self.path.push(PathCommand::MoveTo { point, absolute });

        // End of string or the next character is a possible new command.
        if !skip_wsp(&mut self.chars) || self.chars.peek().map_or(true, |c| c.is_ascii_alphabetic())
        {
            return Ok(());
        }
        skip_comma_wsp(&mut self.chars);

        // If a moveto is followed by multiple pairs of coordinates, the subsequent
        // pairs are treated as implicit lineto commands.
        self.parse_lineto(absolute)
    }

    /// Parse "closepath" command.
    fn parse_closepath(&mut self) -> Result<(), ()> {
        self.path.push(PathCommand::ClosePath);
        Ok(())
    }

    /// Parse "lineto" command.
    fn parse_lineto(&mut self, absolute: IsAbsolute) -> Result<(), ()> {
        parse_arguments!(self, absolute, LineTo, [ point => parse_coord ])
    }

    /// Parse horizontal "lineto" command.
    fn parse_h_lineto(&mut self, absolute: IsAbsolute) -> Result<(), ()> {
        parse_arguments!(self, absolute, HorizontalLineTo, [ x => parse_number ])
    }

    /// Parse vertical "lineto" command.
    fn parse_v_lineto(&mut self, absolute: IsAbsolute) -> Result<(), ()> {
        parse_arguments!(self, absolute, VerticalLineTo, [ y => parse_number ])
    }

    /// Parse cubic Bézier curve command.
    fn parse_curveto(&mut self, absolute: IsAbsolute) -> Result<(), ()> {
        parse_arguments!(self, absolute, CurveTo, [
            control1 => parse_coord, control2 => parse_coord, point => parse_coord
        ])
    }

    /// Parse smooth "curveto" command.
    fn parse_smooth_curveto(&mut self, absolute: IsAbsolute) -> Result<(), ()> {
        parse_arguments!(self, absolute, SmoothCurveTo, [
            control2 => parse_coord, point => parse_coord
        ])
    }

    /// Parse quadratic Bézier curve command.
    fn parse_quadratic_bezier_curveto(&mut self, absolute: IsAbsolute) -> Result<(), ()> {
        parse_arguments!(self, absolute, QuadBezierCurveTo, [
            control1 => parse_coord, point => parse_coord
        ])
    }

    /// Parse smooth quadratic Bézier curveto command.
    fn parse_smooth_quadratic_bezier_curveto(&mut self, absolute: IsAbsolute) -> Result<(), ()> {
        parse_arguments!(self, absolute, SmoothQuadBezierCurveTo, [ point => parse_coord ])
    }

    /// Parse elliptical arc curve command.
    fn parse_elliptical_arc(&mut self, absolute: IsAbsolute) -> Result<(), ()> {
        // Parse a flag whose value is '0' or '1'; otherwise, return Err(()).
        let parse_flag = |iter: &mut Peekable<Cloned<slice::Iter<u8>>>| match iter.next() {
            Some(c) if c == b'0' || c == b'1' => Ok(ArcFlag(c == b'1')),
            _ => Err(()),
        };
        parse_arguments!(self, absolute, EllipticalArc, [
            rx => parse_number,
            ry => parse_number,
            angle => parse_number,
            large_arc_flag => parse_flag,
            sweep_flag => parse_flag,
            point => parse_coord
        ])
    }
}

/// Parse a pair of numbers into CoordPair.
fn parse_coord(iter: &mut Peekable<Cloned<slice::Iter<u8>>>) -> Result<CoordPair, ()> {
    let x = parse_number(iter)?;
    skip_comma_wsp(iter);
    let y = parse_number(iter)?;
    Ok(CoordPair::new(x, y))
}

/// This is a special version which parses the number for SVG Path. e.g. "M 0.6.5" should be parsed
/// as MoveTo with a coordinate of ("0.6", ".5"), instead of treating 0.6.5 as a non-valid floating
/// point number. In other words, the logic here is similar with that of
/// tokenizer::consume_numeric, which also consumes the number as many as possible, but here the
/// input is a Peekable and we only accept an integer of a floating point number.
///
/// The "number" syntax in https://www.w3.org/TR/SVG/paths.html#PathDataBNF
fn parse_number(iter: &mut Peekable<Cloned<slice::Iter<u8>>>) -> Result<CSSFloat, ()> {
    // 1. Check optional sign.
    let sign = if iter
        .peek()
        .map_or(false, |&sign| sign == b'+' || sign == b'-')
    {
        if iter.next().unwrap() == b'-' {
            -1.
        } else {
            1.
        }
    } else {
        1.
    };

    // 2. Check integer part.
    let mut integral_part: f64 = 0.;
    let got_dot = if !iter.peek().map_or(false, |&n| n == b'.') {
        // If the first digit in integer part is neither a dot nor a digit, this is not a number.
        if iter.peek().map_or(true, |n| !n.is_ascii_digit()) {
            return Err(());
        }

        while iter.peek().map_or(false, |n| n.is_ascii_digit()) {
            integral_part = integral_part * 10. + (iter.next().unwrap() - b'0') as f64;
        }

        iter.peek().map_or(false, |&n| n == b'.')
    } else {
        true
    };

    // 3. Check fractional part.
    let mut fractional_part: f64 = 0.;
    if got_dot {
        // Consume '.'.
        iter.next();
        // If the first digit in fractional part is not a digit, this is not a number.
        if iter.peek().map_or(true, |n| !n.is_ascii_digit()) {
            return Err(());
        }

        let mut factor = 0.1;
        while iter.peek().map_or(false, |n| n.is_ascii_digit()) {
            fractional_part += (iter.next().unwrap() - b'0') as f64 * factor;
            factor *= 0.1;
        }
    }

    let mut value = sign * (integral_part + fractional_part);

    // 4. Check exp part. The segment name of SVG Path doesn't include 'E' or 'e', so it's ok to
    //    treat the numbers after 'E' or 'e' are in the exponential part.
    if iter.peek().map_or(false, |&exp| exp == b'E' || exp == b'e') {
        // Consume 'E' or 'e'.
        iter.next();
        let exp_sign = if iter
            .peek()
            .map_or(false, |&sign| sign == b'+' || sign == b'-')
        {
            if iter.next().unwrap() == b'-' {
                -1.
            } else {
                1.
            }
        } else {
            1.
        };

        let mut exp: f64 = 0.;
        while iter.peek().map_or(false, |n| n.is_ascii_digit()) {
            exp = exp * 10. + (iter.next().unwrap() - b'0') as f64;
        }

        value *= f64::powf(10., exp * exp_sign);
    }

    if value.is_finite() {
        Ok(value.min(f32::MAX as f64).max(f32::MIN as f64) as CSSFloat)
    } else {
        Err(())
    }
}

/// Skip all svg whitespaces, and return true if |iter| hasn't finished.
#[inline]
fn skip_wsp(iter: &mut Peekable<Cloned<slice::Iter<u8>>>) -> bool {
    // Note: SVG 1.1 defines the whitespaces as \u{9}, \u{20}, \u{A}, \u{D}.
    //       However, SVG 2 has one extra whitespace: \u{C}.
    //       Therefore, we follow the newest spec for the definition of whitespace,
    //       i.e. \u{9}, \u{20}, \u{A}, \u{C}, \u{D}.
    while iter.peek().map_or(false, |c| c.is_ascii_whitespace()) {
        iter.next();
    }
    iter.peek().is_some()
}

/// Skip all svg whitespaces and one comma, and return true if |iter| hasn't finished.
#[inline]
fn skip_comma_wsp(iter: &mut Peekable<Cloned<slice::Iter<u8>>>) -> bool {
    if !skip_wsp(iter) {
        return false;
    }

    if *iter.peek().unwrap() != b',' {
        return true;
    }
    iter.next();

    skip_wsp(iter)
}
