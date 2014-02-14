/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::DOMExceptionBinding;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::window::Window;
use servo_util::str::DOMString;

#[repr(uint)]
#[deriving(ToStr)]
enum DOMErrorName {
    IndexSizeError = 1,
    HierarchyRequestError = 3,
    WrongDocumentError = 4,
    InvalidCharacterError = 5,
    NoModificationAllowedError = 7,
    NotFoundError = 8,
    NotSupportedError = 9,
    InvalidStateError = 11,
    SyntaxError = 12,
    InvalidModificationError = 13,
    NamespaceError = 14,
    InvalidAccessError = 15,
    SecurityError = 18,
    NetworkError = 19,
    AbortError = 20,
    URLMismatchError = 21,
    QuotaExceededError = 22,
    TimeoutError = 23,
    InvalidNodeTypeError = 24,
    DataCloneError = 25,
    EncodingError
}

pub struct DOMException {
    code: DOMErrorName,
    reflector_: Reflector
}

impl DOMException {
    pub fn new_inherited(code: DOMErrorName) -> DOMException {
        DOMException {
            code: code,
            reflector_: Reflector::new()
        }
    }

    pub fn new(window: &Window, code: DOMErrorName) -> @mut DOMException {
        reflect_dom_object(@mut DOMException::new_inherited(code), window, DOMExceptionBinding::Wrap)
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
            _ => self.code as u16
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
