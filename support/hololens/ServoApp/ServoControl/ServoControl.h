#pragma once
#include "ServoControl.g.h"
#include "OpenGLES.h"
#include "Servo.h"
#include "DefaultUrl.h"

namespace winrt::ServoApp::implementation {
struct ServoControl : ServoControlT<ServoControl>, public servo::ServoDelegate {

  ServoControl();

  void GoBack();
  void GoForward();
  void Reload();
  void Stop();
  void Shutdown();
  Windows::Foundation::Uri LoadURIOrSearch(hstring);

  void OnLoaded(IInspectable const &,
                Windows::UI::Xaml::RoutedEventArgs const &);

  winrt::event_token
  OnURLChanged(Windows::Foundation::EventHandler<hstring> const &handler) {
    return mOnURLChangedEvent.add(handler);
  };
  void OnURLChanged(winrt::event_token const &token) noexcept {
    mOnURLChangedEvent.remove(token);
  }

  winrt::event_token
  OnTitleChanged(Windows::Foundation::EventHandler<hstring> const &handler) {
    return mOnTitleChangedEvent.add(handler);
  };
  void OnTitleChanged(winrt::event_token const &token) noexcept {
    mOnTitleChangedEvent.remove(token);
  }

  winrt::event_token OnHistoryChanged(HistoryChangedDelegate const &handler) {
    return mOnHistoryChangedEvent.add(handler);
  };
  void OnHistoryChanged(winrt::event_token const &token) noexcept {
    mOnHistoryChangedEvent.remove(token);
  }

  winrt::event_token OnLoadStarted(EventDelegate const &handler) {
    return mOnLoadStartedEvent.add(handler);
  };
  void OnLoadStarted(winrt::event_token const &token) noexcept {
    mOnLoadStartedEvent.remove(token);
  }

  winrt::event_token OnLoadEnded(EventDelegate const &handler) {
    return mOnLoadEndedEvent.add(handler);
  };
  void OnLoadEnded(winrt::event_token const &token) noexcept {
    mOnLoadEndedEvent.remove(token);
  }

  winrt::event_token OnCaptureGesturesStarted(EventDelegate const &handler) {
    return mOnCaptureGesturesStartedEvent.add(handler);
  };
  void OnCaptureGesturesStarted(winrt::event_token const &token) noexcept {
    mOnCaptureGesturesStartedEvent.remove(token);
  }

  winrt::event_token OnCaptureGesturesEnded(EventDelegate const &handler) {
    return mOnCaptureGesturesEndedEvent.add(handler);
  };
  void OnCaptureGesturesEnded(winrt::event_token const &token) noexcept {
    mOnCaptureGesturesEndedEvent.remove(token);
  }

  void SetTransientMode(bool transient) { mTransient = transient; }

  void SetArgs(hstring args) { mArgs = args; }

  virtual void WakeUp();
  virtual void OnServoLoadStarted();
  virtual void OnServoLoadEnded();
  virtual void OnServoHistoryChanged(bool, bool);
  virtual void OnServoShutdownComplete();
  virtual void OnServoTitleChanged(winrt::hstring);
  virtual void OnServoAlert(winrt::hstring);
  virtual void OnServoURLChanged(winrt::hstring);
  virtual void Flush();
  virtual void MakeCurrent();
  virtual bool OnServoAllowNavigation(winrt::hstring);
  virtual void OnServoAnimatingChanged(bool);
  virtual void OnServoIMEStateChanged(bool);

private:
  winrt::event<Windows::Foundation::EventHandler<hstring>> mOnURLChangedEvent;
  winrt::event<Windows::Foundation::EventHandler<hstring>> mOnTitleChangedEvent;
  winrt::event<HistoryChangedDelegate> mOnHistoryChangedEvent;
  winrt::event<EventDelegate> mOnLoadStartedEvent;
  winrt::event<EventDelegate> mOnLoadEndedEvent;
  winrt::event<EventDelegate> mOnCaptureGesturesStartedEvent;
  winrt::event<EventDelegate> mOnCaptureGesturesEndedEvent;

  float mDPI = 1;
  hstring mInitialURL = DEFAULT_URL;
  bool mTransient = false;

  Windows::UI::Xaml::Controls::SwapChainPanel ServoControl::Panel();
  void CreateRenderSurface();
  void DestroyRenderSurface();
  void RecoverFromLostDevice();

  void StartRenderLoop();
  void StopRenderLoop();
  void Loop();

  std::optional<Windows::Foundation::Uri> TryParseURI(hstring input) {
    try {
      return Windows::Foundation::Uri(input);
    } catch (hresult_invalid_argument const &) {
      return {};
    }
  }

  void
  OnSurfaceClicked(IInspectable const &,
                   Windows::UI::Xaml::Input::TappedRoutedEventArgs const &);

  void OnSurfaceManipulationDelta(
      IInspectable const &,
      Windows::UI::Xaml::Input::ManipulationDeltaRoutedEventArgs const &);

  void OnSurfaceResized(IInspectable const &,
                        Windows::UI::Xaml::SizeChangedEventArgs const &);

  template <typename Callable> void RunOnUIThread(Callable);
  void RunOnGLThread(std::function<void()>);

  std::unique_ptr<servo::Servo> mServo;
  EGLSurface mRenderSurface{EGL_NO_SURFACE};
  OpenGLES mOpenGLES;
  bool mAnimating = false;
  bool mLooping = false;
  std::vector<std::function<void()>> mTasks;
  CRITICAL_SECTION mGLLock;
  CONDITION_VARIABLE mGLCondVar;
  std::unique_ptr<Concurrency::task<void>> mLoopTask;
  hstring mArgs;
};
} // namespace winrt::ServoApp::implementation

namespace winrt::ServoApp::factory_implementation {
struct ServoControl
    : ServoControlT<ServoControl, implementation::ServoControl> {};
} // namespace winrt::ServoApp::factory_implementation
