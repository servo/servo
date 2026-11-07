/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Ref, RefCell};
use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::MutableHandleValue;
use script_bindings::reflector::{Reflector, reflect_dom_object};

use crate::dom::bindings::codegen::Bindings::DataTransferItemListBinding::DataTransferItemListMethods;
use crate::dom::bindings::codegen::Bindings::FileBinding::FileMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::frozenarray::CachedFrozenArray;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::datatransferitem::DataTransferItem;
use crate::dom::file::File;
use crate::dom::filesystem::FileSystem;
use crate::dom::filesystemdirectoryentry::FileSystemDirectoryEntry;
use crate::dom::filesystementry::FileSystemEntry;
use crate::dom::filesystemfileentry::FileSystemFileEntry;
use crate::dom::window::Window;
use crate::drag::drag_data_store::{DragDataStore, Kind, Mode};

#[dom_struct]
pub(crate) struct DataTransferItemList {
    reflector_: Reflector,
    #[conditional_malloc_size_of]
    #[no_trace]
    data_store: Rc<RefCell<Option<DragDataStore>>>,
    #[ignore_malloc_size_of = "mozjs"]
    frozen_types: CachedFrozenArray,
    /// The entries-api file system for the drag data store, built lazily by
    /// `webkitGetAsEntry()` and shared by all of the store's items.
    /// Must be cleared whenever the store's File items change.
    filesystem: MutNullableDom<FileSystem>,
}

impl DataTransferItemList {
    fn new_inherited(data_store: Rc<RefCell<Option<DragDataStore>>>) -> DataTransferItemList {
        DataTransferItemList {
            reflector_: Reflector::new(),
            frozen_types: CachedFrozenArray::new(),
            data_store,
            filesystem: MutNullableDom::new(None),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        data_store: Rc<RefCell<Option<DragDataStore>>>,
    ) -> DomRoot<DataTransferItemList> {
        reflect_dom_object(
            cx,
            Box::new(DataTransferItemList::new_inherited(data_store)),
            window,
        )
    }

    pub(crate) fn frozen_types(&self, cx: &mut js::context::JSContext, retval: MutableHandleValue) {
        self.frozen_types.get_or_init(
            cx,
            || {
                self.data_store
                    .borrow()
                    .as_ref()
                    .map_or(Vec::new(), |data_store| data_store.types())
            },
            retval,
        );
    }

    pub(crate) fn invalidate_frozen_types(&self) {
        self.frozen_types.clear();
    }

    /// Return the drag data store backing this list, if it is still associated.
    pub(crate) fn data_store(&self) -> Option<Ref<'_, DragDataStore>> {
        Ref::filter_map(self.data_store.borrow(), |data_store| data_store.as_ref()).ok()
    }

    /// Return the entries-api `FileSystem` for the drag data store, building it
    /// from the File items on first use.
    ///
    /// <https://wicg.github.io/entries-api/#html-data>
    /// "During a drag-and-drop operation, file and directory items are associated
    ///  with entries. Each entry is a member of a root directory unique to the
    ///  drag data store."
    pub(crate) fn filesystem(&self, cx: &mut JSContext) -> DomRoot<FileSystem> {
        self.filesystem.or_init(|| self.create_filesystem(cx))
    }

    fn create_filesystem(&self, cx: &mut JSContext) -> DomRoot<FileSystem> {
        let global = self.global();
        let root = FileSystemDirectoryEntry::new(
            cx,
            &global,
            USVString::default(),
            USVString::from(String::from("/")),
        );
        let filesystem = FileSystem::new(
            cx,
            &global,
            USVString::from(String::from("filesystem")),
            &root,
        );
        root.set_filesystem(&filesystem);

        if let Some(store) = self.data_store.borrow().as_ref() {
            for kind in store.iter_item_list() {
                // Non-File items are not represented in the entries tree.
                let Some(file) = kind.as_file(cx, &global) else {
                    continue;
                };
                let name = file.Name().to_string();
                let full_path = USVString::from(format!("/{}", name));
                let entry =
                    FileSystemFileEntry::new(cx, &global, USVString::from(name), full_path, &file);
                entry.set_filesystem(&filesystem);
                root.push_child(entry.upcast::<FileSystemEntry>());
            }
        }

        filesystem
    }
}

