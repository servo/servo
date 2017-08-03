/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use libc::{c_uint, c_ushort, c_int, c_double, size_t, wchar_t};
#[cfg(target_os="linux")]
use libc::c_ulong;
#[cfg(target_os="macos")]
use libc::c_void;

use net_traits::net_error_list::NetError;

pub use self::cef_rect as cef_rect_t;

use std::collections::BTreeMap;

pub type cef_string_map_t = BTreeMap<String, *mut cef_string_t>;
pub type cef_string_multimap_t = BTreeMap<String, Vec<*mut cef_string_t>>;
pub type cef_string_list_t = Vec<String>;
pub enum cef_text_input_context_t {}
pub enum cef_event_handle_t {}

#[cfg(target_os="linux")]
pub type cef_window_handle_t = c_ulong;
#[cfg(target_os="macos")]
pub type cef_window_handle_t = *mut c_void; //NSView*
//#[cfg(target_os="win")]
//pub enum cef_window_handle_t {} //HWND
#[cfg(target_os="linux")]
pub type cef_cursor_handle_t = c_ulong;
#[cfg(target_os="macos")]
pub type cef_cursor_handle_t = *mut c_void; //NSCursor*
//#[cfg(target_os="win")]
//pub enum cef_cursor_handle_t {} //HCURSOR

pub type cef_string_t = cef_string_utf16; //FIXME: this is #defined...
pub type cef_string_userfree_t = *mut cef_string_t; //FIXME: this is #defined...

pub struct cef_string_utf8 {
    pub str: *mut u8,
    pub length: size_t,
    pub dtor: Option<extern "C" fn(str: *mut u8)>,
}
pub type cef_string_utf8_t = cef_string_utf8;
pub type cef_string_userfree_utf8_t = cef_string_utf8;

pub type cef_string_utf16_t = cef_string_utf16;
pub type cef_string_userfree_utf16_t = *mut cef_string_utf16;
pub struct cef_string_utf16 {
    pub str: *mut c_ushort,
    pub length: size_t,
    pub dtor: Option<extern "C" fn(str: *mut c_ushort)>,
}

pub type cef_string_wide_t = cef_string_wide;
pub type cef_string_userfree_wide_t = cef_string_wide;
pub struct cef_string_wide {
    pub str: *mut wchar_t,
    pub length: size_t,
    pub dtor: Option<extern "C" fn(str: *mut wchar_t)>,
}

pub type cef_main_args_t = cef_main_args;
pub struct cef_main_args {
  pub argc: c_int,
  pub argv: *const *const u8
}

pub type cef_color_t = c_uint;

pub enum cef_json_parser_error_t {
  JSON_NO_ERROR = 0,
  JSON_INVALID_ESCAPE,
  JSON_SYNTAX_ERROR,
  JSON_UNEXPECTED_TOKEN,
  JSON_TRAILING_COMMA,
  JSON_TOO_MUCH_NESTING,
  JSON_UNEXPECTED_DATA_AFTER_ROOT,
  JSON_UNSUPPORTED_ENCODING,
  JSON_UNQUOTED_DICTIONARY_KEY,
  JSON_PARSE_ERROR_COUNT
}

///
// Represents the state of a setting.
///
pub enum cef_state_t {
    ///
    // Use the default state for the setting.
    ///
    STATE_DEFAULT = 0,

    ///
    // Enable or allow the setting.
    ///
    STATE_ENABLED,

    ///
    // Disable or disallow the setting.
    ///
    STATE_DISABLED,
}
//
// Existing thread IDs.
//
pub enum cef_thread_id_t {
// BROWSER PROCESS THREADS -- Only available in the browser process.

  //
  // The main thread in the browser. This will be the same as the main
  // application thread if CefInitialize() is called with a
  // CefSettings.multi_threaded_message_loop value of false.
  //
  TID_UI,

  //
  // Used to interact with the database.
  //
  TID_DB,

  //
  // Used to interact with the file system.
  //
  TID_FILE,

  //
  // Used for file system operations that block user interactions.
  // Responsiveness of this thread affects users.
  //
  TID_FILE_USER_BLOCKING,

  //
  // Used to launch and terminate browser processes.
  //
  TID_PROCESS_LAUNCHER,

  //
  // Used to handle slow HTTP cache operations.
  //
  TID_CACHE,

  //
  // Used to process IPC and network messages.
  //
  TID_IO,

// RENDER PROCESS THREADS -- Only available in the render process.

  //
  // The main thread in the renderer. Used for all WebKit and V8 interaction.
  //
  TID_RENDERER,
}

//
// Navigation types.
//
pub enum cef_navigation_type_t {
  NAVIGATION_LINK_CLICKED = 0,
  NAVIGATION_FORM_SUBMITTED,
  NAVIGATION_BACK_FORWARD,
  NAVIGATION_RELOAD,
  NAVIGATION_FORM_RESUBMITTED,
  NAVIGATION_OTHER,
}

//
// Mouse button types.
//
pub enum cef_mouse_button_type_t {
  MBT_LEFT   = 0,
  MBT_MIDDLE,
  MBT_RIGHT,
}

//
// Structure representing mouse event information.
//
pub type cef_mouse_event_t = cef_mouse_event;
pub type CefMouseEvent = cef_mouse_event_t;
pub struct cef_mouse_event {
    //
    // X coordinate relative to the left side of the view.
    //
    pub x: c_int,

    //
    // Y coordinate relative to the top side of the view.
    //
    pub y: c_int,

    //
    // Bit flags describing any pressed modifier keys. See
    // cef_event_flags_t for values.
    //
    pub modifiers: c_uint,
}

//
// Post data elements may represent either bytes or files.
//
pub enum cef_postdataelement_type_t {
  PDE_TYPE_EMPTY  = 0,
  PDE_TYPE_BYTES,
  PDE_TYPE_FILE,
}

//
// Flags used to customize the behavior of CefURLRequest.
//
pub enum cef_urlrequest_flags_t {
  //
  // Default behavior.
  //
  UR_FLAG_NONE                      = 0,

  //
  // If set the cache will be skipped when handling the request.
  //
  UR_FLAG_SKIP_CACHE                = 1 << 0,

  //
  // If set user name, password, and cookies may be sent with the request.
  //
  UR_FLAG_ALLOW_CACHED_CREDENTIALS  = 1 << 1,

  //
  // If set cookies may be sent with the request and saved from the response.
  // UR_FLAG_ALLOW_CACHED_CREDENTIALS must also be set.
  //
  UR_FLAG_ALLOW_COOKIES             = 1 << 2,

  //
  // If set upload progress events will be generated when a request has a body.
  //
  UR_FLAG_REPORT_UPLOAD_PROGRESS    = 1 << 3,

  //
  // If set load timing info will be collected for the request.
  //
  UR_FLAG_REPORT_LOAD_TIMING        = 1 << 4,

  //
  // If set the headers sent and received for the request will be recorded.
  //
  UR_FLAG_REPORT_RAW_HEADERS        = 1 << 5,

  //
  // If set the CefURLRequestClient::OnDownloadData method will not be called.
  //
  UR_FLAG_NO_DOWNLOAD_DATA          = 1 << 6,

  //
  // If set 5XX redirect errors will be propagated to the observer instead of
  // automatically re-tried. This currently only applies for requests
  // originated in the browser process.
  //
  UR_FLAG_NO_RETRY_ON_5XX           = 1 << 7,
}


