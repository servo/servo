/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};

use crate::dom::bindings::codegen::Bindings::FileSystemDirectoryReaderBinding::{
    FileSystemDirectoryReaderMethods, FileSystemEntriesCallback,
};
use crate::dom::bindings::codegen::Bindings::FileSystemEntryBinding::ErrorCallback;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::filesystemdirectoryentry::FileSystemDirectoryEntry;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub(crate) struct FileSystemDirectoryReader {
    reflector_: Reflector,
    dir: Dom<FileSystemDirectoryEntry>,
    idx: Cell<usize>,
    reading_flag: Cell<bool>,
    done_flag: Cell<bool>,
}

impl FileSystemDirectoryReader {
    pub(crate) fn new_inherited(dir: &FileSystemDirectoryEntry) -> FileSystemDirectoryReader {
        FileSystemDirectoryReader {
            reflector_: Reflector::new(),
            dir: Dom::from_ref(dir),
            idx: Cell::new(0),
            reading_flag: Cell::new(false),
            done_flag: Cell::new(false),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        dir: &FileSystemDirectoryEntry,
    ) -> DomRoot<FileSystemDirectoryReader> {
        reflect_dom_object_with_cx(
            Box::new(FileSystemDirectoryReader::new_inherited(dir)),
            global,
            cx,
        )
    }
}

impl FileSystemDirectoryReaderMethods<crate::DomTypeHolder> for FileSystemDirectoryReader {
    /// <https://wicg.github.io/entries-api/#dom-filesystemdirectoryreader-readentries>
    fn ReadEntries(
        &self,
        _success_callback: Rc<FileSystemEntriesCallback>,
        _error_callback: Option<Rc<ErrorCallback>>,
    ) {
        // TODO: Implement per spec 7.3. Need to hook embedder.
    }
}
