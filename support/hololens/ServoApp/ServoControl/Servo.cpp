#include "pch.h"
#include "Servo.h"

namespace winrt::servo {

void on_load_started() { sServo->Delegate().OnServoLoadStarted(); }

void on_load_ended() { sServo->Delegate().OnServoLoadEnded(); }

void on_history_changed(bool back, bool forward) {
  sServo->Delegate().OnServoHistoryChanged(back, forward);
}

void on_shutdown_complete() { sServo->Delegate().OnServoShutdownComplete(); }

void on_alert(const char *message) {
  sServo->Delegate().OnServoAlert(char2hstring(message));
}

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

void set_clipboard_contents(const char* content) {
  // FIXME
}

const char* get_clipboard_contents() {
  // FIXME
  return nullptr;
}

Servo::Servo(GLsizei width, GLsizei height, ServoDelegate &aDelegate)
    : mWindowHeight(height), mWindowWidth(width), mDelegate(aDelegate) {

  capi::CInitOptions o;
  o.args = "--pref dom.webxr.enabled";
  o.url = "https://servo.org";
  o.width = mWindowWidth;
  o.height = mWindowHeight;
  o.density = 1.0;
  o.enable_subpixel_text_antialiasing = false;
  o.vr_pointer = NULL;

  sServo = this; // FIXME;

  capi::CHostCallbacks c;
  c.flush = &flush;
  c.make_current = &make_current;
  c.on_alert = &on_alert;
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

} // namespace winrt::servo
