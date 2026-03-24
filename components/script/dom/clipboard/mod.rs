/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(clippy::module_inception, reason = "The interface name is Clipboard")]
pub(crate) use self::clipboard::*;
pub(crate) mod clipboard;
pub(crate) mod clipboardevent;
pub(crate) mod clipboarditem;