//
// Flags that represent CefURLRequest status.
//
pub enum cef_urlrequest_status_t {
  //
  // Unknown status.
  //
  UR_UNKNOWN = 0,

  //
  // Request succeeded.
  //
  UR_SUCCESS,

  //
  // An IO request is pending, and the caller will be informed when it is
  // completed.
  //
  UR_IO_PENDING,

  //
  // Request was canceled programatically.
  //
  UR_CANCELED,

  //
  // Request failed for some reason.
  //
  UR_FAILED,
}



pub type cef_errorcode_t = NetError;


//
// Key event types.
//
pub enum cef_key_event_type_t {
  KEYEVENT_RAWKEYDOWN = 0,
  KEYEVENT_KEYDOWN,
  KEYEVENT_KEYUP,
  KEYEVENT_CHAR
}

//
// Structure representing keyboard event information.
//
pub type cef_key_event_t = cef_key_event;
pub struct cef_key_event {
  //
  // The type of keyboard event.
  //
  pub t: cef_key_event_type_t,

  //
  // Bit flags describing any pressed modifier keys. See
  // cef_event_flags_t for values.
  //
  pub modifiers: c_uint,

  //
  // The Windows key code for the key event. This value is used by the DOM
  // specification. Sometimes it comes directly from the event (i.e. on
  // Windows) and sometimes it's determined using a mapping function. See
  // WebCore/platform/chromium/KeyboardCodes.h for the list of values.
  //
  pub windows_key_code: c_int,

  //
  // The actual key code generated by the platform.
  //
  pub native_key_code: c_int,

  //
  // Indicates whether the event is considered a "system key" event (see
  // http://msdn.microsoft.com/en-us/library/ms646286(VS.85).aspx for details).
  // This value will always be false on non-Windows platforms.
  //
  pub is_system_key: c_int,

  //
  // The character generated by the keystroke.
  //
  pub character: c_ushort, //FIXME: can be wchar_t also?

  //
  // Same as |character| but unmodified by any concurrently-held modifiers
  // (except shift). This is useful for working out shortcut keys.
  //
  pub unmodified_character: c_ushort, //FIXME: can be wchar_t also?

  //
  // True if the focus is currently on an editable field on the page. This is
  // useful for determining if standard key events should be intercepted.
  //
  pub focus_on_editable_field: c_int,
}

pub type CefKeyEvent = cef_key_event_t;

//
// Structure representing a point.
//
pub type cef_point_t = cef_point;
pub struct cef_point {
  pub x: c_int,
  pub y: c_int,
}

//
// Structure representing a rectangle.
//
pub struct cef_rect {
  pub x: c_int,
  pub y: c_int,
  pub width: c_int,
  pub height: c_int,
}

impl cef_rect {
    pub fn zero() -> cef_rect {
        cef_rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }
}

pub type cef_draggable_region_t = cef_draggable_region;
///
// Structure representing a draggable region.
///
pub struct cef_draggable_region {
  ///
  // Bounds of the region.
  ///
  pub bounds: cef_rect_t,

  ///
  // True (1) this this region is draggable and false (0) otherwise.
  ///
  pub draggable: i32
}

//
// Paint element types.
//
pub enum cef_paint_element_type_t {
  PET_VIEW  = 0,
  PET_POPUP,
}

//
// DOM document types.
//
pub enum cef_dom_document_type_t {
  DOM_DOCUMENT_TYPE_UNKNOWN = 0,
  DOM_DOCUMENT_TYPE_HTML,
  DOM_DOCUMENT_TYPE_XHTML,
  DOM_DOCUMENT_TYPE_PLUGIN,
}

//
// Supported file dialog modes.
//
pub enum cef_file_dialog_mode_t {
  //
  // Requires that the file exists before allowing the user to pick it.
  //
  FILE_DIALOG_OPEN = 0,

  //
  // Like Open, but allows picking multiple files to open.
  //
  FILE_DIALOG_OPEN_MULTIPLE,

  //
  // Allows picking a nonexistent file, and prompts to overwrite if the file
  // already exists.
  //
  FILE_DIALOG_SAVE,
}

//
// Supported value types.
//
pub enum cef_value_type_t {
  VTYPE_INVALID = 0,
  VTYPE_NULL,
  VTYPE_BOOL,
  VTYPE_INT,
  VTYPE_DOUBLE,
  VTYPE_STRING,
  VTYPE_BINARY,
  VTYPE_DICTIONARY,
  VTYPE_LIST,
}

pub type CefValueType = cef_value_type_t;

//
// Existing process IDs.
//
pub enum cef_process_id_t {
  //
  // Browser process.
  //
  PID_BROWSER,
  //
  // Renderer process.
  //
  PID_RENDERER,
}
pub type CefProcessId = cef_process_id_t;

//
// Log severity levels.
//
pub enum cef_log_severity_t {
  //
  // Default logging (currently INFO logging).
  //
  LOGSEVERITY_DEFAULT,

  //
  // Verbose logging.
  //
  LOGSEVERITY_VERBOSE,

  //
  // INFO logging.
  //
  LOGSEVERITY_INFO,

  //
  // WARNING logging.
  //
  LOGSEVERITY_WARNING,

  //
  // ERROR logging.
  //
  LOGSEVERITY_ERROR,

  //
  // ERROR_REPORT logging.
  //
  LOGSEVERITY_ERROR_REPORT,

  //
  // Completely disable logging.
  //
  LOGSEVERITY_DISABLE = 99
}


//
// Initialization settings. Specify NULL or 0 to get the recommended default
// values. Many of these and other settings can also configured using command-
// line switches.
//
pub type cef_settings_t = cef_settings;

#[repr(C)]
pub struct cef_settings {
  //
  // Size of this structure.
  //
  pub size: size_t,

  //
  // Set to true (1) to use a single process for the browser and renderer. This
  // run mode is not officially supported by Chromium and is less stable than
  // the multi-process default. Also configurable using the "single-process"
  // command-line switch.
  //
  pub single_process: c_int,

  //
  // Set to true (1) to disable the sandbox for sub-processes. See
  // cef_sandbox_win.h for requirements to enable the sandbox on Windows. Also
  // configurable using the "no-sandbox" command-line switch.
  //
  pub no_sandbox: c_int,

  //
  // The path to a separate executable that will be launched for sub-processes.
  // By default the browser process executable is used. See the comments on
  // CefExecuteProcess() for details. Also configurable using the
  // "browser-subprocess-path" command-line switch.
  //
  pub browser_subprocess_path: cef_string_t,

  //
  // Set to true (1) to have the browser process message loop run in a separate
  // thread. If false (0) than the CefDoMessageLoopWork() function must be
  // called from your application message loop.
  //
  pub multi_threaded_message_loop: c_int,


  //
  // Set to true (1) to enable windowless (off-screen) rendering support. Do not
  // enable this value if the application does not use windowless rendering as
  // it may reduce rendering performance on some systems.
  //
  pub windowless_rendering_enabled: c_int,

  //
  // Set to true (1) to disable configuration of browser process features using
  // standard CEF and Chromium command-line arguments. Configuration can still
  // be specified using CEF data structures or via the
  // CefApp::OnBeforeCommandLineProcessing() method.
  //
  pub command_line_args_disabled: c_int,

