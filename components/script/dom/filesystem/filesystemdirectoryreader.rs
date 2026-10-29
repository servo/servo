/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object};

use crate::dom::bindings::callback::{ExceptionHandling, RootedCallback, TracedCallback};
use crate::dom::bindings::codegen::Bindings::FileSystemDirectoryReaderBinding::{
    FileSystemDirectoryReaderMethods, FileSystemEntriesCallback,
};
use crate::dom::bindings::codegen::Bindings::FileSystemEntryBinding::ErrorCallback;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::domexception::{DOMErrorName, DOMException};
use crate::dom::filesystemdirectoryentry::FileSystemDirectoryEntry;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub(crate) struct FileSystemDirectoryReader {
    reflector_: Reflector,
    dir: Dom<FileSystemDirectoryEntry>,
    idx: Cell<usize>,
    reading_flag: Cell<bool>,
    done_flag: Cell<bool>,
    reader_error: MutNullableDom<DOMException>,
    pending_callbacks: DomRefCell<Vec<PendingEntriesCallback>>,
    next_callback: Cell<usize>,
}

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct PendingEntriesCallback {
    id: usize,
    callback: TracedCallback<FileSystemEntriesCallback>,
    error: Option<TracedCallback<ErrorCallback>>,
}

/// The action for a `readEntries` call decided synchronously.
#[derive(Clone, Copy)]
enum ReadEntriesAction {
    /// Another read already in progress.
    InvalidState,
    /// A previous read failed.
    ReaderError,
    /// Every entry already produced.
    Done,
    /// Start a new read.
    Read,
}

impl FileSystemDirectoryReader {
    fn new_inherited(dir: &FileSystemDirectoryEntry) -> FileSystemDirectoryReader {
        FileSystemDirectoryReader {
            reflector_: Reflector::new(),
            dir: Dom::from_ref(dir),
            idx: Cell::new(0),
            reading_flag: Cell::new(false),
            done_flag: Cell::new(false),
            reader_error: MutNullableDom::new(None),
            pending_callbacks: Default::default(),
            next_callback: Cell::new(0),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        dir: &FileSystemDirectoryEntry,
    ) -> DomRoot<FileSystemDirectoryReader> {
        reflect_dom_object(
            cx,
            Box::new(FileSystemDirectoryReader::new_inherited(dir)),
            global,
        )
    }
}

impl FileSystemDirectoryReaderMethods<crate::DomTypeHolder> for FileSystemDirectoryReader {
    /// <https://wicg.github.io/entries-api/#dom-filesystemdirectoryreader-readentries>
    fn ReadEntries(
        &self,
        success_callback: RootedCallback<FileSystemEntriesCallback>,
        error_callback: Option<RootedCallback<ErrorCallback>>,
    ) {
        // Steps 1-4. The checks and the flag update run synchronously,
        // before the callback task is queued.
        //
        // 1. If this’s reading flag is true, queue a task to invoke
        //    errorCallback with « a newly created "InvalidStateError"
        //    DOMException » and "report", and abort these steps.
        // 2. If this’s reader error is not null, queue a task to invoke
        //    errorCallback (if given) with « reader error » and "report", and
        //    abort these steps.
        // 3. If this’s done flag is true, queue a task to invoke
        //    successCallback with an empty list and "report", and abort these
        //    steps.
        // 4. Set this’s reading flag to true.
        let action = if self.reading_flag.get() {
            ReadEntriesAction::InvalidState
        } else if self.reader_error.get().is_some() {
            ReadEntriesAction::ReaderError
        } else if self.done_flag.get() {
            ReadEntriesAction::Done
        } else {
            self.reading_flag.set(true);
            ReadEntriesAction::Read
        };

        let id = self.next_callback.get();
        self.pending_callbacks
            .borrow_mut()
            .push(PendingEntriesCallback {
                id,
                callback: success_callback.to_traced(),
                error: error_callback.map(|callback| callback.to_traced()),
            });
        self.next_callback.set(id + 1);

        let this = Trusted::new(self);
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(invoke_read_entries: move |cx| {
                let this = this.root();
                let maybe_index = this
                    .pending_callbacks
                    .borrow()
                    .iter()
                    .position(|val| val.id == id);
                if let Some(index) = maybe_index {
                    let error_callback = this
                        .pending_callbacks
                        .borrow_mut()
                        .get(index)
                        .and_then(|pending| pending.error.as_ref())
                        .map(|callback| callback.root(cx));
                    rooted!(&in(cx) let callback = this
                        .pending_callbacks
                        .borrow_mut()
                        .swap_remove(index)
                        .callback);
                    match action {
                        ReadEntriesAction::InvalidState => {
                            // 1. Invoke errorCallback with « a newly created
                            //    "InvalidStateError" DOMException » and "report".
                            let error = DOMException::new(
                                cx,
                                &this.global(),
                                DOMErrorName::InvalidStateError,
                            );
                            if let Some(error_callback) = error_callback {
                                let _ = error_callback.Call__(
                                    cx,
                                    &error,
                                    ExceptionHandling::Report,
                                );
                            }
                        }
                        ReadEntriesAction::ReaderError => {
                            // 2. Invoke errorCallback (if given) with « reader error » and "report".
                            let error = this
                                .reader_error
                                .get()
                                .expect("reader error must be set");
                            if let Some(error_callback) = error_callback {
                                let _ = error_callback.Call__(
                                    cx,
                                    &error,
                                    ExceptionHandling::Report,
                                );
                            }
                        }
                        ReadEntriesAction::Done => {
                            // 3. Invoke successCallback with an empty list.
                            let _ = callback.Call__(cx, Vec::new(), ExceptionHandling::Report);
                        }
                        ReadEntriesAction::Read => {
                            // 5. In parallel, run these steps:
                            //
                            // 5.1. If this’s directory is null, then..
                            // TODO: (Not reachable until we hook embedder)
                            //
                            // 5.2. Let entries be a non-zero number of
                            //      entries from this’s directory that have
                            //      not yet been produced by this
                            //      FileSystemDirectoryReader, if any.
                            let all_entries = this.dir.children();
                            let start = this.idx.get();
                            let entries = all_entries
                                .get(start..)
                                .unwrap_or(&[])
                                .to_vec();
                            this.idx.set(all_entries.len());
                            //
                            // 5.3. If the previous step failed (for example,
                            //      the directory was deleted or permission is denied), then:
                            //      (Not reachable as we use in-memory entries currently)
                            //
                            //      1. Queue a task to run these steps:
                            //         1. Let error be a an appropriate DOMException.
                            //         2. Set this’s reader error to error.
                            //         3. Set this’s reading flag to false.
                            //         4. Invoke errorCallback (if given) with
                            //            « error » and "report".
                            //      2. Abort these steps.
                            //      TODO: Reading the in-memory children cannot
                            //      currently fail; this is where
                            //      `reader_error` would be set.
                            //
                            // 5.4. Queue a task to run these steps:
                            // 5.4.1. If entries is empty, then set this’s done flag to true.
                            if entries.is_empty() {
                                this.done_flag.set(true);
                            }
                            // 5.4.2. Set this’s reading flag to false.
                            this.reading_flag.set(false);
                            // 5.4.3. Invoke successCallback with « entries » and "report".
                            let _ = callback.Call__(cx, entries, ExceptionHandling::Report);
                        }
                    }
                }
            }));
    }
}
