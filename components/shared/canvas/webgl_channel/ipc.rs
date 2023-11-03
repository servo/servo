/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::io;

use serde::{Deserialize, Serialize};

pub type WebGLSender<T> = ipc_channel::ipc::IpcSender<T>;
pub type WebGLReceiver<T> = ipc_channel::ipc::IpcReceiver<T>;

pub fn webgl_channel<T: Serialize + for<'de> Deserialize<'de>>(
) -> Result<(WebGLSender<T>, WebGLReceiver<T>), io::Error> {
    ipc_channel::ipc::channel()
}
