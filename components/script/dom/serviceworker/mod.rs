/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) use self::serviceworker::*;
pub(crate) mod client;
pub(crate) mod extendableevent;
pub(crate) mod extendablemessageevent;
pub(crate) mod navigationpreloadmanager;
#[allow(
    clippy::module_inception,
    reason = "The interface name is Serviceworker"
)]
pub(crate) mod serviceworker;
pub(crate) mod serviceworkercontainer;
pub(crate) mod serviceworkerglobalscope;
pub(crate) mod serviceworkerregistration;
pub(crate) mod windowclient;
