/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use util::str::DOMString;

pub mod html;
pub mod xml;

pub trait Parser {
    fn parse_chunk(self, input: Chunk);
    fn finish(self);
}

#[derive(JSTraceable, HeapSizeOf, Debug)]
pub enum Chunk {
    Bytes(Vec<u8>),
    Dom(DOMString),
}
