/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#pragma once
#include "App.xaml.g.h"

namespace winrt::ServoApp::implementation {
using namespace winrt::Windows::Foundation;
using namespace winrt::Windows::UI::Xaml;
using namespace winrt::Windows::ApplicationModel;
using namespace winrt::Windows::ApplicationModel::Activation;

struct App : AppT<App> {
  App();

  void createRootFrame(Controls::Frame &, bool, IInspectable const &);
  void OnLaunched(LaunchActivatedEventArgs const &);
  void OnActivated(IActivatedEventArgs const &);
  void OnSuspending(IInspectable const &, SuspendingEventArgs const &);
  void OnNavigationFailed(IInspectable const &,
                          Navigation::NavigationFailedEventArgs const &);
};
} // namespace winrt::ServoApp::implementation
