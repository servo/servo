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
  hstring LoadURIOrSearch(hstring);
  void SendMediaSessionAction(int32_t);

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

  winrt::event_token
  OnMediaSessionMetadata(MediaSessionMetadataDelegate const &handler) {
    return mOnMediaSessionMetadataEvent.add(handler);
  };
  void OnMediaSessionMetadata(winrt::event_token const &token) noexcept {
    mOnMediaSessionMetadataEvent.remove(token);
  }

  winrt::event_token OnMediaSessionPlaybackStateChange(
      Windows::Foundation::EventHandler<int> const &handler) {
    return mOnMediaSessionPlaybackStateChangeEvent.add(handler);
  };
  void
  OnMediaSessionPlaybackStateChange(winrt::event_token const &token) noexcept {
    mOnMediaSessionPlaybackStateChangeEvent.remove(token);
  }

  void SetTransientMode(bool transient) { mTransient = transient; }

  void SetArgs(hstring args) { mArgs = args; }

  virtual void WakeUp();
  virtual void OnServoLoadStarted();
  virtual void OnServoLoadEnded();
  virtual void OnServoHistoryChanged(bool, bool);
  virtual void OnServoShutdownComplete();
  virtual void OnServoTitleChanged(winrt::hstring);
  virtual void OnServoURLChanged(winrt::hstring);
  virtual void Flush();
  virtual void MakeCurrent();
  virtual bool OnServoAllowNavigation(winrt::hstring);
  virtual void OnServoAnimatingChanged(bool);
  virtual void OnServoIMEStateChanged(bool);
  virtual void OnServoMediaSessionMetadata(winrt::hstring, winrt::hstring,
                                           winrt::hstring);
  virtual void OnServoMediaSessionPlaybackStateChange(int);
  virtual void OnServoPromptAlert(winrt::hstring, bool);
  virtual servo::Servo::PromptResult OnServoPromptOkCancel(winrt::hstring, bool);
  virtual servo::Servo::PromptResult OnServoPromptYesNo(winrt::hstring, bool);
  virtual std::optional<hstring> OnServoPromptInput(winrt::hstring,
                                                    winrt::hstring, bool);

private:
  winrt::event<Windows::Foundation::EventHandler<hstring>> mOnURLChangedEvent;
  winrt::event<Windows::Foundation::EventHandler<hstring>> mOnTitleChangedEvent;
  winrt::event<HistoryChangedDelegate> mOnHistoryChangedEvent;
  winrt::event<EventDelegate> mOnLoadStartedEvent;
  winrt::event<EventDelegate> mOnLoadEndedEvent;
  winrt::event<EventDelegate> mOnCaptureGesturesStartedEvent;
  winrt::event<EventDelegate> mOnCaptureGesturesEndedEvent;
  winrt::event<MediaSessionMetadataDelegate> mOnMediaSessionMetadataEvent;
  winrt::event<Windows::Foundation::EventHandler<int>>
      mOnMediaSessionPlaybackStateChangeEvent;

  CRITICAL_SECTION mDialogLock;
  CONDITION_VARIABLE mDialogCondVar;

  std::tuple<Windows::UI::Xaml::Controls::ContentDialogResult,
             std::optional<hstring>>
  PromptSync(hstring title, hstring message, hstring primaryButton,
             std::optional<hstring> secondaryButton,
             std::optional<hstring> input);

  float mDPI = 1;
  hstring mInitialURL = DEFAULT_URL;
  hstring mCurrentUrl = L"";
  bool mTransient = false;

  Windows::UI::Xaml::Controls::SwapChainPanel ServoControl::Panel();
  void CreateRenderSurface();
  void DestroyRenderSurface();
  void RecoverFromLostDevice();

  void StartRenderLoop();
  void StopRenderLoop();
  void Loop();

  void OnSurfaceTapped(IInspectable const &,
                       Windows::UI::Xaml::Input::TappedRoutedEventArgs const &);

  void OnSurfacePointerPressed(
      IInspectable const &,
      Windows::UI::Xaml::Input::PointerRoutedEventArgs const &, bool);

  void OnSurfacePointerCanceled(
      IInspectable const &,
      Windows::UI::Xaml::Input::PointerRoutedEventArgs const &);

  void OnSurfacePointerExited(
      IInspectable const &,
      Windows::UI::Xaml::Input::PointerRoutedEventArgs const &);

  void OnSurfacePointerLost(
      IInspectable const &,
      Windows::UI::Xaml::Input::PointerRoutedEventArgs const &);

  void OnSurfacePointerMoved(
      IInspectable const &,
      Windows::UI::Xaml::Input::PointerRoutedEventArgs const &);

  void OnSurfaceWheelChanged(
      IInspectable const &,
      Windows::UI::Xaml::Input::PointerRoutedEventArgs const &);

  void OnSurfaceManipulationDelta(
      IInspectable const &,
      Windows::UI::Xaml::Input::ManipulationDeltaRoutedEventArgs const &);

  void OnSurfaceResized(IInspectable const &,
                        Windows::UI::Xaml::SizeChangedEventArgs const &);

  template <typename Callable> void RunOnUIThread(Callable);
  void RunOnGLThread(std::function<void()>);

  void TryLoadUri(hstring);

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

  std::optional<servo::Servo::MouseButton> mPressedMouseButton = {};
};
} // namespace winrt::ServoApp::implementation

namespace winrt::ServoApp::factory_implementation {
struct ServoControl
    : ServoControlT<ServoControl, implementation::ServoControl> {};
} // namespace winrt::ServoApp::factory_implementation
