/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
pub(crate) use self::url::*;
#[allow(clippy::module_inception, reason = "The interface name is URL")]
pub(crate) mod url;
pub(crate) mod urlhelper;
pub(crate) mod urlpattern;
pub(crate) mod urlsearchparams;
