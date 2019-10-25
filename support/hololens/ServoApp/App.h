/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#pragma once
#include "App.xaml.g.h"

namespace winrt::ServoApp::implementation {
struct App : AppT<App> {
  App();

  void createRootFrame(winrt::Windows::UI::Xaml::Controls::Frame &, bool,
                       winrt::Windows::Foundation::IInspectable const &);
  void OnLaunched(
      Windows::ApplicationModel::Activation::LaunchActivatedEventArgs const &);
  void App::OnActivated(
      Windows::ApplicationModel::Activation::IActivatedEventArgs const &);
  void OnSuspending(IInspectable const &,
                    Windows::ApplicationModel::SuspendingEventArgs const &);
  void OnNavigationFailed(
      IInspectable const &,
      Windows::UI::Xaml::Navigation::NavigationFailedEventArgs const &);
};
} // namespace winrt::ServoApp::implementation
