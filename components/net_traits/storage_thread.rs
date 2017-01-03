/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::IpcSender;
use servo_url::ServoUrl;

#[derive(Copy, Clone, Deserialize, Serialize, HeapSizeOf)]
pub enum StorageType {
    Session,
    Local,
}

/// Request operations on the storage data associated with a particular url
#[derive(Deserialize, Serialize)]
pub enum StorageThreadMsg {
    /// gets the number of key/value pairs present in the associated storage data
    Length(IpcSender<usize>, ServoUrl, StorageType),

    /// gets the name of the key at the specified index in the associated storage data
    Key(IpcSender<Option<String>>, ServoUrl, StorageType, u32),

    /// Gets the available keys in the associated storage data
    Keys(IpcSender<Vec<String>>, ServoUrl, StorageType),

    /// gets the value associated with the given key in the associated storage data
    GetItem(IpcSender<Option<String>>, ServoUrl, StorageType, String),

    /// sets the value of the given key in the associated storage data
    SetItem(IpcSender<Result<(bool, Option<String>), ()>>, ServoUrl, StorageType, String, String),

    /// removes the key/value pair for the given key in the associated storage data
    RemoveItem(IpcSender<Option<String>>, ServoUrl, StorageType, String),

    /// clears the associated storage data by removing all the key/value pairs
    Clear(IpcSender<bool>, ServoUrl, StorageType),

    /// send a reply when done cleaning up thread resources and then shut it down
    Exit(IpcSender<()>),
}
