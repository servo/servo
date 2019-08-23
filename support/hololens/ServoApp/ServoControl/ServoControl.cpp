#include "pch.h"
#include "ServoControl.h"
#include "ServoControl.g.cpp"
#include <stdlib.h>

using namespace std::placeholders;
using namespace winrt::Windows::UI::Xaml;
using namespace winrt::Windows::UI::Core;
using namespace winrt::Windows::Foundation;
using namespace concurrency;
using namespace winrt::servo;

namespace winrt::ServoApp::implementation {

ServoControl::ServoControl() {
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
  Panel().PointerReleased(
      std::bind(&ServoControl::OnSurfaceClicked, this, _1, _2));
  Panel().ManipulationDelta(
      std::bind(&ServoControl::OnSurfaceManipulationDelta, this, _1, _2));
  InitializeConditionVariable(&mGLCondVar);
  InitializeCriticalSection(&mGLLock);
  CreateRenderSurface();
  StartRenderLoop();
}

Controls::SwapChainPanel ServoControl::Panel() {
  // FIXME: is there a better way of doing this?
  return GetTemplateChild(L"swapChainPanel").as<Controls::SwapChainPanel>();
}

void ServoControl::CreateRenderSurface() {
  if (mRenderSurface == EGL_NO_SURFACE) {
    mRenderSurface = mOpenGLES.CreateSurface(Panel());
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
  auto x = e.Position().X;
  auto y = e.Position().Y;
  auto dx = e.Delta().Translation.X;
  auto dy = e.Delta().Translation.Y;
  RunOnGLThread([=] { mServo->Scroll(dx, dy, x, y); });
  e.Handled(true);
}

void ServoControl::OnSurfaceClicked(IInspectable const &,
                                    Input::PointerRoutedEventArgs const &e) {
  auto coords = e.GetCurrentPoint(Panel());
  auto x = coords.Position().X;
  auto y = coords.Position().Y;
  RunOnGLThread([=] { mServo->Click(x, y); });
  e.Handled(true);
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
Uri ServoControl::LoadURIOrSearch(hstring input) {
  auto uri = TryParseURI(input);
  if (uri == std::nullopt) {
    bool has_dot = wcsstr(input.c_str(), L".") != nullptr;
    hstring input2 = L"https://" + input;
    uri = TryParseURI(input2);
    if (uri == std::nullopt || !has_dot) {
      hstring input3 =
          L"https://duckduckgo.com/html/?q=" + Uri::EscapeComponent(input);
      uri = TryParseURI(input3);
    }
  }
  auto finalUri = uri.value();
  if (!mLooping) {
    mInitialURL = finalUri.ToString();
  } else {
    RunOnGLThread([=] { mServo->LoadUri(finalUri.ToString()); });
  }
  return finalUri;
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
    mServo = std::make_unique<Servo>(mInitialURL, panelWidth, panelHeight, *sd);
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
  cancel_current_task();
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

void ServoControl::OnServoAlert(hstring message) {
  // FIXME: make this sync
  RunOnUIThread([=] {
    Windows::UI::Popups::MessageDialog msg{message};
    msg.ShowAsync();
  });
}

void ServoControl::OnServoTitleChanged(hstring title) {
  RunOnUIThread([=] { mOnTitleChangedEvent(*this, title); });
}

void ServoControl::OnServoURLChanged(hstring url) {
  RunOnUIThread([=] { mOnURLChangedEvent(*this, url); });
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

bool ServoControl::OnServoAllowNavigation(hstring) { return true; }

void ServoControl::OnServoAnimatingChanged(bool animating) {
  EnterCriticalSection(&mGLLock);
  mAnimating = animating;
  LeaveCriticalSection(&mGLLock);
  WakeConditionVariable(&mGLCondVar);
}

void ServoControl::OnServoIMEStateChanged(bool aShow) {
  // FIXME: https://docs.microsoft.com/en-us/windows/win32/winauto/uiauto-implementingtextandtextrange
}

template <typename Callable> void ServoControl::RunOnUIThread(Callable cb) {
  Dispatcher().RunAsync(CoreDispatcherPriority::High, cb);
}

} // namespace winrt::ServoApp::implementation
