/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[expect(clippy::module_inception, reason = "The interface name is WakeLock")]
pub(crate) mod wakelock;
pub(crate) use wakelock::WakeLock;
pub(crate) mod wakelocksentinel;
