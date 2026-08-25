/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) use self::characterdata::*;
pub(crate) mod cdatasection;
#[allow(
    clippy::module_inception,
    reason = "The interface name is CharacterData"
)]
pub(crate) mod characterdata;
pub(crate) mod comment;
pub(crate) mod processinginstruction;
pub(crate) mod text;
