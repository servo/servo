/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use libc::{c_uint, c_ushort, c_int, c_double, size_t, c_void, c_longlong};

pub type cef_string_map_t = c_void;
pub type cef_string_list_t = c_void;
pub type cef_text_input_context_t = c_void;
pub type cef_event_handle_t = c_void;

//these all need to be done...
pub type cef_binary_value = *c_void;
pub type cef_dictionary_value = *c_void;
pub type cef_client_t = c_void;
pub type cef_request_t = c_void;
pub type cef_response_t = c_void;
pub type cef_urlrequest_client_t = c_void;
pub type cef_frame = *c_void;
pub type cef_domnode = *c_void;
pub type cef_load_handler = *c_void;
pub type cef_request = *c_void;
pub type cef_navigation_type = *c_void;
pub type cef_request_context_t = c_void;
pub type cef_window_info_t = c_void;
pub type cef_browser_settings_t = c_void;
pub type cef_v8context = *c_void;
pub type cef_v8exception = *c_void;
pub type cef_v8stack_trace = *c_void;
pub type cef_window_handle_t = c_void; //FIXME: wtf is this

pub type cef_string_t = cef_string_utf8; //FIXME: this is #defined...
pub type cef_string_userfree_t = cef_string_t; //FIXME: this is #defined...

pub type cef_string_utf8_t = cef_string_utf8;
pub struct cef_string_utf8 {
    pub str: *u8,
    pub length: size_t,
    pub dtor: *fn(str: *u8),
}

pub type cef_string_utf16_t = cef_string_utf16;
pub struct cef_string_utf16 {
    pub str: *c_ushort,
    pub length: size_t,
    pub dtor: *fn(str: *c_ushort),
}

pub type cef_string_wide_t = cef_string_wide;
pub struct cef_string_wide {
    pub str: *c_uint, //FIXME: not sure if correct...
    pub length: size_t,
    pub dtor: *fn(str: *c_uint),
}

pub type cef_main_args_t = cef_main_args;
pub struct cef_main_args {
  pub argc: c_int,
  pub argv: **u8
}

pub type cef_color_t = c_uint;

///
// Existing thread IDs.
///
pub enum cef_thread_id_t {
// BROWSER PROCESS THREADS -- Only available in the browser process.

  ///
  // The main thread in the browser. This will be the same as the main
  // application thread if CefInitialize() is called with a
  // CefSettings.multi_threaded_message_loop value of false.
  ///
  TID_UI,

  ///
  // Used to interact with the database.
  ///
  TID_DB,

  ///
  // Used to interact with the file system.
  ///
  TID_FILE,

  ///
  // Used for file system operations that block user interactions.
  // Responsiveness of this thread affects users.
  ///
  TID_FILE_USER_BLOCKING,

  ///
  // Used to launch and terminate browser processes.
  ///
  TID_PROCESS_LAUNCHER,

  ///
  // Used to handle slow HTTP cache operations.
  ///
  TID_CACHE,

  ///
  // Used to process IPC and network messages.
  ///
  TID_IO,

// RENDER PROCESS THREADS -- Only available in the render process.

  ///
  // The main thread in the renderer. Used for all WebKit and V8 interaction.
  ///
  TID_RENDERER,
}

///
// Navigation types.
///
pub enum cef_navigation_type_t {
  NAVIGATION_LINK_CLICKED = 0,
  NAVIGATION_FORM_SUBMITTED,
  NAVIGATION_BACK_FORWARD,
  NAVIGATION_RELOAD,
  NAVIGATION_FORM_RESUBMITTED,
  NAVIGATION_OTHER,
}

///
// Mouse button types.
///
pub enum cef_mouse_button_type_t {
  MBT_LEFT   = 0,
  MBT_MIDDLE,
  MBT_RIGHT,
}

///
// Structure representing mouse event information.
///
pub type cef_mouse_event_t = cef_mouse_event;
pub struct cef_mouse_event {
    ///
    // X coordinate relative to the left side of the view.
    ///
    pub x: c_int,

    ///
    // Y coordinate relative to the top side of the view.
    ///
    pub y: c_int,

    ///
    // Bit flags describing any pressed modifier keys. See
    // cef_event_flags_t for values.
    ///
    pub modifiers: c_uint,
}

///
// Post data elements may represent either bytes or files.
///
pub enum cef_postdataelement_type_t {
  PDE_TYPE_EMPTY  = 0,
  PDE_TYPE_BYTES,
  PDE_TYPE_FILE,
}

///
// Flags used to customize the behavior of CefURLRequest.
///
pub enum cef_urlrequest_flags_t {
  ///
  // Default behavior.
  ///
  UR_FLAG_NONE                      = 0,

  ///
  // If set the cache will be skipped when handling the request.
  ///
  UR_FLAG_SKIP_CACHE                = 1 << 0,

  ///
  // If set user name, password, and cookies may be sent with the request.
  ///
  UR_FLAG_ALLOW_CACHED_CREDENTIALS  = 1 << 1,

  ///
  // If set cookies may be sent with the request and saved from the response.
  // UR_FLAG_ALLOW_CACHED_CREDENTIALS must also be set.
  ///
  UR_FLAG_ALLOW_COOKIES             = 1 << 2,

  ///
  // If set upload progress events will be generated when a request has a body.
  ///
  UR_FLAG_REPORT_UPLOAD_PROGRESS    = 1 << 3,

  ///
  // If set load timing info will be collected for the request.
  ///
  UR_FLAG_REPORT_LOAD_TIMING        = 1 << 4,

  ///
  // If set the headers sent and received for the request will be recorded.
  ///
  UR_FLAG_REPORT_RAW_HEADERS        = 1 << 5,

  ///
  // If set the CefURLRequestClient::OnDownloadData method will not be called.
  ///
  UR_FLAG_NO_DOWNLOAD_DATA          = 1 << 6,

  ///
  // If set 5XX redirect errors will be propagated to the observer instead of
  // automatically re-tried. This currently only applies for requests
  // originated in the browser process.
  ///
  UR_FLAG_NO_RETRY_ON_5XX           = 1 << 7,
}


///
// Flags that represent CefURLRequest status.
///
pub enum cef_urlrequest_status_t {
  ///
  // Unknown status.
  ///
  UR_UNKNOWN = 0,

  ///
  // Request succeeded.
  ///
  UR_SUCCESS,

  ///
  // An IO request is pending, and the caller will be informed when it is
  // completed.
  ///
  UR_IO_PENDING,

  ///
  // Request was canceled programatically.
  ///
  UR_CANCELED,

  ///
  // Request failed for some reason.
  ///
  UR_FAILED,
}



