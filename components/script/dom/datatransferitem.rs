use std::rc::Rc;

use crate::dom::bindings::codegen::Bindings::DataTransferItemBinding;
use crate::dom::bindings::codegen::Bindings::DataTransferItemBinding::{
    DataTransferItemMethods, FunctionStringCallback,
};
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::str::DOMString;

use super::bindings::callback::ExceptionHandling;
use super::file::File;
use super::text::Text;

enum DataTransferItemKinds {
    string(Text),
    file(File),
}

#[dom_struct]
pub struct DataTransferItem {
    reflector_: Reflector,
    kind: DOMString, // 'string' or 'file' enum?
    r#type: DOMString,

    // internal
    item: DataTransferItemKinds,
}

impl DataTransferItemMethods for DataTransferItem {
    fn getAsString(&self, callback: Option<Rc<FunctionStringCallback>>) {
        if let (Some(callback), DataTransferItemKinds::string(text)) = (callback, self.item) {
            callback.Call__(text, ExceptionHandling::Report);
        }
    }
    fn getAsFile(&self) -> Option<File> {
        // should check self.kind DOMString instead of internal state?
        if let DataTransferItemKinds::file(file) = self.item {
            Some(file)
        }
        None
    }
}
