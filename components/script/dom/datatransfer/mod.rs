/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) use self::datatransfer::*;
#[allow(
    clippy::module_inception,
    reason = "The interface name is DataTransfer"
)]
pub(crate) mod datatransfer;
pub(crate) mod datatransferitem;
pub(crate) mod datatransferitemlist;
