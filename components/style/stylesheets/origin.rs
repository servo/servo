/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! [CSS cascade origins](https://drafts.csswg.org/css-cascade/#cascading-origins).

use std::marker::PhantomData;
use std::ops::BitOrAssign;

/// Each style rule has an origin, which determines where it enters the cascade.
///
/// <https://drafts.csswg.org/css-cascade/#cascading-origins>
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
pub enum Origin {
    /// <https://drafts.csswg.org/css-cascade/#cascade-origin-user-agent>
    UserAgent = 1 << 0,

    /// <https://drafts.csswg.org/css-cascade/#cascade-origin-user>
    User = 1 << 1,

    /// <https://drafts.csswg.org/css-cascade/#cascade-origin-author>
    Author = 1 << 2,
}

impl Origin {
    /// Returns an origin that goes in order for `index`.
    ///
    /// This is used for iterating across origins.
    fn from_index(index: i8) -> Option<Self> {
        Some(match index {
            0 => Origin::Author,
            1 => Origin::User,
            2 => Origin::UserAgent,
            _ => return None,
        })
    }
}

bitflags! {
    /// A set of origins. This is equivalent to Gecko's OriginFlags.
    #[cfg_attr(feature = "servo", derive(MallocSizeOf))]
    pub struct OriginSet: u8 {
        /// <https://drafts.csswg.org/css-cascade/#cascade-origin-user-agent>
        const ORIGIN_USER_AGENT = Origin::UserAgent as u8;
        /// <https://drafts.csswg.org/css-cascade/#cascade-origin-user>
        const ORIGIN_USER = Origin::User as u8;
        /// <https://drafts.csswg.org/css-cascade/#cascade-origin-author>
        const ORIGIN_AUTHOR = Origin::Author as u8;
    }
}

impl OriginSet {
    /// Returns an iterator over the origins present in this `OriginSet`.
    ///
    /// See the `OriginSet` documentation for information about the order
    /// origins are iterated.
    pub fn iter(&self) -> OriginSetIterator {
        OriginSetIterator {
            set: *self,
            cur: 0,
        }
    }
}

impl From<Origin> for OriginSet {
    fn from(origin: Origin) -> Self {
        Self::from_bits_truncate(origin as u8)
    }
}

impl BitOrAssign<Origin> for OriginSet {
    fn bitor_assign(&mut self, origin: Origin) {
        *self |= OriginSet::from(origin);
    }
}

/// Iterates over the origins present in an `OriginSet`, in order from
/// highest priority (author) to lower (user agent).
#[derive(Clone)]
pub struct OriginSetIterator {
    set: OriginSet,
    cur: i8,
}

impl Iterator for OriginSetIterator {
    type Item = Origin;

    fn next(&mut self) -> Option<Origin> {
        loop {
            let origin = Origin::from_index(self.cur)?;

            self.cur += 1;

            if self.set.contains(origin.into()) {
                return Some(origin)
            }
        }
    }
}

/// An object that stores a `T` for each origin of the CSS cascade.
#[derive(Debug, Default, MallocSizeOf)]
pub struct PerOrigin<T> {
    /// Data for `Origin::UserAgent`.
    pub user_agent: T,

    /// Data for `Origin::User`.
    pub user: T,

    /// Data for `Origin::Author`.
    pub author: T,
}

impl<T> PerOrigin<T> {
    /// Returns a reference to the per-origin data for the specified origin.
    #[inline]
    pub fn borrow_for_origin(&self, origin: &Origin) -> &T {
        match *origin {
            Origin::UserAgent => &self.user_agent,
            Origin::User => &self.user,
            Origin::Author => &self.author,
        }
    }

    /// Returns a mutable reference to the per-origin data for the specified
    /// origin.
    #[inline]
    pub fn borrow_mut_for_origin(&mut self, origin: &Origin) -> &mut T {
        match *origin {
            Origin::UserAgent => &mut self.user_agent,
            Origin::User => &mut self.user,
            Origin::Author => &mut self.author,
        }
    }

    /// Iterates over references to per-origin extra style data, from highest
    /// level (author) to lowest (user agent).
    pub fn iter_origins(&self) -> PerOriginIter<T> {
        PerOriginIter {
            data: &self,
            cur: 0,
            rev: false,
        }
    }

    /// Iterates over references to per-origin extra style data, from lowest
    /// level (user agent) to highest (author).
    pub fn iter_origins_rev(&self) -> PerOriginIter<T> {
        PerOriginIter {
            data: &self,
            cur: 2,
            rev: true,
        }
    }

    /// Iterates over mutable references to per-origin extra style data, from
    /// highest level (author) to lowest (user agent).
    pub fn iter_mut_origins(&mut self) -> PerOriginIterMut<T> {
        PerOriginIterMut {
            data: self,
            cur: 0,
            _marker: PhantomData,
        }
    }
}

/// Iterator over `PerOrigin<T>`, from highest level (author) to lowest
/// (user agent).
///
/// We rely on this specific order for correctly looking up @font-face,
/// @counter-style and @keyframes rules.
pub struct PerOriginIter<'a, T: 'a> {
    data: &'a PerOrigin<T>,
    cur: i8,
    rev: bool,
}

impl<'a, T> Iterator for PerOriginIter<'a, T> where T: 'a {
    type Item = (&'a T, Origin);

    fn next(&mut self) -> Option<Self::Item> {
        let origin = Origin::from_index(self.cur)?;

        self.cur += if self.rev { -1 } else { 1 };

        Some((self.data.borrow_for_origin(&origin), origin))
    }
}

/// Like `PerOriginIter<T>`, but iterates over mutable references to the
/// per-origin data.
///
/// We must use unsafe code here since it's not possible for the borrow
/// checker to know that we are safely returning a different reference
/// each time from `next()`.
pub struct PerOriginIterMut<'a, T: 'a> {
    data: *mut PerOrigin<T>,
    cur: i8,
    _marker: PhantomData<&'a mut PerOrigin<T>>,
}

impl<'a, T> Iterator for PerOriginIterMut<'a, T> where T: 'a {
    type Item = (&'a mut T, Origin);

    fn next(&mut self) -> Option<Self::Item> {
        let origin = Origin::from_index(self.cur)?;

        self.cur += 1;

        Some((unsafe { (*self.data).borrow_mut_for_origin(&origin) }, origin))
    }
}
