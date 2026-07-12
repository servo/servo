/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use script_bindings::reflector::Reflector;

use crate::dom::bindings::codegen::Bindings::FileSystemEntryBinding::{
    ErrorCallback, FileSystemEntryCallback, FileSystemEntryMethods,
};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::USVString;
use crate::dom::filesystem::FileSystem;

#[dom_struct]
pub(crate) struct FileSystemEntry {
    reflector_: Reflector,
    name: USVString,
    full_path: USVString,
    is_file: bool,
    filesystem: MutNullableDom<FileSystem>,
}

impl FileSystemEntry {
    pub(crate) fn new_inherited(
        name: USVString,
        full_path: USVString,
        is_file: bool,
    ) -> FileSystemEntry {
        FileSystemEntry {
            reflector_: Reflector::new(),
            name,
            full_path,
            is_file,
            filesystem: MutNullableDom::new(None),
        }
    }

    pub(crate) fn set_filesystem(&self, fs: &FileSystem) {
        self.filesystem.set(Some(fs));
    }
}

impl FileSystemEntryMethods<crate::DomTypeHolder> for FileSystemEntry {
    /// <https://wicg.github.io/entries-api/#dom-filesystementry-isfile>
    fn IsFile(&self) -> bool {
        self.is_file
    }

    /// <https://wicg.github.io/entries-api/#dom-filesystementry-isdirectory>
    fn IsDirectory(&self) -> bool {
        !self.is_file
    }

    /// <https://wicg.github.io/entries-api/#dom-filesystementry-name>
    fn Name(&self) -> USVString {
        self.name.clone()
    }

    /// <https://wicg.github.io/entries-api/#dom-filesystementry-fullpath>
    fn FullPath(&self) -> USVString {
        self.full_path.clone()
    }

    /// <https://wicg.github.io/entries-api/#dom-filesystementry-filesystem>
    fn Filesystem(&self) -> DomRoot<FileSystem> {
        self.filesystem
            .get()
            .expect("FileSystemEntry must be associated with a FileSystem")
    }

    /// <https://wicg.github.io/entries-api/#dom-filesystementry-getparent>
    fn GetParent(
        &self,
        _success_callback: Option<Rc<FileSystemEntryCallback>>,
        _error_callback: Option<Rc<ErrorCallback>>,
    ) {
        // TODO: Implement per spec 7.1. Need to hook embedder.
    }
}
