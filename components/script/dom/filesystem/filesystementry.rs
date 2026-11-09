/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::Reflector;

use crate::dom::bindings::callback::{ExceptionHandling, RootedCallback, TracedCallback};
use crate::dom::bindings::codegen::Bindings::FileSystemEntryBinding::{
    ErrorCallback, FileSystemEntryCallback, FileSystemEntryMethods,
};
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::USVString;
use crate::dom::domexception::{DOMErrorName, DOMException};
use crate::dom::filesystem::FileSystem;
use crate::dom::filesystem::path::{evaluate_path, resolve_relative_path};
use crate::dom::filesystemdirectoryentry::FileSystemDirectoryEntry;

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
    callback: Option<TracedCallback<FileSystemEntryCallback>>,
    error: Option<TracedCallback<ErrorCallback>>,
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

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn full_path(&self) -> &str {
        &self.full_path
    }

    /// Returns this entry's root directory.
    pub(crate) fn root(&self) -> DomRoot<FileSystemDirectoryEntry> {
        self.filesystem
            .get()
            .expect("FileSystemEntry must be associated with a FileSystem")
            .root()
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
        error_callback: Option<RootedCallback<ErrorCallback>>,
    ) {
        // Step 1.1. Let path be the result of resolving a relative path with
        // this's full path and '..'.
        let path = resolve_relative_path(self.full_path(), "..");

        // Step 1.2. Let item be the result of evaluating a path with this's
        // root and path.
        //
        // Step 1.3. If item is failure, queue a task to invoke errorCallback
        // (if given) with « a newly created "NotFoundError" DOMException »
        // and "report", and abort these steps.
        let item = evaluate_path(&self.root(), &path)
            .and_then(DomRoot::downcast::<FileSystemDirectoryEntry>)
            .map(|parent| Trusted::new(&*parent));

        let id = self.next_callback.get();
        self.pending_callbacks
            .borrow_mut()
            .push(PendingEntryCallback {
                id,
                callback: success_callback.map(|callback| callback.to_traced()),
                error: error_callback.map(|callback| callback.to_traced()),
            });
        self.next_callback.set(id + 1);

        let this = Trusted::new(self);
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(invoke_get_parent: move |cx| {
                let this = this.root();
                let Some(index) = this
                    .pending_callbacks
                    .borrow()
                    .iter()
                    .position(|val| val.id == id)
                else {
                    return;
                };

                let (success_callback, error_callback) = {
                    let pending = this.pending_callbacks.borrow();
                    let pending = pending
                        .get(index)
                        .expect("callback must still be queued");
                    (
                        pending.callback.as_ref().map(|callback| callback.root(cx)),
                        pending.error.as_ref().map(|callback| callback.root(cx)),
                    )
                };
                this.pending_callbacks.borrow_mut().swap_remove(index);

                // Step 1.4. Let entry be a new directory entry with item's
                // name as name and path as full path.
                //
                // Step 1.5. Queue a task to invoke successCallback with « a
                // new FileSystemDirectoryEntry object associated with entry »
                // and "report".
                match item {
                    Some(item) => {
                        let entry = DomRoot::upcast::<FileSystemEntry>(item.root());
                        if let Some(callback) = success_callback {
                            let _ = callback.Call__(cx, &entry, ExceptionHandling::Report);
                        }
                    },
                    None => {
                        if let Some(error_callback) = error_callback {
                            let error =
                                DOMException::new(cx, &this.global(), DOMErrorName::NotFoundError);
                            let _ = error_callback.Call__(cx, &error, ExceptionHandling::Report);
                        }
                    },
                }
            }));
    }
}
