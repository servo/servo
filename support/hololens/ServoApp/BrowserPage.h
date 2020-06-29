/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#pragma once

#include "BrowserPage.g.h"
#include "ConsoleLog.g.h"
#include "ServoControl/ServoControl.h"
#include "Devtools/Client.h"

namespace winrt::ServoApp::implementation {

using namespace winrt::Windows;
using namespace winrt::Windows::Data::Json;
using namespace winrt::Windows::Foundation;
using namespace winrt::Windows::UI::Xaml;
using namespace winrt::Windows::UI::Xaml::Media;

static const hstring SERVO_SCHEME = L"fxr";
static const hstring SERVO_SCHEME_SLASH_SLASH = L"fxr://";

struct BrowserPage : BrowserPageT<BrowserPage>, public servo::DevtoolsDelegate {
public:
  BrowserPage();

  void OnForwardButtonClicked(IInspectable const &, RoutedEventArgs const &);
  void OnBackButtonClicked(IInspectable const &, RoutedEventArgs const &);
  void OnReloadButtonClicked(IInspectable const &, RoutedEventArgs const &);
  void OnStopButtonClicked(IInspectable const &, RoutedEventArgs const &);
  void OnHomeButtonClicked(IInspectable const &, RoutedEventArgs const &);
  void OnDevtoolsButtonClicked(IInspectable const &, RoutedEventArgs const &);
  void OnJSInputEdited(IInspectable const &, Input::KeyRoutedEventArgs const &);
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
  void OnDevtoolsMessage(servo::DevtoolsMessageLevel, hstring, hstring);
  void ClearConsole();
  void OnDevtoolsDetached();
  Collections::IObservableVector<IInspectable> ConsoleLogs() { return mLogs; };

private:
  void UpdatePref(ServoApp::Pref, Controls::Control);
  void BindServoEvents();
  void BuildPrefList();
  DevtoolsStatus mDevtoolsStatus = DevtoolsStatus::Stopped;
  unsigned int mDevtoolsPort = 0;
  hstring mDevtoolsToken;
  std::unique_ptr<servo::DevtoolsClient> mDevtoolsClient;
  Collections::IObservableVector<IInspectable> mLogs;
};

struct ConsoleLog : ConsoleLogT<ConsoleLog> {
public:
  ConsoleLog(Windows::UI::Color glyph, hstring g, hstring b, hstring s)
      : mGlyph(g), mSource(s), mBody(b) {
    mGlyphColor = UI::Xaml::Media::SolidColorBrush(glyph);
  };
  SolidColorBrush GlyphColor() { return mGlyphColor; };
  hstring Glyph() { return mGlyph; };
  hstring Source() { return mSource; };
  hstring Body() { return mBody; };

private:
  SolidColorBrush mGlyphColor;
  hstring mGlyph;
  hstring mSource;
  hstring mBody;
};

} // namespace winrt::ServoApp::implementation

namespace winrt::ServoApp::factory_implementation {
struct BrowserPage : BrowserPageT<BrowserPage, implementation::BrowserPage> {};
struct ConsoleLog : ConsoleLogT<ConsoleLog, implementation::ConsoleLog> {};
} // namespace winrt::ServoApp::factory_implementation
