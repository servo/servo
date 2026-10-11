/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::{CStr, CString, c_char, c_void};
use std::path::PathBuf;

use embedder_traits::{
    ConsoleLogLevel, Cursor, PermissionFeature, WebResourceRequest, WebResourceResponse,
};
use url::Url;

use crate::download::{
    ServoDownloadContinueCallback, ServoDownloadItem, ServoDownloadUpdatedCallback,
};
use crate::find::{ServoFindResult, ServoFindResultCallback};
use crate::protocol_handler::{ServoWebResourceRequest, ServoWebResourceResponse};
pub use servo_api::LoadStatus;
use servo_api::{WebView, WebViewDelegate};

/// Cursor types matching `embedder_traits::Cursor`.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServoCursorType {
    Pointer = 0,
    Crosshair = 1,
    Hand = 2,
    IBeam = 3,
    Wait = 4,
    Help = 5,
    EastResize = 6,
    NorthResize = 7,
    NorthEastResize = 8,
    NorthWestResize = 9,
    SouthResize = 10,
    SouthEastResize = 11,
    SouthWestResize = 12,
    WestResize = 13,
    NorthSouthResize = 14,
    EastWestResize = 15,
    NorthEastSouthWestResize = 16,
    NorthWestSouthEastResize = 17,
    ColumnResize = 18,
    RowResize = 19,
    MiddlePanning = 20,
    EastPanning = 21,
    NorthPanning = 22,
    NorthEastPanning = 23,
    NorthWestPanning = 24,
    SouthPanning = 25,
    SouthEastPanning = 26,
    SouthWestPanning = 27,
    WestPanning = 28,
    Move = 29,
    VerticalText = 30,
    Cell = 31,
    ContextMenu = 32,
    Alias = 33,
    Progress = 34,
    NoDrop = 35,
    Copy = 36,
    NotAllowed = 37,
    Grab = 38,
    Grabbing = 39,
    Custom = 40,
    None = 41,
    Auto = 42,
}

impl From<Cursor> for ServoCursorType {
    fn from(cursor: Cursor) -> Self {
        match cursor {
            Cursor::Pointer => ServoCursorType::Pointer,
            Cursor::Crosshair => ServoCursorType::Crosshair,
            Cursor::Hand => ServoCursorType::Hand,
            Cursor::IBeam => ServoCursorType::IBeam,
            Cursor::Wait => ServoCursorType::Wait,
            Cursor::Help => ServoCursorType::Help,
            Cursor::EastResize => ServoCursorType::EastResize,
            Cursor::NorthResize => ServoCursorType::NorthResize,
            Cursor::NorthEastResize => ServoCursorType::NorthEastResize,
            Cursor::NorthWestResize => ServoCursorType::NorthWestResize,
            Cursor::SouthResize => ServoCursorType::SouthResize,
            Cursor::SouthEastResize => ServoCursorType::SouthEastResize,
            Cursor::SouthWestResize => ServoCursorType::SouthWestResize,
            Cursor::WestResize => ServoCursorType::WestResize,
            Cursor::NorthSouthResize => ServoCursorType::NorthSouthResize,
            Cursor::EastWestResize => ServoCursorType::EastWestResize,
            Cursor::NorthEastSouthWestResize => ServoCursorType::NorthEastSouthWestResize,
            Cursor::NorthWestSouthEastResize => ServoCursorType::NorthWestSouthEastResize,
            Cursor::ColumnResize => ServoCursorType::ColumnResize,
            Cursor::RowResize => ServoCursorType::RowResize,
            Cursor::MiddlePanning => ServoCursorType::MiddlePanning,
            Cursor::EastPanning => ServoCursorType::EastPanning,
            Cursor::NorthPanning => ServoCursorType::NorthPanning,
            Cursor::NorthEastPanning => ServoCursorType::NorthEastPanning,
            Cursor::NorthWestPanning => ServoCursorType::NorthWestPanning,
            Cursor::SouthPanning => ServoCursorType::SouthPanning,
            Cursor::SouthEastPanning => ServoCursorType::SouthEastPanning,
            Cursor::SouthWestPanning => ServoCursorType::SouthWestPanning,
            Cursor::WestPanning => ServoCursorType::WestPanning,
            Cursor::Move => ServoCursorType::Move,
            Cursor::VerticalText => ServoCursorType::VerticalText,
            Cursor::Cell => ServoCursorType::Cell,
            Cursor::ContextMenu => ServoCursorType::ContextMenu,
            Cursor::Alias => ServoCursorType::Alias,
            Cursor::Progress => ServoCursorType::Progress,
            Cursor::NoDrop => ServoCursorType::NoDrop,
            Cursor::Copy => ServoCursorType::Copy,
            Cursor::NotAllowed => ServoCursorType::NotAllowed,
            Cursor::Grab => ServoCursorType::Grab,
            Cursor::Grabbing => ServoCursorType::Grabbing,
            Cursor::Custom => ServoCursorType::Custom,
            Cursor::None => ServoCursorType::None,
            Cursor::Auto => ServoCursorType::Auto,
        }
    }
}

/// Console log levels matching `embedder_traits::ConsoleLogLevel`.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServoConsoleLogLevel {
    Debug = 0,
    Log = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl From<ConsoleLogLevel> for ServoConsoleLogLevel {
    fn from(level: ConsoleLogLevel) -> Self {
        match level {
            ConsoleLogLevel::Debug => ServoConsoleLogLevel::Debug,
            ConsoleLogLevel::Log => ServoConsoleLogLevel::Log,
            ConsoleLogLevel::Info => ServoConsoleLogLevel::Info,
            ConsoleLogLevel::Warn => ServoConsoleLogLevel::Warn,
            ConsoleLogLevel::Error => ServoConsoleLogLevel::Error,
        }
    }
}

/// Permission feature types that can be requested by web content.
///
/// These correspond to the `PermissionFeature` enum in the Rust API.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServoPermissionType {
    Geolocation = 0,
    Notifications = 1,
    Push = 2,
    Midi = 3,
    Camera = 4,
    Microphone = 5,
    Speaker = 6,
    DeviceInfo = 7,
    BackgroundSync = 8,
    Bluetooth = 9,
    PersistentStorage = 10,
    ScreenWakeLock = 11,
    Gamepad = 12,
}

impl From<PermissionFeature> for ServoPermissionType {
    fn from(feature: PermissionFeature) -> Self {
        match feature {
            PermissionFeature::Geolocation => ServoPermissionType::Geolocation,
            PermissionFeature::Notifications => ServoPermissionType::Notifications,
            PermissionFeature::Push => ServoPermissionType::Push,
            PermissionFeature::Midi => ServoPermissionType::Midi,
            PermissionFeature::Camera => ServoPermissionType::Camera,
            PermissionFeature::Microphone => ServoPermissionType::Microphone,
            PermissionFeature::Speaker => ServoPermissionType::Speaker,
            PermissionFeature::DeviceInfo => ServoPermissionType::DeviceInfo,
            PermissionFeature::BackgroundSync => ServoPermissionType::BackgroundSync,
            PermissionFeature::Bluetooth => ServoPermissionType::Bluetooth,
            PermissionFeature::PersistentStorage => ServoPermissionType::PersistentStorage,
            PermissionFeature::ScreenWakeLock(_) => ServoPermissionType::ScreenWakeLock,
            PermissionFeature::Gamepad => ServoPermissionType::Gamepad,
        }
    }
}