  //
  // The location where cache data will be stored on disk. If empty an in-memory
  // cache will be used for some features and a temporary disk cache for others.
  // HTML5 databases such as localStorage will only persist across sessions if a
  // cache path is specified.
  //
  pub cache_path: cef_string_t,

  //
  // To persist session cookies (cookies without an expiry date or validity
  // interval) by default when using the global cookie manager set this value to
  // true. Session cookies are generally intended to be transient and most Web
  // browsers do not persist them. A |cache_path| value must also be specified to
  // enable this feature. Also configurable using the "persist-session-cookies"
  // command-line switch.
  //
  pub persist_session_cookies: c_int,

  //
  // Value that will be returned as the User-Agent HTTP header. If empty the
  // default User-Agent string will be used. Also configurable using the
  // "user-agent" command-line switch.
  //
  pub user_agent: cef_string_t,

  //
  // Value that will be inserted as the product portion of the default
  // User-Agent string. If empty the Chromium product version will be used. If
  // |userAgent| is specified this value will be ignored. Also configurable
  // using the "product-version" command-line switch.
  //
  pub product_version: cef_string_t,

  //
  // The locale string that will be passed to WebKit. If empty the default
  // locale of "en-US" will be used. This value is ignored on Linux where locale
  // is determined using environment variable parsing with the precedence order:
  // LANGUAGE, LC_ALL, LC_MESSAGES and LANG. Also configurable using the "lang"
  // command-line switch.
  //
  pub locale: cef_string_t,

  //
  // The directory and file name to use for the debug log. If empty, the
  // default name of "debug.log" will be used and the file will be written
  // to the application directory. Also configurable using the "log-file"
  // command-line switch.
  //
  pub log_file: cef_string_t,

  //
  // The log severity. Only messages of this severity level or higher will be
  // logged. Also configurable using the "log-severity" command-line switch with
  // a value of "verbose", "info", "warning", "error", "error-report" or
  // "disable".
  //
  pub log_severity: cef_log_severity_t,

  //
  // Custom flags that will be used when initializing the V8 JavaScript engine.
  // The consequences of using custom flags may not be well tested. Also
  // configurable using the "js-flags" command-line switch.
  //
  pub javascript_flags: cef_string_t,

  //
  // The fully qualified path for the resources directory. If this value is
  // empty the cef.pak and/or devtools_resources.pak files must be located in
  // the module directory on Windows/Linux or the app bundle Resources directory
  // on Mac OS X. Also configurable using the "resources-dir-path" command-line
  // switch.
  //
  pub resources_dir_path: cef_string_t,

  //
  // The fully qualified path for the locales directory. If this value is empty
  // the locales directory must be located in the module directory. This value
  // is ignored on Mac OS X where pack files are always loaded from the app
  // bundle Resources directory. Also configurable using the "locales-dir-path"
  // command-line switch.
  //
  pub locales_dir_path: cef_string_t,

  //
  // Set to true (1) to disable loading of pack files for resources and locales.
  // A resource bundle handler must be provided for the browser and render
  // processes via CefApp::GetResourceBundleHandler() if loading of pack files
  // is disabled. Also configurable using the "disable-pack-loading" command-
  // line switch.
  //
  pub pack_loading_disabled: c_int,

  //
  // Set to a value between 1024 and 65535 to enable remote debugging on the
  // specified port. For example, if 8080 is specified the remote debugging URL
  // will be http://localhost:8080. CEF can be remotely debugged from any CEF or
  // Chrome browser window. Also configurable using the "remote-debugging-port"
  // command-line switch.
  //
  pub remote_debugging_port: c_int,

  //
  // The number of stack trace frames to capture for uncaught exceptions.
  // Specify a positive value to enable the CefV8ContextHandler::
  // OnUncaughtException() callback. Specify 0 (default value) and
  // OnUncaughtException() will not be called. Also configurable using the
  // "uncaught-exception-stack-size" command-line switch.
  //
  pub uncaught_exception_stack_size: c_int,

  //
  // By default CEF V8 references will be invalidated (the IsValid() method will
  // return false) after the owning context has been released. This reduces the
  // need for external record keeping and avoids crashes due to the use of V8
  // references after the associated context has been released.
  //
  // CEF currently offers two context safety implementations with different
  // performance characteristics. The default implementation (value of 0) uses a
  // map of hash values and should provide better performance in situations with
  // a small number contexts. The alternate implementation (value of 1) uses a
  // hidden value attached to each context and should provide better performance
  // in situations with a large number of contexts.
  //
  // If you need better performance in the creation of V8 references and you
  // plan to manually track context lifespan you can disable context safety by
  // specifying a value of -1.
  //
  // Also configurable using the "context-safety-implementation" command-line
  // switch.
  //
  pub context_safety_implementation: c_int,

  //
  // Set to true (1) to ignore errors related to invalid SSL certificates.
  // Enabling this setting can lead to potential security vulnerabilities like
  // "man in the middle" attacks. Applications that load content from the
  // internet should not enable this setting. Also configurable using the
  // "ignore-certificate-errors" command-line switch.
  //
  pub ignore_certificate_errors: c_int,

  //
  // Opaque background color used for accelerated content. By default the
  // background color will be white. Only the RGB components of the specified
  // value will be used. The alpha component must greater than 0 to enable use
  // of the background color but will be otherwise ignored.
  //
  pub background_color: cef_color_t,
}

//
// Structure defining the reference count implementation functions. All
// framework structures must include the cef_base_t structure first.
//
pub type cef_base_t = cef_base;
pub struct cef_base {
  //
  // Size of the data structure.
  //
  pub size: size_t,

  //
  // Increment the reference count.
  //
  pub add_ref: Option<extern "C" fn(base: *mut cef_base) -> c_int>,

  //
  // Decrement the reference count.  Delete this object when no references
  // remain.
  //
  pub release: Option<extern "C" fn(base: *mut cef_base) -> c_int>,

  //
  // Returns the current number of references.
  //
  pub get_refct: Option<extern "C" fn(base: *mut cef_base) -> c_int>,
}

pub type CefBase = *mut cef_base_t;

//
// Class representing window information.
//
pub type cef_window_info_t = cef_window_info;

#[cfg(target_os="linux")]
pub struct cef_window_info {
  pub x: c_uint,
  pub y: c_uint,
  pub width: c_uint,
  pub height: c_uint,

  //
  // Pointer for the parent window.
  //
  pub parent_window: cef_window_handle_t,

  //
  // Set to true (1) to create the browser using windowless (off-screen)
  // rendering. No window will be created for the browser and all rendering will
  // occur via the CefRenderHandler interface. The |parent_window| value will be
  // used to identify monitor info and to act as the parent window for dialogs,
  // context menus, etc. If |parent_window| is not provided then the main screen
  // monitor will be used and some functionality that requires a parent window
  // may not function correctly. In order to create windowless browsers the
  // CefSettings.windowless_rendering_enabled value must be set to true.
  //
  pub windowless_rendering_enabled: c_int,

  //
  // Set to true (1) to enable transparent painting in combination with
  // windowless rendering. When this value is true a transparent background
  // color will be used (RGBA=0x00000000). When this value is false the
  // background will be white and opaque.
  //
  pub transparent_painting_enabled: c_int,

  //
  // Pointer for the new browser window. Only used with windowed rendering.
  //
  pub window: cef_window_handle_t
}