///
// Supported error code values. See net\base\net_error_list.h for complete
// descriptions of the error codes.
///
pub enum cef_errorcode_t {
  ERR_NONE = 0,
  ERR_FAILED = -2,
  ERR_ABORTED = -3,
  ERR_INVALID_ARGUMENT = -4,
  ERR_INVALID_HANDLE = -5,
  ERR_FILE_NOT_FOUND = -6,
  ERR_TIMED_OUT = -7,
  ERR_FILE_TOO_BIG = -8,
  ERR_UNEXPECTED = -9,
  ERR_ACCESS_DENIED = -10,
  ERR_NOT_IMPLEMENTED = -11,
  ERR_CONNECTION_CLOSED = -100,
  ERR_CONNECTION_RESET = -101,
  ERR_CONNECTION_REFUSED = -102,
  ERR_CONNECTION_ABORTED = -103,
  ERR_CONNECTION_FAILED = -104,
  ERR_NAME_NOT_RESOLVED = -105,
  ERR_INTERNET_DISCONNECTED = -106,
  ERR_SSL_PROTOCOL_ERROR = -107,
  ERR_ADDRESS_INVALID = -108,
  ERR_ADDRESS_UNREACHABLE = -109,
  ERR_SSL_CLIENT_AUTH_CERT_NEEDED = -110,
  ERR_TUNNEL_CONNECTION_FAILED = -111,
  ERR_NO_SSL_VERSIONS_ENABLED = -112,
  ERR_SSL_VERSION_OR_CIPHER_MISMATCH = -113,
  ERR_SSL_RENEGOTIATION_REQUESTED = -114,
  ERR_CERT_COMMON_NAME_INVALID = -200,
  ERR_CERT_DATE_INVALID = -201,
  ERR_CERT_AUTHORITY_INVALID = -202,
  ERR_CERT_CONTAINS_ERRORS = -203,
  ERR_CERT_NO_REVOCATION_MECHANISM = -204,
  ERR_CERT_UNABLE_TO_CHECK_REVOCATION = -205,
  ERR_CERT_REVOKED = -206,
  ERR_CERT_INVALID = -207,
  ERR_CERT_END = -208,
  ERR_INVALID_URL = -300,
  ERR_DISALLOWED_URL_SCHEME = -301,
  ERR_UNKNOWN_URL_SCHEME = -302,
  ERR_TOO_MANY_REDIRECTS = -310,
  ERR_UNSAFE_REDIRECT = -311,
  ERR_UNSAFE_PORT = -312,
  ERR_INVALID_RESPONSE = -320,
  ERR_INVALID_CHUNKED_ENCODING = -321,
  ERR_METHOD_NOT_SUPPORTED = -322,
  ERR_UNEXPECTED_PROXY_AUTH = -323,
  ERR_EMPTY_RESPONSE = -324,
  ERR_RESPONSE_HEADERS_TOO_BIG = -325,
  ERR_CACHE_MISS = -400,
  ERR_INSECURE_RESPONSE = -501,
}


///
// Key event types.
///
pub enum cef_key_event_type_t {
  KEYEVENT_RAWKEYDOWN = 0,
  KEYEVENT_KEYDOWN,
  KEYEVENT_KEYUP,
  KEYEVENT_CHAR
}

///
// Structure representing keyboard event information.
///
pub type cef_key_event_t = cef_key_event;
pub struct cef_key_event {
  ///
  // The type of keyboard event.
  ///
  pub t: cef_key_event_type_t,

  ///
  // Bit flags describing any pressed modifier keys. See
  // cef_event_flags_t for values.
  ///
  pub modifiers: c_uint,

  ///
  // The Windows key code for the key event. This value is used by the DOM
  // specification. Sometimes it comes directly from the event (i.e. on
  // Windows) and sometimes it's determined using a mapping function. See
  // WebCore/platform/chromium/KeyboardCodes.h for the list of values.
  ///
  pub windows_key_code: c_int,

  ///
  // The actual key code genenerated by the platform.
  ///
  pub native_key_code: c_int,

  ///
  // Indicates whether the event is considered a "system key" event (see
  // http://msdn.microsoft.com/en-us/library/ms646286(VS.85).aspx for details).
  // This value will always be false on non-Windows platforms.
  ///
  pub is_system_key: c_int,

  ///
  // The character generated by the keystroke.
  ///
  pub character: c_ushort, //FIXME: can be wchar_t also?

  ///
  // Same as |character| but unmodified by any concurrently-held modifiers
  // (except shift). This is useful for working out shortcut keys.
  ///
  pub unmodified_character: c_ushort, //FIXME: can be wchar_t also?

  ///
  // True if the focus is currently on an editable field on the page. This is
  // useful for determining if standard key events should be intercepted.
  ///
  pub focus_on_editable_field: c_int,
}

///
// Structure representing a rectangle.
///
pub type cef_rect_t = cef_rect;
pub struct cef_rect {
  pub x: c_int,
  pub y: c_int,
  pub width: c_int,
  pub height: c_int,
}

///
// Paint element types.
///
pub enum cef_paint_element_type_t {
  PET_VIEW  = 0,
  PET_POPUP,
}

///
// Supported file dialog modes.
///
pub enum cef_file_dialog_mode_t {
  ///
  // Requires that the file exists before allowing the user to pick it.
  ///
  FILE_DIALOG_OPEN = 0,

  ///
  // Like Open, but allows picking multiple files to open.
  ///
  FILE_DIALOG_OPEN_MULTIPLE,

  ///
  // Allows picking a nonexistent file, and prompts to overwrite if the file
  // already exists.
  ///
  FILE_DIALOG_SAVE,
}

///
// Supported value types.
///
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

///
// Existing process IDs.
///
pub enum cef_process_id_t {
  ///
  // Browser process.
  ///
  PID_BROWSER,
  ///
  // Renderer process.
  ///
  PID_RENDERER,
}

///
// Log severity levels.
///
pub enum cef_log_severity_t {
  ///
  // Default logging (currently INFO logging).
  ///
  LOGSEVERITY_DEFAULT,

  ///
  // Verbose logging.
  ///
  LOGSEVERITY_VERBOSE,

  ///
  // INFO logging.
  ///
  LOGSEVERITY_INFO,

  ///
  // WARNING logging.
  ///
  LOGSEVERITY_WARNING,

  ///
  // ERROR logging.
  ///
  LOGSEVERITY_ERROR,

  ///
  // ERROR_REPORT logging.
  ///
  LOGSEVERITY_ERROR_REPORT,

  ///
  // Completely disable logging.
  ///
  LOGSEVERITY_DISABLE = 99
}


///
// Structure representing a message. Can be used on any process and thread.
///
pub type cef_process_message_t = cef_process_message;
pub struct cef_process_message {
  ///
  // Base structure.
  ///
  pub base: cef_base,

  ///
  // Returns true (1) if this object is valid. Do not call any other functions
  // if this function returns false (0).
  ///
  pub is_valid: extern "C" fn(process_message: *mut cef_process_message) -> c_int,

  ///
  // Returns true (1) if the values of this object are read-only. Some APIs may
  // expose read-only objects.
  ///
  pub is_read_only: extern "C" fn(process_message: *mut cef_process_message) -> c_int,

  ///
  // Returns a writable copy of this object.
  ///
  pub copy: extern "C" fn(process_message: *mut cef_process_message) -> *mut cef_process_message,

  ///
  // Returns the message name.
  ///
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_name: extern "C" fn(process_message: *mut cef_process_message) -> *mut cef_string_userfree_t,

  ///
  // Returns the list of arguments.
  ///
  pub get_argument_list: extern "C" fn(process_message: *mut cef_process_message) -> *mut cef_list_value,
}

///
// Initialization settings. Specify NULL or 0 to get the recommended default
// values. Many of these and other settings can also configured using command-
// line switches.
///
pub type cef_settings_t = cef_settings;
pub struct cef_settings {
  ///
  // Size of this structure.
  ///
  pub size: size_t,

  ///
  // Set to true (1) to use a single process for the browser and renderer. This
  // run mode is not officially supported by Chromium and is less stable than
  // the multi-process default. Also configurable using the "single-process"
  // command-line switch.
  ///
  pub single_process: c_int,

  ///
  // Set to true (1) to disable the sandbox for sub-processes. See
  // cef_sandbox_win.h for requirements to enable the sandbox on Windows. Also
  // configurable using the "no-sandbox" command-line switch.
  ///
  pub no_sandbox: c_int,

  ///
  // The path to a separate executable that will be launched for sub-processes.
  // By default the browser process executable is used. See the comments on
  // CefExecuteProcess() for details. Also configurable using the
  // "browser-subprocess-path" command-line switch.
  ///
  pub browser_subprocess_path: cef_string_t,

  ///
  // Set to true (1) to have the browser process message loop run in a separate
  // thread. If false (0) than the CefDoMessageLoopWork() function must be
  // called from your application message loop.
  ///
  pub multi_threaded_message_loop: c_int,

  ///
  // Set to true to enable windowless (off-screen) rendering support. Do not
  // enable this value if the application does not use windowless rendering as
  // it may reduce rendering performance on some systems.
  ///
  pub windowless_rendering_enabled: bool,

