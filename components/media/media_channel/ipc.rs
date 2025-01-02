/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::io;

use serde::{Deserialize, Serialize};

pub type GLPlayerSender<T> = ipc_channel::ipc::IpcSender<T>;
pub type GLPlayerReceiver<T> = ipc_channel::ipc::IpcReceiver<T>;

pub fn glplayer_channel<T: Serialize + for<'de> Deserialize<'de>>(
) -> Result<(GLPlayerSender<T>, GLPlayerReceiver<T>), io::Error> {
    ipc_channel::ipc::channel()
}
