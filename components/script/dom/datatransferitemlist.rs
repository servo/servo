/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::DataTransferItemListBinding::DataTransferItemListMethods;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::datatransferitem::DataTransferItem;
use crate::dom::file::File;

#[dom_struct]
pub struct DataTransferItemList {
    reflector_: Reflector,
}

impl DataTransferItemListMethods for DataTransferItemList {
    fn Length(&self) -> u32 {
        todo!()
    }
    fn Add(&self, data: DOMString, type_: DOMString) -> Option<DomRoot<DataTransferItem>> {
        todo!()
    }
    fn Add_(&self, data: &File) -> Option<DomRoot<DataTransferItem>> {
        todo!()
    }
    fn Remove(&self, index: u32) {
        todo!()
    }
    fn Clear(&self) {
        todo!()
    }
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<DataTransferItem>> {
        todo!()
    }
}