/// Filter pattern for file picker dialogs.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ServoFileFilterPattern {
    /// The filter pattern string (e.g., "image/*", ".pdf", "doc").
    pub pattern: *const c_char,
}

/// Parameters for a file picker dialog.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ServoFilePickerParams {
    /// The origin of the request (NUL terminated UTF-8 string).
    pub origin: *const c_char,
    /// Array of currently selected file paths (NUL terminated UTF-8 strings).
    pub current_paths: *const *const c_char,
    /// Number of currently selected file paths.
    pub current_paths_count: usize,
    /// Array of filter patterns.
    pub filter_patterns: *const ServoFileFilterPattern,
    /// Number of filter patterns.
    pub filter_patterns_count: usize,
    /// Whether multiple files can be selected.
    pub allow_select_multiple: bool,
}

/// A single option in a select element.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ServoSelectElementOption {
    /// Unique identifier for the option.
    pub id: usize,
    /// Label to display to the user (NUL terminated UTF-8 string).
    pub label: *const c_char,
    /// Whether the option is disabled.
    pub is_disabled: bool,
}

/// An option or optgroup in a select element.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ServoSelectElementOptionOrOptgroup {
    /// Type tag: 0 = option, 1 = optgroup.
    pub tag: u8,
    /// Option data (valid when tag == 0).
    pub option: ServoSelectElementOption,
    /// Optgroup label (valid when tag == 1).
    pub optgroup_label: *const c_char,
    /// Optgroup options (valid when tag == 1).
    pub optgroup_options: *const ServoSelectElementOption,
    /// Number of options in optgroup (valid when tag == 1).
    pub optgroup_options_count: usize,
}

/// Parameters for a select element dialog.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ServoSelectElementParams {
    /// Array of options and optgroups.
    pub options: *const ServoSelectElementOptionOrOptgroup,
    /// Number of options/optgroups.
    pub options_count: usize,
    /// Array of currently selected option indices.
    pub selected_options: *const usize,
    /// Number of currently selected options.
    pub selected_options_count: usize,
    /// Whether multiple options can be selected.
    pub allow_select_multiple: bool,
}

/// Callback type for alert dialog responses.
///
/// `result` is always `true` (the user pressed OK).
pub type ServoAlertDialogCallback =
    Option<unsafe extern "C" fn(result: bool, user_data: *mut c_void)>;

/// Callback type for confirm dialog responses.
///
/// `result` is `true` for OK, `false` for Cancel.
pub type ServoConfirmDialogCallback =
    Option<unsafe extern "C" fn(result: bool, user_data: *mut c_void)>;

/// Callback type for prompt dialog responses.
///
/// `result` is the entered text (NUL terminated UTF-8 string) for OK,
/// or `NULL` for Cancel. The string is owned by the embedder and must
/// remain valid until the callback returns.
pub type ServoPromptDialogCallback =
    Option<unsafe extern "C" fn(result: *const c_char, user_data: *mut c_void)>;

/// Callback type for file picker responses.
///
/// `files` is an array of NUL terminated UTF-8 file path strings.
/// `count` is the number of files in the array. Both are `NULL`/0 for Cancel.
/// The strings are owned by the embedder and must remain valid until
/// the callback returns.
pub type ServoFilePickerCallback =
    Option<unsafe extern "C" fn(files: *const *const c_char, count: usize, user_data: *mut c_void)>;

/// Callback type for select element responses.
///
/// `selected_indices` is an array of selected option indices.
/// `count` is the number of selected indices.
/// The array is owned by the embedder and must remain valid until
/// the callback returns.
pub type ServoSelectElementCallback = Option<
    unsafe extern "C" fn(selected_indices: *const usize, count: usize, user_data: *mut c_void),
>;

/// Callback type for permission request responses.
///
/// `granted` is `true` if permission is granted, `false` if denied.
pub type ServoPermissionCallback =
    Option<unsafe extern "C" fn(granted: bool, user_data: *mut c_void)>;

/// Callback type for authentication request responses.
///
/// `username` and `password` are NUL terminated UTF-8 strings for the
/// credentials, or both `NULL` to cancel. The strings are owned by the
/// embedder and must remain valid until the callback returns.
pub type ServoAuthenticationCallback = Option<
    unsafe extern "C" fn(username: *const c_char, password: *const c_char, user_data: *mut c_void),
>;

/// Callback type for before-unload responses.
///
/// `proceed` is `true` to allow the unload, `false` to cancel it.
pub type ServoUnloadCallback = Option<unsafe extern "C" fn(proceed: bool, user_data: *mut c_void)>;

/// Callback type for certificate error responses.
///
/// `allow` is `true` to proceed with the connection despite the certificate error
/// (the certificate will be added to the override list), `false` to cancel the request.
pub type ServoCertErrorCallback = Option<unsafe extern "C" fn(allow: bool, user_data: *mut c_void)>;

/// Callback type for file picker dialog responses.
///
/// `paths` is an array of NUL-terminated UTF-8 file path strings, or NULL to cancel.
/// `path_count` is the number of file paths in `paths`.
/// `user_data` is the user data pointer passed to the trampoline.
pub type ServoFilePickerCallback = Option<
    unsafe extern "C" fn(
        paths: *const *const c_char,
        path_count: usize,
        user_data: *mut c_void,
    ),
>;

/// The delegate that receives notifications about `WebView` events.
///
/// The function pointers can be set to `NULL` for callbacks that are not
/// needed. All callbacks receive the `user_data` pointer that was set in
/// this struct. The callbacks are invoked on the embedder thread that
/// calls `servo_spin_event_loop`.
///
/// Refer to the documentation of the corresponding
/// [`servo::WebViewDelegate`] trait in the Rust API for more information.
///
/// [`servo::WebViewDelegate`]: https://doc.servo.org/servo/trait.WebViewDelegate.html
///
/// # Ownership of callback arguments
///
/// The `webview` argument passed to each callback is a temporary handle.
/// Its lifetime is limited to the duration of the callback. The
/// embedder must not retain or use handle after the callback returns.
/// The callback must not pass it to `servo_webview_free` or any other
/// function that takes ownership of a `WebView`.
///
/// The `user_data` pointer is owned by the embedder.
/// The validity and lifetime of `user_data` is the embedder's responsibility.
///
/// # Safety
///
/// The embedder must ensure that for all function-pointer fields of
/// this struct:
///
/// - A non-null function pointer is a valid C ABI callback function
///   matching the exact signature shown.
/// - The function pointers and `user_data` remain valid for as long as
///   this delegate is associated with any `WebView`.
/// - The callback does not unwind across the FFI boundary.
/// - The `webview` argument is not retained or used after the callback
///   returns and is not passed to `servo_webview_free` or any other
///   function that takes ownership of the `WebView`.
#[repr(C)]
#[derive(Clone)]
pub struct ServoWebViewDelegate {
    /// An opaque pointer passed to all delegate callbacks. May be `NULL`.
    pub user_data: *mut c_void,

    /// Called when the load status of the associated `WebView` changes.
    ///
    /// `load_status` is one of the `SERVO_LOAD_STATUS_*` constants.
    pub notify_load_status_changed: Option<
        unsafe extern "C" fn(webview: *mut WebView, load_status: i32, user_data: *mut c_void),
    >,

    /// Called when the URL of the associated `WebView` changes.
    ///
    /// `url` is a NUL terminated UTF-8 string that is only valid for the
    /// duration of the callback. The embedder must copy it if it needs to
    /// retain the value.
    pub notify_url_changed: Option<
        unsafe extern "C" fn(webview: *mut WebView, url: *const c_char, user_data: *mut c_void),
    >,

