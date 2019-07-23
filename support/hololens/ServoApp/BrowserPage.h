/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#pragma once

#include "BrowserPage.g.h"
#include "ImmersiveView.h"
#include "OpenGLES.h"
#include "Servo.h"

namespace winrt::ServoApp::implementation {

struct Event {
  enum { CLICK, SCROLL, BACK, FORWARD } type;
  std::tuple<float, float> clickCoords;
  std::tuple<float, float, float, float> scrollCoords;
};

struct BrowserPage : BrowserPageT<BrowserPage> {
public:
  BrowserPage();

  void OnImmersiveButtonClicked(Windows::Foundation::IInspectable const &,
                                Windows::UI::Xaml::RoutedEventArgs const &);
  void OnForwardButtonClicked(Windows::Foundation::IInspectable const &,
                              Windows::UI::Xaml::RoutedEventArgs const &);
  void OnBackButtonClicked(Windows::Foundation::IInspectable const &,
                           Windows::UI::Xaml::RoutedEventArgs const &);

  void
  OnSurfaceClicked(Windows::Foundation::IInspectable const &,
                   Windows::UI::Xaml::Input::PointerRoutedEventArgs const &);

  void BrowserPage::OnSurfaceManipulationDelta(
      IInspectable const &,
      Windows::UI::Xaml::Input::ManipulationDeltaRoutedEventArgs const &e);

private:
  void OnVisibilityChanged(
      Windows::UI::Core::CoreWindow const &,
      Windows::UI::Core::VisibilityChangedEventArgs const &args);
  void OnPageLoaded(Windows::Foundation::IInspectable const &,
                    Windows::UI::Xaml::RoutedEventArgs const &);

  void CreateRenderSurface();
  void DestroyRenderSurface();
  void RecoverFromLostDevice();

  void StartRenderLoop();
  void StopRenderLoop();
  void Loop(Concurrency::cancellation_token);
  bool IsLoopRunning();

  Concurrency::cancellation_token_source mLoopCancel;
  std::unique_ptr<Concurrency::task<void>> mLoopTask;
  winrt::ServoApp::ImmersiveViewSource mImmersiveViewSource;
  EGLSurface mRenderSurface{EGL_NO_SURFACE};
  std::unique_ptr<Servo> mServo;

  void BrowserPage::SendEventToServo(Event event);
  std::vector<Event> mEvents;
  std::mutex mEventsMutex;

  OpenGLES mOpenGLES; // FIXME: shared pointer
};
} // namespace winrt::ServoApp::implementation

namespace winrt::ServoApp::factory_implementation {
struct BrowserPage : BrowserPageT<BrowserPage, implementation::BrowserPage> {};
} // namespace winrt::ServoApp::factory_implementation
