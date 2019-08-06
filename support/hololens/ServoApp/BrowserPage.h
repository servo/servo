/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#pragma once

#include "BrowserPage.g.h"
#include "ImmersiveView.h"
#include "ServoControl.h"


namespace winrt::ServoApp::implementation {

struct BrowserPage : BrowserPageT<BrowserPage> {
public:
  BrowserPage();

  void OnImmersiveButtonClicked(Windows::Foundation::IInspectable const &,
                                Windows::UI::Xaml::RoutedEventArgs const &);
  void OnForwardButtonClicked(Windows::Foundation::IInspectable const &,
                              Windows::UI::Xaml::RoutedEventArgs const &);
  void OnBackButtonClicked(Windows::Foundation::IInspectable const &,
                           Windows::UI::Xaml::RoutedEventArgs const &);
  void OnReloadButtonClicked(Windows::Foundation::IInspectable const &,
                              Windows::UI::Xaml::RoutedEventArgs const &);
  void OnStopButtonClicked(Windows::Foundation::IInspectable const &,
                           Windows::UI::Xaml::RoutedEventArgs const &);
  void OnURLEdited(Windows::Foundation::IInspectable const &,
                   Windows::UI::Xaml::Input::KeyRoutedEventArgs const &);
  void Shutdown();

private:
  winrt::ServoApp::ImmersiveViewSource mImmersiveViewSource;
  void BindServoEvents();
};
} // namespace winrt::ServoApp::implementation

namespace winrt::ServoApp::factory_implementation {
struct BrowserPage : BrowserPageT<BrowserPage, implementation::BrowserPage> {};
} // namespace winrt::ServoApp::factory_implementation
