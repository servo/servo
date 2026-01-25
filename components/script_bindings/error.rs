/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::CString;

use js::error::throw_type_error;
use js::jsapi::JS_IsExceptionPending;

use crate::codegen::PrototypeList::proto_id_to_name;
use crate::num::Finite;
use crate::script_runtime::JSContext as SafeJSContext;

/// DOM exceptions that can be thrown by a native DOM method.
/// <https://webidl.spec.whatwg.org/#dfn-error-names-table>
#[derive(Clone, Debug, MallocSizeOf)]
pub enum Error {
    /// IndexSizeError DOMException
    IndexSize(Option<String>),
    /// NotFoundError DOMException
    NotFound(Option<String>),
    /// HierarchyRequestError DOMException
    HierarchyRequest(Option<String>),
    /// WrongDocumentError DOMException
    WrongDocument(Option<String>),
    /// InvalidCharacterError DOMException
    InvalidCharacter(Option<String>),
    /// NotSupportedError DOMException
    NotSupported(Option<String>),
    /// InUseAttributeError DOMException
    InUseAttribute(Option<String>),
    /// InvalidStateError DOMException
    InvalidState(Option<String>),
    /// SyntaxError DOMException
    Syntax(Option<String>),
    /// NamespaceError DOMException
    Namespace(Option<String>),
    /// InvalidAccessError DOMException
    InvalidAccess(Option<String>),
    /// SecurityError DOMException
    Security(Option<String>),
    /// NetworkError DOMException
    Network(Option<String>),
    /// AbortError DOMException
    Abort(Option<String>),
    /// TimeoutError DOMException
    Timeout(Option<String>),
    /// InvalidNodeTypeError DOMException
    InvalidNodeType(Option<String>),
    /// DataCloneError DOMException
    DataClone(Option<String>),
    /// TransactionInactiveError DOMException
    TransactionInactive(Option<String>),
    /// ReadOnlyError DOMException
    ReadOnly(Option<String>),
    /// VersionError DOMException
    Version(Option<String>),
    /// NoModificationAllowedError DOMException
    NoModificationAllowed(Option<String>),
    /// QuotaExceededError DOMException
    QuotaExceeded {
        quota: Option<Finite<f64>>,
        requested: Option<Finite<f64>>,
    },
    /// TypeMismatchError DOMException
    TypeMismatch(Option<String>),
    /// InvalidModificationError DOMException
    InvalidModification(Option<String>),
    /// NotReadableError DOMException
    NotReadable(Option<String>),
    /// DataError DOMException
    Data(Option<String>),
    /// OperationError DOMException
    Operation(Option<String>),
    /// NotAllowedError DOMException
    NotAllowed(Option<String>),
    /// EncodingError DOMException
    Encoding(Option<String>),
    /// ConstraintError DOMException
    Constraint(Option<String>),

    /// TypeError JavaScript Error
    Type(String),
    /// RangeError JavaScript Error
    Range(String),

    /// A JavaScript exception is already pending.
    JSFailed,
}

/// The return type for IDL operations that can throw DOM exceptions.
pub type Fallible<T> = Result<T, Error>;

/// The return type for IDL operations that can throw DOM exceptions and
/// return `()`.
pub type ErrorResult = Fallible<()>;

/// Throw an exception to signal that a `JSObject` can not be converted to a
/// given DOM type.
pub fn throw_invalid_this(cx: SafeJSContext, proto_id: u16) {
    debug_assert!(unsafe { !JS_IsExceptionPending(*cx) });
    let mut vec = "\"this\" object does not implement interface "
        .as_bytes()
        .to_vec();
    vec.extend_from_slice(proto_id_to_name(proto_id).as_bytes());
    let error = CString::new(vec).unwrap();
    unsafe { throw_type_error(*cx, &error) };
}

pub fn throw_constructor_without_new(cx: SafeJSContext, name: &str) {
    debug_assert!(unsafe { !JS_IsExceptionPending(*cx) });
    let mut error = name.as_bytes().to_vec();
    error.extend_from_slice(b" constructor: 'new' is required");
    let error = CString::new(error).unwrap();
    unsafe { throw_type_error(*cx, &error) };
}

#[macro_export]
/// Creates a `CString` using interpolation of runtime expressions.
/// Basically a `format!` that produces a `CString`.
///
/// Because data can come from untrusted sources, it will check the interior for
/// null bytes and replace them with `\u0000`.
macro_rules! cformat {
    ($($arg:tt)*) => {
        {
            use std::io::Write;
            let mut s = Vec::new();
            write!(&mut s, $($arg)*).unwrap();
            std::ffi::CString::new(s).or_else(|s| {
                let s = s.into_vec();
                let mut out = Vec::with_capacity(s.len());
                for b in s {
                    if b == 0 {
                        out.extend_from_slice(b"\\u0000");
                    } else {
                        out.push(b);
                    }
                }
                std::ffi::CString::new(out)
            }).unwrap()
        }
    }
}