impl DataTransferItemListMethods<crate::DomTypeHolder> for DataTransferItemList {
    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-length>
    fn Length(&self) -> u32 {
        // Return zero if the object is in the disabled mode;
        // otherwise it must return the number of items in the drag data store item list.
        self.data_store
            .borrow()
            .as_ref()
            .map_or(0, |data_store| data_store.list_len() as u32)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-item>
    fn IndexedGetter(
        &self,
        cx: &mut js::context::JSContext,
        index: u32,
    ) -> Option<DomRoot<DataTransferItem>> {
        // Step 1 Return null if it isn't associated with a data store
        let option = self.data_store.borrow();
        let data_store = match option.as_ref() {
            Some(value) => value,
            _ => return None,
        };

        // Step 2
        data_store
            .get_by_index(index as usize)
            .map(|(id, _)| DataTransferItem::new(cx, &self.global(), self, *id))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-add>
    fn Add(
        &self,
        cx: &mut js::context::JSContext,
        data: DOMString,
        mut type_: DOMString,
    ) -> Fallible<Option<DomRoot<DataTransferItem>>> {
        // Step 1 If the DataTransferItemList object is not in the read/write mode, return null.
        let index = {
            let mut option = self.data_store.borrow_mut();
            let data_store = match option.as_mut() {
                Some(value) if value.mode() == Mode::ReadWrite => value,
                _ => return Ok(None),
            };

            // Add an item to the drag data store item list whose kind is text,
            // whose type string is equal to the value of the method's second argument, converted to ASCII lowercase,
            // and whose data is the string given by the method's first argument.
            type_.make_ascii_lowercase();
            data_store.add(Kind::Text { data, type_ })
        };

        index.map(|id| {
            self.frozen_types.clear();

            // Step 3 Determine the value of the indexed property corresponding to the newly added item,
            // and return that value (a newly created DataTransferItem object).
            Some(DataTransferItem::new(cx, &self.global(), self, id))
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-add>
    fn Add_(
        &self,
        cx: &mut js::context::JSContext,
        data: &File,
    ) -> Fallible<Option<DomRoot<DataTransferItem>>> {
        let index = {
            // Step 1 If the DataTransferItemList object is not in the read/write mode, return null.
            let mut option = self.data_store.borrow_mut();
            let data_store = match option.as_mut() {
                Some(value) if value.mode() == Mode::ReadWrite => value,
                _ => return Ok(None),
            };

            // Add an item to the drag data store item list whose kind is File,
            // whose type string is the type of the File, converted to ASCII lowercase,
            // and whose data is the same as the File's data.
            let mut type_ = data.file_type();
            type_.make_ascii_lowercase();
            let bytes = data.file_bytes().unwrap_or_default();
            let name = data.name().clone().root();
            data_store.add(Kind::File { bytes, name, type_ })
        };
        index.map(|id| {
            self.frozen_types.clear();
            // The entries root no longer matches the drag data store item list.
            self.filesystem.set(None);

            // Step 3 Determine the value of the indexed property corresponding to the newly added item,
            // and return that value (a newly created DataTransferItem object).
            Some(DataTransferItem::new(cx, &self.global(), self, id))
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-remove>
    fn Remove(&self, index: u32) -> Fallible<()> {
        // Step 1 If the DataTransferItemList object is not in the read/write mode,
        // throw an "InvalidStateError" DOMException.
        let mut option = self.data_store.borrow_mut();
        let data_store = match option.as_mut() {
            Some(value) if value.mode() == Mode::ReadWrite => value,
            _ => return Err(Error::InvalidState(None)),
        };

        let index = index as usize;

        // Step 2 If the drag data store does not contain an indexth item, then return.
        if index < data_store.list_len() {
            // Step 3 Remove the indexth item from the drag data store.
            // The entries root no longer matches the item list if a File was removed.
            if data_store.remove(index).is_some_and(|item| item.is_file()) {
                self.filesystem.set(None);
            }
            self.frozen_types.clear();
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-clear>
    fn Clear(&self) {
        // If the DataTransferItemList object is in the read/write mode, remove all the items from the drag data store.
        // Otherwise, it must do nothing.
        let mut option = self.data_store.borrow_mut();
        let data_store = match option.as_mut() {
            Some(value) if value.mode() == Mode::ReadWrite => value,
            _ => return,
        };

        // If the item list is empty we don't clear it.
        if data_store.list_len() > 0 {
            data_store.clear_list();
            // The entries root no longer matches the drag data store item list.
            self.filesystem.set(None);
            self.frozen_types.clear();
        }
    }
}