  ///
  // Set to true (1) to disable configuration of browser process features using
  // standard CEF and Chromium command-line arguments. Configuration can still
  // be specified using CEF data structures or via the
  // CefApp::OnBeforeCommandLineProcessing() method.
  ///
  pub command_line_args_disabled: c_int,

  ///
  // The location where cache data will be stored on disk. If empty an in-memory
  // cache will be used for some features and a temporary disk cache for others.
  // HTML5 databases such as localStorage will only persist across sessions if a
  // cache path is specified.
  ///
  pub cache_path: cef_string_t,

  ///
  // To persist session cookies (cookies without an expiry date or validity
  // interval) by default when using the global cookie manager set this value to
  // true. Session cookies are generally intended to be transient and most Web
  // browsers do not persist them. A |cache_path| value must also be specified to
  // enable this feature. Also configurable using the "persist-session-cookies"
  // command-line switch.
  ///
  pub persist_session_cookies: c_int,

  ///
  // Value that will be returned as the User-Agent HTTP header. If empty the
  // default User-Agent string will be used. Also configurable using the
  // "user-agent" command-line switch.
  ///
  pub user_agent: cef_string_t,

  ///
  // Value that will be inserted as the product portion of the default
  // User-Agent string. If empty the Chromium product version will be used. If
  // |userAgent| is specified this value will be ignored. Also configurable
  // using the "product-version" command-line switch.
  ///
  pub product_version: cef_string_t,

  ///
  // The locale string that will be passed to WebKit. If empty the default
  // locale of "en-US" will be used. This value is ignored on Linux where locale
  // is determined using environment variable parsing with the precedence order:
  // LANGUAGE, LC_ALL, LC_MESSAGES and LANG. Also configurable using the "lang"
  // command-line switch.
  ///
  pub locale: cef_string_t,

  ///
  // The directory and file name to use for the debug log. If empty, the
  // default name of "debug.log" will be used and the file will be written
  // to the application directory. Also configurable using the "log-file"
  // command-line switch.
  ///
  pub log_file: cef_string_t,

  ///
  // The log severity. Only messages of this severity level or higher will be
  // logged. Also configurable using the "log-severity" command-line switch with
  // a value of "verbose", "info", "warning", "error", "error-report" or
  // "disable".
  ///
  pub log_severity: cef_log_severity_t,

  ///
  // Enable DCHECK in release mode to ease debugging. Also configurable using the
  // "enable-release-dcheck" command-line switch.
  ///
  pub release_dcheck_enabled: c_int,

  ///
  // Custom flags that will be used when initializing the V8 JavaScript engine.
  // The consequences of using custom flags may not be well tested. Also
  // configurable using the "js-flags" command-line switch.
  ///
  pub javascript_flags: cef_string_t,

  ///
  // The fully qualified path for the resources directory. If this value is
  // empty the cef.pak and/or devtools_resources.pak files must be located in
  // the module directory on Windows/Linux or the app bundle Resources directory
  // on Mac OS X. Also configurable using the "resources-dir-path" command-line
  // switch.
  ///
  pub resources_dir_path: cef_string_t,

  ///
  // The fully qualified path for the locales directory. If this value is empty
  // the locales directory must be located in the module directory. This value
  // is ignored on Mac OS X where pack files are always loaded from the app
  // bundle Resources directory. Also configurable using the "locales-dir-path"
  // command-line switch.
  ///
  pub locales_dir_path: cef_string_t,

  ///
  // Set to true (1) to disable loading of pack files for resources and locales.
  // A resource bundle handler must be provided for the browser and render
  // processes via CefApp::GetResourceBundleHandler() if loading of pack files
  // is disabled. Also configurable using the "disable-pack-loading" command-
  // line switch.
  ///
  pub pack_loading_disabled: c_int,

  ///
  // Set to a value between 1024 and 65535 to enable remote debugging on the
  // specified port. For example, if 8080 is specified the remote debugging URL
  // will be http://localhost:8080. CEF can be remotely debugged from any CEF or
  // Chrome browser window. Also configurable using the "remote-debugging-port"
  // command-line switch.
  ///
  pub remote_debugging_port: c_int,

  ///
  // The number of stack trace frames to capture for uncaught exceptions.
  // Specify a positive value to enable the CefV8ContextHandler::
  // OnUncaughtException() callback. Specify 0 (default value) and
  // OnUncaughtException() will not be called. Also configurable using the
  // "uncaught-exception-stack-size" command-line switch.
  ///
  pub uncaught_exception_stack_size: c_int,

  ///
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
  ///
  pub context_safety_implementation: c_int,

  ///
  // Set to true (1) to ignore errors related to invalid SSL certificates.
  // Enabling this setting can lead to potential security vulnerabilities like
  // "man in the middle" attacks. Applications that load content from the
  // internet should not enable this setting. Also configurable using the
  // "ignore-certificate-errors" command-line switch.
  ///
  pub ignore_certificate_errors: c_int,

  ///
  // Opaque background color used for accelerated content. By default the
  // background color will be white. Only the RGB compontents of the specified
  // value will be used. The alpha component must greater than 0 to enable use
  // of the background color but will be otherwise ignored.
  ///
  pub background_color: cef_color_t,
}

///
// Structure defining the reference count implementation functions. All
// framework structures must include the cef_base_t structure first.
///
pub type cef_base_t = cef_base;
pub struct cef_base {
  ///
  // Size of the data structure.
  ///
  pub size: size_t,

  ///
  // Increment the reference count.
  ///
  pub add_ref: extern "C" fn(base: *mut cef_base) -> c_int,

  ///
  // Decrement the reference count.  Delete this object when no references
  // remain.
  ///
  pub release: extern "C" fn(base: *mut cef_base) -> c_int,

  ///
  // Returns the current number of references.
  ///
  pub get_refct: extern "C" fn(base: *mut cef_base) -> c_int,
}

///
// Structure used to create and/or parse command line arguments. Arguments with
// '--', '-' and, on Windows, '/' prefixes are considered switches. Switches
// will always precede any arguments without switch prefixes. Switches can
// optionally have a value specified using the '=' delimiter (e.g.
// "-switch=value"). An argument of "--" will terminate switch parsing with all
// subsequent tokens, regardless of prefix, being interpreted as non-switch
// arguments. Switch names are considered case-insensitive. This structure can
// be used before cef_initialize() is called.
///
pub type cef_command_line_t = cef_command_line;
pub struct cef_command_line {
  ///
  // Base structure.
  ///
  pub base: cef_base,

  ///
  // Returns true (1) if this object is valid. Do not call any other functions
  // if this function returns false (0).
  ///
  pub is_valid: extern "C" fn(cmd: *mut cef_command_line),

  ///
  // Returns true (1) if the values of this object are read-only. Some APIs may
  // expose read-only objects.
  ///
  pub is_read_only: extern "C" fn(cmd: *mut cef_command_line),

  ///
  // Returns a writable copy of this object.
  ///
  pub copy: extern "C" fn(cmd: *mut cef_command_line) -> *mut cef_command_line,

  ///
  // Initialize the command line with the specified |argc| and |argv| values.
  // The first argument must be the name of the program. This function is only
  // supported on non-Windows platforms.
  ///
  pub init_from_argv: extern "C" fn(cmd: *mut cef_command_line, argc: c_int, argv: *u8),

  ///
  // Initialize the command line with the string returned by calling
  // GetCommandLineW(). This function is only supported on Windows.
  ///
  pub init_from_string: extern "C" fn(cmd: *mut cef_command_line, command_line: *cef_string_t),

  ///
  // Reset the command-line switches and arguments but leave the program
  // component unchanged.
  ///
  pub reset: extern "C" fn(cmd: *mut cef_command_line),

  ///
  // Retrieve the original command line string as a vector of strings. The argv
  // array: { program, [(--|-|/)switch[=value]]*, [--], [argument]* }
  ///
  pub get_argv: extern "C" fn(cmd: *mut cef_command_line, argv: *mut cef_string_list_t),