    /// Called when the title of the associated `WebView` changes.
    ///
    /// `title` is a NUL terminated UTF-8 string that is only valid for the
    /// duration of the callback. The embedder must copy it if it needs to
    /// retain the value.
    pub notify_page_title_changed: Option<
        unsafe extern "C" fn(webview: *mut WebView, title: *const c_char, user_data: *mut c_void),
    >,

    /// Called when Servo has rendered a new frame and the embedder should
    /// call `servo_webview_paint` to update the rendering context.
    pub notify_new_frame_ready:
        Option<unsafe extern "C" fn(webview: *mut WebView, user_data: *mut c_void)>,

    // Focus category
    /// Called when the focus state of the associated `WebView` changes.
    ///
    /// `focused` is `true` if the WebView gained focus, `false` if it lost focus.
    pub notify_focus_changed:
        Option<unsafe extern "C" fn(webview: *mut WebView, focused: bool, user_data: *mut c_void)>,

    /// Called when the associated `WebView` gained focus.
    ///
    /// This is a more specific notification than `notify_focus_changed` and is
    /// called when the WebView becomes the active focus target.
    pub notify_got_focus:
        Option<unsafe extern "C" fn(webview: *mut WebView, user_data: *mut c_void)>,

    // Load category (new)
    /// Called when a load has started in the associated `WebView`.
    ///
    /// This is called when the load status transitions to `SERVO_LOAD_STATUS_STARTED`.
    pub notify_load_started:
        Option<unsafe extern "C" fn(webview: *mut WebView, user_data: *mut c_void)>,

    /// Called when a load has ended (completed or failed) in the associated `WebView`.
    ///
    /// This is called when the load status transitions to `SERVO_LOAD_STATUS_COMPLETE`
    /// or when a load error occurs.
    pub notify_load_ended:
        Option<unsafe extern "C" fn(webview: *mut WebView, user_data: *mut c_void)>,

    /// Called when a load error occurs in the associated `WebView`.
    ///
    /// `error_code` is a platform-specific error code. `error_text` is a NUL terminated
    /// UTF-8 string describing the error, valid only for the duration of the callback.
    pub notify_load_error: Option<
        unsafe extern "C" fn(
            webview: *mut WebView,
            error_code: i32,
            error_text: *const c_char,
            user_data: *mut c_void,
        ),
    >,

    // Display category (new)
    /// Called when the cursor type changes in the associated `WebView`.
    ///
    /// `cursor_type` is one of the `SERVO_CURSOR_*` constants.
    pub notify_cursor_changed: Option<
        unsafe extern "C" fn(webview: *mut WebView, cursor_type: i32, user_data: *mut c_void),
    >,

    /// Called when the status text changes in the associated `WebView`.
    ///
    /// `text` is a NUL terminated UTF-8 string that is only valid for the
    /// duration of the callback. The embedder must copy it if it needs to
    /// retain the value. `text` may be `NULL` to indicate no status text.
    pub notify_status_text_changed: Option<
        unsafe extern "C" fn(webview: *mut WebView, text: *const c_char, user_data: *mut c_void),
    >,

    /// Called when the favicon URL changes in the associated `WebView`.
    ///
    /// `url` is a NUL terminated UTF-8 string that is only valid for the
    /// duration of the callback. The embedder must copy it if it needs to
    /// retain the value. `url` may be `NULL` to indicate no favicon.
    pub notify_favicon_changed: Option<
        unsafe extern "C" fn(webview: *mut WebView, url: *const c_char, user_data: *mut c_void),
    >,

    /// Called when a console message is logged by page content in the associated `WebView`.
    ///
    /// `level` is one of the `SERVO_CONSOLE_LOG_LEVEL_*` constants.
    /// `message` is a NUL terminated UTF-8 string that is only valid for the
    /// duration of the callback.
    pub show_console_message: Option<
        unsafe extern "C" fn(
            webview: *mut WebView,
            level: i32,
            message: *const c_char,
            user_data: *mut c_void,
        ),
    >,

    // Dialog category (new)
    /// Called when an `alert()` dialog should be shown.
    ///
    /// `message` is the dialog message (NUL terminated UTF-8 string).
    /// The embedder should show the dialog and call `callback` with `true`
    /// when the user dismisses it.
    pub show_alert_dialog: Option<
        unsafe extern "C" fn(
            webview: *mut WebView,
            message: *const c_char,
            callback: ServoAlertDialogCallback,
            user_data: *mut c_void,
        ),
    >,

    /// Called when a `confirm()` dialog should be shown.
    ///
    /// `message` is the dialog message (NUL terminated UTF-8 string).
    /// The embedder should show the dialog and call `callback` with `true`
    /// for OK or `false` for Cancel.
    pub show_confirm_dialog: Option<
        unsafe extern "C" fn(
            webview: *mut WebView,
            message: *const c_char,
            callback: ServoConfirmDialogCallback,
            user_data: *mut c_void,
        ),
    >,

    /// Called when a `prompt()` dialog should be shown.
    ///
    /// `message` is the dialog message (NUL terminated UTF-8 string).
    /// `default_value` is the initial value for the text field (NUL terminated UTF-8 string).
    /// The embedder should show the dialog and call `callback` with the
    /// entered text for OK, or `NULL` for Cancel.
    pub show_prompt_dialog: Option<
        unsafe extern "C" fn(
            webview: *mut WebView,
            message: *const c_char,
            default_value: *const c_char,
            callback: ServoPromptDialogCallback,
            user_data: *mut c_void,
        ),
    >,

    // Embedder control category (new)
    /// Called when a file picker dialog should be shown (for `<input type=file>`).
    ///
    /// `params` contains the file picker parameters.
    /// The embedder should show the dialog and call `callback` with the
    /// selected files, or `NULL`/0 for Cancel.
    pub show_file_picker: Option<
        unsafe extern "C" fn(
            webview: *mut WebView,
            params: *const ServoFilePickerParams,
            callback: ServoFilePickerCallback,
            user_data: *mut c_void,
        ),
    >,

    /// Called when a select element dropdown should be shown (for `<select>`).
    ///
    /// `params` contains the select element parameters.
    /// The embedder should show the dropdown and call `callback` with the
    /// selected option indices.
    pub show_select_element: Option<
        unsafe extern "C" fn(
            webview: *mut WebView,
            params: *const ServoSelectElementParams,
            callback: ServoSelectElementCallback,
            user_data: *mut c_void,
        ),
    >,

    // Permission category (new)
    /// Called when web content requests permission to access a feature.
    ///
    /// `origin` is the requesting origin (NUL terminated UTF-8 string).
    /// `permission` is the permission type being requested.
    /// The embedder should allow or deny the request and call `callback`
    /// with `true` for granted or `false` for denied.
    pub request_permission: Option<
        unsafe extern "C" fn(
            webview: *mut WebView,
            origin: *const c_char,
            permission: ServoPermissionType,
            callback: ServoPermissionCallback,
            user_data: *mut c_void,
        ),
    >,

    // Auth category (new)
    /// Called when HTTP authentication is required.
    ///
    /// `host` is the hostname (NUL terminated UTF-8 string).
    /// `port` is the port number.
    /// `realm` is the authentication realm (NUL terminated UTF-8 string).
    /// The embedder should prompt for credentials and call `callback` with
    /// the username and password, or both `NULL` to cancel.
    pub request_authentication: Option<
        unsafe extern "C" fn(
            webview: *mut WebView,
            host: *const c_char,
            port: i32,
            realm: *const c_char,
            callback: ServoAuthenticationCallback,
            user_data: *mut c_void,
        ),
    >,

