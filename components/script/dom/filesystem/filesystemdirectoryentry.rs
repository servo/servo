/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::reflect_dom_object_with_cx;

use crate::dom::bindings::codegen::Bindings::FileSystemDirectoryEntryBinding::{
    FileSystemDirectoryEntryMethods, FileSystemFlags,
};
use crate::dom::bindings::codegen::Bindings::FileSystemEntryBinding::{
    ErrorCallback, FileSystemEntryCallback,
};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::filesystem::FileSystem;
use crate::dom::filesystemdirectoryreader::FileSystemDirectoryReader;
use crate::dom::filesystementry::FileSystemEntry;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub(crate) struct FileSystemDirectoryEntry {
    filesystementry: FileSystemEntry,
    children: DomRefCell<Vec<Dom<FileSystemEntry>>>,
}

impl FileSystemDirectoryEntry {
    pub(crate) fn new_inherited(name: USVString, full_path: USVString) -> FileSystemDirectoryEntry {
        FileSystemDirectoryEntry {
            filesystementry: FileSystemEntry::new_inherited(name, full_path, false),
            children: DomRefCell::new(Vec::new()),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        name: USVString,
        full_path: USVString,
    ) -> DomRoot<FileSystemDirectoryEntry> {
        reflect_dom_object_with_cx(
            Box::new(FileSystemDirectoryEntry::new_inherited(name, full_path)),
            global,
            cx,
        )
    }

    pub(crate) fn set_filesystem(&self, fs: &FileSystem) {
        self.filesystementry.set_filesystem(fs);
    }

    pub(crate) fn push_child(&self, child: &FileSystemEntry) {
        self.children.borrow_mut().push(Dom::from_ref(child));
    }
}

impl FileSystemDirectoryEntryMethods<crate::DomTypeHolder> for FileSystemDirectoryEntry {
    /// <https://wicg.github.io/entries-api/#dom-filesystemdirectoryentry-createreader>
    fn CreateReader(&self, cx: &mut JSContext) -> DomRoot<FileSystemDirectoryReader> {
        FileSystemDirectoryReader::new(cx, &self.global(), self)
    }

    /// <https://wicg.github.io/entries-api/#dom-filesystemdirectoryentry-getfile>
    fn GetFile(
        &self,
        _path: Option<Option<USVString>>,
        _options: &FileSystemFlags,
        _success_callback: Option<Rc<FileSystemEntryCallback>>,
        _error_callback: Option<Rc<ErrorCallback>>,
    ) {
        // TODO: Implement per spec 7.2. Need to hook embedder.
    }

    /// <https://wicg.github.io/entries-api/#dom-filesystemdirectoryentry-getdirectory>
    fn GetDirectory(
        &self,
        _path: Option<Option<USVString>>,
        _options: &FileSystemFlags,
        _success_callback: Option<Rc<FileSystemEntryCallback>>,
        _error_callback: Option<Rc<ErrorCallback>>,
    ) {
        // TODO: Implement per spec 7.2. Need to hook embedder.
    }
}