  ///
  // Constructs and returns the represented command line string. Use this
  // function cautiously because quoting behavior is unclear.
  ///
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_command_line_string: extern "C" fn(cmd: *mut cef_command_line) -> *mut cef_string_userfree_t,

  ///
  // Get the program part of the command line string (the first item).
  ///
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_program: extern "C" fn(cmd: *mut cef_command_line) -> *mut cef_string_userfree_t,

  ///
  // Set the program part of the command line string (the first item).
  ///
  pub set_program: extern "C" fn(cmd: *mut cef_command_line, name: *cef_string_t),

  ///
  // Returns true (1) if the command line has switches.
  ///
  pub has_switches: extern "C" fn(cmd: *mut cef_command_line) -> c_int,

  ///
  // Returns true (1) if the command line contains the given switch.
  ///
  pub has_switch: extern "C" fn(cmd: *mut cef_command_line, name: *cef_string_t) -> c_int,

  ///
  // Returns the value associated with the given switch. If the switch has no
  // value or isn't present this function returns the NULL string.
  ///
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_switch_value: extern "C" fn(cmd: *mut cef_command_line, name: *cef_string_t) -> *mut cef_string_userfree_t,

  ///
  // Returns the map of switch names and values. If a switch has no value an
  // NULL string is returned.
  ///
  pub get_switches: extern "C" fn(cmd: *mut cef_command_line, switches: cef_string_map_t),

  ///
  // Add a switch to the end of the command line. If the switch has no value
  // pass an NULL value string.
  ///
  pub append_switch: extern "C" fn(cmd: *mut cef_command_line, name: *cef_string_t),

  ///
  // Add a switch with the specified value to the end of the command line.
  ///
  pub append_switch_with_value: extern "C" fn(cmd: *mut cef_command_line, name: *cef_string_t, value: *cef_string_t),

  ///
  // True if there are remaining command line arguments.
  ///
  pub has_arguments: extern "C" fn(cmd: *mut cef_command_line) -> c_int,

  ///
  // Get the remaining command line arguments.
  ///
  pub get_arguments: extern "C" fn(cmd: *mut cef_command_line, arguments: *mut cef_string_list_t),

  ///
  // Add an argument to the end of the command line.
  ///
  pub append_argument: extern "C" fn(cmd: *mut cef_command_line, argument: *cef_string_t),

  ///
  // Insert a command before the current command. Common for debuggers, like
  // "valgrind" or "gdb --args".
  ///
  pub prepend_wrapper: extern "C" fn(cmd: *mut cef_command_line, wrapper: *cef_string_t),
}


///
// Structure that manages custom scheme registrations.
///
pub type cef_scheme_registrar_t = cef_scheme_registrar;
pub struct cef_scheme_registrar {
  ///
  // Base structure.
  ///
  pub base: cef_base,

  ///
  // Register a custom scheme. This function should not be called for the built-
  // in HTTP, HTTPS, FILE, FTP, ABOUT and DATA schemes.
  //
  // If |is_standard| is true (1) the scheme will be treated as a standard
  // scheme. Standard schemes are subject to URL canonicalization and parsing
  // rules as defined in the Common Internet Scheme Syntax RFC 1738 Section 3.1
  // available at http://www.ietf.org/rfc/rfc1738.txt
  //
  // In particular, the syntax for standard scheme URLs must be of the form:
  // <pre>
  //  [scheme]://[username]:[password]@[host]:[port]/[url-path]
  // </pre Standard scheme URLs must have a host component that is a fully
  // qualified domain name as defined in Section 3.5 of RFC 1034 [13] and
  // Section 2.1 of RFC 1123. These URLs will be canonicalized to
  // "scheme://host/path" in the simplest case and
  // "scheme://username:password@host:port/path" in the most explicit case. For
  // example, "scheme:host/path" and "scheme:///host/path" will both be
  // canonicalized to "scheme://host/path". The origin of a standard scheme URL
  // is the combination of scheme, host and port (i.e., "scheme://host:port" in
  // the most explicit case).
  //
  // For non-standard scheme URLs only the "scheme:" component is parsed and
  // canonicalized. The remainder of the URL will be passed to the handler as-
  // is. For example, "scheme:///some%20text" will remain the same. Non-standard
  // scheme URLs cannot be used as a target for form submission.
  //
  // If |is_local| is true (1) the scheme will be treated as local (i.e., with
  // the same security rules as those applied to "file" URLs). Normal pages
  // cannot link to or access local URLs. Also, by default, local URLs can only
  // perform XMLHttpRequest calls to the same URL (origin + path) that
  // originated the request. To allow XMLHttpRequest calls from a local URL to
  // other URLs with the same origin set the
  // CefSettings.file_access_from_file_urls_allowed value to true (1). To allow
  // XMLHttpRequest calls from a local URL to all origins set the
  // CefSettings.universal_access_from_file_urls_allowed value to true (1).
  //
  // If |is_display_isolated| is true (1) the scheme will be treated as display-
  // isolated. This means that pages cannot display these URLs unless they are
  // from the same scheme. For example, pages in another origin cannot create
  // iframes or hyperlinks to URLs with this scheme.
  //
  // This function may be called on any thread. It should only be called once
  // per unique |scheme_name| value. If |scheme_name| is already registered or
  // if an error occurs this function will return false (0).
  ///
  add_custom_scheme: extern "C" fn(registrar: *mut cef_scheme_registrar,
                               scheme_name: *cef_string_t,
                               is_standard: c_int, is_local: c_int, is_display_isolated: c_int),
}

///
// Structure used to implement a custom resource bundle structure. The functions
// of this structure may be called on multiple threads.
///
pub type cef_resource_bundle_handler_t = cef_resource_bundle_handler;
pub struct cef_resource_bundle_handler {
  ///
  // Base structure.
  ///
  pub base: cef_base,

  ///
  // Called to retrieve a localized translation for the string specified by
  // |message_id|. To provide the translation set |string| to the translation
  // string and return true (1). To use the default translation return false
  // (0). Supported message IDs are listed in cef_pack_strings.h.
  ///
  pub get_localized_string: extern "C" fn(bundle_handler: *mut cef_resource_bundle_handler,
                                  message_id: c_int, string: *mut cef_string_t) -> c_int,

  ///
  // Called to retrieve data for the resource specified by |resource_id|. To
  // provide the resource data set |data| and |data_size| to the data pointer
  // and size respectively and return true (1). To use the default resource data
  // return false (0). The resource data will not be copied and must remain
  // resident in memory. Supported resource IDs are listed in
  // cef_pack_resources.h.
  ///
  pub get_data_resource: extern "C" fn(bundle_handler: *mut cef_resource_bundle_handler,
                               resource_id: c_int, data: **mut c_void, data_size: *mut size_t) -> c_int,
}



///
// Structure representing a list value. Can be used on any process and thread.
///
pub type cef_list_value_t = cef_list_value;
pub struct cef_list_value {
  ///
  // Base structure.
  ///
  pub base: cef_base,

  ///
  // Returns true (1) if this object is valid. Do not call any other functions
  // if this function returns false (0).
  ///
  pub is_valid: extern "C" fn(list_value: *mut cef_list_value) -> c_int,

  ///
  // Returns true (1) if this object is currently owned by another object.
  ///
  pub is_owned: extern "C" fn(list_value: *mut cef_list_value) -> c_int,

  ///
  // Returns true (1) if the values of this object are read-only. Some APIs may
  // expose read-only objects.
  ///
  pub is_read_only: extern "C" fn(list_value: *mut cef_list_value) -> c_int,

  ///
  // Returns a writable copy of this object.
  ///
  pub copy: extern "C" fn(list_value: *mut cef_list_value) -> *mut cef_list_value,

