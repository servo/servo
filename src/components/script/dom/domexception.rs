/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DOMExceptionBinding;
use dom::bindings::codegen::Bindings::DOMExceptionBinding::DOMExceptionConstants;
use dom::bindings::error;
use dom::bindings::error::Error;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use servo_util::str::DOMString;

#[repr(uint)]
#[deriving(Show, Encodable)]
pub enum DOMErrorName {
    IndexSizeError = DOMExceptionConstants::INDEX_SIZE_ERR,
    HierarchyRequestError = DOMExceptionConstants::HIERARCHY_REQUEST_ERR,
    WrongDocumentError = DOMExceptionConstants::WRONG_DOCUMENT_ERR,
    InvalidCharacterError = DOMExceptionConstants::INVALID_CHARACTER_ERR,
    NoModificationAllowedError = DOMExceptionConstants::NO_MODIFICATION_ALLOWED_ERR,
    NotFoundError = DOMExceptionConstants::NOT_FOUND_ERR,
    NotSupportedError = DOMExceptionConstants::NOT_SUPPORTED_ERR,
    InvalidStateError = DOMExceptionConstants::INVALID_STATE_ERR,
    SyntaxError = DOMExceptionConstants::SYNTAX_ERR,
    InvalidModificationError = DOMExceptionConstants::INVALID_MODIFICATION_ERR,
    NamespaceError = DOMExceptionConstants::NAMESPACE_ERR,
    InvalidAccessError = DOMExceptionConstants::INVALID_ACCESS_ERR,
    SecurityError = DOMExceptionConstants::SECURITY_ERR,
    NetworkError = DOMExceptionConstants::NETWORK_ERR,
    AbortError = DOMExceptionConstants::ABORT_ERR,
    URLMismatchError = DOMExceptionConstants::URL_MISMATCH_ERR,
    QuotaExceededError = DOMExceptionConstants::QUOTA_EXCEEDED_ERR,
    TimeoutError = DOMExceptionConstants::TIMEOUT_ERR,
    InvalidNodeTypeError = DOMExceptionConstants::INVALID_NODE_TYPE_ERR,
    DataCloneError = DOMExceptionConstants::DATA_CLONE_ERR,
    EncodingError
}

impl DOMErrorName {
    fn from_error(error: Error) -> DOMErrorName {
        match error {
            error::IndexSize => IndexSizeError,
            error::NotFound => NotFoundError,
            error::HierarchyRequest => HierarchyRequestError,
            error::InvalidCharacter => InvalidCharacterError,
            error::NotSupported => NotSupportedError,
            error::InvalidState => InvalidStateError,
            error::Syntax => SyntaxError,
            error::NamespaceError => NamespaceError,
            error::InvalidAccess => InvalidAccessError,
            error::Security => SecurityError,
            error::Network => NetworkError,
            error::Abort => AbortError,
            error::Timeout => TimeoutError,
            error::FailureUnknown => fail!(),
        }
    }
}

#[deriving(Encodable)]
pub struct DOMException {
    pub code: DOMErrorName,
    pub reflector_: Reflector
}

impl DOMException {
    pub fn new_inherited(code: DOMErrorName) -> DOMException {
        DOMException {
            code: code,
            reflector_: Reflector::new()
        }
    }

    pub fn new(global: &GlobalRef, code: DOMErrorName) -> Temporary<DOMException> {
        reflect_dom_object(box DOMException::new_inherited(code), global, DOMExceptionBinding::Wrap)
    }

    pub fn new_from_error(global: &GlobalRef, code: Error) -> Temporary<DOMException> {
        DOMException::new(global, DOMErrorName::from_error(code))
    }
}

impl Reflectable for DOMException {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}

pub trait DOMExceptionMethods {
    fn Code(&self) -> u16;
    fn Name(&self) -> DOMString;
    fn Message(&self) -> DOMString;
}

impl<'a> DOMExceptionMethods for JSRef<'a, DOMException> {
    // http://dom.spec.whatwg.org/#dom-domexception-code
    fn Code(&self) -> u16 {
        match self.code {
            // http://dom.spec.whatwg.org/#concept-throw
            EncodingError => 0,
            code => code as u16
        }
    }

    // http://dom.spec.whatwg.org/#error-names-0
    fn Name(&self) -> DOMString {
        self.code.to_str()
    }

    // http://dom.spec.whatwg.org/#error-names-0
    fn Message(&self) -> DOMString {
        match self.code {
            IndexSizeError => "The index is not in the allowed range.".to_string(),
            HierarchyRequestError => "The operation would yield an incorrect node tree.".to_string(),
            WrongDocumentError => "The object is in the wrong document.".to_string(),
            InvalidCharacterError => "The string contains invalid characters.".to_string(),
            NoModificationAllowedError => "The object can not be modified.".to_string(),
            NotFoundError => "The object can not be found here.".to_string(),
            NotSupportedError => "The operation is not supported.".to_string(),
            InvalidStateError => "The object is in an invalid state.".to_string(),
            SyntaxError => "The string did not match the expected pattern.".to_string(),
            InvalidModificationError => "The object can not be modified in this way.".to_string(),
            NamespaceError => "The operation is not allowed by Namespaces in XML.".to_string(),
            InvalidAccessError => "The object does not support the operation or argument.".to_string(),
            SecurityError => "The operation is insecure.".to_string(),
            NetworkError => "A network error occurred.".to_string(),
            AbortError => "The operation was aborted.".to_string(),
            URLMismatchError => "The given URL does not match another URL.".to_string(),
            QuotaExceededError => "The quota has been exceeded.".to_string(),
            TimeoutError => "The operation timed out.".to_string(),
            InvalidNodeTypeError => "The supplied node is incorrect or has an incorrect ancestor for this operation.".to_string(),
            DataCloneError => "The object can not be cloned.".to_string(),
            EncodingError => "The encoding operation (either encoded or decoding) failed.".to_string()
        }
    }
}
