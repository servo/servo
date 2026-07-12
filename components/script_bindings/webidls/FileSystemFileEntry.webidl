/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://wicg.github.io/entries-api/#api-fileentry

[Pref="dom_entries_api_enabled", Exposed=Window]
interface FileSystemFileEntry : FileSystemEntry {
  undefined file(FileCallback successCallback,
            optional ErrorCallback errorCallback);
};

callback FileCallback = undefined (File file);
