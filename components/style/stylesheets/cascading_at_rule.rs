/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Cascading at-rule types and traits

use crate::stylesheets::Origin;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};

/// Computes the cascade precedence as according to
/// <http://dev.w3.org/csswg/css-cascade/#cascade-origin>
#[inline]
fn cascade_precendence(origin: Origin, important: bool) -> u8 {
    match (origin, important) {
        (Origin::UserAgent, true) => 1,
        (Origin::User, true) => 2,
        (Origin::Author, true) => 3,
        (Origin::Author, false) => 4,
        (Origin::User, false) => 5,
        (Origin::UserAgent, false) => 6,
    }
}

/// Cascading rule descriptor implementation.
/// This is only used for at-rules which can cascade. These are @viewport and
/// @page, although we don't currently implement @page as such.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToShmem)]
pub struct DescriptorDeclaration<T> {
    /// Origin of the declaration
    pub origin: Origin,
    /// Declaration value
    pub descriptor: T,
    /// Indicates the presence of a !important property.
    pub important: bool,
}

impl<T> DescriptorDeclaration<T> {
    #[allow(missing_docs)]
    pub fn new(origin: Origin, descriptor: T, important: bool) -> Self {
        Self {
            origin,
            descriptor,
            important,
        }
    }
    /// Returns true iff self is equal or higher precedence to the other.
    pub fn higher_or_equal_precendence(&self, other: &Self) -> bool {
        let self_precedence = cascade_precendence(self.origin, self.important);
        let other_precedence = cascade_precendence(other.origin, other.important);

        self_precedence <= other_precedence
    }
}

impl<T> ToCss for DescriptorDeclaration<T>
where
    T: ToCss,
{
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        self.descriptor.to_css(dest)?;
        if self.important {
            dest.write_str(" !important")?;
        }
        dest.write_char(';')
    }
}
