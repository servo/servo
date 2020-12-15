#pragma once
#include "ServoControl.g.h"
#include "Pref.g.h"
#include "OpenGLES.h"
#include "Servo.h"

using namespace winrt::Windows::Foundation::Collections;

namespace winrt::ServoApp::implementation {

struct Pref : PrefT<Pref> {
public:
  Pref(hstring key, IInspectable value, bool isDefault) {
    mKey = key;
    mValue = value;
    mIsDefault = isDefault;
  };
  IInspectable Value() { return mValue; }
  hstring Key() { return mKey; }
  bool IsDefault() { return mIsDefault; }

private:
  hstring mKey;
  IInspectable mValue;
  bool mIsDefault;
};

struct L10NStrings {
  hstring ContextMenuTitle;
  hstring PromptTitle;
  hstring PromptOk;
  hstring PromptCancel;
  hstring PromptYes;
  hstring PromptNo;
  hstring URINotValid;
};

struct ServoControl : ServoControlT<ServoControl>, public servo::ServoDelegate {

  ServoControl();

  IVector<ServoApp::Pref> Preferences();

  void GoBack();
  void GoForward();
  void Reload();
  void Stop();
  void ChangeVisibility(bool);
  void Shutdown();
  hstring LoadURIOrSearch(hstring);
  void GoHome();
  void SendMediaSessionAction(int32_t);

  ServoApp::Pref SetBoolPref(hstring aKey, bool aVal) {
    auto [key, val, isDefault] = servo::Servo::SetBoolPref(aKey, aVal);
    return ServoApp::Pref(key, val, isDefault);
  }

  ServoApp::Pref SetStringPref(hstring aKey, hstring aVal) {
    auto [key, val, isDefault] = servo::Servo::SetStringPref(aKey, aVal);
    return ServoApp::Pref(key, val, isDefault);
  }

  ServoApp::Pref SetIntPref(hstring aKey, int64_t aVal) {
    auto [key, val, isDefault] = servo::Servo::SetIntPref(aKey, aVal);
    return ServoApp::Pref(key, val, isDefault);
  }

  ServoApp::Pref SetFloatPref(hstring aKey, double aVal) {
    auto [key, val, isDefault] = servo::Servo::SetFloatPref(aKey, aVal);
    return ServoApp::Pref(key, val, isDefault);
  }

  ServoApp::Pref ResetPref(hstring aKey) {
    auto [key, val, isDefault] = servo::Servo::ResetPref(aKey);
    return ServoApp::Pref(key, val, isDefault);
  }

  ServoApp::Pref GetPref(hstring aKey) {
    auto [key, val, isDefault] = servo::Servo::GetPref(aKey);
    return ServoApp::Pref(key, val, isDefault);
  }

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

  winrt::event_token
  OnServoPanic(Windows::Foundation::EventHandler<hstring> const &handler) {
    return mOnServoPanic.add(handler);
  };
  void OnServoPanic(winrt::event_token const &token) noexcept {
    mOnServoPanic.remove(token);
  }

  winrt::event_token OnHistoryChanged(HistoryChangedDelegate const &handler) {
    return mOnHistoryChangedEvent.add(handler);
  };
  void OnHistoryChanged(winrt::event_token const &token) noexcept {
    mOnHistoryChangedEvent.remove(token);
  }

