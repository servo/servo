/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::error::throw_type_error;
use js::jsapi::JS_IsExceptionPending;

use crate::codegen::PrototypeList::proto_id_to_name;
use crate::script_runtime::JSContext as SafeJSContext;

/// DOM exceptions that can be thrown by a native DOM method.
/// <https://webidl.spec.whatwg.org/#dfn-error-names-table>
#[derive(Clone, Debug, MallocSizeOf)]
pub enum Error {
    /// IndexSizeError DOMException
    IndexSize,
    /// NotFoundError DOMException
    NotFound,
    /// HierarchyRequestError DOMException
    HierarchyRequest,
    /// WrongDocumentError DOMException
    WrongDocument,
    /// InvalidCharacterError DOMException
    InvalidCharacter,
    /// NotSupportedError DOMException
    NotSupported,
    /// InUseAttributeError DOMException
    InUseAttribute,
    /// InvalidStateError DOMException
    InvalidState,
    /// SyntaxError DOMException
    Syntax,
    /// NamespaceError DOMException
    Namespace,
    /// InvalidAccessError DOMException
    InvalidAccess,
    /// SecurityError DOMException
    Security,
    /// NetworkError DOMException
    Network,
    /// AbortError DOMException
    Abort,
    /// TimeoutError DOMException
    Timeout,
    /// InvalidNodeTypeError DOMException
    InvalidNodeType,
    /// DataCloneError DOMException
    DataClone(Option<String>),
    /// TransactionInactiveError DOMException
    TransactionInactive,
    /// ReadOnlyError DOMException
    ReadOnly,
    /// VersionError DOMException
    Version,
    /// NoModificationAllowedError DOMException
    NoModificationAllowed,
    /// QuotaExceededError DOMException
    QuotaExceeded,
    /// TypeMismatchError DOMException
    TypeMismatch,
    /// InvalidModificationError DOMException
    InvalidModification,
    /// NotReadableError DOMException
    NotReadable,
    /// DataError DOMException
    Data,
    /// OperationError DOMException
    Operation,
    /// NotAllowedError DOMException
    NotAllowed,
    /// EncodingError DOMException
    Encoding,
    /// ConstraintError DOMException
    Constraint,

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
    let error = format!(
        "\"this\" object does not implement interface {}.",
        proto_id_to_name(proto_id)
    );
    unsafe { throw_type_error(*cx, &error) };
}

pub fn throw_constructor_without_new(cx: SafeJSContext, name: &str) {
    debug_assert!(unsafe { !JS_IsExceptionPending(*cx) });
    let error = format!("{} constructor: 'new' is required", name);
    unsafe { throw_type_error(*cx, &error) };
}
