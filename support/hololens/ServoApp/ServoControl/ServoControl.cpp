#include "pch.h"
#include "ServoControl.h"
#include "ServoControl.g.cpp"
#include <stdlib.h>

using namespace std::placeholders;
using namespace winrt::Windows::Graphics::Display;
using namespace winrt::Windows::UI::Xaml;
using namespace winrt::Windows::UI::Popups;
using namespace winrt::Windows::UI::Core;
using namespace winrt::Windows::Foundation;
using namespace winrt::Windows::System;
using namespace winrt::Windows::Devices::Input;
using namespace concurrency;
using namespace winrt::servo;

namespace winrt::ServoApp::implementation {

ServoControl::ServoControl() {
  mDPI = (float)DisplayInformation::GetForCurrentView().ResolutionScale() / 100;
  DefaultStyleKey(winrt::box_value(L"ServoApp.ServoControl"));
  Loaded(std::bind(&ServoControl::OnLoaded, this, _1, _2));
}

void ServoControl::Shutdown() {
  if (mServo != nullptr) {
    if (!mLooping) {
      // FIXME: this should not happen. In that case, we can't send the
      // shutdown event to Servo.
    } else {
      RunOnGLThread([=] { mServo->RequestShutdown(); });
      mLoopTask->wait();
      mLoopTask.reset();
      mServo.reset();
    }
  }
}

void ServoControl::OnLoaded(IInspectable const &, RoutedEventArgs const &) {
  auto panel = Panel();
  panel.Tapped(std::bind(&ServoControl::OnSurfaceTapped, this, _1, _2));
  panel.PointerPressed(
      std::bind(&ServoControl::OnSurfacePointerPressed, this, _1, _2, true));
  panel.PointerReleased(
      std::bind(&ServoControl::OnSurfacePointerPressed, this, _1, _2, false));
  panel.PointerCanceled(
      std::bind(&ServoControl::OnSurfacePointerCanceled, this, _1, _2));
  panel.PointerExited(
      std::bind(&ServoControl::OnSurfacePointerExited, this, _1, _2));
  panel.PointerCaptureLost(
      std::bind(&ServoControl::OnSurfacePointerLost, this, _1, _2));
  panel.PointerMoved(
      std::bind(&ServoControl::OnSurfacePointerMoved, this, _1, _2));
  panel.PointerWheelChanged(
      std::bind(&ServoControl::OnSurfaceWheelChanged, this, _1, _2));
  panel.ManipulationStarted(
      [=](IInspectable const &,
          Input::ManipulationStartedRoutedEventArgs const &e) {
        mOnCaptureGesturesStartedEvent();
        e.Handled(true);
      });
  panel.ManipulationCompleted(
      [=](IInspectable const &,
          Input::ManipulationCompletedRoutedEventArgs const &e) {
        mOnCaptureGesturesEndedEvent();
        e.Handled(true);
      });
  panel.ManipulationDelta(
      std::bind(&ServoControl::OnSurfaceManipulationDelta, this, _1, _2));
  Panel().SizeChanged(std::bind(&ServoControl::OnSurfaceResized, this, _1, _2));
  InitializeConditionVariable(&mGLCondVar);
  InitializeCriticalSection(&mGLLock);
  InitializeConditionVariable(&mDialogCondVar);
  InitializeCriticalSection(&mDialogLock);
  CreateRenderSurface();
  StartRenderLoop();
}

Controls::SwapChainPanel ServoControl::Panel() {
  return GetTemplateChild(L"swapChainPanel").as<Controls::SwapChainPanel>();
}

void ServoControl::CreateRenderSurface() {
  if (mRenderSurface == EGL_NO_SURFACE) {
    mRenderSurface = mOpenGLES.CreateSurface(Panel(), mDPI);
  }
}

void ServoControl::DestroyRenderSurface() {
  mOpenGLES.DestroySurface(mRenderSurface);
  mRenderSurface = EGL_NO_SURFACE;
}

void ServoControl::RecoverFromLostDevice() {
  StopRenderLoop();
  DestroyRenderSurface();
  mOpenGLES.Reset();
  CreateRenderSurface();
  StartRenderLoop();
}

void ServoControl::OnSurfaceManipulationDelta(
    IInspectable const &, Input::ManipulationDeltaRoutedEventArgs const &e) {
  auto x = e.Position().X * mDPI;
  auto y = e.Position().Y * mDPI;
  auto dx = e.Delta().Translation.X * mDPI;
  auto dy = e.Delta().Translation.Y * mDPI;
  RunOnGLThread([=] { mServo->Scroll(dx, dy, x, y); });
  e.Handled(true);
}

void ServoControl::OnSurfaceTapped(IInspectable const &,
                                   Input::TappedRoutedEventArgs const &e) {
  if (e.PointerDeviceType() == PointerDeviceType::Mouse) {
    auto coords = e.GetPosition(Panel());
    auto x = coords.X * mDPI;
    auto y = coords.Y * mDPI;
    RunOnGLThread([=] { mServo->Click(x, y); });
  }
  e.Handled(true);
}

void ServoControl::OnSurfacePointerPressed(
    IInspectable const &, Input::PointerRoutedEventArgs const &e, bool down) {
  auto ty = e.Pointer().PointerDeviceType();
  if (ty == PointerDeviceType::Mouse) {
    auto point = e.GetCurrentPoint(Panel());

    auto x = point.Position().X * mDPI;
    auto y = point.Position().Y * mDPI;
    auto props = point.Properties();
    std::optional<Servo::MouseButton> button = {};

    if (props.IsLeftButtonPressed()) {
      button = Servo::MouseButton::Left;
    } else if (props.IsRightButtonPressed()) {
      button = Servo::MouseButton::Right;
    } else if (props.IsMiddleButtonPressed()) {
      button = Servo::MouseButton::Middle;
    }

    if (!button.has_value() && mPressedMouseButton.has_value()) {
      auto releasedButton = *mPressedMouseButton;
      mPressedMouseButton = {};
      RunOnGLThread([=] { mServo->MouseUp(x, y, releasedButton); });
      e.Handled(true);
    }

    if (button.has_value()) {
      RunOnGLThread([=] { mServo->MouseDown(x, y, *button); });
      e.Handled(true);
    }

    mPressedMouseButton = button;
  } else if (ty == PointerDeviceType::Touch) {
    auto point = e.GetCurrentPoint(Panel());

    auto x = point.Position().X * mDPI;
    auto y = point.Position().Y * mDPI;

    if (down) {
      RunOnGLThread([=] { mServo->TouchDown(x, y, point.PointerId()); });
    } else {
      RunOnGLThread([=] { mServo->TouchUp(x, y, point.PointerId()); });
    }
    e.Handled(true);
  }
}

void ServoControl::OnSurfacePointerCanceled(
    IInspectable const &, Input::PointerRoutedEventArgs const &e) {
  e.Handled(true);
  auto ty = e.Pointer().PointerDeviceType();
  if (ty == PointerDeviceType::Mouse) {
    mPressedMouseButton = {};
  } else if (ty == PointerDeviceType::Touch) {
    auto point = e.GetCurrentPoint(Panel());
    auto x = point.Position().X * mDPI;
    auto y = point.Position().Y * mDPI;
    RunOnGLThread([=] { mServo->TouchCancel(x, y, point.PointerId()); });
  }
}

void ServoControl::OnSurfacePointerExited(
    IInspectable const &, Input::PointerRoutedEventArgs const &e) {
  e.Handled(true);
  auto ty = e.Pointer().PointerDeviceType();
  if (ty == PointerDeviceType::Touch) {
    auto point = e.GetCurrentPoint(Panel());
    auto x = point.Position().X * mDPI;
    auto y = point.Position().Y * mDPI;
    RunOnGLThread([=] { mServo->TouchCancel(x, y, point.PointerId()); });
    ;
  }
}

void ServoControl::OnSurfacePointerLost(
    IInspectable const &, Input::PointerRoutedEventArgs const &e) {
  // According to the documentation:
  // https://docs.microsoft.com/en-us/windows/uwp/design/input/handle-pointer-input#handle-pointer-events
  // we should cancel the event on PointLost. But we keep getting
  // PointerMoved events after PointerLost. Servo doesn't like getting events
  // from a pointer id that has been canceled. So we do nothing.
  e.Handled(true);
  return;
}

void ServoControl::OnSurfacePointerMoved(
    IInspectable const &, Input::PointerRoutedEventArgs const &e) {
  auto ty = e.Pointer().PointerDeviceType();
  auto point = e.GetCurrentPoint(Panel());
  auto x = point.Position().X * mDPI;
  auto y = point.Position().Y * mDPI;
  if (ty == PointerDeviceType::Touch && point.IsInContact()) {
    RunOnGLThread([=] { mServo->TouchMove(x, y, point.PointerId()); });
  } else {
    RunOnGLThread([=] { mServo->MouseMove(x, y); });
  }
  e.Handled(true);
}

void ServoControl::OnSurfaceWheelChanged(
    IInspectable const &, Input::PointerRoutedEventArgs const &e) {
  if (e.Pointer().PointerDeviceType() == PointerDeviceType::Mouse) {
    auto point = e.GetCurrentPoint(Panel());
    auto x = point.Position().X * mDPI;
    auto y = point.Position().Y * mDPI;
    auto delta = point.Properties().MouseWheelDelta() * mDPI;
    RunOnGLThread([=] { mServo->Scroll(0, (float)delta, x, y); });
  }
}

void ServoControl::OnSurfaceResized(IInspectable const &,
                                    SizeChangedEventArgs const &e) {
  auto size = e.NewSize();
  auto w = size.Width * mDPI;
  auto h = size.Height * mDPI;
  RunOnGLThread([=] { mServo->SetSize(w, h); });
}

void ServoControl::GoBack() {
  RunOnGLThread([=] { mServo->GoBack(); });
}
void ServoControl::GoForward() {
  RunOnGLThread([=] { mServo->GoForward(); });
}
void ServoControl::Reload() {
  RunOnGLThread([=] { mServo->Reload(); });
}
void ServoControl::Stop() {
  RunOnGLThread([=] { mServo->Stop(); });
}
hstring ServoControl::LoadURIOrSearch(hstring input) {
  // Initial input is valid
  if (mServo->IsUriValid(input)) {
    TryLoadUri(input);
    return input;
  }

  // Not valid. Maybe it's just missing the scheme.
  hstring with_scheme = L"https://" + input;
  // If the user only types "mozilla" we don't want to go to
  // https://mozilla even though it's a valid url.
  bool has_dot = wcsstr(input.c_str(), L".") != nullptr;
  if (mServo->IsUriValid(with_scheme) && has_dot) {
    TryLoadUri(with_scheme);
    return with_scheme;
  }

  // Doesn't look like a URI. Let's search for the string.
  hstring searchUri =
      L"https://duckduckgo.com/html/?q=" + Uri::EscapeComponent(input);
  TryLoadUri(searchUri);
  return searchUri;
}

void ServoControl::SendMediaSessionAction(int32_t action) {
  RunOnGLThread([=] {
    mServo->SendMediaSessionAction(
        static_cast<Servo::MediaSessionActionType>(action));
  });
}

void ServoControl::TryLoadUri(hstring input) {
  if (!mLooping) {
    mInitialURL = input;
  } else {
    RunOnGLThread([=] {
      if (!mServo->LoadUri(input)) {
        RunOnUIThread([=] {
          MessageDialog msg{L"URI not valid"};
          msg.ShowAsync();
        });
      }
    });
  }
}

void ServoControl::RunOnGLThread(std::function<void()> task) {
  EnterCriticalSection(&mGLLock);
  mTasks.push_back(task);
  LeaveCriticalSection(&mGLLock);
  WakeConditionVariable(&mGLCondVar);
}

/**** GL THREAD LOOP ****/

void ServoControl::Loop() {
  log("BrowserPage::Loop(). GL thread: %i", GetCurrentThreadId());

  mOpenGLES.MakeCurrent(mRenderSurface);

  EGLint panelWidth = 0;
  EGLint panelHeight = 0;
  mOpenGLES.GetSurfaceDimensions(mRenderSurface, &panelWidth, &panelHeight);
  glViewport(0, 0, panelWidth, panelHeight);

  if (mServo == nullptr) {
    log("Entering loop");
    ServoDelegate *sd = static_cast<ServoDelegate *>(this);
    mServo = std::make_unique<Servo>(mInitialURL, mArgs, panelWidth,
                                     panelHeight, mDPI, *sd);
  } else {
    // FIXME: this will fail since create_task didn't pick the thread
    // where Servo was running initially.
    throw winrt::hresult_error(E_FAIL, L"Recovering loop unimplemented");
  }

  mServo->SetBatchMode(true);

  while (true) {
    EnterCriticalSection(&mGLLock);
    while (mTasks.size() == 0 && !mAnimating && mLooping) {
      SleepConditionVariableCS(&mGLCondVar, &mGLLock, INFINITE);
    }
    if (!mLooping) {
      LeaveCriticalSection(&mGLLock);
      break;
    }
    for (auto &&task : mTasks) {
      task();
    }
    mTasks.clear();
    LeaveCriticalSection(&mGLLock);
    mServo->PerformUpdates();
  }
  mServo->DeInit();
}

void ServoControl::StartRenderLoop() {
  if (mLooping) {
#if defined _DEBUG
    throw winrt::hresult_error(E_FAIL, L"GL thread is already looping");
#else
    return;
#endif
  }
  mLooping = true;
  log("BrowserPage::StartRenderLoop(). UI thread: %i", GetCurrentThreadId());
  auto task = Concurrency::create_task([=] { Loop(); });
  mLoopTask = std::make_unique<Concurrency::task<void>>(task);
}

void ServoControl::StopRenderLoop() {
  if (mLooping) {
    EnterCriticalSection(&mGLLock);
    mLooping = false;
    LeaveCriticalSection(&mGLLock);
    WakeConditionVariable(&mGLCondVar);
    mLoopTask->wait();
    mLoopTask.reset();
  }
}

/**** SERVO CALLBACKS ****/

void ServoControl::OnServoLoadStarted() {
  RunOnUIThread([=] { mOnLoadStartedEvent(); });
}

void ServoControl::OnServoLoadEnded() {
  RunOnUIThread([=] { mOnLoadEndedEvent(); });
}

void ServoControl::OnServoHistoryChanged(bool back, bool forward) {
  RunOnUIThread([=] { mOnHistoryChangedEvent(back, forward); });
}

void ServoControl::OnServoShutdownComplete() {
  EnterCriticalSection(&mGLLock);
  mLooping = false;
  LeaveCriticalSection(&mGLLock);
}

void ServoControl::OnServoTitleChanged(hstring title) {
  RunOnUIThread([=] { mOnTitleChangedEvent(*this, title); });
}

void ServoControl::OnServoURLChanged(hstring url) {
  RunOnUIThread([=] {
    mCurrentUrl = url;
    mOnURLChangedEvent(*this, url);
  });
}

void ServoControl::Flush() {
  if (mOpenGLES.SwapBuffers(mRenderSurface) != GL_TRUE) {
    // The call to eglSwapBuffers might not be successful (i.e. due to Device
    // Lost) If the call fails, then we must reinitialize EGL and the GL
    // resources.
    RunOnUIThread([=] { RecoverFromLostDevice(); });
  }
}

void ServoControl::MakeCurrent() { mOpenGLES.MakeCurrent(mRenderSurface); }

void ServoControl::WakeUp() {
  RunOnGLThread([=] {});
}

bool ServoControl::OnServoAllowNavigation(hstring uri) {
  if (mTransient) {
    RunOnUIThread([=] { Launcher::LaunchUriAsync(Uri{uri}); });
  }
  return !mTransient;
}

void ServoControl::OnServoAnimatingChanged(bool animating) {
  EnterCriticalSection(&mGLLock);
  mAnimating = animating;
  LeaveCriticalSection(&mGLLock);
  WakeConditionVariable(&mGLCondVar);
}

void ServoControl::OnServoIMEStateChanged(bool aShow) {
  // FIXME:
  // https://docs.microsoft.com/en-us/windows/win32/winauto/uiauto-implementingtextandtextrange
}

void ServoControl::OnServoMediaSessionMetadata(hstring title, hstring artist,
                                               hstring album) {
  RunOnUIThread([=] { mOnMediaSessionMetadataEvent(title, artist, album); });
}

void ServoControl::OnServoMediaSessionPlaybackStateChange(int state) {
  RunOnUIThread([=] { mOnMediaSessionPlaybackStateChangeEvent(*this, state); });
}

std::tuple<Controls::ContentDialogResult, std::optional<hstring>>
ServoControl::PromptSync(hstring title, hstring message, hstring primaryButton,
                         std::optional<hstring> secondaryButton,
                         std::optional<hstring> input) {

  bool showing = true;
  Controls::ContentDialogResult retButton = Controls::ContentDialogResult::None;
  std::optional<hstring> retString = {};

  EnterCriticalSection(&mDialogLock);

  Dispatcher().RunAsync(CoreDispatcherPriority::High, [&] {
    auto dialog = Controls::ContentDialog();
    dialog.IsPrimaryButtonEnabled(true);
    dialog.PrimaryButtonText(primaryButton);

    if (secondaryButton.has_value()) {
      dialog.IsPrimaryButtonEnabled(true);
      dialog.SecondaryButtonText(*secondaryButton);
    } else {
      dialog.IsPrimaryButtonEnabled(false);
    }

    auto titleBlock = Controls::TextBlock();
    titleBlock.Text(title);

    auto messageBlock = Controls::TextBlock();
    messageBlock.TextWrapping(TextWrapping::Wrap);
    messageBlock.Text(message);
    Controls::StackPanel stack = Controls::StackPanel();
    stack.Children().Append(titleBlock);
    stack.Children().Append(messageBlock);

    dialog.Content(stack);

    auto textbox = Controls::TextBox();
    textbox.KeyDown([=](auto sender, auto args) {
      if (args.Key() == Windows::System::VirtualKey::Enter) {
        dialog.Hide();
      }
    });
    if (input.has_value()) {
      textbox.Text(*input);
      stack.Children().Append(textbox);
    }

    dialog.Closed([&, textbox](Controls::ContentDialog d, auto closed) {
      EnterCriticalSection(&mDialogLock);
      retButton = closed.Result();
      showing = false;
      if (retButton == Controls::ContentDialogResult::Primary &&
          input.has_value()) {
        retString = hstring(textbox.Text());
      }
      LeaveCriticalSection(&mDialogLock);
      WakeConditionVariable(&mDialogCondVar);
    });
    dialog.ShowAsync();
  });

  while (showing) {
    SleepConditionVariableCS(&mDialogCondVar, &mDialogLock, INFINITE);
  }
  LeaveCriticalSection(&mDialogLock);

  return {retButton, retString};
}

void ServoControl::OnServoPromptAlert(winrt::hstring message, bool trusted) {
  auto title = trusted ? L"" : mCurrentUrl + L" says:";
  PromptSync(title, message, L"OK", {}, {});
}

servo::Servo::PromptResult ServoControl::OnServoPromptOkCancel(winrt::hstring message, bool trusted) {
  auto title = trusted ? L"" : mCurrentUrl + L" says:";
  auto [button, string] = PromptSync(title, message, L"OK", L"Cancel", {});
  if (button == Controls::ContentDialogResult::Primary) {
    return servo::Servo::PromptResult::Primary;
  } else if (button == Controls::ContentDialogResult::Secondary) {
    return servo::Servo::PromptResult::Secondary;
  } else {
    return servo::Servo::PromptResult::Dismissed;
  }
}

servo::Servo::PromptResult ServoControl::OnServoPromptYesNo(winrt::hstring message, bool trusted) {
  auto title = trusted ? L"" : mCurrentUrl + L" says:";
  auto [button, string] = PromptSync(title, message, L"Yes", L"No", {});
  if (button == Controls::ContentDialogResult::Primary) {
    return servo::Servo::PromptResult::Primary;
  } else if (button == Controls::ContentDialogResult::Secondary) {
    return servo::Servo::PromptResult::Secondary;
  } else {
    return servo::Servo::PromptResult::Dismissed;
  }
}

std::optional<hstring> ServoControl::OnServoPromptInput(winrt::hstring message,
                                                        winrt::hstring default,
                                                        bool trusted) {
  auto title = trusted ? L"" : mCurrentUrl + L" says:";
  auto [button, string] = PromptSync(title, message, L"Ok", L"Cancel", default);
  return string;
}

template <typename Callable> void ServoControl::RunOnUIThread(Callable cb) {
  Dispatcher().RunAsync(CoreDispatcherPriority::High, cb);
}

} // namespace winrt::ServoApp::implementation
