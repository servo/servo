/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) use self::window::*;
pub(crate) mod dissimilaroriginlocation;
pub(crate) mod dissimilaroriginwindow;
pub(crate) mod history;
pub(crate) mod location;
#[allow(clippy::module_inception, reason = "The interface name is Window")]
pub(crate) mod window;
pub(crate) mod windowproxy;
