/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::DOMExceptionBinding;
use dom::bindings::codegen::BindingDeclarations::DOMExceptionBinding::DOMExceptionConstants;
use dom::bindings::js::{JSRef, Temporary};
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

    pub fn new(window: &JSRef<Window>, code: DOMErrorName) -> Temporary<DOMException> {
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
            IndexSizeError => "The index is not in the allowed range.".to_owned(),
            HierarchyRequestError => "The operation would yield an incorrect node tree.".to_owned(),
            WrongDocumentError => "The object is in the wrong document.".to_owned(),
            InvalidCharacterError => "The string contains invalid characters.".to_owned(),
            NoModificationAllowedError => "The object can not be modified.".to_owned(),
            NotFoundError => "The object can not be found here.".to_owned(),
            NotSupportedError => "The operation is not supported.".to_owned(),
            InvalidStateError => "The object is in an invalid state.".to_owned(),
            SyntaxError => "The string did not match the expected pattern.".to_owned(),
            InvalidModificationError => "The object can not be modified in this way.".to_owned(),
            NamespaceError => "The operation is not allowed by Namespaces in XML.".to_owned(),
            InvalidAccessError => "The object does not support the operation or argument.".to_owned(),
            SecurityError => "The operation is insecure.".to_owned(),
            NetworkError => "A network error occurred.".to_owned(),
            AbortError => "The operation was aborted.".to_owned(),
            URLMismatchError => "The given URL does not match another URL.".to_owned(),
            QuotaExceededError => "The quota has been exceeded.".to_owned(),
            TimeoutError => "The operation timed out.".to_owned(),
            InvalidNodeTypeError => "The supplied node is incorrect or has an incorrect ancestor for this operation.".to_owned(),
            DataCloneError => "The object can not be cloned.".to_owned(),
            EncodingError => "The encoding operation (either encoded or decoding) failed.".to_owned()
        }
    }
}
