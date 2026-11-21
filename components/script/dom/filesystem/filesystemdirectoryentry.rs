/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::reflect_dom_object;

use crate::dom::bindings::callback::RootedCallback;
use crate::dom::bindings::codegen::Bindings::FileSystemDirectoryEntryBinding::{
    FileSystemDirectoryEntryMethods, FileSystemFlags,
};
use crate::dom::bindings::codegen::Bindings::FileSystemEntryBinding::{
    ErrorCallback, FileSystemEntryCallback,
};
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::domexception::DOMErrorName;
use crate::dom::filesystem::FileSystem;
use crate::dom::filesystem::path::{evaluate_path, is_valid_path, resolve_relative_path};
use crate::dom::filesystemdirectoryreader::FileSystemDirectoryReader;
use crate::dom::filesystementry::FileSystemEntry;
use crate::dom::filesystemfileentry::FileSystemFileEntry;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub(crate) struct FileSystemDirectoryEntry {
    filesystementry: FileSystemEntry,
    children: DomRefCell<Vec<Dom<FileSystemEntry>>>,
}

impl FileSystemDirectoryEntry {
    fn new_inherited(name: USVString, full_path: USVString) -> FileSystemDirectoryEntry {
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
        reflect_dom_object(
            cx,
            Box::new(FileSystemDirectoryEntry::new_inherited(name, full_path)),
            global,
        )
    }

    pub(crate) fn set_filesystem(&self, fs: &FileSystem) {
        self.filesystementry.set_filesystem(fs);
    }

    pub(crate) fn push_child(&self, child: &FileSystemEntry) {
        self.children.borrow_mut().push(Dom::from_ref(child));
    }

    /// Returns the member of this directory named `name`, if any.
    pub(crate) fn child(&self, name: &str) -> Option<DomRoot<FileSystemEntry>> {
        self.children
            .borrow()
            .iter()
            .find(|child| child.name() == name)
            .map(|child| DomRoot::from_ref(&**child))
    }

    /// Returns up to `max` children starting at index `start`.
    pub(crate) fn children_batch(&self, start: usize, max: usize) -> Vec<DomRoot<FileSystemEntry>> {
        self.children
            .borrow()
            .iter()
            .skip(start)
            .take(max)
            .map(|child| DomRoot::from_ref(&**child))
            .collect()
    }

    /// Shared helper for getFile() and getDirectory().
    /// Only 1.8 and 1.9 is different, based on whether this is file or dir.
    fn get_entry(
        &self,
        path: Option<Option<USVString>>,
        options: &FileSystemFlags,
        is_file: bool,
        success_callback: Option<RootedCallback<FileSystemEntryCallback>>,
        error_callback: Option<RootedCallback<ErrorCallback>>,
    ) {
        // Step 1.1. If path is undefined or null let path be the empty string.
        let path = path.flatten().unwrap_or_default();

        // Step 1.2. If path is not a valid path, queue a task to invoke
        // errorCallback (if given) with « a newly created "TypeMismatchError"
        // DOMException » and "report", and abort these steps.
        //
        // Step 1.3. If options's create member is true, queue a task to invoke
        // errorCallback (if given) with « a newly created "SecurityError"
        // DOMException » and "report", and abort these steps.
        //
        // Step 1.4. Let path be the result of resolving a relative path with
        // this's full path and path.
        //
        // Step 1.5. Let item be the result of evaluating a path with this's
        // root and path.
        //
        // Step 1.6. If item is failure, queue a task to invoke errorCallback
        // (if given) with « a newly created "NotFoundError" DOMException » and
        // "report", and abort these steps.
        //
        // Step 1.7. If item is not a file (getFile) or directory
        // (getDirectory), queue a task to invoke errorCallback (if given) with
        // « a newly created "TypeMismatchError" DOMException » and "report",
        // and abort these steps.
        let result = if !is_valid_path(&path) {
            Err(DOMErrorName::TypeMismatchError)
        } else if options.create {
            Err(DOMErrorName::SecurityError)
        } else {
            let path = USVString::from(resolve_relative_path(
                self.filesystementry.full_path(),
                &path,
            ));
            evaluate_path(&self.filesystementry.root(), &path)
                .ok_or(DOMErrorName::NotFoundError)
                .and_then(|item| {
                    if item.is_file() == is_file {
                        // Step 1.8. Let entry be a new file or directory entry
                        // with item's name as name and path as full path.
                        // We pin down the value synchronously now to pass
                        // to the queued task.
                        Ok((item.name().to_owned(), path, Trusted::new(&*item)))
                    } else {
                        Err(DOMErrorName::TypeMismatchError)
                    }
                })
        };

        let id = self
            .filesystementry
            .queue_entry_callback(success_callback, error_callback);
        let this = Trusted::new(&self.filesystementry);
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(invoke_get_entry: move |cx| {
                let this = this.root();
                let result = match result {
                    Ok((name, path, item)) => {
                        let item = item.root();
                        if is_file {
                            // Step 1.9. Queue a task to invoke successCallback
                            // with « a new FileSystemFileEntry object associated
                            // with entry » and "report".
                            let file = DomRoot::downcast::<FileSystemFileEntry>(item)
                                .expect("getFile() checked that item is a file")
                                .file();
                            let entry = FileSystemFileEntry::new(
                                cx,
                                &this.global(),
                                USVString::from(name),
                                path,
                                &file,
                            );
                            entry.set_filesystem(&this.filesystem());
                            Ok(DomRoot::upcast::<FileSystemEntry>(entry))
                        } else {
                            // Step 1.9. Queue a task to invoke successCallback
                            // with « a new FileSystemDirectoryEntry associated
                            // with entry » and "report".
                            //
                            // TODO: Return a new directory entry as the spec
                            // requires. Need embedder directory support.
                            Ok(item)
                        }
                    },
                    Err(name) => Err(name),
                };
                this.dispatch_entry_callback(cx, id, result);
            }));
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
        path: Option<Option<USVString>>,
        options: &FileSystemFlags,
        success_callback: Option<RootedCallback<FileSystemEntryCallback>>,
        error_callback: Option<RootedCallback<ErrorCallback>>,
    ) {
        self.get_entry(
            path,
            options,
            true, /*is_file*/
            success_callback,
            error_callback,
        );
    }

    /// <https://wicg.github.io/entries-api/#dom-filesystemdirectoryentry-getdirectory>
    fn GetDirectory(
        &self,
        path: Option<Option<USVString>>,
        options: &FileSystemFlags,
        success_callback: Option<RootedCallback<FileSystemEntryCallback>>,
        error_callback: Option<RootedCallback<ErrorCallback>>,
    ) {
        self.get_entry(
            path,
            options,
            false, /*is_file*/
            success_callback,
            error_callback,
        );
    }
}
