/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::reflect_dom_object_with_cx;

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::codegen::Bindings::FileSystemEntryBinding::ErrorCallback;
use crate::dom::bindings::codegen::Bindings::FileSystemFileEntryBinding::{
    FileCallback, FileSystemFileEntryMethods,
};
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
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
    pending_callbacks: DomRefCell<Vec<PendingFileCallback>>,
    next_callback: Cell<usize>,
}

#[derive(JSTraceable, MallocSizeOf)]
struct PendingFileCallback {
    id: usize,
    #[conditional_malloc_size_of]
    callback: Rc<FileCallback>,
}

impl FileSystemFileEntry {
    fn new_inherited(name: USVString, full_path: USVString, file: &File) -> FileSystemFileEntry {
        FileSystemFileEntry {
            filesystementry: FileSystemEntry::new_inherited(name, full_path, true),
            file: Dom::from_ref(file),
            pending_callbacks: Default::default(),
            next_callback: Cell::new(0),
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
    fn File(&self, success_callback: Rc<FileCallback>, _error_callback: Option<Rc<ErrorCallback>>) {
        // Per spec 7.4: in parallel,
        // 1. (TODO) Evaluate path
        // 2-3. (TODO) errorCallback

        // Note: Step 1 - 3 is meant to re-check if file exists on OS filesystem.
        // It is unreachable for now, as the file data is already
        // stored in-memory on this entry as `File` (set by webkitGetAsEntry).

        // 4. on success, queue a task to invoke successCallback
        // with a new `File` object representing item and "report".

        let id = self.next_callback.get();
        let pending_callback = PendingFileCallback {
            id,
            callback: success_callback,
        };
        self.pending_callbacks.borrow_mut().push(pending_callback);
        self.next_callback.set(id + 1);

        let this = Trusted::new(self);
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(invoke_file_callback: move |cx| {
                let this = this.root();
                let maybe_index = this
                    .pending_callbacks
                    .borrow()
                    .iter()
                    .position(|val| val.id == id);
                if let Some(index) = maybe_index {
                    let callback = this
                        .pending_callbacks
                        .safe_borrow_mut(cx.no_gc())
                        .swap_remove(index)
                        .callback;
                    let file = DomRoot::from_ref(&*this.file);
                    let _ = callback.Call__(cx, &file, ExceptionHandling::Report);
                }
            }));
    }
}
