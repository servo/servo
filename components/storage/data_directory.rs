/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;

use tempfile::TempDir;

/// The directory where Servo should store data such as client storage,
/// web storage, caches, and cookies.
pub enum DataDirectory {
    /// A persistent directory.
    Persistent(PathBuf),
    /// A temporary directory only persisted during this execution.
    Temporary(TempDir),
}

impl DataDirectory {
    /// Return a [`DataDirectory`] that corresponds to a temporary directory.
    pub fn temporary() -> Self {
        Self::Temporary(TempDir::new().expect("Could not create temporary data directory"))
    }

    /// Create a [`DataDirectory`] from an optional [`PathBuf`]. If `None`, a temporary
    /// data directory is created.
    pub fn from_path(path: Option<PathBuf>) -> Self {
        match path {
            Some(path) => {
                std::fs::create_dir_all(&path).expect("Could not create data directory");
                Self::Persistent(path)
            },
            None => Self::temporary(),
        }
    }

    /// Return a [`PathBuf`] to the root of this data directory.
    pub fn path(&self) -> PathBuf {
        match self {
            DataDirectory::Persistent(path) => path.clone(),
            DataDirectory::Temporary(temp_dir) => temp_dir.path().to_owned(),
        }
    }

    /// Return a [`PathBuf`] to a subdirectory of the root of this data directory.
    pub fn subdirectory(&self, name: &str) -> PathBuf {
        self.path().join(name)
    }
}
