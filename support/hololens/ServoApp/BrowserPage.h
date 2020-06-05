/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#pragma once

#include "BrowserPage.g.h"
#include "ServoControl\ServoControl.h"

namespace winrt::ServoApp::implementation {

using namespace winrt::Windows;
using namespace winrt::Windows::Foundation;
using namespace winrt::Windows::UI::Xaml;

static const hstring SERVO_SCHEME = L"fxr";
static const hstring SERVO_SCHEME_SLASH_SLASH = L"fxr://";

struct BrowserPage : BrowserPageT<BrowserPage> {
public:
  BrowserPage();

  void OnForwardButtonClicked(IInspectable const &, RoutedEventArgs const &);
  void OnBackButtonClicked(IInspectable const &, RoutedEventArgs const &);
  void OnReloadButtonClicked(IInspectable const &, RoutedEventArgs const &);
  void OnStopButtonClicked(IInspectable const &, RoutedEventArgs const &);
  void OnHomeButtonClicked(IInspectable const &, RoutedEventArgs const &);
  void OnDevtoolsButtonClicked(IInspectable const &, RoutedEventArgs const &);
  void OnURLEdited(IInspectable const &, Input::KeyRoutedEventArgs const &);
  void OnURLFocused(IInspectable const &);
  void
  OnURLKeyboardAccelerator(IInspectable const &,
                           Input::KeyboardAcceleratorInvokedEventArgs const &);
  void Shutdown();
  void LoadServoURI(Uri uri);
  void SetTransientMode(bool);
  void SetArgs(hstring);
  void OnMediaControlsPlayClicked(IInspectable const &,
                                  RoutedEventArgs const &);
  void OnMediaControlsPauseClicked(IInspectable const &,
                                   RoutedEventArgs const &);
  void OnPrefererenceSearchboxEdited(IInspectable const &,
                                     Input::KeyRoutedEventArgs const &);

private:
  void UpdatePref(ServoApp::Pref, Controls::Control);
  void BindServoEvents();
  void BuildPrefList();
  DevtoolsStatus mDevtoolsStatus = DevtoolsStatus::Stopped;
  unsigned int mDevtoolsPort = 0;
};
} // namespace winrt::ServoApp::implementation

namespace winrt::ServoApp::factory_implementation {
struct BrowserPage : BrowserPageT<BrowserPage, implementation::BrowserPage> {};
} // namespace winrt::ServoApp::factory_implementation
