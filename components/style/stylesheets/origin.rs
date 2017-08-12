/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

///! [CSS cascade origins](https://drafts.csswg.org/css-cascade/#cascading-origins).

/// Each style rule has an origin, which determines where it enters the cascade.
///
/// https://drafts.csswg.org/css-cascade/#cascading-origins
#[derive(Clone, PartialEq, Eq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Origin {
    /// https://drafts.csswg.org/css-cascade/#cascade-origin-us
    UserAgent,

    /// https://drafts.csswg.org/css-cascade/#cascade-origin-author
    Author,

    /// https://drafts.csswg.org/css-cascade/#cascade-origin-user
    User,
}