    // Certificate error category (new)
    /// Called when a certificate error occurs during a navigation request.
    ///
    /// `url` is the URL that triggered the error (NUL terminated UTF-8 string).
    /// `error_code` is one of the `SERVO_CERT_ERROR_*` constants.
    /// `error_text` is a human-readable error description (NUL terminated UTF-8 string).
    /// `certificate_der` is a pointer to the DER-encoded certificate data.
    /// `certificate_der_len` is the length of the certificate data.
    /// `callback` is a function to call with the decision (`true` to allow, `false` to deny).
    /// The embedder must call `callback` exactly once to indicate the decision.
    pub on_certificate_error: Option<
        unsafe extern "C" fn(
            webview: *mut WebView,
            url: *const c_char,
            error_code: i32,
            error_text: *const c_char,
            certificate_der: *const u8,
            certificate_der_len: usize,
            callback: ServoCertErrorCallback,
            user_data: *mut c_void,
        ),
    >,

    // Before unload category (new)
    /// Called when the page is about to be unloaded (beforeunload event).
    ///
    /// The embedder should confirm with the user and call `callback` with
    /// `true` to proceed with unload, or `false` to cancel.
    pub request_unload: Option<
        unsafe extern "C" fn(
            webview: *mut WebView,
            callback: ServoUnloadCallback,
            user_data: *mut c_void,
        ),
    >,

    // Resource interception category (new)
    /// Called when a web resource is about to be loaded. The embedder can
    /// choose to intercept the load and provide a custom response.
    ///
    /// Return `true` to intercept the load (the response must be populated
    /// via the `response` out-parameter), `false` to let the load proceed
    /// normally.
    ///
    /// The `request` and `response` pointers are only valid for the duration
    /// of this callback. The embedder must not retain them.
    pub intercept_request: Option<
        unsafe extern "C" fn(
            webview: *mut WebView,
            request: *const ServoWebResourceRequest,
            response: *mut ServoWebResourceResponse,
            user_data: *mut c_void,
        ) -> bool,
    >,

    // Lifecycle category (new)
    /// Called when the associated `WebView` is about to be closed.
    ///
    /// The embedder should release any resources associated with the WebView
    /// but must not call `servo_webview_free` from within this callback.
    /// The WebView will be automatically freed after this callback returns.
    pub notify_closed: Option<unsafe extern "C" fn(webview: *mut WebView, user_data: *mut c_void)>,

    // Download category (new)
    /// Called when a download is requested (before it starts).
    ///
    /// `download` contains information about the requested download.
    /// The embedder should call `callback` with `true` to allow the download
    /// to proceed, or `false` to cancel it. The `download` pointer is only
    /// valid for the duration of this callback unless the embedder calls
    /// `callback(true)`, in which case the download item remains valid and
    /// the embedder can use the download API functions to query its status
    /// or control it.
    ///
    /// If this callback is not set, downloads are allowed by default.
    pub on_before_download: Option<
        unsafe extern "C" fn(
            webview: *mut WebView,
            download: *mut ServoDownloadItem,
            callback: ServoDownloadContinueCallback,
            user_data: *mut c_void,
        ),
    >,

    /// Called when a download's progress is updated.
    ///
    /// `download` contains the updated download information.
    /// The embedder can use `servo_download_item_get_*` functions to query
    /// the current progress and state.
    pub on_download_updated: Option<
        unsafe extern "C" fn(
            webview: *mut WebView,
            download: *mut ServoDownloadItem,
            user_data: *mut c_void,
        ),
    >,

    // Find category (new)
    /// Called when a find-in-page operation has a result.
    ///
    /// `identifier` is the identifier returned by `servo_webview_find`.
    /// `count` is the total number of matches found.
    /// `result` is a pointer to a `ServoFindResult` struct containing details
    /// about the active match, or `NULL` if no matches were found.
    /// The `result` pointer is only valid for the duration of this callback.
    pub on_find_result: Option<
        unsafe extern "C" fn(
            webview: *mut WebView,
            identifier: c_int,
            count: c_int,
            result: *const ServoFindResult,
            user_data: *mut c_void,
        ),
    >,

    // Window management category (new)
    /// Called when the window containing the `WebView` has been resized.
    ///
    /// `width` and `height` are the new window size in device pixels.
    pub on_window_resized: Option<
        unsafe extern "C" fn(
            webview: *mut WebView,
            width: u32,
            height: u32,
            user_data: *mut c_void,
        ),
    >,

    /// Called when the window containing the `WebView` has been moved.
    ///
    /// `x` and `y` are the new window position in device pixels.
    pub on_window_moved:
        Option<unsafe extern "C" fn(webview: *mut WebView, x: i32, y: i32, user_data: *mut c_void)>,

    /// Called when the window containing the `WebView` has been minimized.
    pub on_window_minimized:
        Option<unsafe extern "C" fn(webview: *mut WebView, user_data: *mut c_void)>,

    /// Called when the window containing the `WebView` has been maximized.
    pub on_window_maximized:
        Option<unsafe extern "C" fn(webview: *mut WebView, user_data: *mut c_void)>,

    /// Called when the window containing the `WebView` has been restored
    /// (from minimized or maximized state).
    pub on_window_restored:
        Option<unsafe extern "C" fn(webview: *mut WebView, user_data: *mut c_void)>,

    /// Called when the window containing the `WebView` has changed fullscreen state.
    ///
    /// `fullscreen` is `true` if the window entered fullscreen, `false` if it exited.
    pub on_window_fullscreen_changed: Option<
        unsafe extern "C" fn(webview: *mut WebView, fullscreen: bool, user_data: *mut c_void),
    >,

    /// Called when the user attempts to close the window containing the `WebView`.
    ///
    /// The embedder can return `false` to prevent the window from closing,
    /// or `true` to allow the close to proceed.
    pub on_window_close_requested:
        Option<unsafe extern "C" fn(webview: *mut WebView, user_data: *mut c_void) -> bool>,

    /// Called when a window move or resize operation has started
    /// (e.g., user dragging window title bar or resize handle).
    pub on_window_move_or_resize_started:
        Option<unsafe extern "C" fn(webview: *mut WebView, user_data: *mut c_void)>,

    // Animation category
    /// Called when the animating state changes.
    ///
    /// When `animating` is true, the embedder should ensure the event loop is pumped regularly.
    pub notify_animating_changed:
        Option<unsafe extern "C" fn(webview: *mut WebView, animating: bool, user_data: *mut c_void)>,

    // History category
    /// Called when the session history changes.
    ///
    /// `entries` is an array of NUL-terminated URL strings.
    /// `entries_count` is the number of URLs in the history.
    /// `current_index` is the index of the current entry in the history.
    pub notify_history_changed: Option<
        unsafe extern "C" fn(
            webview: *mut WebView,
            entries: *const *const c_char,
            entries_count: usize,
            current_index: usize,
            user_data: *mut c_void,
        ),
    >,

    // Crash category
    /// Called when a pipeline panics or crashes.
    ///
    /// `reason` describes the failure (NUL-terminated UTF-8 string).
    /// `backtrace` is the optional backtrace string (or NULL).
    pub notify_crashed: Option<
        unsafe extern "C" fn(
            webview: *mut WebView,
            reason: *const c_char,
            backtrace: *const c_char,
            user_data: *mut c_void,
        ),
    >,

