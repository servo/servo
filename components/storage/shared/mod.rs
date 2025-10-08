/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// These pragmas need to be set once
pub const DB_INIT_PRAGMAS: [&str; 2] =
    ["PRAGMA journal_mode = WAL;", "PRAGMA encoding = 'UTF-16';"];

// These pragmas need to be run once per connection.
pub const DB_PRAGMAS: [&str; 4] = [
    "PRAGMA synchronous = NORMAL;",
    "PRAGMA journal_size_limit = 67108864 -- 64 megabytes;",
    "PRAGMA mmap_size = 67108864 -- 64 megabytes;",
    "PRAGMA cache_size = 2000;",
];
