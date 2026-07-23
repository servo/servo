/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://wicg.github.io/entries-api/#api-directoryreader

[Pref="dom_entries_api_enabled", Exposed=Window]
interface FileSystemDirectoryReader {
  undefined readEntries(FileSystemEntriesCallback successCallback,
                   optional ErrorCallback errorCallback);
};

callback FileSystemEntriesCallback = undefined (sequence<FileSystemEntry> entries);
