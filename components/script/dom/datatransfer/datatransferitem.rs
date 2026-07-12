/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, Ref, RefCell};
use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::codegen::Bindings::DataTransferItemBinding::{
    DataTransferItemMethods, FunctionStringCallback,
};
use crate::dom::bindings::codegen::Bindings::FileBinding::FileMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::file::File;
use crate::dom::filesystem::FileSystem;
use crate::dom::filesystemdirectoryentry::FileSystemDirectoryEntry;
use crate::dom::filesystementry::FileSystemEntry;
use crate::dom::filesystemfileentry::FileSystemFileEntry;
use crate::dom::globalscope::GlobalScope;
use crate::drag_data_store::{DragDataStore, Kind, Mode};

#[dom_struct]
pub(crate) struct DataTransferItem {
    reflector_: Reflector,
    #[conditional_malloc_size_of]
    #[no_trace]
    data_store: Rc<RefCell<Option<DragDataStore>>>,
    id: u16,
    pending_callbacks: DomRefCell<Vec<PendingStringCallback>>,
    next_callback: Cell<usize>,
}

#[derive(JSTraceable, MallocSizeOf)]
struct PendingStringCallback {
    id: usize,
    #[conditional_malloc_size_of]
    callback: Rc<FunctionStringCallback>,
}

impl DataTransferItem {
    fn new_inherited(data_store: Rc<RefCell<Option<DragDataStore>>>, id: u16) -> DataTransferItem {
        DataTransferItem {
            reflector_: Reflector::new(),
            data_store,
            id,
            pending_callbacks: Default::default(),
            next_callback: Cell::new(0),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        data_store: Rc<RefCell<Option<DragDataStore>>>,
        id: u16,
    ) -> DomRoot<DataTransferItem> {
        reflect_dom_object_with_cx(
            Box::new(DataTransferItem::new_inherited(data_store, id)),
            global,
            cx,
        )
    }

    fn item_kind(&self) -> Option<Ref<'_, Kind>> {
        Ref::filter_map(self.data_store.borrow(), |data_store| {
            data_store
                .as_ref()
                .and_then(|data_store| data_store.get_by_id(&self.id))
        })
        .ok()
    }

    fn can_read(&self) -> bool {
        self.data_store
            .borrow()
            .as_ref()
            .is_some_and(|data_store| data_store.mode() != Mode::Protected)
    }
}

impl DataTransferItemMethods<crate::DomTypeHolder> for DataTransferItem {
    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitem-kind>
    fn Kind(&self) -> DOMString {
        self.item_kind()
            .map_or(DOMString::new(), |item| match *item {
                Kind::Text { .. } => DOMString::from("string"),
                Kind::File { .. } => DOMString::from("file"),
            })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitem-type>
    fn Type(&self) -> DOMString {
        self.item_kind()
            .map_or(DOMString::new(), |item| item.type_())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitem-getasstring>
    fn GetAsString(&self, callback: Option<Rc<FunctionStringCallback>>) {
        // Step 1 If the callback is null, return.
        let Some(callback) = callback else {
            return;
        };

        // Step 2 If the DataTransferItem object is not in the read/write mode or the read-only mode, return.
        if !self.can_read() {
            return;
        }

        // Step 3 If the drag data item kind is not text, then return.
        if let Some(string) = self.item_kind().and_then(|item| item.as_string()) {
            let id = self.next_callback.get();
            let pending_callback = PendingStringCallback { id, callback };
            self.pending_callbacks.borrow_mut().push(pending_callback);

            self.next_callback.set(id + 1);
            let this = Trusted::new(self);

            // Step 4 Otherwise, queue a task to invoke callback,
            // passing the actual data of the item represented by the DataTransferItem object as the argument.
            self.global()
                .task_manager()
                .dom_manipulation_task_source()
                .queue(task!(invoke_callback: move |cx| {
                    let maybe_index = this.root().pending_callbacks.borrow().iter().position(|val| val.id == id);
                    if let Some(index) = maybe_index {
                        let callback = this.root().pending_callbacks.borrow_mut().swap_remove(index).callback;
                        let _ = callback.Call__(cx, DOMString::from(string), ExceptionHandling::Report);
                    }
                }));
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitem-getasfile>
    fn GetAsFile(&self, cx: &mut JSContext) -> Option<DomRoot<File>> {
        // Step 1 If the DataTransferItem object is not in the read/write mode or the read-only mode, then return null.
        if !self.can_read() {
            return None;
        }

        // Step 2 If the drag data item kind is not File, then return null.
        // Step 3 Return a new File object representing the actual data
        // of the item represented by the DataTransferItem object.
        self.item_kind()?.as_file(cx, &self.global())
    }

    /// <https://wicg.github.io/entries-api/#dom-datatransferitem-webkitgetasentry>
    fn WebkitGetAsEntry(&self, cx: &mut JSContext) -> Option<DomRoot<FileSystemEntry>> {
        // Step 1. Let store be this’s DataTransfer object’s drag data store.
        // Step 2. If store’s drag data store mode is not read/write mode or read-only mode,
        // return null and abort these steps.
        if !self.can_read() {
            return None;
        }

        // Step 3. Let item be the item in store’s drag data store item list that this represents.
        // Step 4. If item’s kind is not `File`, then return null and abort these steps.
        let file = self.item_kind()?.as_file(cx, &self.global())?;

        // Step 5: Return a new FileSystemEntry object representing the entry.
        let name = file.Name().to_string();

        let file_entry = FileSystemFileEntry::new(
            cx,
            &self.global(),
            USVString::from(name.clone()),
            USVString::from(format!("/{}", name)),
            &file,
        );

        let root = FileSystemDirectoryEntry::new(
            cx,
            &self.global(),
            USVString::default(),
            USVString::from(String::from("/")),
        );

        root.push_child(file_entry.upcast::<FileSystemEntry>());

        let fs = FileSystem::new(
            cx,
            &self.global(),
            USVString::from(String::from("filesystem")),
            &root,
        );

        root.set_filesystem(&fs);
        file_entry.set_filesystem(&fs);

        Some(DomRoot::from_ref(file_entry.upcast::<FileSystemEntry>()))
    }
}
