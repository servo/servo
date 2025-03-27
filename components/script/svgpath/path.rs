// Copyright 2021 the SVG Types Authors
// Copyright 2025 the Servo Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use canvas_traits::canvas::PathSegment;

use crate::svgpath::{Error, Stream};

pub struct PathParser<'a> {
    stream: Stream<'a>,
    state: State,
    last_cmd: u8,
}

impl<'a> PathParser<'a> {
    pub fn new(string: &'a str) -> Self {
        Self {
            stream: Stream::from(string),
            state: State::default(),
            last_cmd: b' ',
        }
    }
}

impl Iterator for PathParser<'_> {
    type Item = Result<PathSegment, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.stream.skip_spaces();

        let Ok(curr_byte) = self.stream.curr_byte() else {
            return None;
        };

        let cmd = if self.last_cmd == b' ' {
            if let move_to @ (b'm' | b'M') = curr_byte {
                self.stream.advance(1);
                move_to
            } else {
                return Some(Err(Error));
            }
        } else if curr_byte.is_ascii_alphabetic() {
            self.stream.advance(1);
            curr_byte
        } else {
            match self.last_cmd {
                b'm' => b'l',
                b'M' => b'L',
                b'z' | b'Z' => return Some(Err(Error)),
                cmd => cmd,
            }
        };

        self.last_cmd = cmd;
        Some(to_point(&mut self.stream, cmd, &mut self.state))
    }
}

#[derive(Default)]
pub struct State {
    start: (f32, f32),
    pos: (f32, f32),
    quad: (f32, f32),
    cubic: (f32, f32),
}

