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

void App::createRootFrame(
    Frame &rootFrame, bool prelaunchActivated,
    winrt::Windows::Foundation::IInspectable const &args) {
  auto content = Window::Current().Content();
  if (content) {
    rootFrame = content.try_as<Frame>();
  }

  if (rootFrame == nullptr) {
    rootFrame = Frame();

    rootFrame.NavigationFailed({this, &App::OnNavigationFailed});

    if (prelaunchActivated == false) {
      if (rootFrame.Content() == nullptr) {
        rootFrame.Navigate(xaml_typename<ServoApp::BrowserPage>(), args);
      }
      Window::Current().Content(rootFrame);
      Window::Current().Activate();
    }
  } else {
    if (prelaunchActivated == false) {
      if (rootFrame.Content() == nullptr) {
        rootFrame.Navigate(xaml_typename<ServoApp::BrowserPage>(), args);
      }
      Window::Current().Activate();
    }
  }
}

void App::OnLaunched(LaunchActivatedEventArgs const &e) {
  Frame rootFrame{nullptr};
  this->createRootFrame(rootFrame, e.PrelaunchActivated(),
                        box_value(e.Arguments()));
}

void App::OnActivated(IActivatedEventArgs const &args) {
  if (args.Kind() == Windows::ApplicationModel::Activation::ActivationKind::
                         CommandLineLaunch) {
    auto cmdLineArgs{args.as<Windows::ApplicationModel::Activation::
                                 CommandLineActivatedEventArgs>()};
    auto cmdLineStr = cmdLineArgs.Operation().Arguments();
    Frame rootFrame{nullptr};
    this->createRootFrame(rootFrame, false, nullptr);
    auto page = rootFrame.Content().try_as<BrowserPage>();
    page->SetArgs(cmdLineStr);
    return;
  }

  if (args.Kind() ==
      Windows::ApplicationModel::Activation::ActivationKind::Protocol) {
    auto protocolActivatedEventArgs{args.as<
        Windows::ApplicationModel::Activation::ProtocolActivatedEventArgs>()};

    Frame rootFrame{nullptr};

    auto content = Window::Current().Content();
    bool isRunning = content != nullptr;
    if (!isRunning) {
      this->createRootFrame(rootFrame, false, nullptr);
    } else {
      rootFrame = content.try_as<Frame>();
    }
    auto page = rootFrame.Content().try_as<BrowserPage>();
    page->LoadFXRURI(protocolActivatedEventArgs.Uri());
  }
}

void App::OnSuspending(IInspectable const &, SuspendingEventArgs const &) {
  // FIXME: Apps can be suspended for various reasons, not just closing them.
  //        * Figure out how to save state like the current URL so it can be
  //          restored if necessary.
  //        * Determine if the user has actually closed the app and shutdown.
  /*auto content = Window::Current().Content();
  Frame rootFrame = content.try_as<Frame>();
  auto page = rootFrame.Content().try_as<BrowserPage>();
  page->Shutdown();*/
}

void App::OnNavigationFailed(IInspectable const &,
                             NavigationFailedEventArgs const &e) {
  throw hresult_error(E_FAIL, hstring(L"Failed to load Page ") +
                                  e.SourcePageType().Name);
}
