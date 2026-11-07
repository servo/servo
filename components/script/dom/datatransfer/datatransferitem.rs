/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, Ref};

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object};

use crate::dom::bindings::callback::{ExceptionHandling, RootedCallback, TracedCallback};
use crate::dom::bindings::codegen::Bindings::DataTransferItemBinding::{
    DataTransferItemMethods, FunctionStringCallback,
};
use crate::dom::bindings::codegen::Bindings::FileBinding::FileMethods;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::datatransferitemlist::DataTransferItemList;
use crate::dom::file::File;
use crate::dom::filesystementry::FileSystemEntry;
use crate::dom::filesystemfileentry::FileSystemFileEntry;
use crate::dom::globalscope::GlobalScope;
use crate::drag::drag_data_store::{Kind, Mode};

#[dom_struct]
pub(crate) struct DataTransferItem {
    reflector_: Reflector,
    /// The `DataTransferItemList` that created this item. It owns the drag data
    /// store and the entries-api `FileSystem` shared by all of its items.
    list: Dom<DataTransferItemList>,
    id: u16,
    pending_callbacks: DomRefCell<Vec<PendingStringCallback>>,
    next_callback: Cell<usize>,
}

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct PendingStringCallback {
    id: usize,
    callback: TracedCallback<FunctionStringCallback>,
}

impl DataTransferItem {
    fn new_inherited(list: &DataTransferItemList, id: u16) -> DataTransferItem {
        DataTransferItem {
            reflector_: Reflector::new(),
            list: Dom::from_ref(list),
            id,
            pending_callbacks: Default::default(),
            next_callback: Cell::new(0),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        list: &DataTransferItemList,
        id: u16,
    ) -> DomRoot<DataTransferItem> {
        reflect_dom_object(
            cx,
            Box::new(DataTransferItem::new_inherited(list, id)),
            global,
        )
    }

    fn item_kind(&self) -> Option<Ref<'_, Kind>> {
        let data_store = self.list.data_store()?;
        Ref::filter_map(data_store, |data_store| data_store.get_by_id(&self.id)).ok()
    }

    fn can_read(&self) -> bool {
        self.list
            .data_store()
            .is_some_and(|data_store| data_store.mode() != Mode::Protected)
    }
}

impl DataTransferItemMethods<crate::DomTypeHolder> for DataTransferItem {
    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitem-kind>
    fn Kind(&self) -> DOMString {
        self.item_kind()
            .map_or(DOMString::new(), |item| match *item {
                Kind::Text { .. } => DOMString::from_static("string"),
                Kind::File { .. } => DOMString::from_static("file"),
            })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitem-type>
    fn Type(&self) -> DOMString {
        self.item_kind()
            .map_or(DOMString::new(), |item| item.type_())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitem-getasstring>
    fn GetAsString(&self, callback: Option<RootedCallback<FunctionStringCallback>>) {
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
            self.pending_callbacks
                .borrow_mut()
                .push(PendingStringCallback {
                    id,
                    callback: callback.to_traced(),
                });

            self.next_callback.set(id + 1);
            let this = Trusted::new(self);

            // Step 4 Otherwise, queue a task to invoke callback,
            // passing the actual data of the item represented by the DataTransferItem object as the argument.
            self.global()
                .task_manager()
                .dom_manipulation_task_source()
                .queue(task!(invoke_callback: move |cx| {
                    let this = this.root();
                    let maybe_index = this.pending_callbacks.borrow().iter().position(|val| val.id == id);
                    if let Some(index) = maybe_index {
                        rooted!(&in(cx) let callback = this
                            .pending_callbacks
                            .safe_borrow_mut(cx.no_gc())
                            .swap_remove(index)
                            .callback);
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
        if !self.item_kind().is_some_and(|item| item.is_file()) {
            return None;
        }
        let global = self.global();

        // <https://wicg.github.io/entries-api/#html-data>
        //
        // "During a drag-and-drop operation, file and directory items are associated
        //  with entries. Each entry is a member of a root directory unique to the
        //  drag data store."
        let filesystem = self.list.filesystem(cx);

        // Step 5. Return a new FileSystemEntry object representing the entry.
        let file = self.item_kind()?.as_file(cx, &global)?;
        let name = file.Name().to_string();
        let full_path = USVString::from(format!("/{}", name));
        let entry = FileSystemFileEntry::new(cx, &global, USVString::from(name), full_path, &file);
        entry.set_filesystem(&filesystem);

        Some(DomRoot::upcast::<FileSystemEntry>(entry))
    }
}
