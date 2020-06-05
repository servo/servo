/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#pragma once

#include "BrowserPage.g.h"
#include "ServoControl\ServoControl.h"

namespace winrt::ServoApp::implementation {

using namespace winrt::Windows::Foundation;

static const hstring SERVO_SCHEME = L"fxr";
static const hstring SERVO_SCHEME_SLASH_SLASH = L"fxr://";

struct BrowserPage : BrowserPageT<BrowserPage> {
public:
  BrowserPage();

  void OnForwardButtonClicked(Windows::Foundation::IInspectable const &,
                              Windows::UI::Xaml::RoutedEventArgs const &);
  void OnBackButtonClicked(Windows::Foundation::IInspectable const &,
                           Windows::UI::Xaml::RoutedEventArgs const &);
  void OnReloadButtonClicked(Windows::Foundation::IInspectable const &,
                             Windows::UI::Xaml::RoutedEventArgs const &);
  void OnStopButtonClicked(Windows::Foundation::IInspectable const &,
                           Windows::UI::Xaml::RoutedEventArgs const &);
  void OnHomeButtonClicked(Windows::Foundation::IInspectable const &,
                           Windows::UI::Xaml::RoutedEventArgs const &);
  void OnDevtoolsButtonClicked(Windows::Foundation::IInspectable const &,
                               Windows::UI::Xaml::RoutedEventArgs const &);
  void OnURLEdited(Windows::Foundation::IInspectable const &,
                   Windows::UI::Xaml::Input::KeyRoutedEventArgs const &);
  void OnURLFocused(Windows::Foundation::IInspectable const &);
  void OnURLKeyboardAccelerator(
      Windows::Foundation::IInspectable const &,
      Windows::UI::Xaml::Input::KeyboardAcceleratorInvokedEventArgs const &);
  void Shutdown();
  void LoadServoURI(Windows::Foundation::Uri uri);
  void SetTransientMode(bool);
  void SetArgs(hstring);
  void OnMediaControlsPlayClicked(Windows::Foundation::IInspectable const &,
                                  Windows::UI::Xaml::RoutedEventArgs const &);
  void OnMediaControlsPauseClicked(Windows::Foundation::IInspectable const &,
                                   Windows::UI::Xaml::RoutedEventArgs const &);
  void OnPrefererenceSearchboxEdited(
      Windows::Foundation::IInspectable const &,
      Windows::UI::Xaml::Input::KeyRoutedEventArgs const &);

private:
  void UpdatePref(ServoApp::Pref, Windows::UI::Xaml::Controls::Control);
  void BindServoEvents();
  void BuildPrefList();
  DevtoolsStatus mDevtoolsStatus = DevtoolsStatus::Stopped;
  unsigned int mDevtoolsPort = 0;
};
} // namespace winrt::ServoApp::implementation

namespace winrt::ServoApp::factory_implementation {
struct BrowserPage : BrowserPageT<BrowserPage, implementation::BrowserPage> {};
} // namespace winrt::ServoApp::factory_implementation
