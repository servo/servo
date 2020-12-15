#include "pch.h"
#include "strutils.h"
#include "ServoControl.h"
#include "ServoControl.g.cpp"
#include "Pref.g.cpp"
#include <stdlib.h>
#include "Keys.h"

using namespace std::placeholders;
using namespace winrt::Windows::ApplicationModel::Resources;
using namespace winrt::Windows::Graphics::Display;
using namespace winrt::Windows::UI::Xaml;
using namespace winrt::Windows::UI::Popups;
using namespace winrt::Windows::UI::Core;
using namespace winrt::Windows::UI::Text::Core;
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

  auto r = ResourceLoader::GetForCurrentView();
  L10NStrings l10NStrings = {r.GetString(L"ContextMenu/title"),
                             r.GetString(L"JavascriptPrompt/title"),
                             r.GetString(L"JavascriptPrompt/ok"),
                             r.GetString(L"JavascriptPrompt/cancel"),
                             r.GetString(L"JavascriptPrompt/yes"),
                             r.GetString(L"JavascriptPrompt/no"),
                             r.GetString(L"URINotValid/Alert")};
  mL10NStrings = std::make_unique<L10NStrings>(l10NStrings);
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
  panel.ManipulationStarted([=](const auto &, const auto &e) {
    mOnCaptureGesturesStartedEvent();
    e.Handled(true);
  });
  panel.ManipulationCompleted([=](const auto &, const auto &e) {
    mOnCaptureGesturesEndedEvent();
    e.Handled(true);
  });
  panel.ManipulationDelta(
      std::bind(&ServoControl::OnSurfaceManipulationDelta, this, _1, _2));
  panel.SizeChanged(std::bind(&ServoControl::OnSurfaceResized, this, _1, _2));

  InitializeTextController();
  InitializeConditionVariable(&mGLCondVar);
  InitializeCriticalSection(&mGLLock);
  InitializeConditionVariable(&mDialogCondVar);
  InitializeCriticalSection(&mDialogLock);
  CreateNativeWindow();
  StartRenderLoop();
}

void ServoControl::InitializeTextController() {
  mInputPane = Windows::UI::ViewManagement::InputPane::GetForCurrentView();
  mInputPane->Hiding([=](const auto &, const auto &) {
    if (mLooping) {
      RunOnGLThread([=] { mServo->IMEDismissed(); });
    }
  });

  auto manager = CoreTextServicesManager::GetForCurrentView();
  mEditContext = manager.CreateEditContext();
  mEditContext->InputPaneDisplayPolicy(CoreTextInputPaneDisplayPolicy::Manual);

  mEditContext->TextRequested([=](const auto &, const auto &e) {
    e.Request().Text(*mFocusedInputText);
  });

  mEditContext->SelectionRequested([=](const auto &, const auto &) {});

  mEditContext->LayoutRequested([=](const auto &, const auto &e) {
    // Necessary to show the preview
    e.Request().LayoutBounds().TextBounds(*mFocusedInputRect);
    e.Request().LayoutBounds().ControlBounds(*mFocusedInputRect);
  });

  mEditContext->TextUpdating([=](const auto &, const auto &e) {
    RunOnGLThread([=] {
      auto text = *hstring2char(e.Text());
      size_t size = strlen(text);
      for (int i = 0; i < size; i++) {
        char letter[2];
        memcpy(letter, &text[i], 1);
        letter[1] = '\0';
        mServo->KeyDown(letter);
        mServo->KeyUp(letter);
      }
    });
    e.Result(CoreTextTextUpdatingResult::Succeeded);
  });

  GotFocus(
      [=](const auto &, const auto &) { mEditContext->NotifyFocusEnter(); });

  LostFocus(
      [=](const auto &, const auto &) { mEditContext->NotifyFocusLeave(); });

  PreviewKeyDown([=](const auto &, const auto &e) {
    auto keystr = KeyToString(e.Key());
    if (keystr.has_value()) {
      RunOnGLThread([=] {
        auto keyname = *keystr;
        mServo->KeyDown(keyname);
      });
    }
  });

  PreviewKeyUp([=](const auto &, const auto &e) {
    auto keystr = KeyToString(e.Key());
    if (keystr.has_value()) {
      RunOnGLThread([=] {
        auto keyname = *keystr;
        mServo->KeyUp(keyname);
      });
    }
  });
}

Controls::SwapChainPanel ServoControl::Panel() {
  return GetTemplateChild(L"swapChainPanel").as<Controls::SwapChainPanel>();
}

void ServoControl::CreateNativeWindow() {
  mPanelWidth = (int)(Panel().ActualWidth() * mDPI);
  mPanelHeight = (int)(Panel().ActualHeight() * mDPI);
  mNativeWindowProperties.Insert(EGLNativeWindowTypeProperty, Panel());
  // How to set size and or scale:
  // Insert(EGLRenderSurfaceSizeProperty),
  // PropertyValue::CreateSize(*renderSurfaceSize));
  mNativeWindowProperties.Insert(EGLRenderResolutionScaleProperty,
                                 PropertyValue::CreateSingle(mDPI));
}

