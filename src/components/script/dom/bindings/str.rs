/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub struct ByteString(Vec<u8>);

impl ByteString {
    pub fn new(value: Vec<u8>) -> ByteString {
        ByteString(value)
    }

    pub fn as_slice<'a>(&'a self) -> &'a [u8] {
        let ByteString(ref vector) = *self;
        vector.as_slice()
    }
}
