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

static char sWakeupEvent[] = "SIGNAL_WAKEUP";

namespace winrt::ServoApp::implementation {
BrowserPage::BrowserPage() {
  InitializeComponent();
  log("BrowserPage::BrowserPage()");
  Loaded(std::bind(&BrowserPage::OnPageLoaded, this, _1, _2));
  Window::Current().CoreWindow().VisibilityChanged(
      std::bind(&BrowserPage::OnVisibilityChanged, this, _1, _2));
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
  Servo::sWakeUp();
}

void BrowserPage::OnBackButtonClicked(IInspectable const &,
                                      RoutedEventArgs const &) {
  SendEventToServo({{Event::BACK}});
}

void BrowserPage::OnForwardButtonClicked(IInspectable const &,
                                         RoutedEventArgs const &) {
  SendEventToServo({{Event::FORWARD}});
}
void BrowserPage::OnImmersiveButtonClicked(IInspectable const &,
                                           RoutedEventArgs const &) {
  if (HolographicSpace::IsAvailable()) {
    log("Holographic space available");
    auto v =
        winrt::Windows::ApplicationModel::Core::CoreApplication::CreateNewView(
            mImmersiveViewSource);
    auto parentId = ApplicationView::GetForCurrentView().Id();
    v.Dispatcher().RunAsync(CoreDispatcherPriority::Normal, [=]() {
      auto winId = ApplicationView::GetForCurrentView().Id();
      ApplicationViewSwitcher::SwitchAsync(winId, parentId);
      log("Immersive view started");
    });
  } else {
    log("Holographic space not available");
  }
}

void BrowserPage::OnVisibilityChanged(CoreWindow const &,
                                      VisibilityChangedEventArgs const &args) {
  auto visible = args.Visible();
  if (visible && !IsLoopRunning()) {
    StartRenderLoop();
  }
  if (!visible && IsLoopRunning()) {
    StopRenderLoop();
  }
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

bool BrowserPage::IsLoopRunning() {
  return mLoopTask != nullptr && !mLoopTask->is_done();
}

void BrowserPage::Loop(cancellation_token cancel) {
  log("BrowserPage::Loop(). GL thread: %i", GetCurrentThreadId());

  HANDLE hEvent = ::CreateEventA(nullptr, FALSE, FALSE, sWakeupEvent);

  Servo::sOnAlert = [=](std::wstring message) {
    // FIXME: make this sync
    swapChainPanel().Dispatcher().RunAsync(
        Windows::UI::Core::CoreDispatcherPriority::High, [=]() {
          Windows::UI::Popups::MessageDialog msg{message};
          msg.ShowAsync();
        });
  };

  Servo::sOnTitleChanged = [=](std::wstring title) {
    swapChainPanel().Dispatcher().RunAsync(CoreDispatcherPriority::High, [=]() {
      ApplicationView::GetForCurrentView().Title(title);
    });
  };

  Servo::sOnURLChanged = [=](std::wstring url) {
    swapChainPanel().Dispatcher().RunAsync(CoreDispatcherPriority::High,
                                           [=]() { urlTextbox().Text(url); });
  };

  Servo::sMakeCurrent = [=]() {
    /* EGLint panelWidth = 0; */
    /* EGLint panelHeight = 0; */
    /* mOpenGLES->GetSurfaceDimensions(mRenderSurface, &panelWidth,
     * &panelHeight); */
    /* glViewport(0, 0, panelWidth, panelHeight); */
    /* mServo->SetSize(panelWidth, panelHeight); */
    mOpenGLES.MakeCurrent(mRenderSurface);
  };

  Servo::sFlush = [=]() {
    if (mOpenGLES.SwapBuffers(mRenderSurface) != GL_TRUE) {
      // The call to eglSwapBuffers might not be successful (i.e. due to Device
      // Lost) If the call fails, then we must reinitialize EGL and the GL
      // resources.
      swapChainPanel().Dispatcher().RunAsync(
          CoreDispatcherPriority::High, [this]() { RecoverFromLostDevice(); });
    }
  };

  mOpenGLES.MakeCurrent(mRenderSurface);

  EGLint panelWidth = 0;
  EGLint panelHeight = 0;
  mOpenGLES.GetSurfaceDimensions(mRenderSurface, &panelWidth, &panelHeight);
  glViewport(0, 0, panelWidth, panelHeight);
  mServo = std::make_unique<Servo>(panelWidth, panelHeight);

  // mServo->SetBatchMode(true);
  // FIXME: ^ this should be necessary as call perform_update
  // ourself. But enabling batch mode will make clicking a link
  // not working because during the click, this thread is not
  // waiting on the hEvent object. See the "wakeup" comment.

  while (!cancel.is_canceled()) {
    // Block until Servo::sWakeUp is called.
    // Or run full speed if animating (see on_animating_changed),
    // it will endup blocking on SwapBuffers to limit rendering to 60FPS
    if (!Servo::sAnimating) {
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
      }
    }
    mEvents.clear();
    mEventsMutex.unlock();
    mServo->PerformUpdates();
  }
  cancel_current_task();
}

void BrowserPage::StartRenderLoop() {
  if (IsLoopRunning()) {
    return;
  }

  auto token = mLoopCancel.get_token();

  Servo::sWakeUp = []() {
    // FIXME: this won't work if it's triggered while the thread is not
    // waiting. We need a better looping logic.
    HANDLE hEvent = ::OpenEventA(EVENT_ALL_ACCESS, FALSE, sWakeupEvent);
    ::SetEvent(hEvent);
  };

  log("BrowserPage::StartRenderLoop(). UI thread: %i", GetCurrentThreadId());

  mLoopTask = std::make_unique<Concurrency::task<void>>(
      Concurrency::create_task([=] { Loop(token); }, token));
}

void BrowserPage::StopRenderLoop() {
  if (IsLoopRunning()) {
    mLoopCancel.cancel();
    mLoopTask->wait();
    mLoopTask.reset();
  }
}

} // namespace winrt::ServoApp::implementation
