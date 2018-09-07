/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for SVG Path.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use std::fmt::{self, Write};
use std::iter::{Cloned, Peekable};
use std::ops::AddAssign;
use std::slice;
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};
use style_traits::values::SequenceWriter;
use values::CSSFloat;
use values::animated::{Animate, Procedure, ToAnimatedZero};
use values::distance::{ComputeSquaredDistance, SquaredDistance};


/// The SVG path data.
///
/// https://www.w3.org/TR/SVG11/paths.html#PathData
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToAnimatedZero,
         ToComputedValue)]
pub struct SVGPathData(Box<[PathCommand]>);

impl SVGPathData {
    /// Return SVGPathData by a slice of PathCommand.
    #[inline]
    pub fn new(cmd: Box<[PathCommand]>) -> Self {
        debug_assert!(!cmd.is_empty());
        SVGPathData(cmd)
    }

    /// Get the array of PathCommand.
    #[inline]
    pub fn commands(&self) -> &[PathCommand] {
        debug_assert!(!self.0.is_empty());
        &self.0
    }

    /// Create a normalized copy of this path by converting each relative command to an absolute
    /// command.
    fn normalize(&self) -> Self {
        let mut state = PathTraversalState {
            subpath_start: CoordPair::new(0.0, 0.0),
            pos: CoordPair::new(0.0, 0.0),
        };
        let result = self.0.iter().map(|seg| seg.normalize(&mut state)).collect::<Vec<_>>();
        SVGPathData(result.into_boxed_slice())
    }
}

impl ToCss for SVGPathData {
    #[inline]
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write
    {
        dest.write_char('"')?;
        {
            let mut writer = SequenceWriter::new(dest, " ");
            for command in self.0.iter() {
                writer.item(command)?;
            }
        }
        dest.write_char('"')
    }
}

impl Parse for SVGPathData {
    // We cannot use cssparser::Parser to parse a SVG path string because the spec wants to make
    // the SVG path string as compact as possible. (i.e. The whitespaces may be dropped.)
    // e.g. "M100 200L100 200" is a valid SVG path string. If we use tokenizer, the first ident
    // is "M100", instead of "M", and this is not correct. Therefore, we use a Peekable
    // str::Char iterator to check each character.
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        let path_string = input.expect_string()?.as_ref();
        if path_string.is_empty() {
            // Treat an empty string as invalid, so we will not set it.
            return Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        // Parse the svg path string as multiple sub-paths.
        let mut path_parser = PathParser::new(path_string);
        while skip_wsp(&mut path_parser.chars) {
            if path_parser.parse_subpath().is_err() {
                return Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError));
            }
        }

        Ok(SVGPathData::new(path_parser.path.into_boxed_slice()))
    }
}

impl Animate for SVGPathData {
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        if self.0.len() != other.0.len() {
            return Err(());
        }

        let result = self.normalize().0
            .iter()
            .zip(other.normalize().0.iter())
            .map(|(a, b)| a.animate(&b, procedure))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(SVGPathData::new(result.into_boxed_slice()))
    }
}

impl ComputeSquaredDistance for SVGPathData {
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        if self.0.len() != other.0.len() {
            return Err(());
        }
        self.normalize().0
            .iter()
            .zip(other.normalize().0.iter())
            .map(|(this, other)| this.compute_squared_distance(&other))
            .sum()
    }
}

/// The SVG path command.
/// The fields of these commands are self-explanatory, so we skip the documents.
/// Note: the index of the control points, e.g. control1, control2, are mapping to the control
/// points of the Bézier curve in the spec.
///
/// https://www.w3.org/TR/SVG11/paths.html#PathData
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf, PartialEq,
         SpecifiedValueInfo)]