  ///
  // Sets the number of values. If the number of values is expanded all new
  // value slots will default to type null. Returns true (1) on success.
  ///
  pub set_size: extern "C" fn(list_value: *mut cef_list_value, size: size_t) -> c_int,

  ///
  // Returns the number of values.
  ///
  pub get_size: extern "C" fn(list_value: *mut cef_list_value) -> size_t,

  ///
  // Removes all values. Returns true (1) on success.
  ///
  pub clear: extern "C" fn(list_value: *mut cef_list_value) -> c_int,

  ///
  // Removes the value at the specified index.
  ///
  pub remove: extern "C" fn(list_value: *mut cef_list_value) -> c_int,

  ///
  // Returns the value type at the specified index.
  ///
  pub get_type: extern "C" fn(list_value: *mut cef_list_value, index: c_int) -> cef_value_type_t,

  ///
  // Returns the value at the specified index as type bool.
  ///
  pub get_bool: extern "C" fn(list_value: *mut cef_list_value, index: c_int) -> c_int,

  ///
  // Returns the value at the specified index as type int.
  ///
  pub get_int: extern "C" fn(list_value: *mut cef_list_value, index: c_int) -> c_int,

  ///
  // Returns the value at the specified index as type double.
  ///
  pub get_double: extern "C" fn(list_value: *mut cef_list_value, index: c_int) -> c_double,

  ///
  // Returns the value at the specified index as type string.
  ///
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_string: extern "C" fn(list_value: *mut cef_list_value, index: c_int) -> *mut cef_string_userfree_t,

  ///
  // Returns the value at the specified index as type binary.
  ///
  pub get_binary: extern "C" fn(list_value: *mut cef_list_value, index: c_int) -> *mut cef_binary_value,

  ///
  // Returns the value at the specified index as type dictionary.
  ///
  pub get_dictionary: extern "C" fn(list_value: *mut cef_list_value, index: c_int) -> *mut cef_dictionary_value,

  ///
  // Returns the value at the specified index as type list.
  ///
  pub get_list: extern "C" fn(list_value: *mut cef_list_value, index: c_int) -> *mut cef_list_value,

  ///
  // Sets the value at the specified index as type null. Returns true (1) if the
  // value was set successfully.
  ///
  pub set_null: extern "C" fn(list_value: *mut cef_list_value, index: c_int) -> c_int,

  ///
  // Sets the value at the specified index as type bool. Returns true (1) if the
  // value was set successfully.
  ///
  pub set_bool: extern "C" fn(list_value: *mut cef_list_value, index: c_int, value: c_int) -> c_int,

  ///
  // Sets the value at the specified index as type int. Returns true (1) if the
  // value was set successfully.
  ///
  pub set_int: extern "C" fn(list_value: *mut cef_list_value, index: c_int, value: c_int) -> c_int,

  ///
  // Sets the value at the specified index as type double. Returns true (1) if
  // the value was set successfully.
  ///
  pub set_double: extern "C" fn(list_value: *mut cef_list_value, index: c_int, value: c_double) -> c_int,

  ///
  // Sets the value at the specified index as type string. Returns true (1) if
  // the value was set successfully.
  ///
  pub set_string: extern "C" fn(list_value: *mut cef_list_value, index: c_int, value: *cef_string_t) -> c_int,

  ///
  // Sets the value at the specified index as type binary. Returns true (1) if
  // the value was set successfully. After calling this function the |value|
  // object will no longer be valid. If |value| is currently owned by another
  // object then the value will be copied and the |value| reference will not
  // change. Otherwise, ownership will be transferred to this object and the
  // |value| reference will be invalidated.
  ///
  pub set_binary: extern "C" fn(list_value: *mut cef_list_value, index: c_int, value: *mut cef_binary_value) -> c_int,

  ///
  // Sets the value at the specified index as type dict. Returns true (1) if the
  // value was set successfully. After calling this function the |value| object
  // will no longer be valid. If |value| is currently owned by another object
  // then the value will be copied and the |value| reference will not change.
  // Otherwise, ownership will be transferred to this object and the |value|
  // reference will be invalidated.
  ///
  pub set_dictionary: extern "C" fn(list_value: *mut cef_list_value, index: c_int, value: *mut cef_dictionary_value) -> c_int,

  ///
  // Sets the value at the specified index as type list. Returns true (1) if the
  // value was set successfully. After calling this function the |value| object
  // will no longer be valid. If |value| is currently owned by another object
  // then the value will be copied and the |value| reference will not change.
  // Otherwise, ownership will be transferred to this object and the |value|
  // reference will be invalidated.
  ///
  pub set_list: extern "C" fn(list_value: *mut cef_list_value, index: c_int, value: *mut cef_list_value) -> c_int,
}

///
// Structure used to implement browser process callbacks. The functions of this
// structure will be called on the browser process main thread unless otherwise
// indicated.
///
pub type cef_browser_process_handler_t = cef_browser_process_handler;
pub struct cef_browser_process_handler {
  ///
  // Base structure.
  ///
  pub base: cef_base,

  ///
  // Called on the browser process UI thread immediately after the CEF context
  // has been initialized.
  ///
  pub on_context_initialized: extern "C" fn(browser_handler: *mut cef_browser_process_handler),

  ///
  // Called before a child process is launched. Will be called on the browser
  // process UI thread when launching a render process and on the browser
  // process IO thread when launching a GPU or plugin process. Provides an
  // opportunity to modify the child process command line. Do not keep a
  // reference to |command_line| outside of this function.
  ///
  pub on_before_child_process_launch: extern "C" fn(browser_handler: *mut cef_browser_process_handler, command_line: *mut cef_command_line),

  ///
  // Called on the browser process IO thread after the main thread has been
  // created for a new render process. Provides an opportunity to specify extra
  // information that will be passed to
  // cef_render_process_handler_t::on_render_thread_created() in the render
  // process. Do not keep a reference to |extra_info| outside of this function.
  ///
  pub on_render_process_thread_created: extern "C" fn(browser_handler: *mut cef_browser_process_handler, extra_info: *mut cef_list_value),
}


///
// Callback structure for cef_browser_host_t::RunFileDialog. The functions of
// this structure will be called on the browser process UI thread.
///
pub type cef_run_file_dialog_callback_t = cef_run_file_dialog_callback;
pub struct cef_run_file_dialog_callback {
  ///
  // Base structure.
  ///
  pub base: cef_base,

  ///
  // Called asynchronously after the file dialog is dismissed. If the selection
  // was successful |file_paths| will be a single value or a list of values
  // depending on the dialog mode. If the selection was cancelled |file_paths|
  // will be NULL.
  ///
  pub cont: extern "C" fn(run_file_dialog_callback: *mut cef_run_file_dialog_callback,
                  browser_host: *mut cef_browser_host,
                  file_paths: *mut cef_string_list_t),
}

///
// Structure used to represent the browser process aspects of a browser window.
// The functions of this structure can only be called in the browser process.
// They may be called on any thread in that process unless otherwise indicated
// in the comments.
///
pub type cef_browser_host_t = cef_browser_host;
pub struct cef_browser_host {
  ///
  // Base structure.
  ///
  pub base: cef_base,

  ///
  // Returns the hosted browser object.
  ///
  pub get_browser: extern "C" fn(browser_host: *mut cef_browser_host) -> *mut cef_browser,

  ///
  // Call this function before destroying a contained browser window. This
  // function performs any internal cleanup that may be needed before the
  // browser window is destroyed. See cef_life_span_handler_t::do_close()
  // documentation for additional usage information.
  ///
  pub parent_window_will_close: extern "C" fn(browser_host: *mut cef_browser_host),

  ///
  // Request that the browser close. The JavaScript 'onbeforeunload' event will
  // be fired. If |force_close| is false (0) the event handler, if any, will be
  // allowed to prompt the user and the user can optionally cancel the close. If
  // |force_close| is true (1) the prompt will not be displayed and the close
  // will proceed. Results in a call to cef_life_span_handler_t::do_close() if
  // the event handler allows the close or if |force_close| is true (1). See
  // cef_life_span_handler_t::do_close() documentation for additional usage
  // information.
  ///
  pub close_browser: extern "C" fn(browser_host: *mut cef_browser_host, force_close: c_int),

