use crate::dom::bindings::codegen::Bindings::DataTransferItemBinding;
use crate::dom::bindings::codegen::Bindings::DataTransferItemBinding::DataTransferItemMethods;
use crate::dom::bindings::str::DOMString;

#[dom_struct]
pub struct DataTransferItem {
    kind: DOMString,
    r#type: DOMString,
}