#[cfg(target_os="macos")]
pub struct cef_window_info {
  pub window_name: cef_string_t,
  pub x: c_uint,
  pub y: c_uint,
  pub width: c_uint,
  pub height: c_uint,

  //
  // Set to true (1) to create the view initially hidden.
  //
  pub hidden: c_int,

  //
  // Pointer for the parent window.
  //
  pub parent_window: cef_window_handle_t,

  //
  // Set to true (1) to create the browser using windowless (off-screen)
  // rendering. No window will be created for the browser and all rendering will
  // occur via the CefRenderHandler interface. The |parent_window| value will be
  // used to identify monitor info and to act as the parent window for dialogs,
  // context menus, etc. If |parent_window| is not provided then the main screen
  // monitor will be used and some functionality that requires a parent window
  // may not function correctly. In order to create windowless browsers the
  // CefSettings.windowless_rendering_enabled value must be set to true.
  //
  pub windowless_rendering_enabled: c_int,

  //
  // Set to true (1) to enable transparent painting in combination with
  // windowless rendering. When this value is true a transparent background
  // color will be used (RGBA=0x00000000). When this value is false the
  // background will be white and opaque.
  //
  pub transparent_painting_enabled: c_int,

  //
  // Pointer for the new browser window. Only used with windowed rendering.
  //
  pub window: cef_window_handle_t
}

pub type CefWindowInfo = cef_window_info_t;

//
// Supported menu item types.
//
pub enum cef_menu_item_type_t {
  MENUITEMTYPE_NONE,
  MENUITEMTYPE_COMMAND,
  MENUITEMTYPE_CHECK,
  MENUITEMTYPE_RADIO,
  MENUITEMTYPE_SEPARATOR,
  MENUITEMTYPE_SUBMENU,
}

//
// Supported context menu type flags.
//
pub enum cef_context_menu_type_flags_t {
  //
  // No node is selected.
  //
  CM_TYPEFLAG_NONE        = 0,
  //
  // The top page is selected.
  //
  CM_TYPEFLAG_PAGE        = 1 << 0,
  //
  // A subframe page is selected.
  //
  CM_TYPEFLAG_FRAME       = 1 << 1,
  //
  // A link is selected.
  //
  CM_TYPEFLAG_LINK        = 1 << 2,
  //
  // A media node is selected.
  //
  CM_TYPEFLAG_MEDIA       = 1 << 3,
  //
  // There is a textual or mixed selection that is selected.
  //
  CM_TYPEFLAG_SELECTION   = 1 << 4,
  //
  // An editable element is selected.
  //
  CM_TYPEFLAG_EDITABLE    = 1 << 5,
}

//
// Supported context menu media types.
//
pub enum cef_context_menu_media_type_t {
  //
  // No special node is in context.
  //
  CM_MEDIATYPE_NONE,
  //
  // An image node is selected.
  //
  CM_MEDIATYPE_IMAGE,
  //
  // A video node is selected.
  //
  CM_MEDIATYPE_VIDEO,
  //
  // An audio node is selected.
  //
  CM_MEDIATYPE_AUDIO,
  //
  // A file node is selected.
  //
  CM_MEDIATYPE_FILE,
  //
  // A plugin node is selected.
  //
  CM_MEDIATYPE_PLUGIN,
}

//
// Supported context menu media state bit flags.
//
pub enum cef_context_menu_media_state_flags_t {
  CM_MEDIAFLAG_NONE                  = 0,
  CM_MEDIAFLAG_ERROR                 = 1 << 0,
  CM_MEDIAFLAG_PAUSED                = 1 << 1,
  CM_MEDIAFLAG_MUTED                 = 1 << 2,
  CM_MEDIAFLAG_LOOP                  = 1 << 3,
  CM_MEDIAFLAG_CAN_SAVE              = 1 << 4,
  CM_MEDIAFLAG_HAS_AUDIO             = 1 << 5,
  CM_MEDIAFLAG_HAS_VIDEO             = 1 << 6,
  CM_MEDIAFLAG_CONTROL_ROOT_ELEMENT  = 1 << 7,
  CM_MEDIAFLAG_CAN_PRINT             = 1 << 8,
  CM_MEDIAFLAG_CAN_ROTATE            = 1 << 9,
}

//
// Supported context menu edit state bit flags.
//
pub enum cef_context_menu_edit_state_flags_t {
  CM_EDITFLAG_NONE            = 0,
  CM_EDITFLAG_CAN_UNDO        = 1 << 0,
  CM_EDITFLAG_CAN_REDO        = 1 << 1,
  CM_EDITFLAG_CAN_CUT         = 1 << 2,
  CM_EDITFLAG_CAN_COPY        = 1 << 3,
  CM_EDITFLAG_CAN_PASTE       = 1 << 4,
  CM_EDITFLAG_CAN_DELETE      = 1 << 5,
  CM_EDITFLAG_CAN_SELECT_ALL  = 1 << 6,
  CM_EDITFLAG_CAN_TRANSLATE   = 1 << 7,
}

//
// Supported event bit flags.
//
pub enum cef_event_flags_t {
  EVENTFLAG_NONE                = 0,
  EVENTFLAG_CAPS_LOCK_ON        = 1 << 0,
  EVENTFLAG_SHIFT_DOWN          = 1 << 1,
  EVENTFLAG_CONTROL_DOWN        = 1 << 2,
  EVENTFLAG_ALT_DOWN            = 1 << 3,
  EVENTFLAG_LEFT_MOUSE_BUTTON   = 1 << 4,
  EVENTFLAG_MIDDLE_MOUSE_BUTTON = 1 << 5,
  EVENTFLAG_RIGHT_MOUSE_BUTTON  = 1 << 6,
  // Mac OS-X command key.
  EVENTFLAG_COMMAND_DOWN        = 1 << 7,
  EVENTFLAG_NUM_LOCK_ON         = 1 << 8,
  EVENTFLAG_IS_KEY_PAD          = 1 << 9,
  EVENTFLAG_IS_LEFT             = 1 << 10,
  EVENTFLAG_IS_RIGHT            = 1 << 11,
}

//
// Time information. Values should always be in UTC.
//
#[repr(C)]
pub struct _cef_time_t {
  year: c_int,          // Four digit year "2007"
  month: c_int,         // 1-based month (values 1 = January, etc.)
  day_of_week: c_int,   // 0-based day of week (0 = Sunday, etc.)
  day_of_month: c_int,  // 1-based day of month (1-31)
  hour: c_int,          // Hour within the current day (0-23)
  minute: c_int,        // Minute within the current hour (0-59)
  second: c_int,        // Second within the current minute (0-59 plus leap
                        //   seconds which may take it up to 60).
  millisecond: c_int,   // Milliseconds within the current second (0-999)
}

pub type cef_time_t = _cef_time_t;

//
// DOM event processing phases.
//
pub enum cef_dom_event_phase_t {
  DOM_EVENT_PHASE_UNKNOWN = 0,
  DOM_EVENT_PHASE_CAPTURING,
  DOM_EVENT_PHASE_AT_TARGET,
  DOM_EVENT_PHASE_BUBBLING,
}

