/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) use self::globalscope::*;
#[expect(clippy::module_inception, reason = "The interface name is GlobalScope")]
pub(crate) mod globalscope;
pub(crate) mod messagechannel;
pub(crate) mod messageport;
pub(crate) mod origin;
pub(crate) mod script_execution;
