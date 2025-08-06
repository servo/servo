/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use base::id::{DomExceptionId, DomExceptionIndex};
use constellation_traits::DomException;
use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::DOMExceptionBinding::{
    DOMExceptionConstants, DOMExceptionMethods,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{
    Reflector, reflect_dom_object, reflect_dom_object_with_proto,
};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::serializable::Serializable;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::structuredclone::StructuredData;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[repr(u16)]
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, Eq, JSTraceable, MallocSizeOf, Ord, PartialEq, PartialOrd)]
pub(crate) enum DOMErrorName {
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
    DataError,
    TransactionInactiveError,
    ReadOnlyError,
    VersionError,
    EncodingError,
    NotReadableError,
    OperationError,
    NotAllowedError,
    ConstraintError,
}

impl DOMErrorName {
    pub(crate) fn from(s: &DOMString) -> Option<DOMErrorName> {
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
            "DataError" => Some(DOMErrorName::DataError),
            "TransactionInactiveError" => Some(DOMErrorName::TransactionInactiveError),
            "ReadOnlyError" => Some(DOMErrorName::ReadOnlyError),
            "VersionError" => Some(DOMErrorName::VersionError),
            "EncodingError" => Some(DOMErrorName::EncodingError),
            "NotReadableError" => Some(DOMErrorName::NotReadableError),
            "OperationError" => Some(DOMErrorName::OperationError),
            "NotAllowedError" => Some(DOMErrorName::NotAllowedError),
            "ConstraintError" => Some(DOMErrorName::ConstraintError),
            _ => None,
        }
    }
}

#[dom_struct]
pub(crate) struct DOMException {
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
            DOMErrorName::DataError => "Provided data is inadequate.",
            DOMErrorName::TransactionInactiveError => {
                "A request was placed against a transaction which is currently not active, or which is finished."
            },
            DOMErrorName::ReadOnlyError => {
                "The mutating operation was attempted in a \"readonly\" transaction."
            },
            DOMErrorName::VersionError => {
                "An attempt was made to open a database using a lower version than the existing version."
            },
            DOMErrorName::EncodingError => {
                "The encoding operation (either encoded or decoding) failed."
            },
            DOMErrorName::NotReadableError => "The I/O read operation failed.",
            DOMErrorName::OperationError => {
                "The operation failed for an operation-specific reason."
            },
            DOMErrorName::NotAllowedError => {
                r#"The request is not allowed by the user agent or the platform in the current context,
                possibly because the user denied permission."#
            },
            DOMErrorName::ConstraintError => {
                "A mutation operation in a transaction failed because a constraint was not satisfied."
            },
        };

        (
            DOMString::from(message),
            DOMString::from(format!("{:?}", code)),
        )
    }

    pub(crate) fn new_inherited(message: DOMString, name: DOMString) -> DOMException {
        DOMException {
            reflector_: Reflector::new(),
            message,
            name,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        code: DOMErrorName,
        can_gc: CanGc,
    ) -> DomRoot<DOMException> {
        let (message, name) = DOMException::get_error_data_by_code(code);

        reflect_dom_object(
            Box::new(DOMException::new_inherited(message, name)),
            global,
            can_gc,
        )
    }

    pub(crate) fn new_with_custom_message(
        global: &GlobalScope,
        code: DOMErrorName,
        message: String,
        can_gc: CanGc,
    ) -> DomRoot<DOMException> {
        let (_, name) = DOMException::get_error_data_by_code(code);

        reflect_dom_object(
            Box::new(DOMException::new_inherited(DOMString::from(message), name)),
            global,
            can_gc,
        )
    }

    // not an IDL stringifier, used internally
    pub(crate) fn stringifier(&self) -> DOMString {
        DOMString::from(format!("{}: {}", self.name, self.message))
    }
}

impl DOMExceptionMethods<crate::DomTypeHolder> for DOMException {
    // https://webidl.spec.whatwg.org/#dom-domexception-domexception
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        message: DOMString,
        name: DOMString,
    ) -> Result<DomRoot<DOMException>, Error> {
        Ok(reflect_dom_object_with_proto(
            Box::new(DOMException::new_inherited(message, name)),
            global,
            proto,
            can_gc,
        ))
    }

    // https://webidl.spec.whatwg.org/#dom-domexception-code
    fn Code(&self) -> u16 {
        match DOMErrorName::from(&self.name) {
            Some(code) if code <= DOMErrorName::DataCloneError => code as u16,
            _ => 0,
        }
    }

    // https://webidl.spec.whatwg.org/#dom-domexception-name
    fn Name(&self) -> DOMString {
        self.name.clone()
    }

    // https://webidl.spec.whatwg.org/#dom-domexception-message
    fn Message(&self) -> DOMString {
        self.message.clone()
    }
}

impl Serializable for DOMException {
    type Index = DomExceptionIndex;
    type Data = DomException;

    // https://webidl.spec.whatwg.org/#idl-DOMException
    fn serialize(&self) -> Result<(DomExceptionId, Self::Data), ()> {
        let serialized = DomException {
            message: self.message.to_string(),
            name: self.name.to_string(),
        };
        Ok((DomExceptionId::new(), serialized))
    }

    // https://webidl.spec.whatwg.org/#idl-DOMException
    fn deserialize(
        owner: &GlobalScope,
        serialized: Self::Data,
        can_gc: CanGc,
    ) -> Result<DomRoot<Self>, ()>
    where
        Self: Sized,
    {
        Ok(Self::new_with_custom_message(
            owner,
            DOMErrorName::from(&DOMString::from_string(serialized.name)).ok_or(())?,
            serialized.message,
            can_gc,
        ))
    }

    fn serialized_storage<'a>(
        data: StructuredData<'a, '_>,
    ) -> &'a mut Option<HashMap<DomExceptionId, Self::Data>> {
        match data {
            StructuredData::Reader(reader) => &mut reader.exceptions,
            StructuredData::Writer(writer) => &mut writer.exceptions,
        }
    }
}
