/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include "pch.h"
#include "Servo.h"
#include "Crash.h"
#include <EGL/egl.h>
#include "../DefaultUrl.h"

namespace winrt::servo {

using namespace Windows::Storage;

void on_load_started() { sServo->Delegate().OnServoLoadStarted(); }

void on_load_ended() { sServo->Delegate().OnServoLoadEnded(); }

void on_history_changed(bool back, bool forward) {
  sServo->Delegate().OnServoHistoryChanged(back, forward);
}

void on_shutdown_complete() { sServo->Delegate().OnServoShutdownComplete(); }

void on_title_changed(const char *title) {
  sServo->Delegate().OnServoTitleChanged(char2hstring(title));
}

void on_url_changed(const char *curl) {
  auto url = char2hstring(curl);
  sServo->CurrentUrl(url);
  sServo->Delegate().OnServoURLChanged(url);
}

void wakeup() {
  if (sServo != nullptr)
    sServo->Delegate().WakeUp();
}

bool on_allow_navigation(const char *url) {
  return sServo->Delegate().OnServoAllowNavigation(char2hstring(url));
};

void on_animating_changed(bool aAnimating) {
  sServo->Delegate().OnServoAnimatingChanged(aAnimating);
}

void on_panic(const char *cbacktrace) {

  if (sLogHandle != INVALID_HANDLE_VALUE) {
    CloseHandle(sLogHandle);
    sLogHandle = INVALID_HANDLE_VALUE;
  }

  auto backtrace = char2hstring(cbacktrace);

  try {
    WriteCrashReport(backtrace, sServo->CurrentUrl());
  } catch (...) {
    log(L"Failed to log panic to crash report");
  }

  // If this is happening in the GL thread, the app can continue running.
  // So let's show the crash report:
  sServo->Delegate().OnServoPanic(backtrace);
  throw hresult_error(E_FAIL, backtrace);
}

void on_ime_show(const char *text, int32_t text_index, bool multiline,
                 int32_t x, int32_t y, int32_t width, int32_t height) {
  hstring htext = L"";
  if (text != nullptr) {
    htext = char2hstring(text);
  }
  sServo->Delegate().OnServoIMEShow(htext, x, y, width, height);
}

void on_ime_hide() { sServo->Delegate().OnServoIMEHide(); }

void set_clipboard_contents(const char *) {
  // FIXME
}

const char *get_clipboard_contents() {
  // FIXME
  return nullptr;
}

void on_media_session_set_position_state(double duration, double position,
                                         double playback_rate) {
  return sServo->Delegate().OnServoMediaSessionPosition(duration, position,
                                                        playback_rate);
}

void on_media_session_metadata(const char *title, const char *album,
                               const char *artist) {
  return sServo->Delegate().OnServoMediaSessionMetadata(
      char2hstring(title), char2hstring(album), char2hstring(artist));
}

void on_media_session_playback_state_change(
    const capi::CMediaSessionPlaybackState state) {
  return sServo->Delegate().OnServoMediaSessionPlaybackStateChange(state);
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
                         const unsigned int port, const char *token) {
  auto state = result == Servo::DevtoolsServerState::Started;
  sServo->Delegate().OnServoDevtoolsStarted(state, port, char2hstring(token));
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

const char *prompt_input(const char *message, const char *def, bool trusted) {
  auto input = sServo->Delegate().OnServoPromptInput(
      char2hstring(message), char2hstring(def), trusted);
  if (input.has_value()) {
    return *hstring2char(*input);
  } else {
    return nullptr;
  }
}

Servo::Servo(std::optional<hstring> initUrl, hstring args, GLsizei width,
             GLsizei height, EGLNativeWindowType eglNativeWindow, float dpi,
             ServoDelegate &aDelegate, bool transient)
    : mWindowHeight(height), mWindowWidth(width), mDelegate(aDelegate) {
  ApplicationDataContainer localSettings =
      ApplicationData::Current().LocalSettings();
  if (!localSettings.Containers().HasKey(L"servoUserPrefs")) {
    ApplicationDataContainer container = localSettings.CreateContainer(
        L"servoUserPrefs", ApplicationDataCreateDisposition::Always);
  }

  auto prefs = localSettings.Containers().Lookup(L"servoUserPrefs");

  std::vector<capi::CPref> cprefs;

  // Ensure few things stay in memories long enough as we send raw
  // pointers to Rust.
  std::vector<std::unique_ptr<char *>> memChar;
  std::vector<std::unique_ptr<bool>> memBool;
  std::vector<std::unique_ptr<int64_t>> memInt;
  std::vector<std::unique_ptr<double>> memDouble;

  for (auto pref : prefs.Values()) {

    auto charkey = hstring2char(pref.Key());
    auto key = *charkey.get();
    memChar.push_back(std::move(charkey));
    auto value = pref.Value();

    auto type = value.as<Windows::Foundation::IPropertyValue>().Type();
    capi::CPref cpref;
    cpref.key = key;
    cpref.pref_type = capi::CPrefType::Missing;
    cpref.value = NULL;
    if (type == Windows::Foundation::PropertyType::Boolean) {
      cpref.pref_type = capi::CPrefType::Bool;
      auto val = std::make_unique<bool>(unbox_value<bool>(value));
      cpref.value = val.get();
      memBool.push_back(std::move(val));
    } else if (type == Windows::Foundation::PropertyType::String) {
      cpref.pref_type = capi::CPrefType::Str;
      auto val = hstring2char(unbox_value<hstring>(value));
      cpref.value = *val.get();
      memChar.push_back(std::move(val));
    } else if (type == Windows::Foundation::PropertyType::Int64) {
      cpref.pref_type = capi::CPrefType::Int;
      auto val = std::make_unique<int64_t>(unbox_value<int64_t>(value));
      cpref.value = val.get();
      memInt.push_back(std::move(val));
    } else if (type == Windows::Foundation::PropertyType::Double) {
      cpref.pref_type = capi::CPrefType::Float;
      auto val = std::make_unique<double>(unbox_value<double>(value));
      cpref.value = val.get();
      memDouble.push_back(std::move(val));
    } else if (type == Windows::Foundation::PropertyType::Empty) {
      cpref.pref_type = capi::CPrefType::Missing;
    } else {
      log(L"skipping pref %s. Unknown type", key);
      continue;
    }
    cprefs.push_back(cpref);
  }

  if (initUrl.has_value()) {
    setNonPersistentHomepage(*initUrl, cprefs);
  } else {
#ifdef OVERRIDE_DEFAULT_URL
    setNonPersistentHomepage(OVERRIDE_DEFAULT_URL, cprefs);
#endif
  }

  if (transient) {
    capi::CPref cpref;
    cpref.key = "dom.webxr.sessionavailable";
    cpref.pref_type = capi::CPrefType::Bool;
    cpref.value = &transient;
    cprefs.push_back(cpref);
  }

  capi::CPrefList prefsList = {cprefs.size(), cprefs.data()};

  capi::CInitOptions o;
  o.prefs = &prefsList;
  o.args = *hstring2char(args);
  o.width = mWindowWidth;
  o.height = mWindowHeight;
  o.density = dpi;
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

  auto current = ApplicationData::Current();
  auto gstLog = std::wstring(current.LocalFolder().Path()) + L"\\gst.log";
  SetEnvironmentVariable(L"GST_DEBUG_FILE", gstLog.c_str());
  // SetEnvironmentVariableA("GST_DEBUG", "4");

  auto filePath = std::wstring(current.LocalFolder().Path()) + L"\\stdout.txt";
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

  capi::CHostCallbacks c = capi::CHostCallbacks{
      .on_load_started = &on_load_started,
      .on_load_ended = &on_load_ended,
      .on_title_changed = &on_title_changed,
      .on_allow_navigation = &on_allow_navigation,
      .on_url_changed = &on_url_changed,
      .on_history_changed = &on_history_changed,
      .on_animating_changed = &on_animating_changed,
      .on_shutdown_complete = &on_shutdown_complete,
      .on_ime_show = &on_ime_show,
      .on_ime_hide = &on_ime_hide,
      .get_clipboard_contents = &get_clipboard_contents,
      .set_clipboard_contents = &set_clipboard_contents,
      .on_media_session_metadata = &on_media_session_metadata,
      .on_media_session_playback_state_change =
          &on_media_session_playback_state_change,
      .on_media_session_set_position_state =
          &on_media_session_set_position_state,
      .prompt_alert = &prompt_alert,
      .prompt_ok_cancel = &prompt_ok_cancel,
      .prompt_yes_no = &prompt_yes_no,
      .prompt_input = &prompt_input,
      .on_devtools_started = &on_devtools_started,
      .show_context_menu = &show_context_menu,
      .on_log_output = &on_log_output,
  };

  capi::register_panic_handler(&on_panic);

  capi::init_with_egl(o, &wakeup, c);
}

Servo::~Servo() {
  sServo = nullptr;
  if (sLogHandle != INVALID_HANDLE_VALUE)
    CloseHandle(sLogHandle);
}

Servo::PrefTuple Servo::SetFloatPref(hstring key, double val) {
  auto ckey = *hstring2char(key);
  capi::set_float_pref(ckey, val);
  auto updatedPref = WrapPref(capi::get_pref(ckey));
  SaveUserPref(updatedPref);
  return updatedPref;
}

Servo::PrefTuple Servo::SetIntPref(hstring key, int64_t val) {
  auto ckey = *hstring2char(key);
  capi::set_int_pref(ckey, val);
  auto updatedPref = WrapPref(capi::get_pref(ckey));
  SaveUserPref(updatedPref);
  return updatedPref;
}

Servo::PrefTuple Servo::SetBoolPref(hstring key, bool val) {
  auto ckey = *hstring2char(key);
  capi::set_bool_pref(ckey, val);
  auto updatedPref = WrapPref(capi::get_pref(ckey));
  SaveUserPref(updatedPref);
  return updatedPref;
}

Servo::PrefTuple Servo::SetStringPref(hstring key, hstring val) {
  auto ckey = *hstring2char(key);
  auto cval = *hstring2char(val);
  capi::set_str_pref(ckey, cval);
  auto updatedPref = WrapPref(capi::get_pref(ckey));
  SaveUserPref(updatedPref);
  return updatedPref;
}

Servo::PrefTuple Servo::ResetPref(hstring key) {
  auto ckey = *hstring2char(key);
  capi::reset_pref(ckey);
  auto updatedPref = WrapPref(capi::get_pref(ckey));
  SaveUserPref(updatedPref);
  return updatedPref;
}

void Servo::GoHome() {
  ApplicationDataContainer localSettings =
      ApplicationData::Current().LocalSettings();
  auto prefs = localSettings.Containers().Lookup(L"servoUserPrefs");
  auto home_pref = prefs.Values().Lookup(L"shell.homepage");
  auto home = home_pref ? unbox_value<hstring>(home_pref) :
#ifdef OVERRIDE_DEFAULT_URL
                        OVERRIDE_DEFAULT_URL
#else
                        FALLBACK_DEFAULT_URL
#endif
      ;
  LoadUri(home);
}

void Servo::SaveUserPref(PrefTuple pref) {
  auto localSettings = ApplicationData::Current().LocalSettings();
  auto values = localSettings.Containers().Lookup(L"servoUserPrefs").Values();
  auto [key, val, isDefault] = pref;
  if (isDefault) {
    values.Remove(key);
  } else {
    values.Insert(key, val);
  }
}

Servo::PrefTuple Servo::WrapPref(capi::CPref pref) {
  winrt::Windows::Foundation::IInspectable val;
  if (pref.pref_type == capi::CPrefType::Bool) {
    val = box_value(*(capi::get_pref_as_bool(pref.value)));
  } else if (pref.pref_type == capi::CPrefType::Int) {
    val = box_value(*(capi::get_pref_as_int(pref.value)));
  } else if (pref.pref_type == capi::CPrefType::Float) {
    val = box_value(*(capi::get_pref_as_float(pref.value)));
  } else if (pref.pref_type == capi::CPrefType::Str) {
    val = box_value(char2hstring(capi::get_pref_as_str(pref.value)));
  }
  auto key = char2hstring(pref.key);
  auto isDefault = pref.is_default;
  Servo::PrefTuple t{key, val, isDefault};
  return t;
}

Servo::PrefTuple Servo::GetPref(hstring key) {
  auto ckey = *hstring2char(key);
  return WrapPref(capi::get_pref(ckey));
}

std::vector<Servo::PrefTuple> Servo::GetPrefs() {
  if (sServo == nullptr) {
    return {};
  }
  auto prefs = capi::get_prefs();
  std::vector<PrefTuple> vec;
  for (auto i = 0; i < prefs.len; i++) {
    auto pref = WrapPref(prefs.list[i]);
    vec.push_back(pref);
  }
  return vec;
}

void setNonPersistentHomepage(hstring url, std::vector<capi::CPref> &cprefs) {
  for (auto cpref : cprefs) {
    if (strcmp(cpref.key, "shell.homepage") == 0) {
      cpref.value = *hstring2char(url);
      return;
    }
  }
  capi::CPref cpref;
  cpref.key = "shell.homepage";
  cpref.pref_type = capi::CPrefType::Str;
  cpref.value = *hstring2char(url);
  cprefs.push_back(cpref);
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
