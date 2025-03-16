/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod number;
mod path;
mod stream;

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Error;

pub(crate) use path::PathParser;
pub(crate) use stream::Stream;
