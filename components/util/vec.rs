/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// TODO(pcwalton): Speed up with SIMD, or better yet, find some way to not do this.
pub fn byte_swap(data: &mut [u8]) {
    let length = data.len();
    // FIXME(rust #27741): Range::step_by is not stable yet as of this writing.
    let mut i = 0;
    while i < length {
        let r = data[i + 2];
        data[i + 2] = data[i + 0];
        data[i + 0] = r;
        i += 4;
    }
}
