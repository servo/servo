/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::DOMExceptionBinding;
use crate::dom::bindings::codegen::Bindings::DOMExceptionBinding::DOMExceptionConstants;
use crate::dom::bindings::codegen::Bindings::DOMExceptionBinding::DOMExceptionMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;

#[repr(u16)]
#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf)]
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
    QuotaExceededError = DOMExceptionConstants::QUOTA_EXCEEDED_ERR,
    TimeoutError = DOMExceptionConstants::TIMEOUT_ERR,
    InvalidNodeTypeError = DOMExceptionConstants::INVALID_NODE_TYPE_ERR,
    DataCloneError = DOMExceptionConstants::DATA_CLONE_ERR,
    NotReadableError = DOMExceptionConstants::NOT_READABLE_ERR,
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
            "QuotaExceededError" => Some(DOMErrorName::QuotaExceededError),
            "TimeoutError" => Some(DOMErrorName::TimeoutError),
            "InvalidNodeTypeError" => Some(DOMErrorName::InvalidNodeTypeError),
            "DataCloneError" => Some(DOMErrorName::DataCloneError),
            "NotReadableError" => Some(DOMErrorName::NotReadableError),
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
            DOMErrorName::QuotaExceededError => "The quota has been exceeded.",
            DOMErrorName::TimeoutError => "The operation timed out.",
            DOMErrorName::InvalidNodeTypeError => {
                "The supplied node is incorrect or has an incorrect ancestor for this operation."
            },
            DOMErrorName::DataCloneError => "The object can not be cloned.",
            DOMErrorName::NotReadableError => "The I/O read operation failed.",
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

        reflect_dom_object(
            Box::new(DOMException::new_inherited(message, name)),
            global,
            DOMExceptionBinding::Wrap,
        )
    }

    pub fn Constructor(
        global: &GlobalScope,
        message: DOMString,
        name: DOMString,
    ) -> Result<DomRoot<DOMException>, Error> {
        Ok(reflect_dom_object(
            Box::new(DOMException::new_inherited(message, name)),
            global,
            DOMExceptionBinding::Wrap,
        ))
    }
}

impl DOMExceptionMethods for DOMException {
    // https://heycam.github.io/webidl/#dfn-DOMException
    fn Code(&self) -> u16 {
        match DOMErrorName::from(&self.name) {
            Some(code) => code as u16,
            None => 0 as u16,
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

    // https://people.mozilla.org/~jorendorff/es6-draft.html#sec-error.prototype.tostring
    fn Stringifier(&self) -> DOMString {
        DOMString::from(format!("{}: {}", self.name, self.message))
    }
}