//
// DOM node types.
//
pub enum cef_dom_node_type_t {
  DOM_NODE_TYPE_UNSUPPORTED = 0,
  DOM_NODE_TYPE_ELEMENT,
  DOM_NODE_TYPE_ATTRIBUTE,
  DOM_NODE_TYPE_TEXT,
  DOM_NODE_TYPE_CDATA_SECTION,
  DOM_NODE_TYPE_PROCESSING_INSTRUCTIONS,
  DOM_NODE_TYPE_COMMENT,
  DOM_NODE_TYPE_DOCUMENT,
  DOM_NODE_TYPE_DOCUMENT_TYPE,
  DOM_NODE_TYPE_DOCUMENT_FRAGMENT,
}

//
// Focus sources.
//
pub enum cef_focus_source_t {
  //
  // The source is explicit navigation via the API (LoadURL(), etc).
  //
  FOCUS_SOURCE_NAVIGATION = 0,
  //
  // The source is a system-generated focus event.
  //
  FOCUS_SOURCE_SYSTEM,
}

//
// Supported JavaScript dialog types.
//
pub enum cef_jsdialog_type_t {
  JSDIALOGTYPE_ALERT = 0,
  JSDIALOGTYPE_CONFIRM,
  JSDIALOGTYPE_PROMPT,
}

//
// Structure representing a size.
//
pub struct _cef_size_t {
  pub width: c_int,
  pub height: c_int,
}

pub type cef_size_t = _cef_size_t;

//
// Structure representing a print job page range.
//
pub struct _cef_page_range_t {
  pub from: c_int,
  pub to: c_int,
}

pub type cef_page_range_t = _cef_page_range_t;

//
// Print job duplex mode values.
//
pub enum cef_duplex_mode_t {
  DUPLEX_MODE_UNKNOWN = -1,
  DUPLEX_MODE_SIMPLEX = 0,
  DUPLEX_MODE_LONG_EDGE = 1,
  DUPLEX_MODE_SHORT_EDGE = 2,
}

//
// Print job color mode values.
//
pub enum cef_color_model_t {
  COLOR_MODEL_UNKNOWN,
  COLOR_MODEL_GRAY,
  COLOR_MODEL_COLOR,
  COLOR_MODEL_CMYK,
  COLOR_MODEL_CMY,
  COLOR_MODEL_KCMY,
  COLOR_MODEL_CMY_K,  // CMY_K represents CMY+K.
  COLOR_MODEL_BLACK,
  COLOR_MODEL_GRAYSCALE,
  COLOR_MODEL_RGB,
  COLOR_MODEL_RGB16,
  COLOR_MODEL_RGBA,
  COLOR_MODEL_COLORMODE_COLOR,  // Used in samsung printer ppds.
  COLOR_MODEL_COLORMODE_MONOCHROME,  // Used in samsung printer ppds.
  COLOR_MODEL_HP_COLOR_COLOR,  // Used in HP color printer ppds.
  COLOR_MODEL_HP_COLOR_BLACK,  // Used in HP color printer ppds.
  COLOR_MODEL_PRINTOUTMODE_NORMAL,  // Used in foomatic ppds.
  COLOR_MODEL_PRINTOUTMODE_NORMAL_GRAY,  // Used in foomatic ppds.
  COLOR_MODEL_PROCESSCOLORMODEL_CMYK,  // Used in canon printer ppds.
  COLOR_MODEL_PROCESSCOLORMODEL_GREYSCALE,  // Used in canon printer ppds.
  COLOR_MODEL_PROCESSCOLORMODEL_RGB,  // Used in canon printer ppds
}

//
// Resource type for a request.
//
pub enum cef_resource_type_t {
  //
  // Top level page.
  //
  RT_MAIN_FRAME = 0,

  //
  // Frame or iframe.
  //
  RT_SUB_FRAME,

  //
  // CSS stylesheet.
  //
  RT_STYLESHEET,

  //
  // External script.
  //
  RT_SCRIPT,

  //
  // Image (jpg/gif/png/etc).
  //
  RT_IMAGE,

  //
  // Font.
  //
  RT_FONT_RESOURCE,

  //
  // Some other subresource. This is the default type if the actual type is
  // unknown.
  //
  RT_SUB_RESOURCE,

  //
  // Object (or embed) tag for a plugin, or a resource that a plugin requested.
  //
  RT_OBJECT,

  //
  // Media resource.
  //
  RT_MEDIA,

  //
  // Main resource of a dedicated worker.
  //
  RT_WORKER,

  //
  // Main resource of a shared worker.
  //
  RT_SHARED_WORKER,

  //
  // Explicitly requested prefetch.
  //
  RT_PREFETCH,

  //
  // Favicon.
  //
  RT_FAVICON,

  //
  // XMLHttpRequest.
  //
  RT_XHR,

  //
  // A request for a <ping>
  //
  RT_PING,

  //
  // Main resource of a service worker.
  //
  RT_SERVICE_WORKER,
}

//
// Transition type for a request. Made up of one source value and 0 or more
// qualifiers.
//
pub enum cef_transition_type_t {
  //
  // Source is a link click or the JavaScript window.open function. This is
  // also the default value for requests like sub-resource loads that are not
  // navigations.
  //
  TT_LINK = 0,

  //
  // Source is some other "explicit" navigation action such as creating a new
  // browser or using the LoadURL function. This is also the default value
  // for navigations where the actual type is unknown.
  //
  TT_EXPLICIT = 1,

  //
  // Source is a subframe navigation. This is any content that is automatically
  // loaded in a non-toplevel frame. For example, if a page consists of several
  // frames containing ads, those ad URLs will have this transition type.
  // The user may not even realize the content in these pages is a separate
  // frame, so may not care about the URL.
  //
  TT_AUTO_SUBFRAME = 3,

  //
  // Source is a subframe navigation explicitly requested by the user that will
  // generate new navigation entries in the back/forward list. These are
  // probably more important than frames that were automatically loaded in
  // the background because the user probably cares about the fact that this
  // link was loaded.
  //
  TT_MANUAL_SUBFRAME = 4,

  //
  // Source is a form submission by the user. NOTE: In some situations
  // submitting a form does not result in this transition type. This can happen
  // if the form uses a script to submit the contents.
  //
  TT_FORM_SUBMIT = 7,

  //
  // Source is a "reload" of the page via the Reload function or by re-visiting
  // the same URL. NOTE: This is distinct from the concept of whether a
  // particular load uses "reload semantics" (i.e. bypasses cached data).
  //
  TT_RELOAD = 8,

  //
  // General mask defining the bits used for the source values.
  //
  TT_SOURCE_MASK = 0xFF,

  // Qualifiers.
  // Any of the core values above can be augmented by one or more qualifiers.
  // These qualifiers further define the transition.

  //
  // Attempted to visit a URL but was blocked.
  //
  TT_BLOCKED_FLAG = 0x00800000,

  //
  // Used the Forward or Back function to navigate among browsing history.
  //
  TT_FORWARD_BACK_FLAG = 0x01000000,

  //
  // The beginning of a navigation chain.
  //
  TT_CHAIN_START_FLAG = 0x10000000,

  //
  // The last transition in a redirect chain.
  //
  TT_CHAIN_END_FLAG = 0x20000000,

  //
  // Redirects caused by JavaScript or a meta refresh tag on the page.
  //
  TT_CLIENT_REDIRECT_FLAG = 0x40000000,

  //
  // Redirects sent from the server by HTTP headers.
  //
  TT_SERVER_REDIRECT_FLAG = 0x80000000,

