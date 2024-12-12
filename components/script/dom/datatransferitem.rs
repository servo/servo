/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::codegen::Bindings::DataTransferItemBinding::{
    DataTransferItemMethods, FunctionStringCallback,
};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::file::File;
use crate::dom::globalscope::GlobalScope;
use crate::drag_data_store::Kind;
use crate::script_runtime::CanGc;

#[dom_struct]
pub struct DataTransferItem {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "TODO"]
    #[no_trace]
    item: Kind,
}

impl DataTransferItem {
    fn new_inherited(item: Kind) -> DataTransferItem {
        DataTransferItem {
            reflector_: Reflector::new(),
            item,
        }
    }

    pub fn new(global: &GlobalScope, item: Kind) -> DomRoot<DataTransferItem> {
        reflect_dom_object(
            Box::new(DataTransferItem::new_inherited(item)),
            global,
            CanGc::note(),
        )
    }
}

impl DataTransferItemMethods<crate::DomTypeHolder> for DataTransferItem {
    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitem-kind>
    fn Kind(&self) -> DOMString {
        match self.item {
            Kind::Text(_) => DOMString::from("string"),
            Kind::File(_) => DOMString::from("file"),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitem-type>
    fn Type(&self) -> DOMString {
        self.item.type_()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitem-getasstring>
    fn GetAsString(&self, callback: Option<Rc<FunctionStringCallback>>) {
        if let (Some(callback), Some(data)) = (callback, self.item.as_string()) {
            let _ = callback.Call__(data, ExceptionHandling::Report);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-datatransferitem-getasfile>
    fn GetAsFile(&self) -> Option<DomRoot<File>> {
        self.item.as_file(&self.global())
    }
}
