/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use computed_values::white_space;

pub trait WhitespaceMethods {
    fn allow_wrap(&self) -> bool;
    fn preserve_newlines(&self) -> bool;
    fn preserve_spaces(&self) -> bool;
}

impl WhitespaceMethods for white_space::T {
    fn allow_wrap(&self) -> bool {
        match *self {
            white_space::T::nowrap |
            white_space::T::pre => false,
            white_space::T::normal |
            white_space::T::pre_wrap |
            white_space::T::pre_line => true,
        }
    }

    fn preserve_newlines(&self) -> bool {
        match *self {
            white_space::T::normal |
            white_space::T::nowrap => false,
            white_space::T::pre |
            white_space::T::pre_wrap |
            white_space::T::pre_line => true,
        }
    }

    fn preserve_spaces(&self) -> bool {
        match *self {
            white_space::T::normal |
            white_space::T::nowrap |
            white_space::T::pre_line => false,
            white_space::T::pre |
            white_space::T::pre_wrap => true,
        }
    }
}