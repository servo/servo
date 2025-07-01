/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::id::WebViewId;
use ipc_channel::ipc::IpcSender;
use malloc_size_of_derive::MallocSizeOf;
use profile_traits::mem::ReportsChan;
use serde::{Deserialize, Serialize};
use servo_url::ServoUrl;

#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, Serialize)]
pub enum StorageType {
    Session,
    Local,
}

/// Request operations on the storage data associated with a particular url
#[derive(Debug, Deserialize, Serialize)]
pub enum StorageThreadMsg {
    /// gets the number of key/value pairs present in the associated storage data
    Length(IpcSender<usize>, StorageType, WebViewId, ServoUrl),

    /// gets the name of the key at the specified index in the associated storage data
    Key(
        IpcSender<Option<String>>,
        StorageType,
        WebViewId,
        ServoUrl,
        u32,
    ),

    /// Gets the available keys in the associated storage data
    Keys(IpcSender<Vec<String>>, StorageType, WebViewId, ServoUrl),

    /// gets the value associated with the given key in the associated storage data
    GetItem(
        IpcSender<Option<String>>,
        StorageType,
        WebViewId,
        ServoUrl,
        String,
    ),

    /// sets the value of the given key in the associated storage data
    SetItem(
        IpcSender<Result<(bool, Option<String>), ()>>,
        StorageType,
        WebViewId,
        ServoUrl,
        String,
        String,
    ),

    /// removes the key/value pair for the given key in the associated storage data
    RemoveItem(
        IpcSender<Option<String>>,
        StorageType,
        WebViewId,
        ServoUrl,
        String,
    ),

    /// clears the associated storage data by removing all the key/value pairs
    Clear(IpcSender<bool>, StorageType, WebViewId, ServoUrl),

    /// clones all storage data of the given top-level browsing context for a new browsing context.
    /// should only be used for sessionStorage.
    Clone {
        sender: IpcSender<()>,
        src: WebViewId,
        dest: WebViewId,
    },

    /// send a reply when done cleaning up thread resources and then shut it down
    Exit(IpcSender<()>),

    /// Measure memory used by this thread and send the report over the provided channel.
    CollectMemoryReport(ReportsChan),
}
