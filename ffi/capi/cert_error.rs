/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::{c_char, c_void};
use std::os::raw::c_int;

/// Certificate error codes matching CEF's `cef_cert_error_t` and net_traits error codes.
///
/// These correspond to common SSL/TLS certificate validation errors.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ServoCertErrorCode {
    /// No error.
    None = 0,

    /// The certificate has expired.
    Expired = 1,

    /// The certificate is not yet valid.
    NotYetValid = 2,

    /// The certificate is self-signed.
    SelfSigned = 3,

    /// The certificate is signed by an unknown authority.
    UnknownCA = 4,

    /// The certificate's common name does not match the host name.
    CommonNameInvalid = 5,

    /// The certificate has been revoked.
    Revoked = 6,

    /// The certificate is invalid (general validation failure).
    Invalid = 7,

    /// The certificate chain is incomplete.
    IncompleteChain = 8,

    /// The certificate uses a weak signature algorithm.
    WeakSignatureAlgorithm = 9,

    /// The certificate uses a weak key.
    WeakKey = 10,

    /// The certificate is not valid for the requested usage.
    InvalidUsage = 11,

    /// The certificate transparency check failed.
    TransparencyRequired = 12,

    /// The certificate was issued by a legacy Symantec CA.
    SymantecLegacy = 13,

    /// A general certificate error occurred.
    Other = -1,
}

impl ServoCertErrorCode {
    /// Convert from a string error description to an error code.
    /// This is a heuristic mapping based on common error messages.
    pub fn from_error_text(text: &str) -> Self {
        let text_lower = text.to_lowercase();
        if text_lower.contains("expired") || text_lower.contains("expiry") {
            ServoCertErrorCode::Expired
        } else if text_lower.contains("not yet valid") || text_lower.contains("not valid before") {
            ServoCertErrorCode::NotYetValid
        } else if text_lower.contains("self signed") || text_lower.contains("self-signed") {
            ServoCertErrorCode::SelfSigned
        } else if text_lower.contains("unknown") && text_lower.contains("authority") {
            ServoCertErrorCode::UnknownCA
        } else if text_lower.contains("common name")
            || text_lower.contains("hostname")
            || text_lower.contains("subject alternative")
        {
            ServoCertErrorCode::CommonNameInvalid
        } else if text_lower.contains("revoked") {
            ServoCertErrorCode::Revoked
        } else if text_lower.contains("incomplete chain") || text_lower.contains("chain incomplete")
        {
            ServoCertErrorCode::IncompleteChain
        } else if text_lower.contains("weak signature")
            || text_lower.contains("sha1")
            || text_lower.contains("md5")
        {
            ServoCertErrorCode::WeakSignatureAlgorithm
        } else if text_lower.contains("weak key")
            || text_lower.contains("key size")
            || text_lower.contains("rsa.*1024")
        {
            ServoCertErrorCode::WeakKey
        } else if text_lower.contains("transparency") || text_lower.contains("ct log") {
            ServoCertErrorCode::TransparencyRequired
        } else if text_lower.contains("symantec") || text_lower.contains("legacy") {
            ServoCertErrorCode::SymantecLegacy
        } else if text_lower.contains("invalid") || text_lower.contains("validation failed") {
            ServoCertErrorCode::Invalid
        } else {
            ServoCertErrorCode::Other
        }
    }
}

