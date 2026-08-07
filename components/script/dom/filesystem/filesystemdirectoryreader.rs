/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::codegen::Bindings::FileSystemDirectoryReaderBinding::{
    FileSystemDirectoryReaderMethods, FileSystemEntriesCallback,
};
use crate::dom::bindings::codegen::Bindings::FileSystemEntryBinding::ErrorCallback;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
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
    pending_callbacks: DomRefCell<Vec<PendingEntriesCallback>>,
    next_callback: Cell<usize>,
}

#[derive(JSTraceable, MallocSizeOf)]
struct PendingEntriesCallback {
    id: usize,
    #[conditional_malloc_size_of]
    callback: Rc<FileSystemEntriesCallback>,
}

impl FileSystemDirectoryReader {
    fn new_inherited(dir: &FileSystemDirectoryEntry) -> FileSystemDirectoryReader {
        FileSystemDirectoryReader {
            reflector_: Reflector::new(),
            dir: Dom::from_ref(dir),
            idx: Cell::new(0),
            reading_flag: Cell::new(false),
            done_flag: Cell::new(false),
            pending_callbacks: Default::default(),
            next_callback: Cell::new(0),
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
        success_callback: Rc<FileSystemEntriesCallback>,
        _error_callback: Option<Rc<ErrorCallback>>,
    ) {
        // Per spec §7.3: queue a task to invoke successCallback with the
        // directory's children that have not yet been produced. The first
        // call returns all children; subsequent calls return an empty list
        // (done flag set).
        let id = self.next_callback.get();
        let pending_callback = PendingEntriesCallback {
            id,
            callback: success_callback,
        };
        self.pending_callbacks.borrow_mut().push(pending_callback);
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
                    let callback = this
                        .pending_callbacks
                        .borrow_mut()
                        .swap_remove(index)
                        .callback;
                    let entries = if this.done_flag.get() {
                        Vec::new()
                    } else {
                        this.done_flag.set(true);
                        this.dir.children()
                    };
                    let _ = callback.Call__(cx, entries, ExceptionHandling::Report);
                }
            }));
    }
}
