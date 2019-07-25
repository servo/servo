/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include "pch.h"
#include "App.h"
#include "BrowserPage.h"

using namespace winrt::Windows::ApplicationModel;
using namespace winrt::Windows::ApplicationModel::Activation;
using namespace winrt::Windows::Foundation;
using namespace winrt::Windows::UI::Xaml;
using namespace winrt::Windows::UI::Xaml::Controls;
using namespace winrt::Windows::UI::Xaml::Navigation;
using namespace winrt::ServoApp;
using namespace winrt::ServoApp::implementation;

App::App() {
  InitializeComponent();
  Suspending({this, &App::OnSuspending});

#if defined _DEBUG &&                                                          \
    !defined DISABLE_XAML_GENERATED_BREAK_ON_UNHANDLED_EXCEPTION
  UnhandledException(
      [this](IInspectable const &, UnhandledExceptionEventArgs const &e) {
        if (IsDebuggerPresent()) {
          auto errorMessage = e.Message();
          __debugbreak();
        }
      });
#endif
}

void App::OnLaunched(LaunchActivatedEventArgs const &e) {
  Frame rootFrame{nullptr};
  auto content = Window::Current().Content();
  if (content) {
    rootFrame = content.try_as<Frame>();
  }

  if (rootFrame == nullptr) {
    rootFrame = Frame();

    rootFrame.NavigationFailed({this, &App::OnNavigationFailed});

    if (e.PrelaunchActivated() == false) {
      if (rootFrame.Content() == nullptr) {
        rootFrame.Navigate(xaml_typename<ServoApp::BrowserPage>(),
                           box_value(e.Arguments()));
      }
      Window::Current().Content(rootFrame);
      Window::Current().Activate();
    }
  } else {
    if (e.PrelaunchActivated() == false) {
      if (rootFrame.Content() == nullptr) {
        rootFrame.Navigate(xaml_typename<ServoApp::BrowserPage>(),
                           box_value(e.Arguments()));
      }
      Window::Current().Activate();
    }
  }
}

void App::OnSuspending([[maybe_unused]] IInspectable const &sender,
                       [[maybe_unused]] SuspendingEventArgs const &e) {
  auto content = Window::Current().Content();
  Frame rootFrame = content.try_as<Frame>();
  auto page = rootFrame.Content().try_as<BrowserPage>();
  page->Shutdown();
}

void App::OnNavigationFailed(IInspectable const &,
                             NavigationFailedEventArgs const &e) {
  throw hresult_error(E_FAIL, hstring(L"Failed to load Page ") +
                                  e.SourcePageType().Name);
}