pub fn to_point(s: &mut Stream<'_>, cmd: u8, state: &mut State) -> Result<PathSegment, Error> {
    let abs = cmd.is_ascii_uppercase();
    let cmd = cmd.to_ascii_lowercase();
    let (dx, dy) = if abs { (0., 0.) } else { state.pos };
    let seg = match cmd {
        b'm' => PathSegment::MoveTo {
            x: s.parse_list_number()? + dx,
            y: s.parse_list_number()? + dy,
        },
        b'l' => PathSegment::LineTo {
            x: s.parse_list_number()? + dx,
            y: s.parse_list_number()? + dy,
        },
        b'h' => PathSegment::LineTo {
            x: s.parse_list_number()? + dx,
            y: state.pos.1,
        },
        b'v' => PathSegment::LineTo {
            x: state.pos.0,
            y: s.parse_list_number()? + dy,
        },
        b'c' => PathSegment::Bezier {
            cp1x: s.parse_list_number()? + dx,
            cp1y: s.parse_list_number()? + dy,
            cp2x: s.parse_list_number()? + dx,
            cp2y: s.parse_list_number()? + dy,
            x: s.parse_list_number()? + dx,
            y: s.parse_list_number()? + dy,
        },
        b's' => PathSegment::Bezier {
            cp1x: state.cubic.0,
            cp1y: state.cubic.1,
            cp2x: s.parse_list_number()? + dx,
            cp2y: s.parse_list_number()? + dy,
            x: s.parse_list_number()? + dx,
            y: s.parse_list_number()? + dy,
        },
        b'q' => PathSegment::Quadratic {
            cpx: s.parse_list_number()? + dx,
            cpy: s.parse_list_number()? + dy,
            x: s.parse_list_number()? + dx,
            y: s.parse_list_number()? + dy,
        },
        b't' => PathSegment::Quadratic {
            cpx: state.quad.0,
            cpy: state.quad.1,
            x: s.parse_list_number()? + dx,
            y: s.parse_list_number()? + dy,
        },
        b'a' => PathSegment::SvgArc {
            radius_x: s.parse_list_number()?,
            radius_y: s.parse_list_number()?,
            rotation: s.parse_list_number()?,
            large_arc: s.parse_flag()?,
            sweep: s.parse_flag()?,
            x: s.parse_list_number()? + dx,
            y: s.parse_list_number()? + dy,
        },
        b'z' => PathSegment::ClosePath,
        _ => return Err(crate::svgpath::Error),
    };

    match seg {
        PathSegment::MoveTo { x, y } => {
            state.start = (x, y);
            state.pos = (x, y);
            state.quad = (x, y);
            state.cubic = (x, y);
        },
        PathSegment::LineTo { x, y } | PathSegment::SvgArc { x, y, .. } => {
            state.pos = (x, y);
            state.quad = (x, y);
            state.cubic = (x, y);
        },
        PathSegment::Bezier {
            cp2x, cp2y, x, y, ..
        } => {
            state.pos = (x, y);
            state.quad = (x, y);
            state.cubic = (x * 2.0 - cp2x, y * 2.0 - cp2y);
        },
        PathSegment::Quadratic { cpx, cpy, x, y, .. } => {
            state.pos = (x, y);
            state.quad = (x * 2.0 - cpx, y * 2.0 - cpy);
            state.cubic = (x, y);
        },
        PathSegment::ClosePath => {
            state.pos = state.start;
            state.quad = state.start;
            state.cubic = state.start;
        },
        _ => {},
    }

    Ok(seg)
}

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test {
        ($name:ident, $text:expr, $( $seg:expr ),*) => (
            #[test]
            fn $name() {
                let mut s = PathParser::new($text);
                $(
                    assert_eq!(s.next().unwrap().unwrap(), $seg);
                )*

                if let Some(res) = s.next() {
                    assert!(res.is_err());
                }
            }
        )
    }

    test!(null, "", );
    test!(not_a_path, "q", );
    test!(not_a_move_to, "L 20 30", );
    test!(stop_on_err_1, "M 10 20 L 30 40 L 50",
        PathSegment::MoveTo { x: 10.0, y: 20.0 },
        PathSegment::LineTo { x: 30.0, y: 40.0 }
    );

    test!(move_to_1, "M 10 20", PathSegment::MoveTo { x: 10.0, y: 20.0 });
    test!(move_to_2, "m 10 20", PathSegment::MoveTo { x: 10.0, y: 20.0 });
    test!(move_to_3, "M 10 20 30 40 50 60",
        PathSegment::MoveTo { x: 10.0, y: 20.0 },
        PathSegment::LineTo { x: 30.0, y: 40.0 },
        PathSegment::LineTo { x: 50.0, y: 60.0 }
    );
    test!(move_to_4, "M 10 20 30 40 50 60 M 70 80 90 100 110 120",
        PathSegment::MoveTo { x: 10.0, y: 20.0 },
        PathSegment::LineTo { x: 30.0, y: 40.0 },
        PathSegment::LineTo { x: 50.0, y: 60.0 },
        PathSegment::MoveTo { x: 70.0, y: 80.0 },
        PathSegment::LineTo { x: 90.0, y: 100.0 },
        PathSegment::LineTo { x: 110.0, y: 120.0 }
    );

    test!(arc_to_1, "M 10 20 A 5 5 30 1 1 20 20",
        PathSegment::MoveTo { x: 10.0, y: 20.0 },
        PathSegment::SvgArc {
            radius_x: 5.0, radius_y: 5.0,
            rotation: 30.0,
            large_arc: true, sweep: true,
            x: 20.0, y: 20.0
        }
    );

    test!(arc_to_2, "M 10 20 a 5 5 30 0 0 20 20",
        PathSegment::MoveTo { x: 10.0, y: 20.0 },
        PathSegment::SvgArc {
            radius_x: 5.0, radius_y: 5.0,
            rotation: 30.0,
            large_arc: false, sweep: false,
            x: 30.0, y: 40.0
        }
    );

    test!(arc_to_10, "M10-20A5.5.3-4 010-.1",
        PathSegment::MoveTo { x: 10.0, y: -20.0 },
        PathSegment::SvgArc {
            radius_x: 5.5, radius_y: 0.3,
            rotation: -4.0,
            large_arc: false, sweep: true,
            x: 0.0, y: -0.1
        }
    );

    test!(separator_1, "M 10 20 L 5 15 C 10 20 30 40 50 60",
        PathSegment::MoveTo { x: 10.0, y: 20.0 },
        PathSegment::LineTo { x: 5.0, y: 15.0 },
        PathSegment::Bezier {
            cp1x: 10.0, cp1y: 20.0,
            cp2x: 30.0, cp2y: 40.0,
            x:    50.0, y:    60.0,
        }
    );

    test!(separator_2, "M 10, 20 L 5, 15 C 10, 20 30, 40 50, 60",
        PathSegment::MoveTo { x: 10.0, y: 20.0 },
        PathSegment::LineTo { x: 5.0, y: 15.0 },
        PathSegment::Bezier {
            cp1x: 10.0, cp1y: 20.0,
            cp2x: 30.0, cp2y: 40.0,
            x:    50.0, y:    60.0,
        }
    );

    test!(separator_3, "M 10,20 L 5,15 C 10,20 30,40 50,60",
        PathSegment::MoveTo { x: 10.0, y: 20.0 },
        PathSegment::LineTo { x: 5.0, y: 15.0 },
        PathSegment::Bezier {
            cp1x: 10.0, cp1y: 20.0,
            cp2x: 30.0, cp2y: 40.0,
            x:    50.0, y:    60.0,
        }
    );

    test!(separator_4, "M10, 20 L5, 15 C10, 20 30 40 50 60",
        PathSegment::MoveTo { x: 10.0, y: 20.0 },
        PathSegment::LineTo { x: 5.0, y: 15.0 },
        PathSegment::Bezier {
            cp1x: 10.0, cp1y: 20.0,
            cp2x: 30.0, cp2y: 40.0,
            x:    50.0, y:    60.0,
        }
    );

    test!(separator_5, "M10 20V30H40V50H60Z",
        PathSegment::MoveTo { x: 10.0, y: 20.0 },
        PathSegment::LineTo { x: 10.0, y: 30.0 },
        PathSegment::LineTo { x: 40.0, y: 30.0 },
        PathSegment::LineTo { x: 40.0, y: 50.0 },
        PathSegment::LineTo { x: 60.0, y: 50.0 },
        PathSegment::ClosePath
    );

    test!(all_segments_1, "M 10 20 L 30 40 H 50 V 60 C 70 80 90 100 110 120 S 130 140 150 160
        Q 170 180 190 200 T 210 220 A 50 50 30 1 1 230 240 Z",
        PathSegment::MoveTo { x: 10.0, y: 20.0 },
        PathSegment::LineTo { x: 30.0, y: 40.0 },
        PathSegment::LineTo { x: 50.0, y: 40.0 },
        PathSegment::LineTo { x: 50.0, y: 60.0 },
        PathSegment::Bezier {
            cp1x:  70.0, cp1y:  80.0,
            cp2x:  90.0, cp2y: 100.0,
            x:    110.0, y:    120.0,
        },
        PathSegment::Bezier {
            cp1x: 130.0, cp1y: 140.0,
            cp2x: 130.0, cp2y: 140.0,
            x:    150.0, y:    160.0,
        },
        PathSegment::Quadratic {
            cpx: 170.0, cpy: 180.0,
            x:   190.0, y:   200.0,
        },
        PathSegment::Quadratic {
            cpx: 210.0, cpy: 220.0,
            x:   210.0, y:   220.0,
        },
        PathSegment::SvgArc {
            radius_x: 50.0, radius_y: 50.0,
            rotation: 30.0,
            large_arc: true, sweep: true,
            x: 230.0, y: 240.0
        },
        PathSegment::ClosePath
    );

    test!(all_segments_2, "m 10 20 l 30 40 h 50 v 60 c 70 80 90 100 110 120 s 130 140 150 160
        q 170 180 190 200 t 210 220 a 50 50 30 1 1 230 240 z",
        PathSegment::MoveTo { x: 10.0, y: 20.0 },
        PathSegment::LineTo { x: 40.0, y: 60.0 },
        PathSegment::LineTo { x: 90.0, y: 60.0 },
        PathSegment::LineTo { x: 90.0, y: 120.0 },
        PathSegment::Bezier {
            cp1x: 160.0, cp1y: 200.0,
            cp2x: 180.0, cp2y: 220.0,
            x:    200.0, y:    240.0,
        },
        PathSegment::Bezier {
            cp1x: 220.0, cp1y: 260.0, //?
            cp2x: 330.0, cp2y: 380.0,
            x:    350.0, y:    400.0,
        },
        PathSegment::Quadratic {
            cpx: 520.0, cpy: 580.0,
            x:   540.0, y:   600.0,
        },
        PathSegment::Quadratic {
            cpx: 560.0, cpy: 620.0, //?
            x:   750.0, y:   820.0
        },
        PathSegment::SvgArc {
            radius_x: 50.0, radius_y: 50.0,
            rotation: 30.0,
            large_arc: true, sweep: true,
            x: 980.0, y: 1060.0
        },
        PathSegment::ClosePath
    );

    test!(close_path_1, "M10 20 L 30 40 ZM 100 200 L 300 400",
        PathSegment::MoveTo { x: 10.0, y: 20.0 },
        PathSegment::LineTo { x: 30.0, y: 40.0 },
        PathSegment::ClosePath,
        PathSegment::MoveTo { x: 100.0, y: 200.0 },
        PathSegment::LineTo { x: 300.0, y: 400.0 }
    );

    test!(close_path_2, "M10 20 L 30 40 zM 100 200 L 300 400",
        PathSegment::MoveTo { x: 10.0, y: 20.0 },
        PathSegment::LineTo { x: 30.0, y: 40.0 },
        PathSegment::ClosePath,
        PathSegment::MoveTo { x: 100.0, y: 200.0 },
        PathSegment::LineTo { x: 300.0, y: 400.0 }
    );

    test!(close_path_3, "M10 20 L 30 40 Z Z Z",
        PathSegment::MoveTo { x: 10.0, y: 20.0 },
        PathSegment::LineTo { x: 30.0, y: 40.0 },
        PathSegment::ClosePath,
        PathSegment::ClosePath,
        PathSegment::ClosePath
    );

    // first token should be EndOfStream
    test!(invalid_1, "M\t.", );

    // ClosePath can't be followed by a number
    test!(invalid_2, "M 0 0 Z 2",
        PathSegment::MoveTo { x: 0.0, y: 0.0 },
        PathSegment::ClosePath
    );

    // ClosePath can be followed by any command
    test!(invalid_3, "M 0 0 Z H 10",
        PathSegment::MoveTo { x: 0.0, y: 0.0 },
        PathSegment::ClosePath,
        PathSegment::LineTo { x: 10.0, y: 0.0 }
    );
}
