// Copyright 2018 the SVG Types Authors
// Copyright 2025 the Servo Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::str::FromStr;

use crate::svgpath::{Error, Stream};

/// An [SVG number](https://www.w3.org/TR/SVG2/types.html#InterfaceSVGNumber).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Number(pub f32);

impl std::str::FromStr for Number {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let mut s = Stream::from(text);
        let n = s.parse_number()?;
        s.skip_spaces();
        if !s.at_end() {
            return Err(Error);
        }

        Ok(Self(n))
    }
}

impl Stream<'_> {
    /// Parses number from the stream.
    ///
    /// This method will detect a number length and then
    /// will pass a substring to the `f64::from_str` method.
    ///
    /// <https://www.w3.org/TR/SVG2/types.html#InterfaceSVGNumber>
    pub fn parse_number(&mut self) -> Result<f32, Error> {
        // Strip off leading whitespaces.
        self.skip_spaces();

        if self.at_end() {
            return Err(Error);
        }

        self.parse_number_impl().map_err(|_| Error)
    }

    fn parse_number_impl(&mut self) -> Result<f32, Error> {
        let start = self.pos();

        let mut c = self.curr_byte()?;

        // Consume sign.
        if matches!(c, b'+' | b'-') {
            self.advance(1);
            c = self.curr_byte()?;
        }

        // Consume integer.
        match c {
            b'0'..=b'9' => self.skip_digits(),
            b'.' => {},
            _ => return Err(Error),
        }

        // Consume fraction.
        if let Ok(b'.') = self.curr_byte() {
            self.advance(1);
            self.skip_digits();
        }

        if let Ok(c) = self.curr_byte() {
            if matches!(c, b'e' | b'E') {
                let c2 = self.next_byte()?;
                // Check for `em`/`ex`.
                if c2 != b'm' && c2 != b'x' {
                    self.advance(1);

                    match self.curr_byte()? {
                        b'+' | b'-' => {
                            self.advance(1);
                            self.skip_digits();
                        },
                        b'0'..=b'9' => self.skip_digits(),
                        _ => {
                            return Err(Error);
                        },
                    }
                }
            }
        }

        let s = self.slice_back(start);

        // Use the default f32 parser now.
        if let Ok(n) = f32::from_str(s) {
            // inf, nan, etc. are an error.
            if n.is_finite() {
                return Ok(n);
            }
        }

        Err(Error)
    }

    /// Parses number from a list of numbers.
    pub fn parse_list_number(&mut self) -> Result<f32, Error> {
        if self.at_end() {
            return Err(Error);
        }

        let n = self.parse_number()?;
        self.skip_spaces();
        self.parse_list_separator();
        Ok(n)
    }
}

/// A pull-based [`<list-of-numbers>`] parser.
///
/// # Examples
///
/// ```
/// use svgtypes::NumberListParser;
///
/// let mut p = NumberListParser::from("10, 20 -50");
/// assert_eq!(p.next().unwrap().unwrap(), 10.0);
/// assert_eq!(p.next().unwrap().unwrap(), 20.0);
/// assert_eq!(p.next().unwrap().unwrap(), -50.0);
/// assert_eq!(p.next().is_none(), true);
/// ```
///
/// [`<list-of-numbers>`]: https://www.w3.org/TR/SVG2/types.html#InterfaceSVGNumberList
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NumberListParser<'a>(Stream<'a>);

impl<'a> From<&'a str> for NumberListParser<'a> {
    #[inline]
    fn from(v: &'a str) -> Self {
        NumberListParser(Stream::from(v))
    }
}

impl Iterator for NumberListParser<'_> {
    type Item = Result<f32, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.at_end() {
            None
        } else {
            let v = self.0.parse_list_number();
            if v.is_err() {
                self.0.jump_to_end();
            }

            Some(v)
        }
    }
}

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use crate::svgpath::Stream;

    macro_rules! test_p {
        ($name:ident, $text:expr, $result:expr) => (
            #[test]
            fn $name() {
                let mut s = Stream::from($text);
                assert_eq!(s.parse_number().unwrap(), $result);
            }
        )
    }

    test_p!(parse_1,  "0", 0.0);
    test_p!(parse_2,  "1", 1.0);
    test_p!(parse_3,  "-1", -1.0);
    test_p!(parse_4,  " -1 ", -1.0);
    test_p!(parse_5,  "  1  ", 1.0);
    test_p!(parse_6,  ".4", 0.4);
    test_p!(parse_7,  "-.4", -0.4);
    test_p!(parse_8,  "-.4text", -0.4);
    test_p!(parse_9,  "-.01 text", -0.01);
    test_p!(parse_10, "-.01 4", -0.01);
    test_p!(parse_11, ".0000000000008", 0.0000000000008);
    test_p!(parse_12, "1000000000000", 1000000000000.0);
    test_p!(parse_13, "123456.123456", 123456.123456);
    test_p!(parse_14, "+10", 10.0);
    test_p!(parse_15, "1e2", 100.0);
    test_p!(parse_16, "1e+2", 100.0);
    test_p!(parse_17, "1E2", 100.0);
    test_p!(parse_18, "1e-2", 0.01);
    test_p!(parse_19, "1ex", 1.0);
    test_p!(parse_20, "1em", 1.0);
    test_p!(parse_21, "12345678901234567890", 12345678901234567000.0);
    test_p!(parse_22, "0.", 0.0);
    test_p!(parse_23, "1.3e-2", 0.013);
    // test_number!(parse_24, "1e", 1.0); // TODO: this
}