#[allow(missing_docs)]
#[repr(C, u8)]
pub enum PathCommand {
    /// The unknown type.
    /// https://www.w3.org/TR/SVG/paths.html#__svg__SVGPathSeg__PATHSEG_UNKNOWN
    Unknown,
    /// The "moveto" command.
    MoveTo { point: CoordPair, absolute: bool },
    /// The "lineto" command.
    LineTo { point: CoordPair, absolute: bool },
    /// The horizontal "lineto" command.
    HorizontalLineTo { x: CSSFloat, absolute: bool },
    /// The vertical "lineto" command.
    VerticalLineTo { y: CSSFloat, absolute: bool },
    /// The cubic Bézier curve command.
    CurveTo { control1: CoordPair, control2: CoordPair, point: CoordPair, absolute: bool },
    /// The smooth curve command.
    SmoothCurveTo { control2: CoordPair, point: CoordPair, absolute: bool },
    /// The quadratic Bézier curve command.
    QuadBezierCurveTo { control1: CoordPair, point: CoordPair, absolute: bool },
    /// The smooth quadratic Bézier curve command.
    SmoothQuadBezierCurveTo { point: CoordPair, absolute: bool },
    /// The elliptical arc curve command.
    EllipticalArc {
        rx: CSSFloat,
        ry: CSSFloat,
        angle: CSSFloat,
        large_arc_flag: bool,
        sweep_flag: bool,
        point: CoordPair,
        absolute: bool
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
            MoveTo { mut point, absolute } => {
                if !absolute {
                    point += state.pos;
                }
                state.pos = point;
                state.subpath_start = point;
                MoveTo { point, absolute: true }
            },
            LineTo { mut point, absolute } => {
                if !absolute {
                    point += state.pos;
                }
                state.pos = point;
                LineTo { point, absolute: true }
            },
            HorizontalLineTo { mut x, absolute } => {
                if !absolute {
                    x += state.pos.0;
                }
                state.pos.0 = x;
                HorizontalLineTo { x, absolute: true }
            },
            VerticalLineTo { mut y, absolute } => {
                if !absolute {
                    y += state.pos.1;
                }
                state.pos.1 = y;
                VerticalLineTo { y, absolute: true }
            },
            CurveTo { mut control1, mut control2, mut point, absolute } => {
                if !absolute {
                    control1 += state.pos;
                    control2 += state.pos;
                    point += state.pos;
                }
                state.pos = point;
                CurveTo { control1, control2, point, absolute: true }
            },
            SmoothCurveTo { mut control2, mut point, absolute } => {
                if !absolute {
                    control2 += state.pos;
                    point += state.pos;
                }
                state.pos = point;
                SmoothCurveTo { control2, point, absolute: true }
            },
            QuadBezierCurveTo { mut control1, mut point, absolute } => {
                if !absolute {
                    control1 += state.pos;
                    point += state.pos;
                }
                state.pos = point;
                QuadBezierCurveTo { control1, point, absolute: true }
            },
            SmoothQuadBezierCurveTo { mut point, absolute } => {
                if !absolute {
                    point += state.pos;
                }
                state.pos = point;
                SmoothQuadBezierCurveTo { point, absolute: true }
            },
            EllipticalArc { rx, ry, angle, large_arc_flag, sweep_flag, mut point, absolute } => {
                if !absolute {
                    point += state.pos;
                }
                state.pos = point;
                EllipticalArc { rx, ry, angle, large_arc_flag, sweep_flag, point, absolute: true }
            },
        }
    }
}

impl ToCss for PathCommand {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write
    {
        use self::PathCommand::*;
        match *self {
            Unknown => dest.write_char('X'),
            ClosePath => dest.write_char('Z'),
            MoveTo { point, absolute } => {
                dest.write_char(if absolute { 'M' } else { 'm' })?;
                dest.write_char(' ')?;
                point.to_css(dest)
            }
            LineTo { point, absolute } => {
                dest.write_char(if absolute { 'L' } else { 'l' })?;
                dest.write_char(' ')?;
                point.to_css(dest)
            }
            CurveTo { control1, control2, point, absolute } => {
                dest.write_char(if absolute { 'C' } else { 'c' })?;
                dest.write_char(' ')?;
                control1.to_css(dest)?;
                dest.write_char(' ')?;
                control2.to_css(dest)?;
                dest.write_char(' ')?;
                point.to_css(dest)
            },
            QuadBezierCurveTo { control1, point, absolute } => {
                dest.write_char(if absolute { 'Q' } else { 'q' })?;
                dest.write_char(' ')?;
                control1.to_css(dest)?;
                dest.write_char(' ')?;
                point.to_css(dest)
            },
            EllipticalArc { rx, ry, angle, large_arc_flag, sweep_flag, point, absolute } => {
                dest.write_char(if absolute { 'A' } else { 'a' })?;
                dest.write_char(' ')?;
                rx.to_css(dest)?;
                dest.write_char(' ')?;
                ry.to_css(dest)?;
                dest.write_char(' ')?;
                angle.to_css(dest)?;
                dest.write_char(' ')?;
                (large_arc_flag as i32).to_css(dest)?;
                dest.write_char(' ')?;
                (sweep_flag as i32).to_css(dest)?;
                dest.write_char(' ')?;
                point.to_css(dest)
            },
            HorizontalLineTo { x, absolute } => {
                dest.write_char(if absolute { 'H' } else { 'h' })?;
                dest.write_char(' ')?;
                x.to_css(dest)
            },
            VerticalLineTo { y, absolute } => {
                dest.write_char(if absolute { 'V' } else { 'v' })?;
                dest.write_char(' ')?;
                y.to_css(dest)
            },
            SmoothCurveTo { control2, point, absolute } => {
                dest.write_char(if absolute { 'S' } else { 's' })?;
                dest.write_char(' ')?;
                control2.to_css(dest)?;
                dest.write_char(' ')?;
                point.to_css(dest)
            },
            SmoothQuadBezierCurveTo { point, absolute } => {
                dest.write_char(if absolute { 'T' } else { 't' })?;
                dest.write_char(' ')?;
                point.to_css(dest)
            },
        }
    }
}

