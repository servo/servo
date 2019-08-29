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

  // 7 filter modules.
  static char *pfilters[64] = {
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

  o.vslogger_mod_list = pfilters;	// servo log modules
  o.vslogger_mod_size = 7;			// Important: Number of modules in pfilters

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

winrt::hstring char2hstring(const char *c_str) {
  // FIXME: any better way of doing this?
  auto str = std::string(c_str);
  int size_needed = MultiByteToWideChar(CP_UTF8, 0, &str[0], (int)str.size(), NULL, 0);
  std::wstring str2(size_needed, 0);
  MultiByteToWideChar(CP_UTF8, 0, &str[0], (int)str.size(), &str2[0], size_needed);
  winrt::hstring str3 {str2};
  return str3;
}

} // namespace servo