  //
  // Used to test whether a transition involves a redirect.
  //
  TT_IS_REDIRECT_MASK = 0xC0000000,

  //
  // General mask defining the bits used for the qualifiers.
  //
  TT_QUALIFIER_MASK = 0xFFFFFF00,
}

//
// Process termination status values.
//
pub enum cef_termination_status_t {
  //
  // Non-zero exit status.
  //
  TS_ABNORMAL_TERMINATION,

  //
  // SIGKILL or task manager kill.
  //
  TS_PROCESS_WAS_KILLED,

  //
  // Segmentation fault.
  //
  TS_PROCESS_CRASHED,
}

//
// V8 access control values.
//
pub enum cef_v8_accesscontrol_t {
  V8_ACCESS_CONTROL_DEFAULT               = 0,
  V8_ACCESS_CONTROL_ALL_CAN_READ          = 1,
  V8_ACCESS_CONTROL_ALL_CAN_WRITE         = 1 << 1,
  V8_ACCESS_CONTROL_PROHIBITS_OVERWRITING = 1 << 2
}

//
// V8 property attribute values.
//
pub enum cef_v8_propertyattribute_t {
  V8_PROPERTY_ATTRIBUTE_NONE       = 0,       // Writeable, Enumerable,
                                              //   Configurable
  V8_PROPERTY_ATTRIBUTE_READONLY   = 1 << 0,  // Not writeable
  V8_PROPERTY_ATTRIBUTE_DONTENUM   = 1 << 1,  // Not enumerable
  V8_PROPERTY_ATTRIBUTE_DONTDELETE = 1 << 2   // Not configurable
}

//
// XML node types.
//
pub enum cef_xml_node_type_t {
  XML_NODE_UNSUPPORTED = 0,
  XML_NODE_PROCESSING_INSTRUCTION,
  XML_NODE_DOCUMENT_TYPE,
  XML_NODE_ELEMENT_START,
  XML_NODE_ELEMENT_END,
  XML_NODE_ATTRIBUTE,
  XML_NODE_TEXT,
  XML_NODE_CDATA,
  XML_NODE_ENTITY_REFERENCE,
  XML_NODE_WHITESPACE,
  XML_NODE_COMMENT,
}

//
// Geoposition error codes.
//
pub enum cef_geoposition_error_code_t {
  GEOPOSITON_ERROR_NONE = 0,
  GEOPOSITON_ERROR_PERMISSION_DENIED,
  GEOPOSITON_ERROR_POSITION_UNAVAILABLE,
  GEOPOSITON_ERROR_TIMEOUT,
}

//
// Structure representing geoposition information. The properties of this
// structure correspond to those of the JavaScript Position object although
// their types may differ.
//
pub struct _cef_geoposition_t {
  //
  // Latitude in decimal degrees north (WGS84 coordinate frame).
  //
  pub latitude: c_double,

  //
  // Longitude in decimal degrees west (WGS84 coordinate frame).
  //
  pub longitude: c_double,

  //
  // Altitude in meters (above WGS84 datum).
  //
  pub altitude: c_double,

  //
  // Accuracy of horizontal position in meters.
  //
  pub accuracy: c_double,

  //
  // Accuracy of altitude in meters.
  //
  pub altitude_accuracy: c_double,

  //
  // Heading in decimal degrees clockwise from true north.
  //
  pub heading: c_double,

  //
  // Horizontal component of device velocity in meters per second.
  //
  pub speed: c_double,

  //
  // Time of position measurement in milliseconds since Epoch in UTC time. This
  // is taken from the host computer's system clock.
  //
  pub timestamp: cef_time_t,

  //
  // Error code, see enum above.
  //
  pub error_code: cef_geoposition_error_code_t,

  //
  // Human-readable error message.
  //
  pub error_message: cef_string_t,
}

pub type cef_geoposition_t = _cef_geoposition_t;

pub type CefGeoposition = cef_geoposition_t;

//
// Cookie information.
//
pub struct _cef_cookie_t {
  //
  // The cookie name.
  //
  pub name: cef_string_t,

  //
  // The cookie value.
  //
  pub value: cef_string_t,

  //
  // If |domain| is empty a host cookie will be created instead of a domain
  // cookie. Domain cookies are stored with a leading "." and are visible to
  // sub-domains whereas host cookies are not.
  //
  pub domain: cef_string_t,

  //
  // If |path| is non-empty only URLs at or below the path will get the cookie
  // value.
  //
  pub path: cef_string_t,

  //
  // If |secure| is true the cookie will only be sent for HTTPS requests.
  //
  pub secure: c_int,

  //
  // If |httponly| is true the cookie will only be sent for HTTP requests.
  //
  pub httponly: c_int,

  //
  // The cookie creation date. This is automatically populated by the system on
  // cookie creation.
  //
  pub creation: cef_time_t,

  //
  // The cookie last access date. This is automatically populated by the system
  // on access.
  //
  pub last_access: cef_time_t,

  //
  // The cookie expiration date is only valid if |has_expires| is true.
  //
  pub has_expires: c_int,
  pub expires: cef_time_t,
}

pub type cef_cookie_t = _cef_cookie_t;

pub type CefCookie = cef_cookie_t;

//
// Popup window features.
//
pub struct _cef_popup_features_t {
  pub x: c_int,
  pub x_set: c_int,
  pub y: c_int,
  pub y_set: c_int,
  pub width: c_int,
  pub width_set: c_int,
  pub height: c_int,
  pub height_set: c_int,

  pub menu_bar_visible: c_int,
  pub status_bar_visible: c_int,
  pub tool_bar_visible: c_int,
  pub location_bar_visible: c_int,
  pub scrollbars_visible: c_int,
  pub resizable: c_int,

  pub fullscreen: c_int,
  pub dialog: c_int,
  pub additional_features: cef_string_list_t,
}

pub type cef_popup_features_t = _cef_popup_features_t;

pub type CefPopupFeatures = cef_popup_features_t;

// FIXME(pcwalton): Really should be a set of bitflags.
pub enum cef_drag_operations_mask_t {
    DRAG_OPERATION_NONE    = 0,
    DRAG_OPERATION_COPY    = 1,
    DRAG_OPERATION_LINK    = 2,
    DRAG_OPERATION_GENERIC = 4,
    DRAG_OPERATION_PRIVATE = 8,
    DRAG_OPERATION_MOVE    = 16,
    DRAG_OPERATION_DELETE  = 32,
    DRAG_OPERATION_EVERY   = 0xffffffff,
}

pub enum cef_xml_encoding_type_t {
    XML_ENCODING_NONE = 0,
    XML_ENCODING_UTF8,
    XML_ENCODING_UTF16LE,
    XML_ENCODING_UTF16BE,
    XML_ENCODING_ASCII,
}

///
// The manner in which a link click should be opened.
///
pub enum cef_window_open_disposition_t {
    WOD_UNKNOWN = 0,
    WOD_SUPPRESS_OPEN,
    WOD_CURRENT_TAB,
    WOD_SINGLETON_TAB,
    WOD_NEW_FOREGROUND_TAB,
    WOD_NEW_BACKGROUND_TAB,
    WOD_NEW_POPUP,
    WOD_NEW_WINDOW,
    WOD_SAVE_TO_DISK,
    WOD_OFF_THE_RECORD,
    WOD_IGNORE_ACTION
}