impl ToAnimatedZero for PathCommand {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        use self::PathCommand::*;
        let absolute = true;
        match self {
            &ClosePath => Ok(ClosePath),
            &Unknown => Ok(Unknown),
            &MoveTo { ref point, .. } => Ok(MoveTo { point: point.to_animated_zero()?, absolute }),
            &LineTo { ref point, .. } => Ok(LineTo { point: point.to_animated_zero()?, absolute }),
            &HorizontalLineTo { x, .. } => {
                Ok(HorizontalLineTo { x: x.to_animated_zero()?, absolute })
            },
            &VerticalLineTo { y, .. } => {
                Ok(VerticalLineTo { y: y.to_animated_zero()?, absolute })
            },
            &CurveTo { ref control1, ref control2, ref point, .. } => {
                Ok(CurveTo {
                    control1: control1.to_animated_zero()?,
                    control2: control2.to_animated_zero()?,
                    point: point.to_animated_zero()?,
                    absolute,
                })
            },
            &SmoothCurveTo { ref control2, ref point, .. } => {
                Ok(SmoothCurveTo {
                    control2: control2.to_animated_zero()?,
                    point: point.to_animated_zero()?,
                    absolute,
                })
            },
            &QuadBezierCurveTo { ref control1, ref point, .. } => {
                Ok(QuadBezierCurveTo {
                    control1: control1.to_animated_zero()?,
                    point: point.to_animated_zero()?,
                    absolute,
                })
            },
            &SmoothQuadBezierCurveTo { ref point, .. } => {
                Ok(SmoothQuadBezierCurveTo { point: point.to_animated_zero()?, absolute })
            },
            &EllipticalArc { rx, ry, angle, large_arc_flag, sweep_flag, ref point, .. } => {
                Ok(EllipticalArc {
                    rx: rx.to_animated_zero()?,
                    ry: ry.to_animated_zero()?,
                    angle: angle.to_animated_zero()?,
                    large_arc_flag,
                    sweep_flag,
                    point: point.to_animated_zero()?,
                    absolute,
                })
            },
        }
    }
}


/// The path coord type.
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf, PartialEq,
         SpecifiedValueInfo, ToAnimatedZero, ToCss)]
#[repr(C)]
pub struct CoordPair(CSSFloat, CSSFloat);

impl CoordPair {
    /// Create a CoordPair.
    #[inline]
    pub fn new(x: CSSFloat, y: CSSFloat) -> Self {
        CoordPair(x, y)
    }
}

impl AddAssign for CoordPair {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
        self.1 += other.1;
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

