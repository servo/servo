/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::DOMExceptionBinding;
use dom::bindings::codegen::DOMExceptionBinding::DOMExceptionConstants;
use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::window::Window;
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

    pub fn new(window: &JS<Window>, code: DOMErrorName) -> JS<DOMException> {
        reflect_dom_object(~DOMException::new_inherited(code), window, DOMExceptionBinding::Wrap)
    }
}

impl Reflectable for DOMException {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}

impl DOMException {
    // http://dom.spec.whatwg.org/#dom-domexception-code
    pub fn Code(&self) -> u16 {
        match self.code {
            // http://dom.spec.whatwg.org/#concept-throw
            EncodingError => 0,
            code => code as u16
        }
    }

    // http://dom.spec.whatwg.org/#error-names-0
    pub fn Name(&self) -> DOMString {
        self.code.to_str()
    }

    // http://dom.spec.whatwg.org/#error-names-0
    pub fn Message(&self) -> DOMString {
        match self.code {
            IndexSizeError => ~"The index is not in the allowed range.",
            HierarchyRequestError => ~"The operation would yield an incorrect node tree.",
            WrongDocumentError => ~"The object is in the wrong document.",
            InvalidCharacterError => ~"The string contains invalid characters.",
            NoModificationAllowedError => ~"The object can not be modified.",
            NotFoundError => ~"The object can not be found here.",
            NotSupportedError => ~"The operation is not supported.",
            InvalidStateError => ~"The object is in an invalid state.",
            SyntaxError => ~"The string did not match the expected pattern.",
            InvalidModificationError => ~"The object can not be modified in this way.",
            NamespaceError => ~"The operation is not allowed by Namespaces in XML.",
            InvalidAccessError => ~"The object does not support the operation or argument.",
            SecurityError => ~"The operation is insecure.",
            NetworkError => ~"A network error occurred.",
            AbortError => ~"The operation was aborted.",
            URLMismatchError => ~"The given URL does not match another URL.",
            QuotaExceededError => ~"The quota has been exceeded.",
            TimeoutError => ~"The operation timed out.",
            InvalidNodeTypeError => ~"The supplied node is incorrect or has an incorrect ancestor for this operation.",
            DataCloneError => ~"The object can not be cloned.",
            EncodingError => ~"The encoding operation (either encoded or decoding) failed."
        }
    }
}
