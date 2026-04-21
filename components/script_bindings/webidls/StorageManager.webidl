/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://storage.spec.whatwg.org/#api

[SecureContext]
interface mixin NavigatorStorage {
  [SameObject, Pref="dom_storage_manager_api_enabled"] readonly attribute StorageManager storage;
};
Navigator includes NavigatorStorage;
WorkerNavigator includes NavigatorStorage;

[SecureContext, Exposed=(Window,Worker)]
interface StorageManager {
  [NewObject]
  Promise<boolean> persisted();

  [Exposed=Window, NewObject]
  Promise<boolean> persist();

  [NewObject]
  Promise<StorageEstimate> estimate();
};

dictionary StorageEstimate {
  unsigned long long usage;
  unsigned long long quota;
};
