/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include "pch.h"
#include "logs.h"
#include "BrowserPage.h"
#include "BrowserPage.g.cpp"
#include "ImmersiveView.h"
#include "OpenGLES.h"

using namespace std::placeholders;
using namespace winrt::Windows::UI::Xaml;
using namespace winrt::Windows::UI::Core;
using namespace winrt::Windows::UI::ViewManagement;
using namespace winrt::Windows::Foundation;
using namespace winrt::Windows::Graphics::Holographic;
using namespace concurrency;
using namespace servo;

namespace winrt::ServoApp::implementation {
BrowserPage::BrowserPage() {
  log("BrowserPage::BrowserPage()");
  InitializeComponent();
  Loaded(std::bind(&BrowserPage::OnPageLoaded, this, _1, _2));
  Window::Current().CoreWindow().VisibilityChanged(
      std::bind(&BrowserPage::OnVisibilityChanged, this, _1, _2));
}

void BrowserPage::Shutdown() {
  log("BrowserPage::Shutdown()");

  if (mServo != nullptr) {
    if (!IsLoopRunning()) {
      // FIXME: this should not happen. In that case, we can't send the
      // shutdown event to Servo.
    } else {
      HANDLE hEvent = ::CreateEventA(nullptr, FALSE, FALSE, sShutdownEvent);
      SendEventToServo({{Event::SHUTDOWN}});
      log("Waiting for Servo to shutdown");
      ::WaitForSingleObject(hEvent, INFINITE);
      StopRenderLoop();
      mServo.reset();
    }
  }
}

void BrowserPage::OnPageLoaded(IInspectable const &, RoutedEventArgs const &) {
  log("BrowserPage::OnPageLoaded()");
  CreateRenderSurface();
  StartRenderLoop();
  swapChainPanel().PointerReleased(
      std::bind(&BrowserPage::OnSurfaceClicked, this, _1, _2));
  swapChainPanel().ManipulationDelta(
      std::bind(&BrowserPage::OnSurfaceManipulationDelta, this, _1, _2));
}

void BrowserPage::OnVisibilityChanged(CoreWindow const &,
                                      VisibilityChangedEventArgs const &args) {
  auto visible = args.Visible();

  // FIXME: for now, this is disabled as we get this message before shutdown,
  // stopping the event loop, which we can't recover from yet (see comment in
  // Loop())

  // if (visible && !IsLoopRunning()) {
  //  StartRenderLoop();
  //}
  // if (!visible && IsLoopRunning()) {
  //  StopRenderLoop();
  //}
}

void BrowserPage::CreateRenderSurface() {
  if (mRenderSurface == EGL_NO_SURFACE) {
    mRenderSurface = mOpenGLES.CreateSurface(swapChainPanel());
  }
}

void BrowserPage::DestroyRenderSurface() {
  mOpenGLES.DestroySurface(mRenderSurface);
  mRenderSurface = EGL_NO_SURFACE;
}

void BrowserPage::RecoverFromLostDevice() {
  StopRenderLoop();
  DestroyRenderSurface();
  mOpenGLES.Reset();
  CreateRenderSurface();
  StartRenderLoop();
}

/**** GL THREAD LOOP ****/

bool BrowserPage::IsLoopRunning() {
  return mLoopTask != nullptr && !mLoopTask->is_done();
}

void BrowserPage::Loop(cancellation_token cancel) {
  log("BrowserPage::Loop(). GL thread: %i", GetCurrentThreadId());

  HANDLE hEvent = ::CreateEventA(nullptr, FALSE, FALSE, sWakeupEvent);

  mOpenGLES.MakeCurrent(mRenderSurface);

  EGLint panelWidth = 0;
  EGLint panelHeight = 0;
  mOpenGLES.GetSurfaceDimensions(mRenderSurface, &panelWidth, &panelHeight);
  glViewport(0, 0, panelWidth, panelHeight);

  if (mServo == nullptr) {
    log("Entering loop");
    ServoDelegate *sd = static_cast<ServoDelegate *>(this);
    mServo = std::make_unique<Servo>(panelWidth, panelHeight, *sd);
  } else {
    // FIXME: this will fail since create_task didn't pick the thread
    // where Servo was running initially.
    throw winrt::hresult_error(E_FAIL, L"Recovering loop unimplemented");
  }

  // mServo->SetBatchMode(true);
  // FIXME: ^ this should be necessary as we call perform_update
  // ourself. But enabling batch mode will make clicking a link
  // not working because during the click, this thread is not
  // waiting on the hEvent object. See the "wakeup" comment.

  log("Entering loop");
  while (!cancel.is_canceled()) {
    // Block until wakeup is called.
    // Or run full speed if animating (see OnAnimatingChanged),
    // it will endup blocking on Flush to limit rendering to 60FPS
    if (!mAnimating) {
      ::WaitForSingleObject(hEvent, INFINITE);
    }
    mEventsMutex.lock();
    for (auto &&e : mEvents) {
      switch (e.type) {
      case Event::CLICK: {
        auto [x, y] = e.clickCoords;
        mServo->Click(x, y);
        break;
      }
      case Event::SCROLL: {
        auto [x, y, dx, dy] = e.scrollCoords;
        mServo->Scroll(x, y, dx, dy);
        break;
      }
      case Event::FORWARD:
        mServo->GoForward();
        break;
      case Event::BACK:
        mServo->GoBack();
        break;
      case Event::RELOAD:
        mServo->Reload();
        break;
      case Event::STOP:
        mServo->Stop();
        break;
      case Event::SHUTDOWN:
        log("Requesting Servo to shutdown");
        mServo->RequestShutdown();
        break;
      }
    }
    mEvents.clear();
    mEventsMutex.unlock();
    mServo->PerformUpdates();
  }
  log("Leaving loop");
  mServo->DeInit();
  cancel_current_task();
} // namespace winrt::ServoApp::implementation

void BrowserPage::StartRenderLoop() {
  if (IsLoopRunning()) {
#if defined _DEBUG
    throw winrt::hresult_error(E_FAIL, L"GL thread is already looping");
#else
    return;
#endif
  }
  log("BrowserPage::StartRenderLoop(). UI thread: %i", GetCurrentThreadId());
  auto token = mLoopCancel.get_token();
  mLoopTask = std::make_unique<Concurrency::task<void>>(
      Concurrency::create_task([=] { Loop(token); }, token));
}

void BrowserPage::StopRenderLoop() {
  if (IsLoopRunning()) {
    mLoopCancel.cancel();
    WakeUp();
    mLoopTask->wait();
    mLoopTask.reset();
  }
}

/**** SERVO CALLBACKS ****/

void BrowserPage::OnLoadStarted() {
  RunOnUIThread([=] {
    reloadButton().IsEnabled(false);
    stopButton().IsEnabled(true);
  });
}

void BrowserPage::OnLoadEnded() {
  RunOnUIThread([=] {
    reloadButton().IsEnabled(true);
    stopButton().IsEnabled(false);
  });
}

void BrowserPage::OnHistoryChanged(bool back, bool forward) {
  RunOnUIThread([=] {
    backButton().IsEnabled(back);
    forwardButton().IsEnabled(forward);
  });
}

void BrowserPage::OnShutdownComplete() {
  log("Servo notified ShutdownComplete");
  HANDLE hEvent = ::OpenEventA(EVENT_ALL_ACCESS, FALSE, sShutdownEvent);
  ::SetEvent(hEvent);
}

void BrowserPage::OnAlert(std::wstring message) {
  // FIXME: make this sync
  RunOnUIThread([=] {
    Windows::UI::Popups::MessageDialog msg{message};
    msg.ShowAsync();
  });
}

void BrowserPage::OnTitleChanged(std::wstring title) {
  RunOnUIThread([=] { ApplicationView::GetForCurrentView().Title(title); });
}

void BrowserPage::OnURLChanged(std::wstring url) {
  RunOnUIThread([=] { urlTextbox().Text(url); });
}

void BrowserPage::Flush() {
  if (mOpenGLES.SwapBuffers(mRenderSurface) != GL_TRUE) {
    // The call to eglSwapBuffers might not be successful (i.e. due to Device
    // Lost) If the call fails, then we must reinitialize EGL and the GL
    // resources.
    RunOnUIThread([=] { RecoverFromLostDevice(); });
  }
}

void BrowserPage::MakeCurrent() { mOpenGLES.MakeCurrent(mRenderSurface); }

void BrowserPage::WakeUp() {
  // FIXME: this won't work if it's triggered while the thread is not
  // waiting. We need a better looping logic.
  HANDLE hEvent = ::OpenEventA(EVENT_ALL_ACCESS, FALSE, sWakeupEvent);
  ::SetEvent(hEvent);
}

bool BrowserPage::OnAllowNavigation(std::wstring) { return true; }

void BrowserPage::OnAnimatingChanged(bool animating) { mAnimating = animating; }

template <typename Callable> void BrowserPage::RunOnUIThread(Callable cb) {
  swapChainPanel().Dispatcher().RunAsync(
      Windows::UI::Core::CoreDispatcherPriority::High, cb);
}

/**** USER INTERACTIONS WITH UI ****/

void BrowserPage::OnBackButtonClicked(IInspectable const &,
                                      RoutedEventArgs const &) {
  SendEventToServo({{Event::BACK}});
}

void BrowserPage::OnForwardButtonClicked(IInspectable const &,
                                         RoutedEventArgs const &) {
  SendEventToServo({{Event::FORWARD}});
}

void BrowserPage::OnReloadButtonClicked(IInspectable const &,
                                        RoutedEventArgs const &) {
  SendEventToServo({{Event::RELOAD}});
}

void BrowserPage::OnStopButtonClicked(IInspectable const &,
                                      RoutedEventArgs const &) {
  SendEventToServo({{Event::STOP}});
}

void BrowserPage::OnImmersiveButtonClicked(IInspectable const &,
                                           RoutedEventArgs const &) {
  if (HolographicSpace::IsAvailable()) {
    log("Holographic space available");
    auto v =
        winrt::Windows::ApplicationModel::Core::CoreApplication::CreateNewView(
            mImmersiveViewSource);
    auto parentId = ApplicationView::GetForCurrentView().Id();
    v.Dispatcher().RunAsync(CoreDispatcherPriority::Normal, [=] {
      auto winId = ApplicationView::GetForCurrentView().Id();
      ApplicationViewSwitcher::SwitchAsync(winId, parentId);
      log("Immersive view started");
    });
  } else {
    log("Holographic space not available");
  }
}

void BrowserPage::OnSurfaceManipulationDelta(
    IInspectable const &, Input::ManipulationDeltaRoutedEventArgs const &e) {
  auto x = e.Position().X;
  auto y = e.Position().Y;
  auto dx = e.Delta().Translation.X;
  auto dy = e.Delta().Translation.Y;
  Event event = {{Event::SCROLL}};
  event.scrollCoords = {dx, dy, x, y};
  SendEventToServo(event);
  e.Handled(true);
}

void BrowserPage::OnSurfaceClicked(IInspectable const &,
                                   Input::PointerRoutedEventArgs const &e) {
  auto coords = e.GetCurrentPoint(swapChainPanel());
  auto x = coords.Position().X;
  auto y = coords.Position().Y;
  SendEventToServo({{Event::CLICK}, {x, y}});
  e.Handled(true);
}

void BrowserPage::SendEventToServo(Event event) {
  mEventsMutex.lock();
  mEvents.push_back(event);
  mEventsMutex.unlock();
  WakeUp();
}

} // namespace winrt::ServoApp::implementation