///
// Cursor type values.
///
pub enum cef_cursor_type_t {
    CT_POINTER = 0,
    CT_CROSS,
    CT_HAND,
    CT_IBEAM,
    CT_WAIT,
    CT_HELP,
    CT_EASTRESIZE,
    CT_NORTHRESIZE,
    CT_NORTHEASTRESIZE,
    CT_NORTHWESTRESIZE,
    CT_SOUTHRESIZE,
    CT_SOUTHEASTRESIZE,
    CT_SOUTHWESTRESIZE,
    CT_WESTRESIZE,
    CT_NORTHSOUTHRESIZE,
    CT_EASTWESTRESIZE,
    CT_NORTHEASTSOUTHWESTRESIZE,
    CT_NORTHWESTSOUTHEASTRESIZE,
    CT_COLUMNRESIZE,
    CT_ROWRESIZE,
    CT_MIDDLEPANNING,
    CT_EASTPANNING,
    CT_NORTHPANNING,
    CT_NORTHEASTPANNING,
    CT_NORTHWESTPANNING,
    CT_SOUTHPANNING,
    CT_SOUTHEASTPANNING,
    CT_SOUTHWESTPANNING,
    CT_WESTPANNING,
    CT_MOVE,
    CT_VERTICALTEXT,
    CT_CELL,
    CT_CONTEXTMENU,
    CT_ALIAS,
    CT_PROGRESS,
    CT_NODROP,
    CT_COPY,
    CT_NONE,
    CT_NOTALLOWED,
    CT_ZOOMIN,
    CT_ZOOMOUT,
    CT_GRAB,
    CT_GRABBING,
    CT_CUSTOM,
}

///
// Screen information used when window rendering is disabled. This structure is
// passed as a parameter to CefRenderHandler::GetScreenInfo and should be filled
// in by the client.
///
pub struct _cef_screen_info {
  ///
  // Device scale factor. Specifies the ratio between physical and logical
  // pixels.
  ///
  pub device_scale_factor: f32,

  ///
  // The screen depth in bits per pixel.
  ///
  pub depth: i32,

  ///
  // The bits per color component. This assumes that the colors are balanced
  // equally.
  ///
  pub depth_per_component: i32,

  ///
  // This can be true for black and white printers.
  ///
  pub is_monochrome: i32,

  ///
  // This is set from the rcMonitor member of MONITORINFOEX, to whit:
  //   "A RECT structure that specifies the display monitor rectangle,
  //   expressed in virtual-screen coordinates. Note that if the monitor
  //   is not the primary display monitor, some of the rectangle's
  //   coordinates may be negative values."
  //
  // The |rect| and |available_rect| properties are used to determine the
  // available surface for rendering popup views.
  ///
  pub rect: cef_rect_t,

  ///
  // This is set from the rcWork member of MONITORINFOEX, to whit:
  //   "A RECT structure that specifies the work area rectangle of the
  //   display monitor that can be used by applications, expressed in
  //   virtual-screen coordinates. Windows uses this rectangle to
  //   maximize an application on the monitor. The rest of the area in
  //   rcMonitor contains system windows such as the task bar and side
  //   bars. Note that if the monitor is not the primary display monitor,
  //   some of the rectangle's coordinates may be negative values".
  //
  // The |rect| and |available_rect| properties are used to determine the
  // available surface for rendering popup views.
  ///
  pub available_rect: cef_rect_t,
}

pub type cef_screen_info_t = _cef_screen_info;
pub type CefScreenInfo = cef_screen_info_t;

///
// Browser initialization settings. Specify NULL or 0 to get the recommended
// default values. The consequences of using custom values may not be well
// tested. Many of these and other settings can also configured using command-
// line switches.
///
pub struct _cef_browser_settings {
    ///
    // Size of this structure.
    ///
    pub size: u64,

    ///
    // The maximum rate in frames per second (fps) that CefRenderHandler::OnPaint
    // will be called for a windowless browser. The actual fps may be lower if
    // the browser cannot generate frames at the requested rate. The minimum
    // value is 1 and the maximum value is 60 (default 30).
    ///
    pub windowless_frame_rate: i32,

    // The below values map to WebPreferences settings.

    ///
    // Font settings.
    ///
    pub standard_font_family: cef_string_t,
    pub fixed_font_family: cef_string_t,
    pub serif_font_family: cef_string_t,
    pub sans_serif_font_family: cef_string_t,
    pub cursive_font_family: cef_string_t,
    pub fantasy_font_family: cef_string_t,
    pub default_font_size: i32,
    pub default_fixed_font_size: i32,
    pub minimum_font_size: i32,
    pub minimum_logical_font_size: i32,

    ///
    // Default encoding for Web content. If empty "ISO-8859-1" will be used. Also
    // configurable using the "default-encoding" command-line switch.
    ///
    pub default_encoding: cef_string_t,

    ///
    // Controls the loading of fonts from remote sources. Also configurable using
    // the "disable-remote-fonts" command-line switch.
    ///
    pub remote_fonts: cef_state_t,

    ///
    // Controls whether JavaScript can be executed. Also configurable using the
    // "disable-javascript" command-line switch.
    ///
    pub javascript: cef_state_t,

    ///
    // Controls whether JavaScript can be used for opening windows. Also
    // configurable using the "disable-javascript-open-windows" command-line
    // switch.
    ///
    pub javascript_open_windows: cef_state_t,

    ///
    // Controls whether JavaScript can be used to close windows that were not
    // opened via JavaScript. JavaScript can still be used to close windows that
    // were opened via JavaScript or that have no back/forward history. Also
    // configurable using the "disable-javascript-close-windows" command-line
    // switch.
    ///
    pub javascript_close_windows: cef_state_t,

    ///
    // Controls whether JavaScript can access the clipboard. Also configurable
    // using the "disable-javascript-access-clipboard" command-line switch.
    ///
    pub javascript_access_clipboard: cef_state_t,

    ///
    // Controls whether DOM pasting is supported in the editor via
    // execCommand("paste"). The |javascript_access_clipboard| setting must also
    // be enabled. Also configurable using the "disable-javascript-dom-paste"
    // command-line switch.
    ///
    pub javascript_dom_paste: cef_state_t,

    ///
    // Controls whether the caret position will be drawn. Also configurable using
    // the "enable-caret-browsing" command-line switch.
    ///
    pub caret_browsing: cef_state_t,

    ///
    // Controls whether the Java plugin will be loaded. Also configurable using
    // the "disable-java" command-line switch.
    ///
    pub java: cef_state_t,

    ///
    // Controls whether any plugins will be loaded. Also configurable using the
    // "disable-plugins" command-line switch.
    ///
    pub plugins: cef_state_t,

    ///
    // Controls whether file URLs will have access to all URLs. Also configurable
    // using the "allow-universal-access-from-files" command-line switch.
    ///
    pub universal_access_from_file_urls: cef_state_t,

    ///
    // Controls whether file URLs will have access to other file URLs. Also
    // configurable using the "allow-access-from-files" command-line switch.
    ///
    pub file_access_from_file_urls: cef_state_t,

    ///
    // Controls whether web security restrictions (same-origin policy) will be
    // enforced. Disabling this setting is not recommend as it will allow risky
    // security behavior such as cross-site scripting (XSS). Also configurable
    // using the "disable-web-security" command-line switch.
    ///
    pub web_security: cef_state_t,

