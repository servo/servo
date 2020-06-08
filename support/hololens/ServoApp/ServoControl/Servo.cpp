#include "pch.h"
#include "Servo.h"
#include <EGL/egl.h>

namespace winrt::servo {

void on_load_started() { sServo->Delegate().OnServoLoadStarted(); }

void on_load_ended() { sServo->Delegate().OnServoLoadEnded(); }

void on_history_changed(bool back, bool forward) {
  sServo->Delegate().OnServoHistoryChanged(back, forward);
}

void on_shutdown_complete() { sServo->Delegate().OnServoShutdownComplete(); }

void on_title_changed(const char *title) {
  sServo->Delegate().OnServoTitleChanged(char2hstring(title));
}

void on_url_changed(const char *url) {
  sServo->Delegate().OnServoURLChanged(char2hstring(url));
}

void wakeup() { sServo->Delegate().WakeUp(); }

bool on_allow_navigation(const char *url) {
  return sServo->Delegate().OnServoAllowNavigation(char2hstring(url));
};

void on_animating_changed(bool aAnimating) {
  sServo->Delegate().OnServoAnimatingChanged(aAnimating);
}

void on_panic(const char *backtrace) {
  if (sLogHandle != INVALID_HANDLE_VALUE) {
    CloseHandle(sLogHandle);
    sLogHandle = INVALID_HANDLE_VALUE;
  }
  throw hresult_error(E_FAIL, char2hstring(backtrace));
}

void on_ime_state_changed(bool aShow) {
  sServo->Delegate().OnServoIMEStateChanged(aShow);
}

void set_clipboard_contents(const char *) {
  // FIXME
}

const char *get_clipboard_contents() {
  // FIXME
  return nullptr;
}

void on_media_session_metadata(const char *title, const char *album,
                               const char *artist) {
  return sServo->Delegate().OnServoMediaSessionMetadata(
      char2hstring(title), char2hstring(album), char2hstring(artist));
}

void on_media_session_playback_state_change(
    const capi::CMediaSessionPlaybackState state) {
  return sServo->Delegate().OnServoMediaSessionPlaybackStateChange(static_cast<int>(state));
}

void prompt_alert(const char *message, bool trusted) {
  sServo->Delegate().OnServoPromptAlert(char2hstring(message), trusted);
}

void show_context_menu(const char *title, const char *const *items_list,
                       uint32_t items_size) {
  std::optional<hstring> opt_title = {};
  if (title != nullptr) {
    opt_title = char2hstring(title);
  }
  std::vector<winrt::hstring> items;
  for (uint32_t i = 0; i < items_size; i++) {
    items.push_back(char2hstring(items_list[i]));
  }
  sServo->Delegate().OnServoShowContextMenu(opt_title, items);
}

void on_devtools_started(Servo::DevtoolsServerState result,
                         const unsigned int port) {
  sServo->Delegate().OnServoDevtoolsStarted(
      result == Servo::DevtoolsServerState::Started, port);
}

void on_log_output(const char *buffer, uint32_t buffer_length) {
  OutputDebugStringA(buffer);

  if (sLogHandle == INVALID_HANDLE_VALUE) {
    return;
  }

  DWORD bytesWritten;
  auto writeResult =
      WriteFile(sLogHandle, buffer, buffer_length, &bytesWritten, nullptr);

  if (writeResult == FALSE || bytesWritten != buffer_length)
    throw std::runtime_error(
        "Failed to write log message to the log file: error code " +
        std::to_string(GetLastError()));
}

Servo::PromptResult prompt_ok_cancel(const char *message, bool trusted) {
  return sServo->Delegate().OnServoPromptOkCancel(char2hstring(message),
                                                  trusted);
}

Servo::PromptResult prompt_yes_no(const char *message, bool trusted) {
  return sServo->Delegate().OnServoPromptYesNo(char2hstring(message), trusted);
}

const char *prompt_input(const char *message, const char *default,
                         bool trusted) {
  auto input = sServo->Delegate().OnServoPromptInput(
      char2hstring(message), char2hstring(default), trusted);
  if (input.has_value()) {
    return *hstring2char(*input);
  } else {
    return nullptr;
  }
}

Servo::Servo(hstring url, hstring args, GLsizei width, GLsizei height,
             EGLNativeWindowType eglNativeWindow, float dpi,
             ServoDelegate &aDelegate)
    : mWindowHeight(height), mWindowWidth(width), mDelegate(aDelegate) {
  SetEnvironmentVariableA("PreviewRuntimeEnabled", "1");

  capi::CInitOptions o;
  hstring defaultPrefs = L" --pref dom.webxr.enabled --devtools";
  o.args = *hstring2char(args + defaultPrefs);
  o.url = *hstring2char(url);
  o.width = mWindowWidth;
  o.height = mWindowHeight;
  o.density = dpi;
  o.enable_subpixel_text_antialiasing = false;
  o.native_widget = eglNativeWindow;

  // Note about logs:
  // By default: all modules are enabled. Only warn level-logs are displayed.
  // To change the log level, add "--vslogger-level debug" to o.args.
  // To only print logs from specific modules, add their names to pfilters.
  // For example:
  // static char *pfilters[] = {
  //   "servo",
  //   "simpleservo",
  //   "script::dom::bindings::error", // Show JS errors by default.
  //   "canvas::webgl_thread", // Show GL errors by default.
  //   "compositing",
  //   "constellation",
  // };
  // o.vslogger_mod_list = pfilters;
  // o.vslogger_mod_size = sizeof(pfilters) / sizeof(pfilters[0]);

  o.vslogger_mod_list = NULL;
  o.vslogger_mod_size = 0;

  sServo = this; // FIXME;

#ifndef _DEBUG
  char buffer[1024];
  bool logToFile = GetEnvironmentVariableA("FirefoxRealityLogStdout", buffer,
                                           sizeof(buffer)) != 0;
#else
  bool logToFile = true;
#endif
  if (logToFile) {
    auto current = winrt::Windows::Storage::ApplicationData::Current();
    auto filePath =
        std::wstring(current.LocalFolder().Path()) + L"\\stdout.txt";
    sLogHandle =
        CreateFile2(filePath.c_str(), GENERIC_WRITE, 0, CREATE_ALWAYS, nullptr);
    if (sLogHandle == INVALID_HANDLE_VALUE) {
      throw std::runtime_error("Failed to open the log file: error code " +
                               std::to_string(GetLastError()));
    }

    if (SetFilePointer(sLogHandle, 0, nullptr, FILE_END) ==
        INVALID_SET_FILE_POINTER) {
      throw std::runtime_error(
          "Failed to set file pointer to the end of file: error code " +
          std::to_string(GetLastError()));
    }
  }

  capi::CHostCallbacks c;
  c.on_load_started = &on_load_started;
  c.on_load_ended = &on_load_ended;
  c.on_title_changed = &on_title_changed;
  c.on_url_changed = &on_url_changed;
  c.on_history_changed = &on_history_changed;
  c.on_animating_changed = &on_animating_changed;
  c.on_shutdown_complete = &on_shutdown_complete;
  c.on_allow_navigation = &on_allow_navigation;
  c.on_ime_state_changed = &on_ime_state_changed;
  c.get_clipboard_contents = &get_clipboard_contents;
  c.set_clipboard_contents = &set_clipboard_contents;
  c.on_media_session_metadata = &on_media_session_metadata;
  c.on_media_session_playback_state_change =
      &on_media_session_playback_state_change;
  c.prompt_alert = &prompt_alert;
  c.prompt_ok_cancel = &prompt_ok_cancel;
  c.prompt_yes_no = &prompt_yes_no;
  c.prompt_input = &prompt_input;
  c.on_devtools_started = &on_devtools_started;
  c.show_context_menu = &show_context_menu;
  c.on_log_output = &on_log_output;

  capi::register_panic_handler(&on_panic);

  capi::init_with_egl(o, &wakeup, c);
}

Servo::~Servo() {
  sServo = nullptr;
  if (sLogHandle != INVALID_HANDLE_VALUE)
    CloseHandle(sLogHandle);
}

winrt::hstring char2hstring(const char *c_str) {
  // FIXME: any better way of doing this?
  auto str = std::string(c_str);
  int size_needed =
      MultiByteToWideChar(CP_UTF8, 0, &str[0], (int)str.size(), NULL, 0);
  std::wstring str2(size_needed, 0);
  MultiByteToWideChar(CP_UTF8, 0, &str[0], (int)str.size(), &str2[0],
                      size_needed);
  winrt::hstring str3{str2};
  return str3;
}

std::unique_ptr<char *> hstring2char(hstring hstr) {
  const wchar_t *wc = hstr.c_str();
  size_t size = hstr.size() + 1;
  char *str = new char[size];
  size_t converted = 0;
  wcstombs_s(&converted, str, size, wc, hstr.size());
  return std::make_unique<char *>(str);
}

} // namespace winrt::servo
