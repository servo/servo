/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::IpcSender;
use url::Url;

#[derive(Copy, Clone, Deserialize, Serialize, HeapSizeOf)]
pub enum StorageType {
    Session,
    Local
}

/// Request operations on the storage data associated with a particular url
#[derive(Deserialize, Serialize)]
pub enum StorageThreadMsg {
    /// gets the number of key/value pairs present in the associated storage data
    Length(IpcSender<usize>, Url, StorageType),

    /// gets the name of the key at the specified index in the associated storage data
    Key(IpcSender<Option<String>>, Url, StorageType, u32),

    /// Gets the available keys in the associated storage data
    Keys(IpcSender<Vec<String>>, Url, StorageType),

    /// gets the value associated with the given key in the associated storage data
    GetItem(IpcSender<Option<String>>, Url, StorageType, String),

    /// sets the value of the given key in the associated storage data
    SetItem(IpcSender<Result<(bool, Option<String>), ()>>, Url, StorageType, String, String),

    /// removes the key/value pair for the given key in the associated storage data
    RemoveItem(IpcSender<Option<String>>, Url, StorageType, String),

    /// clears the associated storage data by removing all the key/value pairs
    Clear(IpcSender<bool>, Url, StorageType),

    /// shut down this thread
    Exit
}

/// Handle to a storage thread
pub type StorageThread = IpcSender<StorageThreadMsg>;


