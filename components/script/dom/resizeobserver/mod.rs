/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) use self::resizeobserver::*;
#[allow(
    clippy::module_inception,
    reason = "The interface name is ResizeObserver"
)]
pub(crate) mod resizeobserver;
pub(crate) mod resizeobserverentry;
pub(crate) mod resizeobserversize;
