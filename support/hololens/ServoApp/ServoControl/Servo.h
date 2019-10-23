/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#pragma once

#include "pch.h"
#include "logs.h"
#include <stdlib.h>

namespace winrt::servo {

namespace capi {
extern "C" {
#include <simpleservo.h>
}
} // namespace capi

hstring char2hstring(const char *);
std::unique_ptr<char *> hstring2char(hstring);

class ServoDelegate {
public:
  // Called from any thread
  virtual void WakeUp() = 0;
  // Called from GL thread
  virtual void OnServoLoadStarted() = 0;
  virtual void OnServoLoadEnded() = 0;
  virtual void OnServoHistoryChanged(bool, bool) = 0;
  virtual void OnServoShutdownComplete() = 0;
  virtual void OnServoTitleChanged(hstring) = 0;
  virtual void OnServoAlert(hstring) = 0;
  virtual void OnServoURLChanged(hstring) = 0;
  virtual bool OnServoAllowNavigation(hstring) = 0;
  virtual void OnServoAnimatingChanged(bool) = 0;
  virtual void OnServoIMEStateChanged(bool) = 0;
  virtual void Flush() = 0;
  virtual void MakeCurrent() = 0;

protected:
  virtual ~ServoDelegate(){};
};

class Servo {
public:
  Servo(hstring, hstring, GLsizei, GLsizei, float, ServoDelegate &);
  ~Servo();
  ServoDelegate &Delegate() { return mDelegate; }

  void PerformUpdates() { capi::perform_updates(); }
  void DeInit() { capi::deinit(); }
  void RequestShutdown() { capi::request_shutdown(); }
  void SetBatchMode(bool mode) { capi::set_batch_mode(mode); }
  void GoForward() { capi::go_forward(); }
  void GoBack() { capi::go_back(); }
  void Click(float x, float y) { capi::click((int32_t)x, (int32_t)y); }
  void Reload() { capi::reload(); }
  void Stop() { capi::stop(); }
  void LoadUri(hstring uri) { capi::load_uri(*hstring2char(uri)); }
  void Scroll(float dx, float dy, float x, float y) {
    capi::scroll((int32_t)dx, (int32_t)dy, (int32_t)x, (int32_t)y);
  }
  void SetSize(GLsizei width, GLsizei height) {
    if (width != mWindowWidth || height != mWindowHeight) {
      mWindowWidth = width;
      mWindowHeight = height;
      capi::resize(mWindowWidth, mWindowHeight);
    }
  }

private:
  ServoDelegate &mDelegate;
  GLsizei mWindowWidth;
  GLsizei mWindowHeight;
};

// This is sad. We need a static pointer to Servo because we use function
// pointer as callback in Servo, and these functions need a way to get
// the Servo instance. See https://github.com/servo/servo/issues/22967
static Servo *sServo = nullptr;

} // namespace winrt::servo
