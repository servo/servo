/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) use self::navigator::*;
#[allow(clippy::module_inception, reason = "The interface name is Navigator")]
pub(crate) mod navigator;
pub(crate) mod navigatorinfo;
pub(crate) mod plugin;
pub(crate) mod pluginarray;