            match self.chars.next() {
                Some(command) => {
                    let abs = command.is_ascii_uppercase();
                    macro_rules! parse_command {
                        ( $($($p:pat)|+ => $parse_func:ident,)* ) => {
                            match command {
                                $(
                                    $($p)|+ => {
                                        skip_wsp(&mut self.chars);
                                        self.$parse_func(abs)?;
                                    },
                                )*
                                _ => return Err(()),
                            }
                        }
                    }
                    parse_command!(
                        b'Z' | b'z' => parse_closepath,
                        b'L' | b'l' => parse_lineto,
                        b'H' | b'h' => parse_h_lineto,
                        b'V' | b'v' => parse_v_lineto,
                        b'C' | b'c' => parse_curveto,
                        b'S' | b's' => parse_smooth_curveto,
                        b'Q' | b'q' => parse_quadratic_bezier_curveto,
                        b'T' | b't' => parse_smooth_quadratic_bezier_curveto,
                        b'A' | b'a' => parse_elliprical_arc,
                    );
                },
                _ => break, // no more commands.
            }
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
        let absolute = command == b'M';
        self.path.push(PathCommand::MoveTo { point, absolute } );

        // End of string or the next character is a possible new command.
        if !skip_wsp(&mut self.chars) ||
           self.chars.peek().map_or(true, |c| c.is_ascii_alphabetic()) {
            return Ok(());
        }
        skip_comma_wsp(&mut self.chars);

        // If a moveto is followed by multiple pairs of coordinates, the subsequent
        // pairs are treated as implicit lineto commands.
        self.parse_lineto(absolute)
    }

    /// Parse "closepath" command.
    fn parse_closepath(&mut self, _absolute: bool) -> Result<(), ()> {
        self.path.push(PathCommand::ClosePath);
        Ok(())
    }

    /// Parse "lineto" command.
    fn parse_lineto(&mut self, absolute: bool) -> Result<(), ()> {
        parse_arguments!(self, absolute, LineTo, [ point => parse_coord ])
    }

    /// Parse horizontal "lineto" command.
    fn parse_h_lineto(&mut self, absolute: bool) -> Result<(), ()> {
        parse_arguments!(self, absolute, HorizontalLineTo, [ x => parse_number ])
    }

    /// Parse vertical "lineto" command.
    fn parse_v_lineto(&mut self, absolute: bool) -> Result<(), ()> {
        parse_arguments!(self, absolute, VerticalLineTo, [ y => parse_number ])
    }

    /// Parse cubic Bézier curve command.
    fn parse_curveto(&mut self, absolute: bool) -> Result<(), ()> {
        parse_arguments!(self, absolute, CurveTo, [
            control1 => parse_coord, control2 => parse_coord, point => parse_coord
        ])
    }

    /// Parse smooth "curveto" command.
    fn parse_smooth_curveto(&mut self, absolute: bool) -> Result<(), ()> {
        parse_arguments!(self, absolute, SmoothCurveTo, [
            control2 => parse_coord, point => parse_coord
        ])
    }

    /// Parse quadratic Bézier curve command.
    fn parse_quadratic_bezier_curveto(&mut self, absolute: bool) -> Result<(), ()> {
        parse_arguments!(self, absolute, QuadBezierCurveTo, [
            control1 => parse_coord, point => parse_coord
        ])
    }

    /// Parse smooth quadratic Bézier curveto command.
    fn parse_smooth_quadratic_bezier_curveto(&mut self, absolute: bool) -> Result<(), ()> {
        parse_arguments!(self, absolute, SmoothQuadBezierCurveTo, [ point => parse_coord ])
    }

    /// Parse elliptical arc curve command.
    fn parse_elliprical_arc(&mut self, absolute: bool) -> Result<(), ()> {
        // Parse a flag whose value is '0' or '1'; otherwise, return Err(()).
        let parse_flag = |iter: &mut Peekable<Cloned<slice::Iter<u8>>>| -> Result<bool, ()> {
            match iter.next() {
                Some(c) if c == b'0' || c == b'1' => Ok(c == b'1'),
                _ => Err(()),
            }
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
    let sign = if iter.peek().map_or(false, |&sign| sign == b'+' || sign == b'-') {
        if iter.next().unwrap() == b'-' { -1. } else { 1. }
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
            integral_part =
                integral_part * 10. + (iter.next().unwrap() - b'0') as f64;
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
        let exp_sign = if iter.peek().map_or(false, |&sign| sign == b'+' || sign == b'-') {
            if iter.next().unwrap() == b'-' { -1. } else { 1. }
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
        Ok(value.min(::std::f32::MAX as f64).max(::std::f32::MIN as f64) as CSSFloat)
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