  ///
  // Set focus for the browser window. If |enable| is true (1) focus will be set
  // to the window. Otherwise, focus will be removed.
  ///
  pub set_focus: extern "C" fn(browser_host: *mut cef_browser_host, force_close: c_int),

  ///
  // Retrieve the window handle for this browser.
  ///
  pub get_window_handle: extern "C" fn(browser_host: *mut cef_browser_host) -> *mut cef_window_handle_t,

  ///
  // Retrieve the window handle of the browser that opened this browser. Will
  // return NULL for non-popup windows. This function can be used in combination
  // with custom handling of modal windows.
  ///
  pub get_opener_window_handle: extern "C" fn(browser_host: *mut cef_browser_host) -> *mut cef_window_handle_t,

  ///
  // Returns the client for this browser.
  ///
  pub get_client: extern "C" fn(browser_host: *mut cef_browser_host) -> *mut cef_client_t,

  ///
  // Returns the request context for this browser.
  ///
  pub get_request_context: extern "C" fn(browser_host: *mut cef_browser_host) -> *mut cef_request_context_t,

  ///
  // Get the current zoom level. The default zoom level is 0.0. This function
  // can only be called on the UI thread.
  ///
  pub get_zoom_level: extern "C" fn(browser_host: *mut cef_browser_host) -> c_double,

  ///
  // Change the zoom level to the specified value. Specify 0.0 to reset the zoom
  // level. If called on the UI thread the change will be applied immediately.
  // Otherwise, the change will be applied asynchronously on the UI thread.
  ///
  pub set_zoom_level: extern "C" fn(browser_host: *mut cef_browser_host, zoomLevel: c_double),

  ///
  // Call to run a file chooser dialog. Only a single file chooser dialog may be
  // pending at any given time. |mode| represents the type of dialog to display.
  // |title| to the title to be used for the dialog and may be NULL to show the
  // default title ("Open" or "Save" depending on the mode). |default_file_name|
  // is the default file name to select in the dialog. |accept_types| is a list
  // of valid lower-cased MIME types or file extensions specified in an input
  // element and is used to restrict selectable files to such types. |callback|
  // will be executed after the dialog is dismissed or immediately if another
  // dialog is already pending. The dialog will be initiated asynchronously on
  // the UI thread.
  ///
  pub run_file_dialog: extern "C" fn(browser_host: *mut cef_browser_host,
                             mode: cef_file_dialog_mode_t, title: *cef_string_t,
                             default_file_name: *cef_string_t, accept_types: *mut cef_string_list_t,
                             callback: *mut cef_run_file_dialog_callback),

  ///
  // Download the file at |url| using cef_download_handler_t.
  ///
  pub start_download: extern "C" fn(browser_host: *mut cef_browser_host, url: *cef_string_t),

  ///
  // Print the current browser contents.
  ///
  pub print: extern "C" fn(browser_host: *mut cef_browser_host),

  ///
  // Search for |searchText|. |identifier| can be used to have multiple searches
  // running simultaniously. |forward| indicates whether to search forward or
  // backward within the page. |matchCase| indicates whether the search should
  // be case-sensitive. |findNext| indicates whether this is the first request
  // or a follow-up.
  ///
  pub find: extern "C" fn(browser_host: *mut cef_browser_host, identifier: c_int, searchText: *cef_string_t,
                  forward: c_int, matchCase: c_int, findNext: c_int),

  ///
  // Cancel all searches that are currently going on.
  ///
  pub stop_finding: extern "C" fn(browser_host: *mut cef_browser_host, clearSelection: c_int),

  ///
  // Open developer tools in its own window.
  ///
  pub show_dev_tools: extern "C" fn(browser_host: *mut cef_browser_host,
                            windowInfo: *cef_window_info_t,
                            client: *mut cef_client_t,
                            settings: *cef_browser_settings_t),

  ///
  // Explicitly close the developer tools window if one exists for this browser
  // instance.
  ///
  pub close_dev_tools: extern "C" fn(browser_host: *mut cef_browser_host),

  ///
  // Set whether mouse cursor change is disabled.
  ///
  pub set_mouse_cursor_change_disabled: extern "C" fn(browser_host: *mut cef_browser_host,
                                              disabled: c_int),

  ///
  // Returns true (1) if mouse cursor change is disabled.
  ///
  pub is_mouse_cursor_change_disabled: extern "C" fn(browser_host: *mut cef_browser_host) -> c_int,

  ///
  // Returns true (1) if window rendering is disabled.
  ///
  pub is_window_rendering_disabled: extern "C" fn(browser_host: *mut cef_browser_host) -> c_int,

  ///
  // Notify the browser that the widget has been resized. The browser will first
  // call cef_render_handler_t::GetViewRect to get the new size and then call
  // cef_render_handler_t::OnPaint asynchronously with the updated regions. This
  // function is only used when window rendering is disabled.
  ///
  pub was_resized: extern "C" fn(browser_host: *mut cef_browser_host),

  ///
  // Notify the browser that it has been hidden or shown. Layouting and
  // cef_render_handler_t::OnPaint notification will stop when the browser is
  // hidden. This function is only used when window rendering is disabled.
  ///
  pub was_hidden: extern "C" fn(browser_host: *mut cef_browser_host, hidden: c_int),

  ///
  // Send a notification to the browser that the screen info has changed. The
  // browser will then call cef_render_handler_t::GetScreenInfo to update the
  // screen information with the new values. This simulates moving the webview
  // window from one display to another, or changing the properties of the
  // current display. This function is only used when window rendering is
  // disabled.
  ///
  pub notify_screen_info_changed: extern "C" fn(browser_host: *mut cef_browser_host),

  ///
  // Invalidate the |dirtyRect| region of the view. The browser will call
  // cef_render_handler_t::OnPaint asynchronously with the updated regions. This
  // function is only used when window rendering is disabled.
  ///
  pub invalidate: extern "C" fn(browser_host: *mut cef_browser_host,
                        dirtyRect: *cef_rect, t: cef_paint_element_type_t),

  ///
  // Send a key event to the browser.
  ///
  pub send_key_event: extern "C" fn(browser_host: *mut cef_browser_host,
                            event: *cef_key_event),

  ///
  // Send a mouse click event to the browser. The |x| and |y| coordinates are
  // relative to the upper-left corner of the view.
  ///
  pub send_mouse_click_event: extern "C" fn(browser_host: *mut cef_browser_host,
                            event: *cef_mouse_event,
                            t: cef_mouse_button_type_t,
                            mouseUp: c_int, clickCount: c_int),

  ///
  // Send a mouse move event to the browser. The |x| and |y| coordinates are
  // relative to the upper-left corner of the view.
  ///
  pub send_mouse_move_event: extern "C" fn(browser_host: *mut cef_browser_host,
                            event: *cef_mouse_event, mouseLeave: c_int),

  ///
  // Send a mouse wheel event to the browser. The |x| and |y| coordinates are
  // relative to the upper-left corner of the view. The |deltaX| and |deltaY|
  // values represent the movement delta in the X and Y directions respectively.
  // In order to scroll inside select popups with window rendering disabled
  // cef_render_handler_t::GetScreenPoint should be implemented properly.
  ///
  pub send_mouse_wheel_event: extern "C" fn(browser_host: *mut cef_browser_host,
                            event: *cef_mouse_event, deltaX: c_int, deltaY: c_int),

  ///
  // Send a focus event to the browser.
  ///
  pub send_focus_event: extern "C" fn(browser_host: *mut cef_browser_host, setFocus: c_int),

  ///
  // Send a capture lost event to the browser.
  ///
  pub send_capture_lost_event: extern "C" fn(browser_host: *mut cef_browser_host),

