/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::reflect_dom_object_with_cx;

use crate::dom::bindings::codegen::Bindings::FileSystemEntryBinding::ErrorCallback;
use crate::dom::bindings::codegen::Bindings::FileSystemFileEntryBinding::{
    FileCallback, FileSystemFileEntryMethods,
};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::file::File;
use crate::dom::filesystem::FileSystem;
use crate::dom::filesystementry::FileSystemEntry;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub(crate) struct FileSystemFileEntry {
    filesystementry: FileSystemEntry,
    file: Dom<File>,
}

impl FileSystemFileEntry {
    pub(crate) fn new_inherited(
        name: USVString,
        full_path: USVString,
        file: &File,
    ) -> FileSystemFileEntry {
        FileSystemFileEntry {
            filesystementry: FileSystemEntry::new_inherited(name, full_path, true),
            file: Dom::from_ref(file),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        name: USVString,
        full_path: USVString,
        file: &File,
    ) -> DomRoot<FileSystemFileEntry> {
        reflect_dom_object_with_cx(
            Box::new(FileSystemFileEntry::new_inherited(name, full_path, file)),
            global,
            cx,
        )
    }

    pub(crate) fn set_filesystem(&self, fs: &FileSystem) {
        self.filesystementry.set_filesystem(fs);
    }
}

impl FileSystemFileEntryMethods<crate::DomTypeHolder> for FileSystemFileEntry {
    /// <https://wicg.github.io/entries-api/#dom-filesystemfileentry-file>
    fn File(
        &self,
        _success_callback: Rc<FileCallback>,
        _error_callback: Option<Rc<ErrorCallback>>,
    ) {
        // TODO: Implement per spec 7.4. No need to hook embedder here.
        // It is done at `filesystementry`.
    }
}
