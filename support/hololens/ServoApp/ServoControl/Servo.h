/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#pragma once

#include "pch.h"
#include <EGL/egl.h>
#include "strutils.h"
#include <stdlib.h>

namespace winrt::servo {

namespace capi {
extern "C" {
#include <simpleservo.h>
}
} // namespace capi

using namespace capi;

hstring char2hstring(const char *);
std::unique_ptr<char *> hstring2char(hstring);
void setNonPersistentHomepage(hstring, std::vector<capi::CPref> &);

class ServoDelegate;

class Servo {
public:
  Servo(std::optional<hstring>, hstring, GLsizei, GLsizei, EGLNativeWindowType,
        float, ServoDelegate &, bool);
  ~Servo();
  ServoDelegate &Delegate() { return mDelegate; }
  hstring CurrentUrl() { return mUrl; }
  void CurrentUrl(hstring url) { mUrl = url; }

  typedef std::tuple<hstring, winrt::Windows::Foundation::IInspectable, bool>
      PrefTuple;
  static std::vector<PrefTuple> GetPrefs();
  static PrefTuple GetPref(hstring key);
  static PrefTuple SetBoolPref(hstring key, bool val);
  static PrefTuple SetStringPref(hstring key, hstring val);
  static PrefTuple SetIntPref(hstring key, int64_t val);
  static PrefTuple SetFloatPref(hstring key, double val);
  static PrefTuple ResetPref(hstring key);

  typedef CMouseButton MouseButton;
  typedef CPromptResult PromptResult;
  typedef CContextMenuResult ContextMenuResult;
  typedef CMediaSessionActionType MediaSessionActionType;
  typedef CMediaSessionPlaybackState MediaSessionPlaybackState;
  typedef CDevtoolsServerState DevtoolsServerState;
  typedef CPrefType CPrefType;

  void PerformUpdates() { perform_updates(); }
  void DeInit() { deinit(); }
  void RequestShutdown() { request_shutdown(); }
  void SetBatchMode(bool mode) { set_batch_mode(mode); }
  void GoForward() { go_forward(); }
  void GoBack() { go_back(); }
  void Click(float x, float y) { click(x, y); }
  void MouseDown(float x, float y, CMouseButton b) { mouse_down(x, y, b); }
  void MouseUp(float x, float y, CMouseButton b) { mouse_up(x, y, b); }
  void TouchDown(float x, float y, int32_t id) { touch_down(x, y, id); }
  void TouchUp(float x, float y, int32_t id) { touch_up(x, y, id); }
  void TouchMove(float x, float y, int32_t id) { touch_move(x, y, id); }
  void TouchCancel(float x, float y, int32_t id) { touch_cancel(x, y, id); }
  void MouseMove(float x, float y) { mouse_move(x, y); }
  void KeyDown(const char *k) { key_down(k); }
  void KeyUp(const char *k) { key_up(k); }

  void Reload() {
    clear_cache();
    reload();
  }
  void Stop() { stop(); }
  bool LoadUri(hstring uri) { return load_uri(*hstring2char(uri)); }
  void ChangeVisibility(bool visible) { change_visibility(visible); }
  bool IsUriValid(hstring uri) { return is_uri_valid(*hstring2char(uri)); }
  void GoHome();
  void Scroll(float dx, float dy, float x, float y) {
    scroll((int32_t)dx, (int32_t)dy, (int32_t)x, (int32_t)y);
  }
  void SetSize(GLsizei width, GLsizei height) {
    if (width != mWindowWidth || height != mWindowHeight) {
      mWindowWidth = width;
      mWindowHeight = height;
      resize(mWindowWidth, mWindowHeight);
    }
  }
  void SendMediaSessionAction(CMediaSessionActionType action) {
    media_session_action(action);
  }
  void ContextMenuClosed(CContextMenuResult res, unsigned int idx) {
    on_context_menu_closed(res, idx);
  }
  void IMEDismissed() { ime_dismissed(); }

private:
  ServoDelegate &mDelegate;
  hstring mUrl;
  GLsizei mWindowWidth;
  GLsizei mWindowHeight;
  static void SaveUserPref(PrefTuple);
  static PrefTuple WrapPref(CPref cpref);
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
  virtual void OnServoPanic(hstring) = 0;
  virtual void OnServoIMEShow(hstring text, int32_t x, int32_t y, int32_t width,
                              int32_t height) = 0;
  virtual void OnServoIMEHide() = 0;
  virtual void OnServoDevtoolsStarted(bool, const unsigned int, hstring) = 0;
  virtual void OnServoMediaSessionMetadata(hstring, hstring, hstring) = 0;
  virtual void OnServoMediaSessionPosition(double, double, double) = 0;
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