  winrt::event_token
  OnDevtoolsStatusChanged(DevtoolsStatusChangedDelegate const &handler) {
    return mOnDevtoolsStatusChangedEvent.add(handler);
  };
  void OnDevtoolsStatusChanged(winrt::event_token const &token) noexcept {
    mOnDevtoolsStatusChangedEvent.remove(token);
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
  OnMediaSessionPosition(MediaSessionPositionDelegate const &handler) {
    return mOnMediaSessionPositionEvent.add(handler);
  };
  void OnMediaSessionPosition(winrt::event_token const &token) noexcept {
    mOnMediaSessionPositionEvent.remove(token);
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
  virtual bool OnServoAllowNavigation(winrt::hstring);
  virtual void OnServoAnimatingChanged(bool);
  virtual void OnServoPanic(hstring);
  virtual void OnServoIMEHide();
  virtual void OnServoIMEShow(hstring text, int32_t, int32_t, int32_t, int32_t);
  virtual void OnServoMediaSessionMetadata(winrt::hstring, winrt::hstring,
                                           winrt::hstring);
  virtual void OnServoMediaSessionPlaybackStateChange(int);
  virtual void OnServoMediaSessionPosition(double, double, double);
  virtual void OnServoPromptAlert(winrt::hstring, bool);
  virtual void OnServoShowContextMenu(std::optional<winrt::hstring>,
                                      std::vector<winrt::hstring>);
  virtual servo::Servo::PromptResult OnServoPromptOkCancel(winrt::hstring,
                                                           bool);
  virtual servo::Servo::PromptResult OnServoPromptYesNo(winrt::hstring, bool);
  virtual std::optional<hstring> OnServoPromptInput(winrt::hstring,
                                                    winrt::hstring, bool);
  virtual void OnServoDevtoolsStarted(bool, const unsigned int, winrt::hstring);

  DevtoolsStatus GetDevtoolsStatus();

private:
  winrt::event<Windows::Foundation::EventHandler<hstring>> mOnURLChangedEvent;
  winrt::event<Windows::Foundation::EventHandler<hstring>> mOnTitleChangedEvent;
  winrt::event<Windows::Foundation::EventHandler<hstring>> mOnServoPanic;
  winrt::event<HistoryChangedDelegate> mOnHistoryChangedEvent;
  winrt::event<DevtoolsStatusChangedDelegate> mOnDevtoolsStatusChangedEvent;
  winrt::event<EventDelegate> mOnLoadStartedEvent;
  winrt::event<EventDelegate> mOnLoadEndedEvent;
  winrt::event<EventDelegate> mOnCaptureGesturesStartedEvent;
  winrt::event<EventDelegate> mOnCaptureGesturesEndedEvent;
  winrt::event<MediaSessionMetadataDelegate> mOnMediaSessionMetadataEvent;
  winrt::event<MediaSessionPositionDelegate> mOnMediaSessionPositionEvent;
  winrt::event<Windows::Foundation::EventHandler<int>>
      mOnMediaSessionPlaybackStateChangeEvent;

  CRITICAL_SECTION mDialogLock;
  CONDITION_VARIABLE mDialogCondVar;

  std::tuple<Windows::UI::Xaml::Controls::ContentDialogResult,
             std::optional<hstring>>
  PromptSync(hstring title, hstring message, hstring primaryButton,
             std::optional<hstring> secondaryButton,
             std::optional<hstring> input);

  int mPanelHeight = 0;
  int mPanelWidth = 0;
  float mDPI = 1;
  hstring mCurrentUrl = L"";
  bool mTransient = false;
  std::optional<hstring> mInitUrl = {};

  Windows::UI::Xaml::Controls::SwapChainPanel Panel();
  void CreateNativeWindow();
  EGLNativeWindowType GetNativeWindow();
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
  void InitializeTextController();

  std::unique_ptr<servo::Servo> mServo;
  PropertySet mNativeWindowProperties;
  OpenGLES mOpenGLES;
  bool mAnimating = false;
  bool mLooping = false;
  std::vector<std::function<void()>> mTasks;
  CRITICAL_SECTION mGLLock;
  CONDITION_VARIABLE mGLCondVar;
  std::unique_ptr<Concurrency::task<void>> mLoopTask;
  hstring mArgs;
  std::optional<servo::Servo::MouseButton> mPressedMouseButton = {};
  std::unique_ptr<L10NStrings> mL10NStrings = nullptr;

  std::optional<Windows::UI::Text::Core::CoreTextEditContext> mEditContext;
  std::optional<Windows::UI::ViewManagement::InputPane> mInputPane;

  std::optional<Windows::Foundation::Rect> mFocusedInputRect;
  std::optional<hstring> mFocusedInputText;
};
} // namespace winrt::ServoApp::implementation

namespace winrt::ServoApp::factory_implementation {
struct ServoControl
    : ServoControlT<ServoControl, implementation::ServoControl> {};
struct Pref : PrefT<Pref, implementation::Pref> {};
} // namespace winrt::ServoApp::factory_implementation
