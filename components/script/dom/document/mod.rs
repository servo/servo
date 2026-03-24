/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) use self::document::*;
#[allow(clippy::module_inception, reason = "The interface name is Bluetooth")]
pub(crate) mod document;
pub(crate) mod document_embedder_controls;
pub(crate) mod document_event_handler;
pub(crate) mod documentfragment;
pub(crate) mod documentorshadowroot;
pub(crate) mod documenttype;
