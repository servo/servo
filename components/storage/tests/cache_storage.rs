/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use servo_base::generic_channel::{self, GenericSend};
use storage::CacheStorageThreadFactory;
use storage_traits::cache_storage::{CacheStorageThreadHandle, CacheStorageThreadMessage};

#[test]
fn test_exit() {
    let handle: CacheStorageThreadHandle = CacheStorageThreadFactory::new(None, false);

    let (sender, receiver) = generic_channel::channel().unwrap();
    handle
        .send(CacheStorageThreadMessage::Exit(sender.into()))
        .unwrap();
    receiver.recv().unwrap();

    // Workaround for https://github.com/servo/servo/issues/32912
    #[cfg(windows)]
    std::thread::sleep(std::time::Duration::from_millis(1000));
}
