/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::error::Error as StdError;

use libc::ENOSPC;
use rusqlite::{Error as RusqliteError, ffi};

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

pub(crate) fn is_sqlite_disk_full_error(error: &RusqliteError) -> bool {
    fn has_enospc(mut source: Option<&(dyn StdError + 'static)>) -> bool {
        while let Some(err) = source {
            if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
                if io_err.raw_os_error() == Some(ENOSPC) {
                    return true;
                }
            }
            source = err.source();
        }
        false
    }

    // Walk the full chain (including `error` itself).
    let saw_enospc = has_enospc(Some(error as &(dyn StdError + 'static)));

    match error {
        RusqliteError::SqliteFailure(sqlite_err, _) => {
            // High confidence "database or disk is full".
            if sqlite_err.code == ffi::ErrorCode::DiskFull ||
                sqlite_err.extended_code == ffi::SQLITE_FULL
            {
                return true;
            }

            // Only treat IO errors as quota-related if ENOSPC is present.
            if saw_enospc &&
                matches!(
                    sqlite_err.extended_code,
                    ffi::SQLITE_IOERR |
                        ffi::SQLITE_IOERR_WRITE |
                        ffi::SQLITE_IOERR_FSYNC |
                        ffi::SQLITE_IOERR_DIR_FSYNC |
                        ffi::SQLITE_IOERR_TRUNCATE |
                        ffi::SQLITE_IOERR_MMAP
                )
            {
                return true;
            }

            false
        },
        _ => saw_enospc,
    }
}