    ///
    // Controls whether image URLs will be loaded from the network. A cached image
    // will still be rendered if requested. Also configurable using the
    // "disable-image-loading" command-line switch.
    ///
    pub image_loading: cef_state_t,

    ///
    // Controls whether standalone images will be shrunk to fit the page. Also
    // configurable using the "image-shrink-standalone-to-fit" command-line
    // switch.
    ///
    pub image_shrink_standalone_to_fit: cef_state_t,

    ///
    // Controls whether text areas can be resized. Also configurable using the
    // "disable-text-area-resize" command-line switch.
    ///
    pub text_area_resize: cef_state_t,

    ///
    // Controls whether the tab key can advance focus to links. Also configurable
    // using the "disable-tab-to-links" command-line switch.
    ///
    pub tab_to_links: cef_state_t,

    ///
    // Controls whether local storage can be used. Also configurable using the
    // "disable-local-storage" command-line switch.
    ///
    pub local_storage: cef_state_t,

    ///
    // Controls whether databases can be used. Also configurable using the
    // "disable-databases" command-line switch.
    ///
    pub databases: cef_state_t,

    ///
    // Controls whether the application cache can be used. Also configurable using
    // the "disable-application-cache" command-line switch.
    ///
    pub application_cache: cef_state_t,

    ///
    // Controls whether WebGL can be used. Note that WebGL requires hardware
    // support and may not work on all systems even when enabled. Also
    // configurable using the "disable-webgl" command-line switch.
    ///
    pub webgl: cef_state_t,

    ///
    // Opaque background color used for the browser before a document is loaded
    // and when no document color is specified. By default the background color
    // will be the same as CefSettings.background_color. Only the RGB compontents
    // of the specified value will be used. The alpha component must greater than
    // 0 to enable use of the background color but will be otherwise ignored.
    ///
    pub background_color: cef_color_t,

    ///
    // Comma delimited ordered list of language codes without any whitespace that
    // will be used in the "Accept-Language" HTTP header. May be set globally
    // using the CefBrowserSettings.accept_language_list value. If both values are
    // empty then "en-US,en" will be used.
    ///
    pub accept_language_list: cef_string_t,
}

pub type cef_browser_settings_t = _cef_browser_settings;
pub type CefBrowserSettings = cef_browser_settings_t;


///
// Structure representing cursor information. |buffer| will be
// |size.width|*|size.height|*4 bytes in size and represents a BGRA image with
// an upper-left origin.
///
pub struct _cef_cursor_info {
    pub hotspot: cef_point_t,
    pub image_scale_factor: f32,
    pub buffer: *mut isize,
    pub size: cef_size_t,
}

pub type cef_cursor_info_t = _cef_cursor_info;
pub type CefCursorInfo = cef_cursor_info_t;

///
// Return value types.
///
pub enum cef_return_value_t {
    ///
    // Cancel immediately.
    ///
    RV_CANCEL = 0,

    ///
    // Continue immediately.
    ///
    RV_CONTINUE,

    ///
    // Continue asynchronously (usually via a callback).
    ///
    RV_CONTINUE_ASYNC,
}



///
// Request context initialization settings. Specify NULL or 0 to get the
// recommended default values.
///
pub struct _cef_request_context_settings {
  ///
  // Size of this structure.
  ///
  pub size: size_t,

  ///
  // The location where cache data will be stored on disk. If empty then
  // browsers will be created in "incognito mode" where in-memory caches are
  // used for storage and no data is persisted to disk. HTML5 databases such as
  // localStorage will only persist across sessions if a cache path is
  // specified. To share the global browser cache and related configuration set
  // this value to match the CefSettings.cache_path value.
  ///
  pub cache_path: cef_string_t,

  ///
  // To persist session cookies (cookies without an expiry date or validity
  // interval) by default when using the global cookie manager set this value to
  // true. Session cookies are generally intended to be transient and most Web
  // browsers do not persist them. Can be set globally using the
  // CefSettings.persist_session_cookies value. This value will be ignored if
  // |cache_path| is empty or if it matches the CefSettings.cache_path value.
  ///
  pub persist_session_cookies: i32,

  ///
  // Set to true (1) to ignore errors related to invalid SSL certificates.
  // Enabling this setting can lead to potential security vulnerabilities like
  // "man in the middle" attacks. Applications that load content from the
  // internet should not enable this setting. Can be set globally using the
  // CefSettings.ignore_certificate_errors value. This value will be ignored if
  // |cache_path| matches the CefSettings.cache_path value.
  ///
  pub ignore_certificate_errors: i32,

  ///
  // Comma delimited ordered list of language codes without any whitespace that
  // will be used in the "Accept-Language" HTTP header. Can be set globally
  // using the CefSettings.accept_language_list value or overridden on a per-
  // browser basis using the CefBrowserSettings.accept_language_list value. If
  // all values are empty then "en-US,en" will be used. This value will be
  // ignored if |cache_path| matches the CefSettings.cache_path value.
  ///
  pub accept_language_list: cef_string_t,
}

pub type cef_request_context_settings_t = _cef_request_context_settings;
pub type CefRequestContextSettings = cef_request_context_settings_t;

///
// Margin type for PDF printing.
///
pub enum cef_pdf_print_margin_type_t {
  ///
  // Default margins.
  ///
  PDF_PRINT_MARGIN_DEFAULT = 0,

  ///
  // No margins.
  ///
  PDF_PRINT_MARGIN_NONE,

  ///
  // Minimum margins.
  ///
  PDF_PRINT_MARGIN_MINIMUM,

  ///
  // Custom margins using the |margin_*| values from cef_pdf_print_settings_t.
  ///
  PDF_PRINT_MARGIN_CUSTOM,
}

///
// Structure representing PDF print settings.
///
pub struct cef_pdf_print_settings {
  ///
  // Page title to display in the header. Only used if |header_footer_enabled|
  // is set to true (1).
  ///
  pub header_footer_title: cef_string_t,

  ///
  // URL to display in the footer. Only used if |header_footer_enabled| is set
  // to true (1).
  ///
  pub header_footer_url: cef_string_t,

  ///
  // Output page size in microns. If either of these values is less than or
  // equal to zero then the default paper size (A4) will be used.
  ///
  pub page_width: i32,
  pub page_height: i32,

  ///
  // Margins in millimeters. Only used if |margin_type| is set to
  // PDF_PRINT_MARGIN_CUSTOM.
  ///
  pub margin_top: f64,
  pub margin_right: f64,
  pub margin_bottom: f64,
  pub margin_left: f64,

  ///
  // Margin type.
  ///
  pub margin_type: cef_pdf_print_margin_type_t,

  ///
  // Set to true (1) to print headers and footers or false (0) to not print
  // headers and footers.
  ///
  pub header_footer_enabled: i32,

  ///
  // Set to true (1) to print the selection only or false (0) to print all.
  ///
  pub selection_only: i32,

  ///
  // Set to true (1) for landscape mode or false (0) for portrait mode.
  ///
  pub landscape: i32,

  ///
  // Set to true (1) to print background graphics or false (0) to not print
  // background graphics.
  ///
  pub backgrounds_enabled: i32,
}
pub type cef_pdf_print_settings_t = cef_pdf_print_settings;
pub type CefPdfPrintSettings = cef_pdf_print_settings;
