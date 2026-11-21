/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::context::JSContext;
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

    pub(crate) fn is_file(&self) -> bool {
        self.is_file
    }

    pub(crate) fn filesystem(&self) -> DomRoot<FileSystem> {
        self.filesystem
            .get()
            .expect("FileSystemEntry must be associated with a FileSystem")
    }

    /// Returns this entry's root directory.
    pub(crate) fn root(&self) -> DomRoot<FileSystemDirectoryEntry> {
        self.filesystem().root()
    }

    /// Queue `success_callback` and `error_callback` for the task that will
    /// invoke one of them, and returns the id used to look them up.
    pub(crate) fn queue_entry_callback(
        &self,
        success_callback: Option<RootedCallback<FileSystemEntryCallback>>,
        error_callback: Option<RootedCallback<ErrorCallback>>,
    ) -> usize {
        let id = self.next_callback.get();
        self.pending_callbacks
            .borrow_mut()
            .push(PendingEntryCallback {
                id,
                callback: success_callback.map(|callback| callback.to_traced()),
                error: error_callback.map(|callback| callback.to_traced()),
            });
        self.next_callback.set(id + 1);
        id
    }

    /// Invoke the callbacks queued under `id` with `result`, unless the task was cancelled.
    pub(crate) fn dispatch_entry_callback(
        &self,
        cx: &mut JSContext,
        id: usize,
        result: Result<DomRoot<FileSystemEntry>, DOMErrorName>,
    ) {
        let Some(index) = self
            .pending_callbacks
            .borrow()
            .iter()
            .position(|val| val.id == id)
        else {
            return;
        };

        let (success_callback, error_callback) = {
            let pending = self.pending_callbacks.borrow();
            let pending = pending.get(index).expect("callback must still be queued");
            (
                pending.callback.as_ref().map(|callback| callback.root(cx)),
                pending.error.as_ref().map(|callback| callback.root(cx)),
            )
        };
        self.pending_callbacks
            .safe_borrow_mut(cx)
            .swap_remove(index);

        match result {
            Ok(entry) => {
                if let Some(callback) = success_callback {
                    let _ = callback.Call__(cx, &entry, ExceptionHandling::Report);
                }
            },
            Err(name) => {
                if let Some(error_callback) = error_callback {
                    let error = DOMException::new(cx, &self.global(), name);
                    let _ = error_callback.Call__(cx, &error, ExceptionHandling::Report);
                }
            },
        }
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
        self.filesystem()
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
        let item = evaluate_path(&self.root(), &path)
            .and_then(DomRoot::downcast::<FileSystemDirectoryEntry>)
            .map(|item| Trusted::new(&*item));

        let id = self.queue_entry_callback(success_callback, error_callback);

        let this = Trusted::new(self);
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(invoke_get_parent: move |cx| {
                let this = this.root();
                let result = match item {
                    // Step 1.3. If item is failure, queue a task to invoke
                    // errorCallback (if given) with « a newly created
                    // "NotFoundError" DOMException » and "report", and abort
                    // these steps.
                    None => Err(DOMErrorName::NotFoundError),
                    // Step 1.4. Let entry be a new directory entry with item's
                    // name as name and path as full path.
                    //
                    // Step 1.5. Queue a task to invoke successCallback with « a
                    // new FileSystemDirectoryEntry object associated with entry »
                    // and "report".
                    //
                    // TODO: Return a new directory entry as the spec requires.
                    // We can fix this once directory members are resolved by path
                    // with embedder support.
                    Some(item) => Ok(DomRoot::upcast::<FileSystemEntry>(item.root())),
                };
                this.dispatch_entry_callback(cx, id, result);
            }));
    }
}
