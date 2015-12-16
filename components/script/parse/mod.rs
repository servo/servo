/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub mod html;
pub mod xml;

pub trait Parser {
    fn parse_chunk(self, input: String);
    fn finish(self);
}
