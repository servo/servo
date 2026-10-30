/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![expect(
    clippy::module_inception,
    reason = "The interface name is CloseWatcher"
)]
pub(crate) use self::closewatcher::*;
pub(crate) mod closewatcher;
pub(crate) mod closewatchermanager;
pub(crate) mod internalclosewatcher;
