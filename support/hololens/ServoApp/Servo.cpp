#include "pch.h"
#include "Servo.h"

// FIXME: rename mozilla to something else
namespace servo {

void on_load_started() { sServo->Delegate().OnLoadStarted(); }
void on_load_ended() { sServo->Delegate().OnLoadEnded(); }
void on_history_changed(bool back, bool forward) {
  sServo->Delegate().OnHistoryChanged(back, forward);
}
void on_shutdown_complete() { sServo->Delegate().OnShutdownComplete(); }
void on_alert(const char *message) {
  sServo->Delegate().OnAlert(char2w(message));
}
void on_title_changed(const char *title) {
  sServo->Delegate().OnTitleChanged(char2w(title));
}
void on_url_changed(const char *url) {
  sServo->Delegate().OnURLChanged(char2w(url));
}
void flush() { sServo->Delegate().Flush(); }
void make_current() { sServo->Delegate().MakeCurrent(); }
void wakeup() { sServo->Delegate().WakeUp(); }
bool on_allow_navigation(const char *url) {
 return sServo->Delegate().OnAllowNavigation(char2w(url));
};
void on_animating_changed(bool aAnimating) {
  sServo->Delegate().OnAnimatingChanged(aAnimating);
}

Servo::Servo(GLsizei width, GLsizei height, ServoDelegate &aDelegate)
    : mWindowHeight(height), mWindowWidth(width), mDelegate(aDelegate) {

  capi::CInitOptions o;
  o.args = NULL;
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

  init_with_egl(o, &wakeup, c);
}

Servo::~Servo() { sServo = nullptr; }

std::wstring char2w(const char *c_str) {
  auto str = std::string(c_str);
  int size_needed =
      MultiByteToWideChar(CP_UTF8, 0, &str[0], (int)str.size(), NULL, 0);
  std::wstring str2(size_needed, 0);
  MultiByteToWideChar(CP_UTF8, 0, &str[0], (int)str.size(), &str2[0],
                      size_needed);
  return str2;
}

} // namespace servo
