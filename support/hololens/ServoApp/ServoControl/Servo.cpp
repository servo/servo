#include "pch.h"
#include "Servo.h"

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

void flush() { sServo->Delegate().Flush(); }

void make_current() { sServo->Delegate().MakeCurrent(); }

void wakeup() { sServo->Delegate().WakeUp(); }

bool on_allow_navigation(const char *url) {
  return sServo->Delegate().OnServoAllowNavigation(char2hstring(url));
};

void on_animating_changed(bool aAnimating) {
  sServo->Delegate().OnServoAnimatingChanged(aAnimating);
}

void on_panic(const char *backtrace) {
  throw hresult_error(E_FAIL, char2hstring(backtrace));
}

void on_ime_state_changed(bool aShow) {
  sServo->Delegate().OnServoIMEStateChanged(aShow);
}

void set_clipboard_contents(const char *content) {
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
  return sServo->Delegate().OnServoMediaSessionPlaybackStateChange(state);
}

void prompt_alert(const char *message, bool trusted) {
  sServo->Delegate().OnServoPromptAlert(char2hstring(message), trusted);
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
             float dpi, ServoDelegate &aDelegate)
    : mWindowHeight(height), mWindowWidth(width), mDelegate(aDelegate) {

  capi::CInitOptions o;
  hstring defaultPrefs = L" --pref dom.webxr.enabled  --devtools=6000";
  o.args = *hstring2char(args + defaultPrefs);
  o.url = *hstring2char(url);
  o.width = mWindowWidth;
  o.height = mWindowHeight;
  o.density = dpi;
  o.enable_subpixel_text_antialiasing = false;
  o.vr_pointer = NULL;

  // 7 filter modules.
  /* Sample list of servo modules to filter.
  static char *pfilters[] = {
          "servo",
          "simpleservo",
          "simpleservo::jniapi",
          "simpleservo::gl_glue::egl",
          // Show JS errors by default.
          "script::dom::bindings::error",
          // Show GL errors by default.
          "canvas::webgl_thread",
          "compositing::compositor",
          "constellation::constellation",
  };
  */

  // Example Call when *pfilters[] is used:
  // o.vslogger_mod_list = pfilters; // servo log modules
  // o.vslogger_mod_size = sizeof(pfilters) / sizeof(pfilters[0]); //
  // Important: Number of modules in pfilters
  o.vslogger_mod_list = NULL;
  o.vslogger_mod_size = 0;

  sServo = this; // FIXME;

  capi::CHostCallbacks c;
  c.flush = &flush;
  c.make_current = &make_current;
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

  capi::register_panic_handler(&on_panic);

  capi::init_with_egl(o, &wakeup, c);
}

Servo::~Servo() { sServo = nullptr; }

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
