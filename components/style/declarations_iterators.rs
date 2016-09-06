/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A set of simple iterators to be reused for doing the cascade.

use properties::{Importance, PropertyDeclaration};


/// An iterator that applies the declarations in a list that matches the given
/// importance.
#[derive(Clone)]
pub struct SimpleDeclarationsIterator<'a> {
    declarations: &'a [(PropertyDeclaration, Importance)],
    importance: Importance,
    index: usize,
}

impl<'a> SimpleDeclarationsIterator<'a> {
    pub fn new(declarations: &'a [(PropertyDeclaration, Importance)],
               importance: Importance) -> Self {
        SimpleDeclarationsIterator {
            declarations: declarations,
            importance: importance,
            index: 0,
        }
    }
}

impl<'a> Iterator for SimpleDeclarationsIterator<'a> {
    type Item = &'a PropertyDeclaration;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index == self.declarations.len() {
                return None;
            }

            let (ref decl, importance) =
                self.declarations[self.declarations.len() - self.index - 1];

            self.index += 1;

            if importance == self.importance {
                return Some(decl)
            }
        }
    }
}

/// An iterator that applies the declarations in a list without checking any
/// order.
#[derive(Clone)]
pub struct RawDeclarationsIterator<'a> {
    declarations: &'a [(PropertyDeclaration, Importance)],
    index: usize,
}

impl<'a> RawDeclarationsIterator<'a> {
    pub fn new(declarations: &'a [(PropertyDeclaration, Importance)]) -> Self {
        RawDeclarationsIterator {
            declarations: declarations,
            index: 0,
        }
    }
}

impl<'a> Iterator for RawDeclarationsIterator<'a> {
    type Item = &'a PropertyDeclaration;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.declarations.len() {
            return None;
        }

        let (ref decl, _) =
            self.declarations[self.declarations.len() - self.index - 1];

        self.index += 1;

        return Some(decl)
    }
}