  ///
  // Get the NSTextInputContext implementation for enabling IME on Mac when
  // window rendering is disabled.
  ///
  pub get_nstext_input_context: extern "C" fn(browser_host: *mut cef_browser_host) -> cef_text_input_context_t,

  ///
  // Handles a keyDown event prior to passing it through the NSTextInputClient
  // machinery.
  ///
  pub handle_key_event_before_text_input_client: extern "C" fn(browser_host: *mut cef_browser_host,
                                                       key_event: *mut cef_event_handle_t),

  ///
  // Performs any additional actions after NSTextInputClient handles the event.
  ///
  pub handle_key_event_after_text_input_client: extern "C" fn(browser_host: *mut cef_browser_host,
                                                       key_event: *mut cef_event_handle_t),
}


///
// Structure used to represent a browser window. When used in the browser
// process the functions of this structure may be called on any thread unless
// otherwise indicated in the comments. When used in the render process the
// functions of this structure may only be called on the main thread.
///
pub type cef_browser_t = cef_browser;
pub struct cef_browser {
  ///
  // Base structure.
  ///
  pub base: cef_base,

  ///
  // Returns the browser host object. This function can only be called in the
  // browser process.
  ///
  pub get_host: extern "C" fn(browser: *mut cef_browser) -> *mut cef_browser_host,

  ///
  // Returns true (1) if the browser can navigate backwards.
  ///
  pub can_go_back: extern "C" fn(browser: *mut cef_browser) -> c_int,

  ///
  // Navigate backwards.
  ///
  pub go_back: extern "C" fn(browser: *mut cef_browser),

  ///
  // Returns true (1) if the browser can navigate forwards.
  ///
  pub can_go_forward: extern "C" fn(browser: *mut cef_browser) -> c_int,

  ///
  // Navigate forwards.
  ///
  pub go_forward: extern "C" fn(browser: *mut cef_browser),

  ///
  // Returns true (1) if the browser is currently loading.
  ///
  pub is_loading: extern "C" fn(browser: *mut cef_browser) -> c_int,

  ///
  // Reload the current page.
  ///
  pub reload: extern "C" fn(browser: *mut cef_browser),

  ///
  // Reload the current page ignoring any cached data.
  ///
  pub reload_ignore_cache: extern "C" fn(browser: *mut cef_browser),

  ///
  // Stop loading the page.
  ///
  pub stop_load: extern "C" fn(browser: *mut cef_browser),

  ///
  // Returns the globally unique identifier for this browser.
  ///
  pub get_identifier: extern "C" fn(browser: *mut cef_browser) -> c_int,

  ///
  // Returns true (1) if this object is pointing to the same handle as |that|
  // object.
  ///
  pub is_same: extern "C" fn(browser: *mut cef_browser, that: *mut cef_browser) -> c_int,

  ///
  // Returns true (1) if the window is a popup window.
  ///
  pub is_popup: extern "C" fn(browser: *mut cef_browser) -> c_int,

  ///
  // Returns true (1) if a document has been loaded in the browser.
  ///
  pub has_document: extern "C" fn(browser: *mut cef_browser) -> c_int,

  ///
  // Returns the main (top-level) frame for the browser window.
  ///
  pub get_main_frame: extern "C" fn(browser: *mut cef_browser) -> *mut cef_frame,

  ///
  // Returns the focused frame for the browser window.
  ///
  pub get_focused_frame: extern "C" fn(browser: *mut cef_browser) -> *mut cef_frame,

  ///
  // Returns the frame with the specified identifier, or NULL if not found.
  ///
  pub get_frame_byident: extern "C" fn(browser: *mut cef_browser, identifier: c_longlong) -> *mut cef_frame,

  ///
  // Returns the frame with the specified name, or NULL if not found.
  ///
  pub get_frame: extern "C" fn(browser: *mut cef_browser, name: *cef_string_t) -> *mut cef_frame,

  ///
  // Returns the number of frames that currently exist.
  ///
  pub get_frame_count: extern "C" fn(browser: *mut cef_browser) -> size_t,

  ///
  // Returns the identifiers of all existing frames.
  ///
  pub get_frame_identifiers: extern "C" fn(browser: *mut cef_browser,
                             identifiersCount: *mut size_t,
                             identifiers: *mut c_longlong),

  ///
  // Returns the names of all existing frames.
  ///
  pub get_frame_names: extern "C" fn(browser: *mut cef_browser, names: *mut cef_string_list_t),

  //
  // Send a message to the specified |target_process|. Returns true (1) if the
  // message was sent successfully.
  ///
  pub send_process_message: extern "C" fn(browser: *mut cef_browser, target_process: cef_process_id_t,
                             message: *mut cef_process_message) -> c_int,
}

///
// Structure used to implement render process callbacks. The functions of this
// structure will be called on the render process main thread (TID_RENDERER)
// unless otherwise indicated.
///
pub type cef_render_process_handler_t = cef_render_process_handler;
pub struct cef_render_process_handler {
  ///
  // Base structure.
  ///
  pub base: cef_base,

  ///
  // Called after the render process main thread has been created. |extra_info|
  // is a read-only value originating from
  // cef_browser_process_handler_t::on_render_process_thread_created(). Do not
  // keep a reference to |extra_info| outside of this function.
  ///
  pub on_render_thread_created: extern "C" fn(render_handler: *mut cef_render_process_handler, extra_info: *mut cef_list_value),

  ///
  // Called after WebKit has been initialized.
  ///
  pub on_web_kit_initialized: extern "C" fn(render_handler: *mut cef_render_process_handler),

  ///
  // Called after a browser has been created. When browsing cross-origin a new
  // browser will be created before the old browser with the same identifier is
  // destroyed.
  ///
  pub on_browser_created: extern "C" fn(render_handler: *mut cef_render_process_handler, browser: *mut cef_browser),

  ///
  // Called before a browser is destroyed.
  ///
  pub on_browser_destroyed: extern "C" fn(render_handler: *mut cef_render_process_handler, browser: *mut cef_browser),

  ///
  // Return the handler for browser load status events.
  ///
  pub get_load_handler: extern "C" fn(render_handler: *mut cef_render_process_handler) -> *mut cef_load_handler,

  ///
  // Called before browser navigation. Return true (1) to cancel the navigation
  // or false (0) to allow the navigation to proceed. The |request| object
  // cannot be modified in this callback.
  ///
  pub on_before_navigation: extern "C" fn(render_handler: *mut cef_render_process_handler,
                              browser: *mut cef_browser,
                              frame: *mut cef_frame,
                              request: *mut cef_request,
                              navigation_type: *mut cef_navigation_type,
                              is_redirect: c_int) -> c_int,

  ///
  // Called immediately after the V8 context for a frame has been created. To
  // retrieve the JavaScript 'window' object use the
  // cef_v8context_t::get_global() function. V8 handles can only be accessed
  // from the thread on which they are created. A task runner for posting tasks
  // on the associated thread can be retrieved via the
  // cef_v8context_t::get_task_runner() function.
  ///
  pub on_context_created: extern "C" fn(render_handler: *mut cef_render_process_handler,
                                browser: *mut cef_browser,
                                frame: *mut cef_frame,
                                context: *mut cef_v8context),

  ///
  // Called immediately before the V8 context for a frame is released. No
  // references to the context should be kept after this function is called.
  ///
  pub on_context_released: extern "C" fn(render_handler: *mut cef_render_process_handler,
                                 browser: *mut cef_browser,
                                 frame: *mut cef_frame,
                                 context: *mut cef_v8context),

  ///
  // Called for global uncaught exceptions in a frame. Execution of this
  // callback is disabled by default. To enable set
  // CefSettings.uncaught_exception_stack_size  0.
  ///
  pub on_uncaught_exception: extern "C" fn(render_handler: *mut cef_render_process_handler,
                                 browser: *mut cef_browser,
                                 frame: *mut cef_frame,
                                 context: *mut cef_v8context,
                                 exception: *mut cef_v8exception,
                                 stackTrace: *mut cef_v8stack_trace),

