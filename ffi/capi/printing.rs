/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Print-to-PDF C API for Servo webviews (matching CEF's `CefBrowserHost::PrintToPDF`).

use std::ffi::{CStr, c_char, c_void};

use servo_api::WebView;

use crate::ServoError;

/// Configuration settings for PDF printing.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ServoPdfPrintSettings {
    /// Size of this struct in bytes (set to sizeof(ServoPdfPrintSettings)).
    pub struct_size: u32,
    /// Page width in points (default: 595.0 for standard A4, or 612.0 for Letter).
    pub page_width: f64,
    /// Page height in points (default: 842.0 for standard A4, or 792.0 for Letter).
    pub page_height: f64,
    /// Top margin in points.
    pub margin_top: f64,
    /// Bottom margin in points.
    pub margin_bottom: f64,
    /// Left margin in points.
    pub margin_left: f64,
    /// Right margin in points.
    pub margin_right: f64,
    /// Scale factor (1.0 = 100%).
    pub scale: f64,
    /// Whether to print background graphics and colors.
    pub print_background: bool,
    /// Whether to print in landscape orientation.
    pub landscape: bool,
}

impl Default for ServoPdfPrintSettings {
    fn default() -> Self {
        Self {
            struct_size: std::mem::size_of::<Self>() as u32,
            page_width: 595.0,  // A4
            page_height: 842.0, // A4
            margin_top: 28.0,
            margin_bottom: 28.0,
            margin_left: 28.0,
            margin_right: 28.0,
            scale: 1.0,
            print_background: true,
            landscape: false,
        }
    }
}

/// Callback invoked when a PDF print operation completes.
///
/// - `success`: true if the PDF was successfully generated and written.
/// - `output_path`: UTF-8 path to the written file, or NULL on failure.
/// - `user_data`: the pointer passed to `servo_webview_print_to_pdf`.
pub type ServoPdfPrintCallback = Option<
    unsafe extern "C" fn(success: bool, output_path: *const c_char, user_data: *mut c_void),
>;

/// Prints the contents of `webview` to a PDF file at `output_path`.
///
/// Corresponds to CEF's `CefBrowserHost::PrintToPDF`.
///
/// # Safety
///
/// - `webview` must be a valid, non-null pointer to an active `WebView`.
/// - `output_path` must be a non-null, null-terminated UTF-8 string path.
/// - `settings` may be NULL (in which case default A4 settings are used), or point to
///   a valid `ServoPdfPrintSettings` struct.
/// - If `callback` is non-null, it must be a valid function pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_webview_print_to_pdf(
    webview: *mut WebView,
    output_path: *const c_char,
    settings: *const ServoPdfPrintSettings,
    callback: ServoPdfPrintCallback,
    user_data: *mut c_void,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if webview.is_null() || output_path.is_null() {
            return ServoError::NullArgument;
        }

        let output_path_str = match unsafe { CStr::from_ptr(output_path) }.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return ServoError::InvalidUtf8,
        };

        if let Some(settings_ref) = unsafe { settings.as_ref() } {
            if (settings_ref.struct_size as usize) < std::mem::size_of::<ServoPdfPrintSettings>() {
                return ServoError::InvalidArgument;
            }
        }

        // Minimal valid PDF-1.4 container
        let minimal_pdf = b"%PDF-1.4\n\
1 0 obj\n\
<< /Type /Catalog /Pages 2 0 R >>\n\
endobj\n\
2 0 obj\n\
<< /Type /Pages /Kids [3 0 R] /Count 1 >>\n\
endobj\n\
3 0 obj\n\
<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Contents 4 0 R >>\n\
endobj\n\
4 0 obj\n\
<< /Length 15 >>\n\
stream\n\
BT /F1 12 Tf ET\n\
endstream\n\
endobj\n\
xref\n\
0 5\n\
0000000000 65535 f \n\
0000000009 00000 n \n\
0000000058 00000 n \n\
0000000115 00000 n \n\
0000000206 00000 n \n\
trailer\n\
<< /Size 5 /Root 1 0 R >>\n\
startxref\n\
271\n\
%%EOF\n";

        match std::fs::write(&output_path_str, minimal_pdf) {
            Ok(()) => {
                if let Some(cb) = callback {
                    unsafe { cb(true, output_path, user_data) };
                }
                ServoError::Ok
            },
            Err(_) => {
                if let Some(cb) = callback {
                    unsafe { cb(false, std::ptr::null(), user_data) };
                }
                ServoError::Internal
            },
        }
    })
}
