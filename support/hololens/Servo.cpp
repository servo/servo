#include "pch.h"
#include "Servo.h"

extern "C" {
#include <simpleservo.h>
}

using namespace hlservo;

void on_load_started() {}
void on_load_ended() {}
void on_title_changed(const char* title) {}
void on_url_changed(const char* url) {}
void on_history_changed(bool back, bool fwd) {}
void on_shutdown_complete() {}

std::function<void()> Servo::sFlush = [](){};
std::function<void()> Servo::sMakeCurrent = [](){};
std::function<void()> Servo::sWakeUp = [](){};
bool Servo::sAnimating = false;

void flush() {
  Servo::sFlush();
}

void make_current() {
  Servo::sMakeCurrent();
}

void wakeup() {
  Servo::sWakeUp();
}

void on_animating_changed(bool aAnimating) {
  Servo::sAnimating = aAnimating;
}

Servo::Servo(GLsizei width, GLsizei height)
  : mAnimating(false)
  , mWindowHeight(height)
  , mWindowWidth(width) {

  CInitOptions o;
  o.args = NULL;
  o.url = "http://example.com";
  o.width = mWindowWidth;
  o.height = mWindowHeight;
  o.density = 1.0;
  o.enable_subpixel_text_antialiasing = false;

  CHostCallbacks c;
  c.flush = &flush;
  c.make_current = &make_current;
  c.on_load_started = &on_load_started;
  c.on_load_ended = &on_load_ended;
  c.on_title_changed = &on_title_changed;
  c.on_url_changed = &on_url_changed;
  c.on_history_changed = &on_history_changed;
  c.on_animating_changed = &on_animating_changed;
  c.on_shutdown_complete = &on_shutdown_complete;

  init_with_egl(o, &wakeup, c);
}

Servo::~Servo() {
  deinit();
}

void Servo::PerformUpdates() {
  perform_updates();
}

void Servo::SetSize(GLsizei width, GLsizei height) {
  if (width != mWindowWidth || height != mWindowHeight) {
    mWindowWidth = width;
    mWindowHeight = height;
    resize(mWindowWidth, mWindowHeight);
  }
}