    // IME category
    /// Called when an editable element is focused and requests an IME.
    /// `text` is the current text in the input element (NUL-terminated UTF-8).
    /// `cursor_pos` is the cursor insertion point, or -1 if not set.
    pub show_ime: Option<
        unsafe extern "C" fn(
            webview: *mut WebView,
            text: *const c_char,
            cursor_pos: i32,
            user_data: *mut c_void,
        ),
    >,
    /// Called when the editable element loses focus or requests dismissing the IME.
    pub hide_ime: Option<
        unsafe extern "C" fn(
            webview: *mut WebView,
            user_data: *mut c_void,
        ),
    >,

    // File picker category
    /// Called when the web page requests a file picker dialog (e.g. `<input type="file">`).
    /// `allow_multiple` is `true` if multiple file selection is permitted.
    /// `callback` must be invoked with selected file paths (or NULL/0 to cancel).
    /// `user_data` must be passed to `callback`.
    pub show_file_picker: Option<
        unsafe extern "C" fn(
            webview: *mut WebView,
            allow_multiple: bool,
            callback: ServoFilePickerCallback,
            user_data: *mut c_void,
        ),
    >,
}

/// Inherent methods for `ServoWebViewDelegate`.
impl ServoWebViewDelegate {
    /// Returns the user data pointer.
    pub fn user_data(&self) -> *mut c_void {
        self.user_data
    }
}

impl WebViewDelegate for ServoWebViewDelegate {
    fn notify_load_status_changed(&self, mut webview: WebView, load_status: LoadStatus) {
        let Some(callback) = self.notify_load_status_changed else {
            return;
        };

        let load_status = load_status as i32;

        // SAFETY: The embedder is assumed to uphold the safety requirements of the
        // `ServoWebViewDelegate` struct.
        //
        // The `webview` raw pointer is derived from a valid `webview` handle.
        unsafe { callback(&mut webview as *mut WebView, load_status, self.user_data) };

        // Also fire the new load start/end callbacks based on the load status.
        // LoadStatus::Started -> notify_load_started
        // LoadStatus::Complete -> notify_load_ended
        // (HeadParsed doesn't trigger start/end)
        match load_status {
            x if x == LoadStatus::Started as i32 => {
                if let Some(cb) = self.notify_load_started {
                    unsafe { cb(&mut webview as *mut WebView, self.user_data) };
                }
            },
            x if x == LoadStatus::Complete as i32 => {
                if let Some(cb) = self.notify_load_ended {
                    unsafe { cb(&mut webview as *mut WebView, self.user_data) };
                }
            },
            _ => {},
        }
    }

    fn notify_url_changed(&self, mut webview: WebView, url: Url) {
        let Some(callback) = self.notify_url_changed else {
            return;
        };

        // Interior NUL bytes cannot appear in a parsed URL, but guard against
        // conversion failure rather than panicking across the FFI boundary.
        let Ok(url) = CString::new(url.as_str()) else {
            return;
        };

        // SAFETY: The embedder is assumed to uphold the safety requirements of the
        // `ServoWebViewDelegate` struct.
        //
        // The `webview` raw pointer is derived from a valid `webview` handle.
        // `url` outlives the call.
        unsafe { callback(&mut webview as *mut WebView, url.as_ptr(), self.user_data) };
    }

    fn notify_page_title_changed(&self, mut webview: WebView, title: Option<String>) {
        let Some(callback) = self.notify_page_title_changed else {
            return;
        };

        // Guard against interior NUL bytes rather than panicking across the
        // FFI boundary.
        let Ok(title) = CString::new(title.unwrap_or_default()) else {
            return;
        };

        // SAFETY: The embedder is assumed to uphold the safety requirements of the
        // `ServoWebViewDelegate` struct.
        //
        // The `webview` raw pointer is derived from a valid `webview` handle.
        // `title` outlives the call.
        unsafe { callback(&mut webview as *mut WebView, title.as_ptr(), self.user_data) };
    }

    fn notify_new_frame_ready(&self, mut webview: WebView) {
        let Some(callback) = self.notify_new_frame_ready else {
            return;
        };

        // SAFETY: The embedder is assumed to uphold the safety requirements of the
        // `ServoWebViewDelegate` struct.
        //
        // The `webview` raw pointer is derived from a valid `webview` handle.
        unsafe { callback(&mut webview as *mut WebView, self.user_data) };
    }

    fn notify_focus_changed(&self, mut webview: WebView, focused: bool) {
        let Some(callback) = self.notify_focus_changed else {
            return;
        };

        // SAFETY: The embedder is assumed to uphold the safety requirements of the
        // `ServoWebViewDelegate` struct.
        //
        // The `webview` raw pointer is derived from a valid `webview` handle.
        unsafe { callback(&mut webview as *mut WebView, focused, self.user_data) };
    }

    fn notify_got_focus(&self, mut webview: WebView) {
        let Some(callback) = self.notify_got_focus else {
            return;
        };

        // SAFETY: The embedder is assumed to uphold the safety requirements of the
        // `ServoWebViewDelegate` struct.
        //
        // The `webview` raw pointer is derived from a valid `webview` handle.
        unsafe { callback(&mut webview as *mut WebView, self.user_data) };
    }

    fn notify_status_text_changed(&self, mut webview: WebView, status: Option<String>) {
        let Some(callback) = self.notify_status_text_changed else {
            return;
        };

        // Guard against interior NUL bytes rather than panicking across the
        // FFI boundary.
        let Ok(text) = CString::new(status.unwrap_or_default()) else {
            return;
        };

        // SAFETY: The embedder is assumed to uphold the safety requirements of the
        // `ServoWebViewDelegate` struct.
        //
        // The `webview` raw pointer is derived from a valid `webview` handle.
        // `text` outlives the call.
        unsafe { callback(&mut webview as *mut WebView, text.as_ptr(), self.user_data) };
    }

    fn notify_cursor_changed(&self, mut webview: WebView, cursor: Cursor) {
        let Some(callback) = self.notify_cursor_changed else {
            return;
        };

        let cursor_type = ServoCursorType::from(cursor) as i32;

        // SAFETY: The embedder is assumed to uphold the safety requirements of the
        // `ServoWebViewDelegate` struct.
        //
        // The `webview` raw pointer is derived from a valid `webview` handle.
        unsafe { callback(&mut webview as *mut WebView, cursor_type, self.user_data) };
    }

    fn notify_favicon_changed(&self, mut webview: WebView) {
        let Some(callback) = self.notify_favicon_changed else {
            return;
        };

        // The favicon URL is not currently exposed by the Servo API, so the
        // callback receives `NULL`. The favicon image itself can be queried via
        // `servo_webview_...` APIs.
        unsafe {
            callback(
                &mut webview as *mut WebView,
                std::ptr::null(),
                self.user_data,
            )
        };
    }

    fn show_console_message(&self, mut webview: WebView, level: ConsoleLogLevel, message: String) {
        let Some(callback) = self.show_console_message else {
            return;
        };

        let level_int = ServoConsoleLogLevel::from(level) as i32;

        // Guard against interior NUL bytes rather than panicking across the
        // FFI boundary.
        let Ok(msg) = CString::new(message) else {
            return;
        };

        // SAFETY: The embedder is assumed to uphold the safety requirements of the
        // `ServoWebViewDelegate` struct.
        //
        // The `webview` raw pointer is derived from a valid `webview` handle.
        // `msg` outlives the call.
        unsafe {
            callback(
                &mut webview as *mut WebView,
                level_int,
                msg.as_ptr(),
                self.user_data,
            )
        };
    }

