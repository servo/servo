/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::id::WebViewId;
use ipc_channel::ipc::IpcSender;

/// Messages sent from the bluetooth threads to the embedder.
pub enum BluetoothToEmbedderMsg {
    /// Open dialog to select bluetooth device.
    GetSelectedBluetoothDevice(WebViewId, Vec<String>, IpcSender<Option<String>>),
}
