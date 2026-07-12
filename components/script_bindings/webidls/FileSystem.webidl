/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://wicg.github.io/entries-api/#api-domfilesystem

[Pref="dom_entries_api_enabled", Exposed=Window]
interface FileSystem {
  readonly attribute USVString name;
  readonly attribute FileSystemDirectoryEntry root;
};