    fn show_embedder_control(
        &self,
        mut webview: WebView,
        embedder_control: servo_api::EmbedderControl,
    ) {
        use servo_api::EmbedderControl;

        match embedder_control {
            EmbedderControl::SimpleDialog(simple_dialog) => {
                use servo_api::SimpleDialog;

                match simple_dialog {
                    SimpleDialog::Alert(alert) => {
                        let Some(callback) = self.show_alert_dialog else {
                            // Auto-confirm if no handler
                            alert.confirm();
                            return;
                        };
                        let Ok(message) = CString::new(alert.message()) else {
                            alert.confirm();
                            return;
                        };
                        let responder: Box<dyn FnOnce(bool)> = Box::new(move |_result: bool| {
                            alert.confirm();
                        });
                        let responder_ptr = Box::into_raw(Box::new(responder)) as *mut c_void;
                        unsafe {
                            callback(
                                &mut webview as *mut WebView,
                                message.as_ptr(),
                                Some(alert_trampoline),
                                responder_ptr,
                            );
                        }
                    },
                    SimpleDialog::Confirm(confirm) => {
                        let Some(callback) = self.show_confirm_dialog else {
                            confirm.dismiss();
                            return;
                        };
                        let Ok(message) = CString::new(confirm.message()) else {
                            confirm.dismiss();
                            return;
                        };
                        let responder: Box<dyn FnOnce(bool)> = Box::new(move |result: bool| {
                            if result {
                                confirm.confirm();
                            } else {
                                confirm.dismiss();
                            }
                        });
                        let responder_ptr = Box::into_raw(Box::new(responder)) as *mut c_void;
                        unsafe {
                            callback(
                                &mut webview as *mut WebView,
                                message.as_ptr(),
                                Some(confirm_trampoline),
                                responder_ptr,
                            );
                        }
                    },
                    SimpleDialog::Prompt(mut prompt) => {
                        let Some(callback) = self.show_prompt_dialog else {
                            prompt.dismiss();
                            return;
                        };
                        let Ok(message) = CString::new(prompt.message()) else {
                            prompt.dismiss();
                            return;
                        };
                        let Ok(default_val) = CString::new(prompt.current_value()) else {
                            prompt.dismiss();
                            return;
                        };
                        let responder: Box<dyn FnOnce(Option<String>)> = Box::new(move |result| {
                            if let Some(text) = result {
                                prompt.set_current_value(&text);
                                prompt.confirm();
                            } else {
                                prompt.dismiss();
                            }
                        });
                        let responder_ptr = Box::into_raw(Box::new(responder)) as *mut c_void;
                        unsafe {
                            callback(
                                &mut webview as *mut WebView,
                                message.as_ptr(),
                                default_val.as_ptr(),
                                Some(prompt_trampoline),
                                responder_ptr,
                            );
                        }
                    },
                }
            },
            EmbedderControl::FilePicker(mut file_picker) => {
                let Some(callback) = self.show_file_picker else {
                    file_picker.dismiss();
                    return;
                };
                let allow_multiple = file_picker.allow_select_multiple();
                let responder: Box<dyn FnOnce(Option<Vec<PathBuf>>)> = Box::new(move |paths| {
                    if let Some(paths) = paths {
                        file_picker.select(&paths);
                    } else {
                        file_picker.dismiss();
                    }
                });
                let responder_ptr = Box::into_raw(Box::new(responder)) as *mut c_void;
                unsafe {
                    callback(
                        &mut webview as *mut WebView,
                        allow_multiple,
                        Some(file_picker_trampoline),
                        responder_ptr,
                    );
                }
            },
            EmbedderControl::InputMethod(ime) => {
                if let Some(callback) = self.show_ime {
                    let text = CString::new(ime.text()).unwrap_or_default();
                    let cursor_pos = ime.insertion_point().map(|p| p as i32).unwrap_or(-1);
                    unsafe {
                        callback(
                            &mut webview as *mut WebView,
                            text.as_ptr(),
                            cursor_pos,
                            self.user_data,
                        );
                    }
                }
            },
            EmbedderControl::SelectElement(select_element) => {
                select_element.submit();
            },
            _ => {
                // ColorPicker, ContextMenu - not yet exposed to C
            },
        }
    }

    fn request_permission(&self, mut webview: WebView, request: servo_api::PermissionRequest) {
        let Some(callback) = self.request_permission else {
            request.deny();
            return;
        };

        // The origin of the requesting page is not currently exposed by the
        // Servo `PermissionRequest` API, so an empty origin is provided.
        let origin = String::new();
        let permission_type = ServoPermissionType::from(request.feature());

        let Ok(origin_cstr) = CString::new(origin) else {
            request.deny();
            return;
        };

        // Create responder
        let responder = move |granted: bool| {
            if granted {
                request.allow();
            } else {
                request.deny();
            }
        };
        let responder: Box<dyn FnOnce(bool)> = Box::new(responder);
        let responder_ptr = Box::into_raw(Box::new(responder)) as *mut c_void;

        // Call C callback with trampoline
        unsafe {
            callback(
                &mut webview as *mut WebView,
                origin_cstr.as_ptr(),
                permission_type,
                Some(permission_trampoline),
                responder_ptr,
            )
        };
    }

    fn request_authentication(
        &self,
        mut webview: WebView,
        request: servo_api::AuthenticationRequest,
    ) {
        let Some(callback) = self.request_authentication else {
            return; // Default: no credentials
        };

        let url = request.url();
        let host = url.host_str().unwrap_or("");
        let port = url
            .port()
            .unwrap_or(if url.scheme() == "https" { 443 } else { 80 });
        let realm = ""; // Realm not directly available, could be extracted from WWW-Authenticate header

        let Ok(host_cstr) = CString::new(host) else {
            return;
        };
        let Ok(realm_cstr) = CString::new(realm) else {
            return;
        };

        let responder = move |username: String, password: String| {
            request.authenticate(username, password);
        };
        let responder: Box<dyn FnOnce(String, String)> = Box::new(responder);
        let responder_ptr = Box::into_raw(Box::new(responder)) as *mut c_void;

        unsafe {
            callback(
                &mut webview as *mut WebView,
                host_cstr.as_ptr(),
                port as i32,
                realm_cstr.as_ptr(),
                Some(authentication_trampoline),
                responder_ptr,
            )
        };
    }

    fn certificate_error(
        &self,
        mut webview: WebView,
        certificate_error: servo_api::CertificateError,
        request: servo_api::CertificateErrorRequest,
    ) {
        let Some(callback) = self.on_certificate_error else {
            // Default: deny the request
            request.deny();
            return;
        };

        let Ok(url_cstr) = CString::new(certificate_error.url.as_str()) else {
            request.deny();
            return;
        };
        let Ok(error_text_cstr) = CString::new(certificate_error.error_text) else {
            request.deny();
            return;
        };

        // Create responder
        let responder = move |allow: bool| {
            if allow {
                request.allow();
            } else {
                request.deny();
            }
        };
        let responder: Box<dyn FnOnce(bool)> = Box::new(responder);
        let responder_ptr = Box::into_raw(Box::new(responder)) as *mut c_void;

        // Call C callback with trampoline
        unsafe {
            callback(
                &mut webview as *mut WebView,
                url_cstr.as_ptr(),
                certificate_error.error_code,
                error_text_cstr.as_ptr(),
                certificate_error.certificate_der.as_ptr(),
                certificate_error.certificate_der.len(),
                Some(certificate_error_trampoline),
                responder_ptr,
            )
        };
    }

