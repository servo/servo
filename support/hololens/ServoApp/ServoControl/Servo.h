/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#pragma once

#include "pch.h"
#include <EGL/egl.h>
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

class ServoDelegate;

class Servo {
public:
  Servo(hstring, hstring, GLsizei, GLsizei, EGLNativeWindowType, float,
        ServoDelegate &);
  ~Servo();
  ServoDelegate &Delegate() { return mDelegate; }

  typedef std::tuple<hstring, winrt::Windows::Foundation::IInspectable, bool>
      PrefTuple;
  static std::vector<PrefTuple> GetPrefs();
  static PrefTuple GetPref(hstring key);
  static PrefTuple SetBoolPref(hstring key, bool val);
  static PrefTuple SetStringPref(hstring key, hstring val);
  static PrefTuple SetIntPref(hstring key, int64_t val);
  static PrefTuple SetFloatPref(hstring key, double val);
  static PrefTuple ResetPref(hstring key);

  typedef capi::CMouseButton MouseButton;
  typedef capi::CPromptResult PromptResult;
  typedef capi::CContextMenuResult ContextMenuResult;
  typedef capi::CMediaSessionActionType MediaSessionActionType;
  typedef capi::CMediaSessionPlaybackState MediaSessionPlaybackState;
  typedef capi::CDevtoolsServerState DevtoolsServerState;
  typedef capi::CPrefType CPrefType;

  void PerformUpdates() { capi::perform_updates(); }
  void DeInit() { capi::deinit(); }
  void RequestShutdown() { capi::request_shutdown(); }
  void SetBatchMode(bool mode) { capi::set_batch_mode(mode); }
  void GoForward() { capi::go_forward(); }
  void GoBack() { capi::go_back(); }
  void Click(float x, float y) { capi::click(x, y); }
  void MouseDown(float x, float y, capi::CMouseButton b) {
    capi::mouse_down(x, y, b);
  }
  void MouseUp(float x, float y, capi::CMouseButton b) {
    capi::mouse_up(x, y, b);
  }
  void TouchDown(float x, float y, int32_t id) { capi::touch_down(x, y, id); }
  void TouchUp(float x, float y, int32_t id) { capi::touch_up(x, y, id); }
  void TouchMove(float x, float y, int32_t id) { capi::touch_move(x, y, id); }
  void TouchCancel(float x, float y, int32_t id) {
    capi::touch_cancel(x, y, id);
  }
  void MouseMove(float x, float y) { capi::mouse_move(x, y); }

  void Reload() { capi::reload(); }
  void Stop() { capi::stop(); }
  bool LoadUri(hstring uri) { return capi::load_uri(*hstring2char(uri)); }
  void ChangeVisibility(bool visible) { capi::change_visibility(visible); }
  bool IsUriValid(hstring uri) {
    return capi::is_uri_valid(*hstring2char(uri));
  }
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
  void SendMediaSessionAction(capi::CMediaSessionActionType action) {
    capi::media_session_action(action);
  }
  void ContextMenuClosed(capi::CContextMenuResult res, unsigned int idx) {
    capi::on_context_menu_closed(res, idx);
  }

private:
  ServoDelegate &mDelegate;
  GLsizei mWindowWidth;
  GLsizei mWindowHeight;
  static void SaveUserPref(PrefTuple);
  static PrefTuple WrapPref(capi::CPref cpref);
};

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
  virtual void OnServoURLChanged(hstring) = 0;
  virtual bool OnServoAllowNavigation(hstring) = 0;
  virtual void OnServoAnimatingChanged(bool) = 0;
  virtual void OnServoIMEStateChanged(bool) = 0;
  virtual void OnServoDevtoolsStarted(bool, const unsigned int) = 0;
  virtual void OnServoMediaSessionMetadata(hstring, hstring, hstring) = 0;
  virtual void OnServoMediaSessionPlaybackStateChange(int) = 0;
  virtual void OnServoPromptAlert(hstring, bool) = 0;
  virtual void OnServoShowContextMenu(std::optional<hstring>,
                                      std::vector<hstring>) = 0;
  virtual Servo::PromptResult OnServoPromptOkCancel(hstring, bool) = 0;
  virtual Servo::PromptResult OnServoPromptYesNo(hstring, bool) = 0;
  virtual std::optional<hstring> OnServoPromptInput(hstring, hstring, bool) = 0;

protected:
  virtual ~ServoDelegate(){};
};

// This is sad. We need a static pointer to Servo because we use function
// pointer as callback in Servo, and these functions need a way to get
// the Servo instance. See https://github.com/servo/servo/issues/22967
static Servo *sServo = nullptr;
static HANDLE sLogHandle = INVALID_HANDLE_VALUE;

} // namespace winrt::servo
