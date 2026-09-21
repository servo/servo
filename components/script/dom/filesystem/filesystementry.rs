/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::Reflector;

use crate::dom::bindings::callback::{ExceptionHandling, RootedCallback, TracedCallback};
use crate::dom::bindings::codegen::Bindings::FileSystemBinding::FileSystemMethods;
use crate::dom::bindings::codegen::Bindings::FileSystemEntryBinding::{
    ErrorCallback, FileSystemEntryCallback, FileSystemEntryMethods,
};
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
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
    pending_callbacks: DomRefCell<Vec<PendingEntryCallback>>,
    next_callback: Cell<usize>,
}

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct PendingEntryCallback {
    id: usize,
    callback: TracedCallback<FileSystemEntryCallback>,
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
            pending_callbacks: Default::default(),
            next_callback: Cell::new(0),
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
        success_callback: Option<RootedCallback<FileSystemEntryCallback>>,
        _error_callback: Option<RootedCallback<ErrorCallback>>,
    ) {
        let Some(callback) = success_callback else {
            return;
        };
        // Per spec 7.1: in parallel,
        // 1. (TODO) Let `path` be the result of resolve ".." relative to this's full path.
        // 2. (TODO) Let `item` be the result of evaluating a path with this’s root and path.
        // 3. (TODO) Queue errorCallback if `item` is failure.
        // 4 - 5.  Queue a task to invoke
        // successCallback with the parent directory entry.
        //
        // NOTE: For now, the parent is always the root directory, which is correct
        // as the `FileSystemEntry` can only be created by webkitGetAsEntry(), which is
        // single level DnD.

        let id = self.next_callback.get();
        self.pending_callbacks
            .borrow_mut()
            .push(PendingEntryCallback {
                id,
                callback: callback.to_traced(),
            });
        self.next_callback.set(id + 1);

        let this = Trusted::new(self);
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(invoke_get_parent: move |cx| {
                let this = this.root();
                let maybe_index = this
                    .pending_callbacks
                    .borrow()
                    .iter()
                    .position(|val| val.id == id);
                if let Some(index) = maybe_index {
                    rooted!(&in(cx) let callback = this
                        .pending_callbacks
                        .safe_borrow_mut(cx.no_gc())
                        .swap_remove(index)
                        .callback);
                    let entry = DomRoot::upcast::<FileSystemEntry>(this.Filesystem().Root());
                    let _ = callback.Call__(cx, &entry, ExceptionHandling::Report);
                }
            }));
    }
}