    fn request_unload(&self, mut webview: WebView, request: servo_api::AllowOrDenyRequest) {
        let Some(callback) = self.request_unload else {
            request.allow(); // Default: allow unload
            return;
        };

        let responder = move |proceed: bool| {
            if proceed {
                request.allow();
            } else {
                request.deny();
            }
        };
        let responder: Box<dyn FnOnce(bool)> = Box::new(responder);
        let responder_ptr = Box::into_raw(Box::new(responder)) as *mut c_void;

        unsafe {
            callback(
                &mut webview as *mut WebView,
                Some(unload_trampoline),
                responder_ptr,
            )
        };
    }

    fn load_web_resource(&self, mut webview: WebView, load: servo_api::WebResourceLoad) {
        let Some(callback) = self.intercept_request else {
            return; // Default: don't intercept
        };

        // Convert the Rust request to a C request
        let c_request = ServoWebResourceRequest::from_web_resource_request(load.request());

        // Create a response object for the embedder to populate
        let request_url = load.request().url.as_url().clone();
        let mut c_response = ServoWebResourceResponse::new(request_url);

        // Call the C callback
        let intercepted = unsafe {
            callback(
                &mut webview as *mut WebView,
                &c_request as *const _,
                &mut c_response as *mut _,
                self.user_data,
            )
        };

        if intercepted {
            // Intercept the load with the provided response
            let response = c_response.into_web_resource_response();
            load.intercept(response);
        }
        // If not intercepted, the load proceeds normally (default behavior)
    }

    fn notify_closed(&self, mut webview: WebView) {
        let Some(callback) = self.notify_closed else {
            return;
        };

        // SAFETY: The embedder is assumed to uphold the safety requirements of the
        // `ServoWebViewDelegate` struct.
        //
        // The `webview` raw pointer is derived from a valid `webview` handle.
        unsafe { callback(&mut webview as *mut WebView, self.user_data) };
    }

    fn on_before_download(
        &self,
        mut webview: WebView,
        download: servo_api::DownloadItem,
        continue_callback: servo_api::DownloadContinueCallback,
    ) {
        let Some(callback) = self.on_before_download else {
            // Default: allow the download
            continue_callback(true);
            return;
        };

        // Create the C download item
        let c_download = Box::into_raw(ServoDownloadItem::from_download_item(download));

        // Create the trampoline callback
        let responder = move |continue_download: bool| {
            continue_callback(continue_download);
        };
        let responder: Box<dyn FnOnce(bool)> = Box::new(responder);
        let responder_ptr = Box::into_raw(Box::new(responder)) as *mut c_void;

        // Call the C callback
        unsafe {
            callback(
                &mut webview as *mut WebView,
                c_download,
                Some(download_continue_trampoline),
                responder_ptr as *mut c_void,
            )
        };
    }

    fn on_download_updated(&self, mut webview: WebView, download: servo_api::DownloadItem) {
        let Some(callback) = self.on_download_updated else {
            return;
        };

        // Create the C download item
        let c_download = Box::into_raw(ServoDownloadItem::from_download_item(download));

        // Call the C callback
        unsafe { callback(&mut webview as *mut WebView, c_download, self.user_data) };
    }

    fn on_find_result(
        &self,
        mut webview: WebView,
        identifier: i32,
        count: i32,
        result: Option<servo_constellation_traits::FindResult>,
    ) {
        let Some(callback) = self.on_find_result else {
            return;
        };

        let c_result = result.map(|r| {
            let find_result = crate::find::ServoFindResult::from(r);
            Box::into_raw(Box::new(find_result))
        });

        // Call the C callback
        unsafe {
            callback(
                &mut webview as *mut WebView,
                identifier,
                count,
                c_result.unwrap_or(std::ptr::null()),
                self.user_data,
            )
        };

        // Free the result if it was allocated
        if let Some(ptr) = c_result {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    fn notify_window_resized(&self, mut webview: WebView, width: u32, height: u32) {
        let Some(callback) = self.on_window_resized else {
            return;
        };

        unsafe { callback(&mut webview as *mut WebView, width, height, self.user_data) };
    }

    fn notify_window_moved(&self, mut webview: WebView, x: i32, y: i32) {
        let Some(callback) = self.on_window_moved else {
            return;
        };

        unsafe { callback(&mut webview as *mut WebView, x, y, self.user_data) };
    }

    fn notify_window_minimized(&self, mut webview: WebView) {
        let Some(callback) = self.on_window_minimized else {
            return;
        };

        unsafe { callback(&mut webview as *mut WebView, self.user_data) };
    }

    fn notify_window_maximized(&self, mut webview: WebView) {
        let Some(callback) = self.on_window_maximized else {
            return;
        };

        unsafe { callback(&mut webview as *mut WebView, self.user_data) };
    }

    fn notify_window_restored(&self, mut webview: WebView) {
        let Some(callback) = self.on_window_restored else {
            return;
        };

        unsafe { callback(&mut webview as *mut WebView, self.user_data) };
    }

    fn notify_window_fullscreen_changed(&self, mut webview: WebView, fullscreen: bool) {
        let Some(callback) = self.on_window_fullscreen_changed else {
            return;
        };

        unsafe { callback(&mut webview as *mut WebView, fullscreen, self.user_data) };
    }

    fn notify_window_close_requested(&self, mut webview: WebView) -> bool {
        let Some(callback) = self.on_window_close_requested else {
            return true; // Default: allow close
        };

        unsafe { callback(&mut webview as *mut WebView, self.user_data) }
    }

    fn notify_window_move_or_resize_started(&self, mut webview: WebView) {
        let Some(callback) = self.on_window_move_or_resize_started else {
            return;
        };

        unsafe { callback(&mut webview as *mut WebView, self.user_data) };
    }

    fn notify_animating_changed(&self, mut webview: WebView, animating: bool) {
        let Some(callback) = self.notify_animating_changed else {
            return;
        };
        unsafe {
            callback(
                &mut webview as *mut WebView,
                animating,
                self.user_data,
            )
        };
    }

    fn notify_history_changed(&self, mut webview: WebView, entries: Vec<Url>, current: usize) {
        let Some(callback) = self.notify_history_changed else {
            return;
        };
        let c_strings: Vec<CString> = entries
            .iter()
            .filter_map(|url| CString::new(url.as_str()).ok())
            .collect();
        let c_ptrs: Vec<*const c_char> = c_strings.iter().map(|s| s.as_ptr()).collect();
        unsafe {
            callback(
                &mut webview as *mut WebView,
                c_ptrs.as_ptr(),
                c_ptrs.len(),
                current,
                self.user_data,
            )
        };
    }

    fn notify_crashed(&self, mut webview: WebView, reason: String, backtrace: Option<String>) {
        let Some(callback) = self.notify_crashed else {
            return;
        };
        let reason_c = CString::new(reason).unwrap_or_default();
        let backtrace_c = backtrace.and_then(|b| CString::new(b).ok());
        let backtrace_ptr = backtrace_c.as_ref().map_or(std::ptr::null(), |b| b.as_ptr());
        unsafe {
            callback(
                &mut webview as *mut WebView,
                reason_c.as_ptr(),
                backtrace_ptr,
                self.user_data,
            )
        };
    }

    fn hide_embedder_control(
        &self,
        mut webview: WebView,
        _control_id: servo_api::EmbedderControlId,
    ) {
        if let Some(callback) = self.hide_ime {
            unsafe {
                callback(&mut webview as *mut WebView, self.user_data);
            }
        }
    }
}

// Trampoline functions for C callbacks to call back into Rust

unsafe extern "C" fn permission_trampoline(granted: bool, user_data: *mut c_void) {
    if user_data.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let responder = Box::from_raw(user_data as *mut Box<dyn FnOnce(bool)>);
        responder(granted);
    }));
}

