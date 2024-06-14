/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::DOMExceptionBinding::{
    DOMExceptionConstants, DOMExceptionMethods,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{
    reflect_dom_object, reflect_dom_object_with_proto, Reflector,
};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;

#[repr(u16)]
#[derive(Clone, Copy, Debug, Eq, JSTraceable, MallocSizeOf, Ord, PartialEq, PartialOrd)]
pub enum DOMErrorName {
    IndexSizeError = DOMExceptionConstants::INDEX_SIZE_ERR,
    HierarchyRequestError = DOMExceptionConstants::HIERARCHY_REQUEST_ERR,
    WrongDocumentError = DOMExceptionConstants::WRONG_DOCUMENT_ERR,
    InvalidCharacterError = DOMExceptionConstants::INVALID_CHARACTER_ERR,
    NoModificationAllowedError = DOMExceptionConstants::NO_MODIFICATION_ALLOWED_ERR,
    NotFoundError = DOMExceptionConstants::NOT_FOUND_ERR,
    NotSupportedError = DOMExceptionConstants::NOT_SUPPORTED_ERR,
    InUseAttributeError = DOMExceptionConstants::INUSE_ATTRIBUTE_ERR,
    InvalidStateError = DOMExceptionConstants::INVALID_STATE_ERR,
    SyntaxError = DOMExceptionConstants::SYNTAX_ERR,
    InvalidModificationError = DOMExceptionConstants::INVALID_MODIFICATION_ERR,
    NamespaceError = DOMExceptionConstants::NAMESPACE_ERR,
    InvalidAccessError = DOMExceptionConstants::INVALID_ACCESS_ERR,
    SecurityError = DOMExceptionConstants::SECURITY_ERR,
    NetworkError = DOMExceptionConstants::NETWORK_ERR,
    AbortError = DOMExceptionConstants::ABORT_ERR,
    TypeMismatchError = DOMExceptionConstants::TYPE_MISMATCH_ERR,
    URLMismatchError = DOMExceptionConstants::URL_MISMATCH_ERR,
    QuotaExceededError = DOMExceptionConstants::QUOTA_EXCEEDED_ERR,
    TimeoutError = DOMExceptionConstants::TIMEOUT_ERR,
    InvalidNodeTypeError = DOMExceptionConstants::INVALID_NODE_TYPE_ERR,
    DataCloneError = DOMExceptionConstants::DATA_CLONE_ERR,
    EncodingError,
    NotReadableError,
    OperationError,
}

impl DOMErrorName {
    pub fn from(s: &DOMString) -> Option<DOMErrorName> {
        match s.as_ref() {
            "IndexSizeError" => Some(DOMErrorName::IndexSizeError),
            "HierarchyRequestError" => Some(DOMErrorName::HierarchyRequestError),
            "WrongDocumentError" => Some(DOMErrorName::WrongDocumentError),
            "InvalidCharacterError" => Some(DOMErrorName::InvalidCharacterError),
            "NoModificationAllowedError" => Some(DOMErrorName::NoModificationAllowedError),
            "NotFoundError" => Some(DOMErrorName::NotFoundError),
            "NotSupportedError" => Some(DOMErrorName::NotSupportedError),
            "InUseAttributeError" => Some(DOMErrorName::InUseAttributeError),
            "InvalidStateError" => Some(DOMErrorName::InvalidStateError),
            "SyntaxError" => Some(DOMErrorName::SyntaxError),
            "InvalidModificationError" => Some(DOMErrorName::InvalidModificationError),
            "NamespaceError" => Some(DOMErrorName::NamespaceError),
            "InvalidAccessError" => Some(DOMErrorName::InvalidAccessError),
            "SecurityError" => Some(DOMErrorName::SecurityError),
            "NetworkError" => Some(DOMErrorName::NetworkError),
            "AbortError" => Some(DOMErrorName::AbortError),
            "TypeMismatchError" => Some(DOMErrorName::TypeMismatchError),
            "URLMismatchError" => Some(DOMErrorName::URLMismatchError),
            "QuotaExceededError" => Some(DOMErrorName::QuotaExceededError),
            "TimeoutError" => Some(DOMErrorName::TimeoutError),
            "InvalidNodeTypeError" => Some(DOMErrorName::InvalidNodeTypeError),
            "DataCloneError" => Some(DOMErrorName::DataCloneError),
            "EncodingError" => Some(DOMErrorName::EncodingError),
            "NotReadableError" => Some(DOMErrorName::NotReadableError),
            "OperationError" => Some(DOMErrorName::OperationError),
            _ => None,
        }
    }
}

#[dom_struct]
pub struct DOMException {
    reflector_: Reflector,
    message: DOMString,
    name: DOMString,
}

impl DOMException {
    fn get_error_data_by_code(code: DOMErrorName) -> (DOMString, DOMString) {
        let message = match &code {
            DOMErrorName::IndexSizeError => "The index is not in the allowed range.",
            DOMErrorName::HierarchyRequestError => {
                "The operation would yield an incorrect node tree."
            },
            DOMErrorName::WrongDocumentError => "The object is in the wrong document.",
            DOMErrorName::InvalidCharacterError => "The string contains invalid characters.",
            DOMErrorName::NoModificationAllowedError => "The object can not be modified.",
            DOMErrorName::NotFoundError => "The object can not be found here.",
            DOMErrorName::NotSupportedError => "The operation is not supported.",
            DOMErrorName::InUseAttributeError => "The attribute already in use.",
            DOMErrorName::InvalidStateError => "The object is in an invalid state.",
            DOMErrorName::SyntaxError => "The string did not match the expected pattern.",
            DOMErrorName::InvalidModificationError => "The object can not be modified in this way.",
            DOMErrorName::NamespaceError => "The operation is not allowed by Namespaces in XML.",
            DOMErrorName::InvalidAccessError => {
                "The object does not support the operation or argument."
            },
            DOMErrorName::SecurityError => "The operation is insecure.",
            DOMErrorName::NetworkError => "A network error occurred.",
            DOMErrorName::AbortError => "The operation was aborted.",
            DOMErrorName::TypeMismatchError => "The given type does not match any expected type.",
            DOMErrorName::URLMismatchError => "The given URL does not match another URL.",
            DOMErrorName::QuotaExceededError => "The quota has been exceeded.",
            DOMErrorName::TimeoutError => "The operation timed out.",
            DOMErrorName::InvalidNodeTypeError => {
                "The supplied node is incorrect or has an incorrect ancestor for this operation."
            },
            DOMErrorName::DataCloneError => "The object can not be cloned.",
            DOMErrorName::EncodingError => {
                "The encoding operation (either encoded or decoding) failed."
            },
            DOMErrorName::NotReadableError => "The I/O read operation failed.",
            DOMErrorName::OperationError => {
                "The operation failed for an operation-specific reason."
            },
        };

        (
            DOMString::from(message),
            DOMString::from(format!("{:?}", code)),
        )
    }

    fn new_inherited(message_: DOMString, name_: DOMString) -> DOMException {
        DOMException {
            reflector_: Reflector::new(),
            message: message_,
            name: name_,
        }
    }

    pub fn new(global: &GlobalScope, code: DOMErrorName) -> DomRoot<DOMException> {
        let (message, name) = DOMException::get_error_data_by_code(code);

        reflect_dom_object(Box::new(DOMException::new_inherited(message, name)), global)
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        message: DOMString,
        name: DOMString,
    ) -> Result<DomRoot<DOMException>, Error> {
        Ok(reflect_dom_object_with_proto(
            Box::new(DOMException::new_inherited(message, name)),
            global,
            proto,
        ))
    }

    // not an IDL stringifier, used internally
    pub fn stringifier(&self) -> DOMString {
        DOMString::from(format!("{}: {}", self.name, self.message))
    }
}

impl DOMExceptionMethods for DOMException {
    // https://heycam.github.io/webidl/#dom-domexception-code
    fn Code(&self) -> u16 {
        match DOMErrorName::from(&self.name) {
            Some(code) if code <= DOMErrorName::DataCloneError => code as u16,
            _ => 0,
        }
    }

    // https://heycam.github.io/webidl/#idl-DOMException-error-names
    fn Name(&self) -> DOMString {
        self.name.clone()
    }

    // https://heycam.github.io/webidl/#error-names
    fn Message(&self) -> DOMString {
        self.message.clone()
    }
}
