/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use std::ascii::StrAsciiExt;
use cssparser::ast::{ComponentValue, Ident, SkipWhitespaceIterable, SkipWhitespaceIterator};


pub fn one_component_value<'a>(input: &'a [ComponentValue]) -> Result<&'a ComponentValue, ()> {
    let mut iter = input.skip_whitespace();
    match iter.next() {
        Some(value) => if iter.next().is_none() { Ok(value) } else { Err(()) },
        None => Err(())
    }
}


pub fn get_ident_lower(component_value: &ComponentValue) -> Result<String, ()> {
    match component_value {
        &Ident(ref value) => Ok(value.as_slice().to_ascii_lower()),
        _ => Err(()),
    }
}


pub struct BufferedIter<E, I> {
    iter: I,
    buffer: Option<E>,
}

impl<E, I: Iterator<E>> BufferedIter<E, I> {
    pub fn new(iter: I) -> BufferedIter<E, I> {
        BufferedIter {
            iter: iter,
            buffer: None,
        }
    }

    #[inline]
    pub fn push_back(&mut self, value: E) {
        assert!(self.buffer.is_none());
        self.buffer = Some(value);
    }
}

impl<E, I: Iterator<E>> Iterator<E> for BufferedIter<E, I> {
    #[inline]
    fn next(&mut self) -> Option<E> {
        if self.buffer.is_some() {
            self.buffer.take()
        }
        else {
            self.iter.next()
        }
    }
}


pub type ParserIter<'a> = BufferedIter<&'a ComponentValue, SkipWhitespaceIterator<'a>>;