  ///
  // Called when a new node in the the browser gets focus. The |node| value may
  // be NULL if no specific node has gained focus. The node object passed to
  // this function represents a snapshot of the DOM at the time this function is
  // executed. DOM objects are only valid for the scope of this function. Do not
  // keep references to or attempt to access any DOM objects outside the scope
  // of this function.
  ///
  pub on_focused_node_changed: extern "C" fn(render_handler: *mut cef_render_process_handler,
                                 browser: *mut cef_browser,
                                 frame: *mut cef_frame,
                                 node: *mut cef_domnode),

  ///
  // Called when a new message is received from a different process. Return true
  // (1) if the message was handled or false (0) otherwise. Do not keep a
  // reference to or attempt to access the message outside of this callback.
  ///
  pub on_process_message_received: extern "C" fn(render_handler: *mut cef_render_process_handler,
                                 browser: *mut cef_browser,
                                 source_process: cef_process_id_t,
                                 message: *mut cef_process_message) ->c_int,
}

///
// Implement this structure to provide handler implementations. Methods will be
// called by the process and/or thread indicated.
///
pub type cef_app_t = cef_app;
pub struct cef_app {
  ///
  // Base structure.
  ///
  pub base: cef_base,

  ///
  // Provides an opportunity to view and/or modify command-line arguments before
  // processing by CEF and Chromium. The |process_type| value will be NULL for
  // the browser process. Do not keep a reference to the cef_command_line_t
  // object passed to this function. The CefSettings.command_line_args_disabled
  // value can be used to start with an NULL command-line object. Any values
  // specified in CefSettings that equate to command-line arguments will be set
  // before this function is called. Be cautious when using this function to
  // modify command-line arguments for non-browser processes as this may result
  // in undefined behavior including crashes.
  ///
  pub on_before_command_line_processing: extern "C" fn(app: *mut cef_app_t, process_type: *cef_string_t, command_line: *mut cef_command_line),

  ///
  // Provides an opportunity to register custom schemes. Do not keep a reference
  // to the |registrar| object. This function is called on the main thread for
  // each process and the registered schemes should be the same across all
  // processes.
  ///
  pub on_register_custom_schemes: extern "C" fn(app: *mut cef_app_t, registrar: *mut cef_scheme_registrar),

  ///
  // Return the handler for resource bundle events. If
  // CefSettings.pack_loading_disabled is true (1) a handler must be returned.
  // If no handler is returned resources will be loaded from pack files. This
  // function is called by the browser and render processes on multiple threads.
  ///
  pub get_resource_bundle_handler: extern "C" fn(app: *mut cef_app_t) -> *mut cef_resource_bundle_handler,

  ///
  // Return the handler for functionality specific to the browser process. This
  // function is called on multiple threads in the browser process.
  ///
  pub get_browser_process_handler: extern "C" fn(app: *mut cef_app_t) -> *mut cef_browser_process_handler,

  ///
  // Return the handler for functionality specific to the render process. This
  // function is called on the render process main thread.
  ///
  pub get_render_process_handler: extern "C" fn(app: *mut cef_app_t) -> *mut cef_render_process_handler,
}


///
// Structure used to make a URL request. URL requests are not associated with a
// browser instance so no cef_client_t callbacks will be executed. URL requests
// can be created on any valid CEF thread in either the browser or render
// process. Once created the functions of the URL request object must be
// accessed on the same thread that created it.
///
pub type cef_urlrequest_t = cef_urlrequest;
pub struct cef_urlrequest {
  ///
  // Base structure.
  ///
  pub base: cef_base,

  ///
  // Returns the request object used to create this URL request. The returned
  // object is read-only and should not be modified.
  ///
  pub get_request: extern "C" fn(url_req: *mut cef_urlrequest) -> *mut cef_request_t,

  ///
  // Returns the client.
  ///
  pub get_client: extern "C" fn(url_req: *mut cef_urlrequest) -> *mut cef_urlrequest_client_t,

  ///
  // Returns the request status.
  ///
  pub get_request_status: extern "C" fn(url_req: *mut cef_urlrequest) -> cef_urlrequest_status_t,

  ///
  // Returns the request error if status is UR_CANCELED or UR_FAILED, or 0
  // otherwise.
  ///
  pub get_request_error: extern "C" fn(url_req: *mut cef_urlrequest) -> cef_errorcode_t,

  ///
  // Returns the response, or NULL if no response information is available.
  // Response information will only be available after the upload has completed.
  // The returned object is read-only and should not be modified.
  ///
  pub get_response: extern "C" fn(url_req: *mut cef_urlrequest) -> *mut cef_response_t,

  ///
  // Cancel the request.
  ///
  pub cancel: extern "C" fn(url_req: *mut cef_urlrequest),
}



///
// Structure used to represent a single element in the request post data. The
// functions of this structure may be called on any thread.
///
pub type cef_post_data_element_t = cef_post_data_element;
pub struct cef_post_data_element {
  ///
  // Base structure.
  ///
  pub base: cef_base,

  ///
  // Returns true (1) if this object is read-only.
  ///
  pub is_read_only: extern "C" fn(post_data_element: *mut cef_post_data_element) -> c_int,

  ///
  // Remove all contents from the post data element.
  ///
  pub set_to_empty: extern "C" fn(post_data_element: *mut cef_post_data_element),

  ///
  // The post data element will represent a file.
  ///
  pub set_to_file: extern "C" fn(post_data_element: *mut cef_post_data_element, fileName: *cef_string_t),

  ///
  // The post data element will represent bytes.  The bytes passed in will be
  // copied.
  ///
  pub set_to_bytes: extern "C" fn(post_data_element: *mut cef_post_data_element,
                          size: size_t, bytes: *c_void),

  ///
  // Return the type of this post data element.
  ///
  pub get_type: extern "C" fn(post_data_element: *mut cef_post_data_element) -> cef_postdataelement_type_t,

  ///
  // Return the file name.
  ///
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_file: extern "C" fn(post_data_element: *mut cef_post_data_element) -> *mut cef_string_userfree_t,

  ///
  // Return the number of bytes.
  ///
  pub get_bytes_count: extern "C" fn(post_data_element: *mut cef_post_data_element) -> size_t,

  ///
  // Read up to |size| bytes into |bytes| and return the number of bytes
  // actually read.
  ///
  pub get_bytes: extern "C" fn(post_data_element: *mut cef_post_data_element,
                             size: size_t, bytes: *mut c_void) -> size_t,
}


///
// Structure used to represent post data for a web request. The functions of
// this structure may be called on any thread.
///
pub type cef_post_data_t = cef_post_data;
pub struct cef_post_data {
  ///
  // Base structure.
  ///
  pub base: cef_base_t,

  ///
  // Returns true (1) if this object is read-only.
  ///
  pub is_read_only: extern "C" fn(post_data: *mut cef_post_data) -> c_int,

  ///
  // Returns the number of existing post data elements.
  ///
  pub get_element_count: extern "C" fn(post_data: *mut cef_post_data) -> size_t,

  ///
  // Retrieve the post data elements.
  ///
  pub get_elements: extern "C" fn(post_data: *mut cef_post_data,
                          elements_count: *mut size_t, elements: **mut cef_post_data_element),

  ///
  // Remove the specified post data element.  Returns true (1) if the removal
  // succeeds.
  ///
  pub remove_element: extern "C" fn(post_data: *mut cef_post_data,
                            element: *mut cef_post_data_element) -> c_int,

  ///
  // Add the specified post data element.  Returns true (1) if the add succeeds.
  ///
  pub add_element: extern "C" fn(post_data: *mut cef_post_data,
                            element: *mut cef_post_data_element) -> c_int,

  ///
  // Remove all existing post data elements.
  ///
  pub remove_elements: extern "C" fn(post_data: *mut cef_post_data),
}
