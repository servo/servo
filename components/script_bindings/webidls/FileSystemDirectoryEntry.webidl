/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://wicg.github.io/entries-api/#api-directoryentry

[Pref="dom_entries_api_enabled", Exposed=Window]
interface FileSystemDirectoryEntry : FileSystemEntry {
  FileSystemDirectoryReader createReader();
  undefined getFile(optional USVString? path,
               optional FileSystemFlags options = {},
               optional FileSystemEntryCallback successCallback,
               optional ErrorCallback errorCallback);
  undefined getDirectory(optional USVString? path,
                    optional FileSystemFlags options = {},
                    optional FileSystemEntryCallback successCallback,
                    optional ErrorCallback errorCallback);
};

dictionary FileSystemFlags {
  boolean create = false;
  boolean exclusive = false;
};
