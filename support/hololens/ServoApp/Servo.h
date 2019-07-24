/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#pragma once

#include "pch.h"
#include "logs.h"

namespace servo {

namespace capi {
extern "C" {
#include <simpleservo.h>
}
} // namespace capi

class ServoDelegate {
public:
  // Called from any thread
  virtual void WakeUp() = 0;
  // Called from GL thread
  virtual void OnLoadStarted() = 0;
  virtual void OnLoadEnded() = 0;
  virtual void OnHistoryChanged(bool, bool) = 0;
  virtual void OnShutdownComplete() = 0;
  virtual void OnTitleChanged(std::wstring) = 0;
  virtual void OnAlert(std::wstring) = 0;
  virtual void OnURLChanged(std::wstring) = 0;
  virtual void Flush() = 0;
  virtual void MakeCurrent() = 0;
  virtual bool OnAllowNavigation(std::wstring) = 0;
  virtual void OnAnimatingChanged(bool) = 0;

protected:
  virtual ~ServoDelegate(){log("A1");};
};

class Servo {
public:
  Servo(GLsizei, GLsizei, ServoDelegate &);
  ~Servo();
  ServoDelegate &Delegate() { return mDelegate; }

  void PerformUpdates() { capi::perform_updates(); }
  void RequestShutdown() { capi::request_shutdown(); }
  void SetBatchMode(bool mode) { capi::set_batch_mode(mode); }
  void GoForward() { capi::go_forward(); }
  void GoBack() { capi::go_back(); }
  void Click(float x, float y) { capi::click(x, y); }
  void Reload() { capi::reload(); }
  void Stop() { capi::stop(); }
  void Scroll(float dx, float dy, float x, float y) {
    capi::scroll(dx, dy, x, y);
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

std::wstring char2w(const char *c_str);

} // namespace servo