unsafe extern "C" fn authentication_trampoline(
    username: *const c_char,
    password: *const c_char,
    user_data: *mut c_void,
) {
    if user_data.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let responder = Box::from_raw(user_data as *mut Box<dyn FnOnce(String, String)>);
        let username = if username.is_null() {
            String::new()
        } else {
            CStr::from_ptr(username).to_string_lossy().into_owned()
        };
        let password = if password.is_null() {
            String::new()
        } else {
            CStr::from_ptr(password).to_string_lossy().into_owned()
        };
        responder(username, password);
    }));
}

unsafe extern "C" fn unload_trampoline(proceed: bool, user_data: *mut c_void) {
    if user_data.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let responder = Box::from_raw(user_data as *mut Box<dyn FnOnce(bool)>);
        responder(proceed);
    }));
}

unsafe extern "C" fn certificate_error_trampoline(allow: bool, user_data: *mut c_void) {
    if user_data.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let responder = Box::from_raw(user_data as *mut Box<dyn FnOnce(bool)>);
        responder(allow);
    }));
}

unsafe extern "C" fn download_continue_trampoline(continue_download: bool, user_data: *mut c_void) {
    if user_data.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let responder = Box::from_raw(user_data as *mut Box<dyn FnOnce(bool)>);
        responder(continue_download);
    }));
}

unsafe extern "C" fn alert_trampoline(_result: bool, user_data: *mut c_void) {
    if user_data.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let responder = Box::from_raw(user_data as *mut Box<dyn FnOnce(bool)>);
        responder(true);
    }));
}

unsafe extern "C" fn confirm_trampoline(result: bool, user_data: *mut c_void) {
    if user_data.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let responder = Box::from_raw(user_data as *mut Box<dyn FnOnce(bool)>);
        responder(result);
    }));
}

unsafe extern "C" fn prompt_trampoline(result: *const c_char, user_data: *mut c_void) {
    if user_data.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let responder = Box::from_raw(user_data as *mut Box<dyn FnOnce(Option<String>)>);
        let text = if result.is_null() {
            None
        } else {
            CStr::from_ptr(result).to_str().ok().map(|s| s.to_owned())
        };
        responder(text);
    }));
}

unsafe extern "C" fn file_picker_trampoline(
    paths: *const *const c_char,
    path_count: usize,
    user_data: *mut c_void,
) {
    if user_data.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let responder = Box::from_raw(
            user_data as *mut Box<dyn FnOnce(Option<Vec<PathBuf>>)>,
        );
        if paths.is_null() || path_count == 0 {
            responder(None);
            return;
        }
        let mut result_paths = Vec::with_capacity(path_count);
        for i in 0..path_count {
            let p = *paths.add(i);
            if !p.is_null() {
                if let Ok(s) = CStr::from_ptr(p).to_str() {
                    result_paths.push(PathBuf::from(s));
                }
            }
        }
        responder(Some(result_paths));
    }));
}

/// Cursor type constants for `notify_cursor_changed` delegate callback.
/// These match `embedder_traits::Cursor` and `ServoCursorType`.
pub const SERVO_CURSOR_POINTER: i32 = 0;
pub const SERVO_CURSOR_CROSSHAIR: i32 = 1;
pub const SERVO_CURSOR_HAND: i32 = 2;
pub const SERVO_CURSOR_IBEAM: i32 = 3;
pub const SERVO_CURSOR_WAIT: i32 = 4;
pub const SERVO_CURSOR_HELP: i32 = 5;
pub const SERVO_CURSOR_EAST_RESIZE: i32 = 6;
pub const SERVO_CURSOR_NORTH_RESIZE: i32 = 7;
pub const SERVO_CURSOR_NORTH_EAST_RESIZE: i32 = 8;
pub const SERVO_CURSOR_NORTH_WEST_RESIZE: i32 = 9;
pub const SERVO_CURSOR_SOUTH_RESIZE: i32 = 10;
pub const SERVO_CURSOR_SOUTH_EAST_RESIZE: i32 = 11;
pub const SERVO_CURSOR_SOUTH_WEST_RESIZE: i32 = 12;
pub const SERVO_CURSOR_WEST_RESIZE: i32 = 13;
pub const SERVO_CURSOR_NORTH_SOUTH_RESIZE: i32 = 14;
pub const SERVO_CURSOR_EAST_WEST_RESIZE: i32 = 15;
pub const SERVO_CURSOR_NORTH_EAST_SOUTH_WEST_RESIZE: i32 = 16;
pub const SERVO_CURSOR_NORTH_WEST_SOUTH_EAST_RESIZE: i32 = 17;
pub const SERVO_CURSOR_COLUMN_RESIZE: i32 = 18;
pub const SERVO_CURSOR_ROW_RESIZE: i32 = 19;
pub const SERVO_CURSOR_MIDDLE_PANNING: i32 = 20;
pub const SERVO_CURSOR_EAST_PANNING: i32 = 21;
pub const SERVO_CURSOR_NORTH_PANNING: i32 = 22;
pub const SERVO_CURSOR_NORTH_EAST_PANNING: i32 = 23;
pub const SERVO_CURSOR_NORTH_WEST_PANNING: i32 = 24;
pub const SERVO_CURSOR_SOUTH_PANNING: i32 = 25;
pub const SERVO_CURSOR_SOUTH_EAST_PANNING: i32 = 26;
pub const SERVO_CURSOR_SOUTH_WEST_PANNING: i32 = 27;
pub const SERVO_CURSOR_WEST_PANNING: i32 = 28;
pub const SERVO_CURSOR_MOVE: i32 = 29;
pub const SERVO_CURSOR_VERTICAL_TEXT: i32 = 30;
pub const SERVO_CURSOR_CELL: i32 = 31;
pub const SERVO_CURSOR_CONTEXT_MENU: i32 = 32;
pub const SERVO_CURSOR_ALIAS: i32 = 33;
pub const SERVO_CURSOR_PROGRESS: i32 = 34;
pub const SERVO_CURSOR_NO_DROP: i32 = 35;
pub const SERVO_CURSOR_COPY: i32 = 36;
pub const SERVO_CURSOR_NOT_ALLOWED: i32 = 37;
pub const SERVO_CURSOR_GRAB: i32 = 38;
pub const SERVO_CURSOR_GRABBING: i32 = 39;
pub const SERVO_CURSOR_CUSTOM: i32 = 40;
pub const SERVO_CURSOR_NONE: i32 = 41;
pub const SERVO_CURSOR_AUTO: i32 = 42;

/// Console log level constants for `show_console_message` delegate callback.
/// These match `embedder_traits::ConsoleLogLevel` and `ServoConsoleLogLevel`.
pub const SERVO_CONSOLE_LOG_LEVEL_DEBUG: i32 = 0;
pub const SERVO_CONSOLE_LOG_LEVEL_LOG: i32 = 1;
pub const SERVO_CONSOLE_LOG_LEVEL_INFO: i32 = 2;
pub const SERVO_CONSOLE_LOG_LEVEL_WARN: i32 = 3;
pub const SERVO_CONSOLE_LOG_LEVEL_ERROR: i32 = 4;