/// C-compatible constants for certificate error codes.
/// These are provided for C callers that cannot use the enum directly.
pub const SERVO_CERT_ERROR_NONE: i32 = 0;
pub const SERVO_CERT_ERROR_EXPIRED: i32 = 1;
pub const SERVO_CERT_ERROR_NOT_YET_VALID: i32 = 2;
pub const SERVO_CERT_ERROR_SELF_SIGNED: i32 = 3;
pub const SERVO_CERT_ERROR_UNKNOWN_CA: i32 = 4;
pub const SERVO_CERT_ERROR_COMMON_NAME_INVALID: i32 = 5;
pub const SERVO_CERT_ERROR_REVOKED: i32 = 6;
pub const SERVO_CERT_ERROR_INVALID: i32 = 7;
pub const SERVO_CERT_ERROR_INCOMPLETE_CHAIN: i32 = 8;
pub const SERVO_CERT_ERROR_WEAK_SIGNATURE_ALGORITHM: i32 = 9;
pub const SERVO_CERT_ERROR_WEAK_KEY: i32 = 10;
pub const SERVO_CERT_ERROR_INVALID_USAGE: i32 = 11;
pub const SERVO_CERT_ERROR_TRANSPARENCY_REQUIRED: i32 = 12;
pub const SERVO_CERT_ERROR_SYMANTEC_LEGACY: i32 = 13;
pub const SERVO_CERT_ERROR_OTHER: i32 = -1;

/// Certificate error information passed to the embedder.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ServoCertificateError {
    /// The URL that triggered the certificate error (NUL-terminated UTF-8).
    pub url: *const c_char,
    /// The certificate error code.
    pub error_code: c_int,
    /// Human-readable error description (NUL-terminated UTF-8).
    pub error_text: *const c_char,
    /// The DER-encoded certificate that caused the error.
    pub certificate_der: *const u8,
    /// Length of the certificate DER data.
    pub certificate_der_len: usize,
}

/// Callback type for certificate error decisions.
///
/// The embedder must call this callback to indicate whether to allow or deny
/// the request despite the certificate error.
///
/// - `allow`: `true` to proceed with the connection (certificate will be added to override list),
///   `false` to cancel the request with a certificate error.
/// - `user_data`: The opaque pointer passed to `on_certificate_error`.
pub type ServoCertErrorCallback = Option<unsafe extern "C" fn(allow: bool, user_data: *mut c_void)>;

/// Continue a certificate error decision.
///
/// This function must be called by the embedder in response to the
/// `on_certificate_error` delegate callback to indicate whether to proceed
/// with the connection despite the certificate error.
///
/// # Safety
///
/// - `callback` must be a valid function pointer previously provided to
///   the `on_certificate_error` callback.
/// - `user_data` must be the same pointer passed to `on_certificate_error`.
/// - This function must not be called after the `on_certificate_error` callback returns
///   unless the callback was explicitly designed to support deferred decisions.
/// - The callback must not unwind across the FFI boundary.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_certificate_error_continue(
    callback: ServoCertErrorCallback,
    allow: bool,
    user_data: *mut c_void,
) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if let Some(cb) = callback {
            // SAFETY: The caller is responsible for ensuring the callback and user_data
            // are valid and that the callback doesn't unwind.
            unsafe { cb(allow, user_data) };
        }
    }));
}

impl From<ServoCertErrorCode> for i32 {
    fn from(code: ServoCertErrorCode) -> Self {
        code as i32
    }
}

impl From<i32> for ServoCertErrorCode {
    fn from(code: i32) -> Self {
        match code {
            0 => ServoCertErrorCode::None,
            1 => ServoCertErrorCode::Expired,
            2 => ServoCertErrorCode::NotYetValid,
            3 => ServoCertErrorCode::SelfSigned,
            4 => ServoCertErrorCode::UnknownCA,
            5 => ServoCertErrorCode::CommonNameInvalid,
            6 => ServoCertErrorCode::Revoked,
            7 => ServoCertErrorCode::Invalid,
            8 => ServoCertErrorCode::IncompleteChain,
            9 => ServoCertErrorCode::WeakSignatureAlgorithm,
            10 => ServoCertErrorCode::WeakKey,
            11 => ServoCertErrorCode::InvalidUsage,
            12 => ServoCertErrorCode::TransparencyRequired,
            13 => ServoCertErrorCode::SymantecLegacy,
            _ => ServoCertErrorCode::Other,
        }
    }
}
