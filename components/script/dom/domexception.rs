/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DOMExceptionBinding;
use dom::bindings::codegen::Bindings::DOMExceptionBinding::DOMExceptionConstants;
use dom::bindings::codegen::Bindings::DOMExceptionBinding::DOMExceptionMethods;
use dom::bindings::error;
use dom::bindings::error::Error;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use servo_util::str::DOMString;

#[repr(uint)]
#[deriving(Show)]
#[jstraceable]
pub enum DOMErrorName {
    IndexSizeError = DOMExceptionConstants::INDEX_SIZE_ERR as uint,
    HierarchyRequestError = DOMExceptionConstants::HIERARCHY_REQUEST_ERR as uint,
    WrongDocumentError = DOMExceptionConstants::WRONG_DOCUMENT_ERR as uint,
    InvalidCharacterError = DOMExceptionConstants::INVALID_CHARACTER_ERR as uint,
    NoModificationAllowedError = DOMExceptionConstants::NO_MODIFICATION_ALLOWED_ERR as uint,
    NotFoundError = DOMExceptionConstants::NOT_FOUND_ERR as uint,
    NotSupportedError = DOMExceptionConstants::NOT_SUPPORTED_ERR as uint,
    InvalidStateError = DOMExceptionConstants::INVALID_STATE_ERR as uint,
    SyntaxError = DOMExceptionConstants::SYNTAX_ERR as uint,
    InvalidModificationError = DOMExceptionConstants::INVALID_MODIFICATION_ERR as uint,
    NamespaceError = DOMExceptionConstants::NAMESPACE_ERR as uint,
    InvalidAccessError = DOMExceptionConstants::INVALID_ACCESS_ERR as uint,
    SecurityError = DOMExceptionConstants::SECURITY_ERR as uint,
    NetworkError = DOMExceptionConstants::NETWORK_ERR as uint,
    AbortError = DOMExceptionConstants::ABORT_ERR as uint,
    URLMismatchError = DOMExceptionConstants::URL_MISMATCH_ERR as uint,
    QuotaExceededError = DOMExceptionConstants::QUOTA_EXCEEDED_ERR as uint,
    TimeoutError = DOMExceptionConstants::TIMEOUT_ERR as uint,
    InvalidNodeTypeError = DOMExceptionConstants::INVALID_NODE_TYPE_ERR as uint,
    DataCloneError = DOMExceptionConstants::DATA_CLONE_ERR as uint,
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
            error::DataClone => DataCloneError,
            error::FailureUnknown => panic!(),
        }
    }
}

#[dom_struct]
pub struct DOMException {
    code: DOMErrorName,
    reflector_: Reflector
}

impl DOMException {
    fn new_inherited(code: DOMErrorName) -> DOMException {
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

impl<'a> DOMExceptionMethods for JSRef<'a, DOMException> {
    // http://dom.spec.whatwg.org/#dom-domexception-code
    fn Code(self) -> u16 {
        match self.code {
            // http://dom.spec.whatwg.org/#concept-throw
            EncodingError => 0,
            code => code as u16
        }
    }

    // http://dom.spec.whatwg.org/#error-names-0
    fn Name(self) -> DOMString {
        self.code.to_string()
    }

    // http://dom.spec.whatwg.org/#error-names-0
    fn Message(self) -> DOMString {
        let message = match self.code {
            IndexSizeError => "The index is not in the allowed range.",
            HierarchyRequestError => "The operation would yield an incorrect node tree.",
            WrongDocumentError => "The object is in the wrong document.",
            InvalidCharacterError => "The string contains invalid characters.",
            NoModificationAllowedError => "The object can not be modified.",
            NotFoundError => "The object can not be found here.",
            NotSupportedError => "The operation is not supported.",
            InvalidStateError => "The object is in an invalid state.",
            SyntaxError => "The string did not match the expected pattern.",
            InvalidModificationError => "The object can not be modified in this way.",
            NamespaceError => "The operation is not allowed by Namespaces in XML.",
            InvalidAccessError => "The object does not support the operation or argument.",
            SecurityError => "The operation is insecure.",
            NetworkError => "A network error occurred.",
            AbortError => "The operation was aborted.",
            URLMismatchError => "The given URL does not match another URL.",
            QuotaExceededError => "The quota has been exceeded.",
            TimeoutError => "The operation timed out.",
            InvalidNodeTypeError => "The supplied node is incorrect or has an incorrect ancestor for this operation.",
            DataCloneError => "The object can not be cloned.",
            EncodingError => "The encoding operation (either encoded or decoding) failed."
        };

        message.to_string()
    }
}
