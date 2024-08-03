/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DataTransferItemBinding::{
    DataTransferItemMethods, FunctionStringCallback,
};
use crate::dom::bindings::import::module::Rc;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::file::File;

#[dom_struct]
pub struct DataTransferItem {
    reflector_: Reflector,
    kind: DomRefCell<DOMString>,
    type_: DomRefCell<DOMString>,
}

impl DataTransferItem {
    pub fn new_inherited(kind: DOMString, type_: DOMString) -> DataTransferItem {
        DataTransferItem {
            reflector_: Reflector::new(),
            kind: DomRefCell::new(kind),
            type_: DomRefCell::new(type_),
        }
    }
}

impl DataTransferItemMethods for DataTransferItem {
    // https://html.spec.whatwg.org/multipage/#dom-datatransferitem-kind
    fn Kind(&self) -> DOMString {
        self.kind.borrow().clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-datatransferitem-type
    fn Type(&self) -> DOMString {
        self.type_.borrow().clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-datatransferitem-getasstring
    fn GetAsString(&self, callback: Option<Rc<FunctionStringCallback>>) {
        todo!()
    }

    // https://html.spec.whatwg.org/multipage/#dom-datatransferitem-getasfile
    fn GetAsFile(&self) -> Option<DomRoot<File>> {
        todo!()
    }
}
