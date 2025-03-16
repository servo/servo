// Copyright 2018 the SVG Types Authors
// Copyright 2025 the Servo Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::svgpath::Error;

/// A streaming text parsing interface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Stream<'a> {
    text: &'a str,
    pos: usize,
}

impl<'a> From<&'a str> for Stream<'a> {
    #[inline]
    fn from(text: &'a str) -> Self {
        Stream { text, pos: 0 }
    }
}

impl<'a> Stream<'a> {
    /// Returns the current position in bytes.
    #[inline]
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Sets current position equal to the end.
    ///
    /// Used to indicate end of parsing on error.
    #[inline]
    pub fn jump_to_end(&mut self) {
        self.pos = self.text.len();
    }

    /// Checks if the stream is reached the end.
    ///
    /// Any [`pos()`] value larger than original text length indicates stream end.
    ///
    /// Accessing stream after reaching end via safe methods will produce
    /// an error.
    ///
    /// Accessing stream after reaching end via *_unchecked methods will produce
    /// a Rust's bound checking error.
    ///
    /// [`pos()`]: #method.pos
    #[inline]
    pub fn at_end(&self) -> bool {
        self.pos >= self.text.len()
    }

    /// Returns a byte from a current stream position.
    #[inline]
    pub fn curr_byte(&self) -> Result<u8, Error> {
        if self.at_end() {
            return Err(Error);
        }

        Ok(self.curr_byte_unchecked())
    }

    /// Returns a byte from a current stream position.
    ///
    /// # Panics
    ///
    /// - if the current position is after the end of the data
    #[inline]
    pub fn curr_byte_unchecked(&self) -> u8 {
        self.text.as_bytes()[self.pos]
    }

    /// Checks that current byte is equal to provided.
    ///
    /// Returns `false` if no bytes left.
    #[inline]
    pub fn is_curr_byte_eq(&self, c: u8) -> bool {
        if !self.at_end() {
            self.curr_byte_unchecked() == c
        } else {
            false
        }
    }

    /// Returns a next byte from a current stream position.
    #[inline]
    pub fn next_byte(&self) -> Result<u8, Error> {
        if self.pos + 1 >= self.text.len() {
            return Err(Error);
        }

        Ok(self.text.as_bytes()[self.pos + 1])
    }

    /// Advances by `n` bytes.
    #[inline]
    pub fn advance(&mut self, n: usize) {
        debug_assert!(self.pos + n <= self.text.len());
        self.pos += n;
    }

    /// Skips whitespaces.
    ///
    /// Accepted values: `' ' \n \r \t`.
    pub fn skip_spaces(&mut self) {
        while !self.at_end() && matches!(self.curr_byte_unchecked(), b' ' | b'\t' | b'\n' | b'\r') {
            self.advance(1);
        }
    }

    /// Consumes bytes by the predicate.
    pub fn skip_bytes<F>(&mut self, f: F)
    where
        F: Fn(&Stream<'_>, u8) -> bool,
    {
        while !self.at_end() {
            let c = self.curr_byte_unchecked();
            if f(self, c) {
                self.advance(1);
            } else {
                break;
            }
        }
    }

    /// Slices data from `pos` to the current position.
    #[inline]
    pub fn slice_back(&self, pos: usize) -> &'a str {
        &self.text[pos..self.pos]
    }

    /// Skips digits.
    pub fn skip_digits(&mut self) {
        self.skip_bytes(|_, c| c.is_ascii_digit());
    }

    #[inline]
    pub(crate) fn parse_list_separator(&mut self) {
        if self.is_curr_byte_eq(b',') {
            self.advance(1);
        }
    }

    // By the SVG spec 'large-arc' and 'sweep' must contain only one char
    // and can be written without any separators, e.g.: 10 20 30 01 10 20.
    pub(crate) fn parse_flag(&mut self) -> Result<bool, Error> {
        self.skip_spaces();

        let c = self.curr_byte()?;
        match c {
            b'0' | b'1' => {
                self.advance(1);
                if self.is_curr_byte_eq(b',') {
                    self.advance(1);
                }
                self.skip_spaces();

                Ok(c == b'1')
            },
            _ => Err(Error),
        }
    }
}