EGLNativeWindowType ServoControl::GetNativeWindow() {
  EGLNativeWindowType win =
      static_cast<EGLNativeWindowType>(winrt::get_abi(mNativeWindowProperties));

  return win;
}

void ServoControl::RecoverFromLostDevice() {
  StopRenderLoop();
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
  Focus(FocusState::Programmatic);
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
  auto w = (size.Width * mDPI);
  auto h = (size.Height * mDPI);
  RunOnGLThread([=] { mServo->SetSize((GLsizei)w, (GLsizei)h); });
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
void ServoControl::ChangeVisibility(bool visible) {
  RunOnGLThread([=] { mServo->ChangeVisibility(visible); });
}
void ServoControl::Stop() {
  RunOnGLThread([=] { mServo->Stop(); });
}
void ServoControl::GoHome() {
  RunOnGLThread([=] { mServo->GoHome(); });
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
  auto escapedInput = Uri::EscapeComponent(input);
  std::wstring searchUri =
      unbox_value<hstring>(std::get<1>(Servo::GetPref(L"shell.searchpage")))
          .c_str();
  std::wstring formatted = format(searchUri, escapedInput.c_str());
  hstring finalUri{formatted};
  TryLoadUri(finalUri);
  return finalUri;
}

void ServoControl::SendMediaSessionAction(int32_t action) {
  RunOnGLThread([=] {
    mServo->SendMediaSessionAction(
        static_cast<Servo::MediaSessionActionType>(action));
  });
}

void ServoControl::TryLoadUri(hstring input) {
  if (mLooping) {
    RunOnGLThread([=] {
      if (!mServo->LoadUri(input)) {
        RunOnUIThread([=] {
          MessageDialog msg{mL10NStrings->URINotValid};
          msg.ShowAsync();
        });
      }
    });
  } else {
    mInitUrl = input;
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
  log(L"BrowserPage::Loop(). GL thread: %i", GetCurrentThreadId());

  if (mServo == nullptr) {
    log(L"Entering loop");
    ServoDelegate *sd = static_cast<ServoDelegate *>(this);
    EGLNativeWindowType win = GetNativeWindow();
    mServo = std::make_unique<Servo>(mInitUrl, mArgs, mPanelWidth, mPanelHeight,
                                     win, mDPI, *sd, mTransient);
  } else {
    // FIXME: this will fail since create_task didn't pick the thread
    // where Servo was running initially.
    throw winrt::hresult_error(E_FAIL, L"Recovering loop unimplemented");
  }

  mServo->SetBatchMode(true);

  while (true) {
    EnterCriticalSection(&mGLLock);
    try {
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
    } catch (hresult_error const &e) {
      log(L"GL Thread exception: %s", e.message().c_str());
      throw e;
    } catch (...) {
      log(L"GL Thread exception");
      throw winrt::hresult_error(E_FAIL, L"GL Thread exception");
    }
  }
  mServo->DeInit();
}

void ServoControl::StartRenderLoop() {
  if (mLooping) {
    throw winrt::hresult_error(E_FAIL, L"GL thread is already looping");
  }
  mLooping = true;
  log(L"BrowserPage::StartRenderLoop(). UI thread: %i", GetCurrentThreadId());
  auto task = Concurrency::create_task([=] {
    try {
      Loop();
    } catch (...) {
      // Do our best to recover. Exception has been logged at that point.
      mLooping = false;
      mLoopTask.reset();
      mServo.reset();
      LeaveCriticalSection(&mGLLock);
    }
  });
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

void ServoControl::WakeUp() {
  RunOnGLThread([=] {});
}

bool ServoControl::OnServoAllowNavigation(hstring uri) {
  if (mTransient) {
    RunOnUIThread([=] { Launcher::LaunchUriAsync(Uri{uri}); });
  }
  return !mTransient;
}

void ServoControl::OnServoPanic(hstring backtrace) {
  RunOnUIThread([=] { mOnServoPanic(*this, backtrace); });
}

void ServoControl::OnServoAnimatingChanged(bool animating) {
  EnterCriticalSection(&mGLLock);
  mAnimating = animating;
  LeaveCriticalSection(&mGLLock);
  WakeConditionVariable(&mGLCondVar);
}

void ServoControl::OnServoIMEHide() {
  RunOnUIThread([=] { mInputPane->TryHide(); });
}

void ServoControl::OnServoIMEShow(hstring text, int32_t x, int32_t y,
                                  int32_t width, int32_t height) {
  RunOnUIThread([=] {
    mEditContext->NotifyFocusEnter();
    // FIXME: The simpleservo on_ime_show callback comes with a input method
    // type parameter that could be used to set the input scope here.
    mEditContext->InputScope(CoreTextInputScope::Text);
    // offset of the Servo SwapChainPanel.
    auto transform = Panel().TransformToVisual(Window::Current().Content());
    auto offset = transform.TransformPoint(Point(0, 0));
    mFocusedInputRect =
        Rect(x + offset.X, y + offset.Y, (float)width, (float)height);
    mFocusedInputText = text;
    mInputPane->TryShow();
  });
}

void ServoControl::OnServoMediaSessionPosition(double duration, double position,
                                               double playback_rate) {
  RunOnUIThread(
      [=] { mOnMediaSessionPositionEvent(duration, position, playback_rate); });
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
      dialog.IsSecondaryButtonEnabled(true);
      dialog.SecondaryButtonText(*secondaryButton);
    } else {
      dialog.IsSecondaryButtonEnabled(false);
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
  auto titlefmt =
      format(mL10NStrings->PromptTitle.c_str(), mCurrentUrl.c_str());
  hstring title{trusted ? L"" : titlefmt};
  PromptSync(title, message, mL10NStrings->PromptOk, {}, {});
}

Servo::PromptResult ServoControl::OnServoPromptOkCancel(winrt::hstring message,
                                                        bool trusted) {
  auto titlefmt =
      format(mL10NStrings->PromptTitle.c_str(), mCurrentUrl.c_str());
  hstring title{trusted ? L"" : titlefmt};
  auto [button, string] = PromptSync(title, message, mL10NStrings->PromptOk,
                                     mL10NStrings->PromptCancel, {});
  if (button == Controls::ContentDialogResult::Primary) {
    return Servo::PromptResult::Primary;
  } else if (button == Controls::ContentDialogResult::Secondary) {
    return Servo::PromptResult::Secondary;
  } else {
    return Servo::PromptResult::Dismissed;
  }
}

Servo::PromptResult ServoControl::OnServoPromptYesNo(winrt::hstring message,
                                                     bool trusted) {
  auto titlefmt =
      format(mL10NStrings->PromptTitle.c_str(), mCurrentUrl.c_str());
  hstring title{trusted ? L"" : titlefmt};
  auto [button, string] = PromptSync(title, message, mL10NStrings->PromptYes,
                                     mL10NStrings->PromptNo, {});
  if (button == Controls::ContentDialogResult::Primary) {
    return Servo::PromptResult::Primary;
  } else if (button == Controls::ContentDialogResult::Secondary) {
    return Servo::PromptResult::Secondary;
  } else {
    return Servo::PromptResult::Dismissed;
  }
}

std::optional<hstring> ServoControl::OnServoPromptInput(winrt::hstring message,
                                                        winrt::hstring def,
                                                        bool trusted) {
  auto titlefmt =
      format(mL10NStrings->PromptTitle.c_str(), mCurrentUrl.c_str());
  hstring title{trusted ? L"" : titlefmt};
  auto [button, string] = PromptSync(title, message, mL10NStrings->PromptOk,
                                     mL10NStrings->PromptCancel, def);
  return string;
}

void ServoControl::OnServoDevtoolsStarted(bool success, const unsigned int port,
                                          hstring token) {
  RunOnUIThread([=] {
    auto status = success ? DevtoolsStatus::Running : DevtoolsStatus::Failed;
    // This port works, let's save it for future use.
    Servo::SetIntPref(L"devtools.server.port", port);
    mOnDevtoolsStatusChangedEvent(status, port, token);
  });
}

void ServoControl::OnServoShowContextMenu(std::optional<hstring> title,
                                          std::vector<winrt::hstring> items) {
  RunOnUIThread([=] {
    auto titlestr = mL10NStrings->ContextMenuTitle;
    MessageDialog msg{title.value_or(titlestr)};
    for (auto i = 0; i < items.size(); i++) {
      UICommand cmd{items[i], [=](auto) {
                      RunOnGLThread([=] {
                        mServo->ContextMenuClosed(
                            Servo::ContextMenuResult::Selected, i);
                      });
                    }};
      msg.Commands().Append(cmd);
    }
    UICommand cancel{mL10NStrings->PromptCancel, [=](auto) {
                       RunOnGLThread([=] {
                         mServo->ContextMenuClosed(
                             Servo::ContextMenuResult::Dismissed_, 0);
                       });
                     }};
    msg.Commands().Append(cancel);
    msg.CancelCommandIndex((uint32_t)items.size());
    msg.ShowAsync();
  });
}

template <typename Callable> void ServoControl::RunOnUIThread(Callable cb) {
  Dispatcher().RunAsync(CoreDispatcherPriority::High, cb);
}

Collections::IVector<ServoApp::Pref> ServoControl::Preferences() {
  std::vector<ServoApp::Pref> prefs;
  for (auto [key, val, isDefault] : Servo::GetPrefs()) {
    prefs.push_back(ServoApp::Pref(key, val, isDefault));
  }
  return winrt::single_threaded_observable_vector<ServoApp::Pref>(
      std::move(prefs));
}

} // namespace winrt::ServoApp::implementation
