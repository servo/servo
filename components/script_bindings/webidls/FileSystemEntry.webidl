/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://wicg.github.io/entries-api/#api-entry

[Pref="dom_entries_api_enabled", Exposed=Window]
interface FileSystemEntry {
  readonly attribute boolean isFile;
  readonly attribute boolean isDirectory;
  readonly attribute USVString name;
  readonly attribute USVString fullPath;
  readonly attribute FileSystem filesystem;

  undefined getParent(optional FileSystemEntryCallback successCallback,
                 optional ErrorCallback errorCallback);
};

callback ErrorCallback = undefined (DOMException err);
callback FileSystemEntryCallback = undefined (FileSystemEntry entry);
