/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Helper to iterate over `OriginFlags` bits.

use gecko_bindings::structs::OriginFlags;
use gecko_bindings::structs::OriginFlags_Author;
use gecko_bindings::structs::OriginFlags_User;
use gecko_bindings::structs::OriginFlags_UserAgent;
use stylesheets::Origin;

impl OriginFlags {
    /// Returns an iterator over the origins present in the `OriginFlags`,
    /// in order from highest priority (author) to lower (user agent).
    pub fn iter(self) -> OriginFlagsIter {
        OriginFlagsIter {
            origin_flags: self,
            cur: 0,
        }
    }
}

/// Iterates over the origins present in an `OriginFlags`, in order from
/// highest priority (author) to lower (user agent).
pub struct OriginFlagsIter {
    origin_flags: OriginFlags,
    cur: usize,
}

impl Iterator for OriginFlagsIter {
    type Item = Origin;

    fn next(&mut self) -> Option<Origin> {
        loop {
            let (bit, origin) = match self.cur {
                0 => (OriginFlags_Author, Origin::Author),
                1 => (OriginFlags_User, Origin::User),
                2 => (OriginFlags_UserAgent, Origin::UserAgent),
                _ => return None,
            };

            self.cur += 1;

            if (self.origin_flags & bit).0 != 0 {
                return Some(origin);
            }
        }
    }
}
