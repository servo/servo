/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) use self::attributes::*;
pub(crate) use self::element::*;
pub(crate) mod attributes;
pub(crate) mod create;
#[allow(clippy::module_inception, reason = "The interface name is element")]
pub(crate) mod element;
pub(crate) mod focus;
pub(crate) mod namednodemap;
