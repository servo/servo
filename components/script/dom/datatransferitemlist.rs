/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::DataTransferItemListBinding::DataTransferItemListMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::datatransferitem::DataTransferItem;
use crate::dom::file::File;
use crate::dom::window::Window;

#[dom_struct]
pub struct DataTransferItemList {
    reflector_: Reflector,
}

impl DataTransferItemList {
    pub fn new_inherited() -> DataTransferItemList {
        DataTransferItemList {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(window: &Window) -> DomRoot<DataTransferItemList> {
        reflect_dom_object(Box::new(DataTransferItemList::new_inherited()), window)
    }
}

impl DataTransferItemListMethods for DataTransferItemList {
    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-length>
    fn Length(&self) -> u32 {
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-add>
    fn Add(&self, data: DOMString, type_: DOMString) -> Option<DomRoot<DataTransferItem>> {
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-add>
    fn Add_(&self, data: &File) -> Option<DomRoot<DataTransferItem>> {
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-remove>
    fn Remove(&self, index: u32) {
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-clear>
    fn Clear(&self) {
        todo!()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitemlist-item>
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<DataTransferItem>> {
        todo!()
    }
}
