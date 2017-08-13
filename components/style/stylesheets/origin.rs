/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

///! [CSS cascade origins](https://drafts.csswg.org/css-cascade/#cascading-origins).

use std::marker::PhantomData;

/// Each style rule has an origin, which determines where it enters the cascade.
///
/// https://drafts.csswg.org/css-cascade/#cascading-origins
#[derive(Clone, PartialEq, Eq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Origin {
    /// https://drafts.csswg.org/css-cascade/#cascade-origin-us
    UserAgent,

    /// https://drafts.csswg.org/css-cascade/#cascade-origin-user
    User,

    /// https://drafts.csswg.org/css-cascade/#cascade-origin-author
    Author,
}

/// An object that stores a `T` for each origin of the CSS cascade.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Debug, Default)]
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

/// An object that can be cleared.
pub trait PerOriginClear {
    /// Clears the object.
    fn clear(&mut self);
}

impl<T> PerOriginClear for PerOrigin<T> where T: PerOriginClear {
    fn clear(&mut self) {
        self.user_agent.clear();
        self.user.clear();
        self.author.clear();
    }
}

/// Iterator over `PerOrigin<T>`, from highest level (author) to lowest
/// (user agent).
///
/// We rely on this specific order for correctly looking up @font-face,
/// @counter-style and @keyframes rules.
pub struct PerOriginIter<'a, T: 'a> {
    data: &'a PerOrigin<T>,
    cur: usize,
}

impl<'a, T> Iterator for PerOriginIter<'a, T> where T: 'a {
    type Item = (&'a T, Origin);

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.cur {
            0 => (&self.data.author, Origin::Author),
            1 => (&self.data.user, Origin::User),
            2 => (&self.data.user_agent, Origin::UserAgent),
            _ => return None,
        };
        self.cur += 1;
        Some(result)
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
    cur: usize,
    _marker: PhantomData<&'a mut PerOrigin<T>>,
}

impl<'a, T> Iterator for PerOriginIterMut<'a, T> where T: 'a {
    type Item = (&'a mut T, Origin);

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.cur {
            0 => (unsafe { &mut (*self.data).author }, Origin::Author),
            1 => (unsafe { &mut (*self.data).user }, Origin::User),
            2 => (unsafe { &mut (*self.data).user_agent }, Origin::UserAgent),
            _ => return None,
        };
        self.cur += 1;
        Some(result)
    }
}
